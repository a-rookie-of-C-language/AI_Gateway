use axum::http::StatusCode;
use axum::Json;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::infrastructure::http::AppState::AppState;
use crate::shared::response;
use crate::shared::token_estimator::estimate_tokens;

pub fn estimate_request_tokens(payload: &CompletionRequest) -> u64 {
    payload
        .messages
        .iter()
        .map(|m| estimate_tokens(&m.content) + 4)
        .sum()
}

pub async fn check_and_consume_tokens(
    state: &AppState,
    estimated_tokens: u64,
    tenant: &TenantIdentity,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state.try_consume_tokens(estimated_tokens, &tenant.tenant_id, &tenant.app_id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(response::err(StatusCode::PAYMENT_REQUIRED, "quota exceeded")),
        Err(e) => {
            tracing::error!("quota check failed: {}", e);
            Err(response::err(StatusCode::INTERNAL_SERVER_ERROR, "quota service unavailable"))
        }
    }
}
