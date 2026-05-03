use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, Sse},
    Json,
};
use futures_util::{stream, StreamExt};

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::domain::supporting::observability_audit::TraceRecord::TraceRecord;
use crate::infrastructure::http::AppState::AppState;
use crate::shared::json_extractor::UnifiedJson;
use crate::shared::response;
use crate::shared::validator::validate_request;

const MAX_ROLLBACK_RETRIES: u32 = 3;

async fn retry_release_tokens(
    state: &AppState,
    tokens: u64,
    tenant_id: &str,
    app_id: &str,
    request_id: &str,
) {
    for attempt in 0..MAX_ROLLBACK_RETRIES {
        match state.release_tokens(tokens, tenant_id, app_id).await {
            Ok(_) => return,
            Err(e) => {
                tracing::warn!(
                    request_id = %request_id,
                    attempt,
                    error = %e,
                    "quota rollback failed, retrying"
                );
                if attempt < MAX_ROLLBACK_RETRIES - 1 {
                    tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                }
            }
        }
    }
    tracing::error!(request_id = %request_id, "quota rollback failed after all retries");
}

pub async fn chat_stream(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantIdentity>,
    headers: HeaderMap,
    UnifiedJson(payload): UnifiedJson<CompletionRequest>,
) -> Result<Sse<futures_util::stream::BoxStream<'static, Result<Event, Infallible>>>, (StatusCode, Json<serde_json::Value>)> {
    validate_request(&payload)?;

    let estimated_tokens: u64 = payload
        .messages
        .iter()
        .map(|m| crate::shared::token_estimator::estimate_tokens(&m.content) + 4)
        .sum();

    match state.try_consume_tokens(estimated_tokens, &tenant.tenant_id, &tenant.app_id).await {
        Ok(true) => {}
        Ok(false) => return Err(response::err(StatusCode::PAYMENT_REQUIRED, "quota exceeded")),
        Err(e) => {
            tracing::error!("quota check failed: {}", e);
            return Err(response::err(StatusCode::INTERNAL_SERVER_ERROR, "quota service unavailable"));
        }
    }

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    let trace = TraceRecord {
        request_id,
        provider: "openai-compatible".to_string(),
        span_id: None,
    };

    tracing::info!(
        tenant_id = %tenant.tenant_id,
        app_id = %tenant.app_id,
        request_id = %trace.request_id,
        provider = %trace.provider,
        estimated_tokens = estimated_tokens,
        "chat stream request"
    );

    let streaming = match state.chat_service.stream_complete(payload).await {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(request_id = %trace.request_id, "provider stream error: {:?}", err);
            retry_release_tokens(&state, estimated_tokens, &tenant.tenant_id, &tenant.app_id, &trace.request_id).await;
            let evs = stream::iter(vec![
                Ok(Event::default().event("error").data(serde_json::json!({"message": "upstream service error"}).to_string())),
                Ok(Event::default().event("done").data("{\"finish_reason\":\"error\"}")),
            ]);
            return Ok(Sse::new(Box::pin(evs)));
        }
    };

    let upstream = streaming.stream;
    let usage_rx = streaming.usage_rx;

    let has_dao = state.token_usage_dao.is_some();
    if has_dao {
        let dao = state.token_usage_dao.clone();
        let tenant_id = tenant.tenant_id.clone();
        let app_id = tenant.app_id.clone();
        let req_id = trace.request_id.clone();
        let app_state = state.clone();
        tokio::spawn(async move {
            let result = tokio::time::timeout(Duration::from_secs(10), async {
                match usage_rx.await {
                    Ok(Some(mut usage)) => {
                        if usage.request_id.is_empty() {
                            usage.request_id = req_id.clone();
                        }
                        if usage.tenant_id.is_empty() {
                            usage.tenant_id = tenant_id.clone();
                        }
                        if usage.app_id.is_empty() {
                            usage.app_id = app_id.clone();
                        }
                        let actual = usage.total_tokens as u64;
                        if actual > estimated_tokens {
                            if let Err(e) = app_state.try_consume_tokens(actual - estimated_tokens, &tenant_id, &app_id).await {
                                tracing::warn!(request_id = %req_id, "streaming quota top-up failed: {}", e);
                            }
                        } else if actual < estimated_tokens {
                            retry_release_tokens(&app_state, estimated_tokens - actual, &tenant_id, &app_id, &req_id).await;
                        }
                        if let Some(ref dao) = dao {
                            if let Err(e) = dao.insert(&usage).await {
                                tracing::error!("failed to persist streaming token usage: {}", e);
                                retry_release_tokens(&app_state, estimated_tokens, &tenant_id, &app_id, &req_id).await;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {
                        tracing::error!("usage oneshot channel dropped");
                    }
                }
            }).await;

            if result.is_err() {
                tracing::warn!(request_id = %req_id, "streaming usage persistence timed out");
            }
        });
    } else {
        let _ = usage_rx;
    }

    let trace_id = trace.request_id.clone();
    let out = upstream.map(move |item| -> Result<Event, Infallible> {
        match item {
            Ok(node) => {
                tracing::debug!(request_id = %trace_id, raw = %node, "provider raw event");
                Ok(Event::default().event("raw").data(node.to_string()))
            }
            Err(err) => Ok(Event::default().event("error").data(
                serde_json::json!({"message": err.to_string()}).to_string(),
            )),
        }
    });
    let done = stream::iter(vec![Ok(Event::default().event("done").data("{\"finish_reason\":\"stop\"}"))]);
    let merged: futures_util::stream::BoxStream<'static, Result<Event, Infallible>> = Box::pin(out.chain(done));

    Ok(Sse::new(merged))
}
