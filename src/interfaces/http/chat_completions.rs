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

pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantIdentity>,
    headers: HeaderMap,
    Json(payload): Json<CompletionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let estimated_tokens: u64 = payload
        .messages
        .iter()
        .map(|m| m.content.len() as u64)
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
                    }
                }
            }
            Ok(response::ok(json!(data)))
        }
        Err(err) => Err(response::err(StatusCode::BAD_GATEWAY, &err.to_string())),
    }
}
