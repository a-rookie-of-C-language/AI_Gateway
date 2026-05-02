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
use crate::shared::response;

const MAX_MESSAGES: usize = 128;
const MAX_MESSAGE_CONTENT_LEN: usize = 128 * 1024;
const VALID_ROLES: &[&str] = &["system", "user", "assistant", "tool"];

fn validate_request(payload: &CompletionRequest) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if payload.messages.is_empty() {
        return Err(response::err(StatusCode::BAD_REQUEST, "messages must not be empty"));
    }
    if payload.messages.len() > MAX_MESSAGES {
        return Err(response::err(
            StatusCode::BAD_REQUEST,
            &format!("messages count exceeds limit of {}", MAX_MESSAGES),
        ));
    }
    for (i, msg) in payload.messages.iter().enumerate() {
        if !VALID_ROLES.contains(&msg.role.as_str()) {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("invalid role '{}' at message index {}", msg.role, i),
            ));
        }
        if msg.content.len() > MAX_MESSAGE_CONTENT_LEN {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("message content exceeds {} bytes at index {}", MAX_MESSAGE_CONTENT_LEN, i),
            ));
        }
    }
    Ok(())
}

pub async fn chat_stream(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantIdentity>,
    headers: HeaderMap,
    Json(payload): Json<CompletionRequest>,
) -> Result<Sse<futures_util::stream::BoxStream<'static, Result<Event, Infallible>>>, (StatusCode, Json<serde_json::Value>)> {
    validate_request(&payload)?;

    let estimated_tokens: u64 = payload
        .messages
        .iter()
        .map(|m| (m.content.chars().count() as u64 + 2) / 3)
        .sum();

    match state.try_consume_tokens(estimated_tokens).await {
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
            if let Err(e) = state.release_tokens(estimated_tokens).await {
                tracing::error!(request_id = %trace.request_id, "quota rollback on stream error failed: {}", e);
            }
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
                            if let Err(e) = app_state.try_consume_tokens(actual - estimated_tokens).await {
                                tracing::warn!(request_id = %req_id, "streaming quota top-up failed: {}", e);
                            }
                        }
                        if let Some(ref dao) = dao {
                            if let Err(e) = dao.insert(&usage).await {
                                tracing::error!("failed to persist streaming token usage: {}", e);
                                if let Err(rollback_err) = app_state.release_tokens(estimated_tokens).await {
                                    tracing::error!(request_id = %req_id, "streaming quota rollback failed: {}", rollback_err);
                                }
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
