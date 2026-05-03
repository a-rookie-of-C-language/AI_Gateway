use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::json;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::quota_billing::TokenUsage::TokenUsage;
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

pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantIdentity>,
    headers: HeaderMap,
    Json(payload): Json<CompletionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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
        "chat completion request"
    );

    match state.chat_service.complete(payload).await {
        Ok(data) => {
            if let Some(tt) = data.total_tokens {
                let actual = tt as u64;
                if actual > estimated_tokens {
                    if let Err(e) = state.try_consume_tokens(actual - estimated_tokens).await {
                        tracing::warn!(request_id = %trace.request_id, "quota top-up failed: {}", e);
                    }
                } else if actual < estimated_tokens {
                    if let Err(e) = state.release_tokens(estimated_tokens - actual).await {
                        tracing::warn!(request_id = %trace.request_id, "quota rollback failed: {}", e);
                    }
                }
            }
            if let (Some(pt), Some(ct), Some(tt)) = (data.prompt_tokens, data.completion_tokens, data.total_tokens) {
                if let Some(ref dao) = state.token_usage_dao {
                    let usage = TokenUsage {
                        request_id: trace.request_id.clone(),
                        tenant_id: tenant.tenant_id.clone(),
                        app_id: tenant.app_id.clone(),
                        model: data.model.clone(),
                        prompt_tokens: pt,
                        completion_tokens: ct,
                        total_tokens: tt,
                        created_at: chrono::Utc::now(),
                    };
                    if let Err(e) = dao.insert(&usage).await {
                        tracing::error!(request_id = %trace.request_id, "failed to persist token usage: {}", e);
                        if let Err(rollback_err) = state.release_tokens(estimated_tokens).await {
                            tracing::error!(request_id = %trace.request_id, "quota rollback failed: {}", rollback_err);
                        }
                    }
                }
            }
            Ok(response::ok(json!(data)))
        }
        Err(err) => {
            tracing::error!(request_id = %trace.request_id, "provider error: {:?}", err);
            if let Err(e) = state.release_tokens(estimated_tokens).await {
                tracing::error!(request_id = %trace.request_id, "quota rollback on provider error failed: {}", e);
            }
            Err(response::err(StatusCode::BAD_GATEWAY, "upstream service error"))
        }
    }
}
