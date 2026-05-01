use std::convert::Infallible;

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, Sse},
    Json,
};
use tokio_stream::iter;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::domain::supporting::observability_audit::TraceRecord::TraceRecord;
use crate::infrastructure::http::AppState::AppState;
use crate::shared::response;

pub async fn chat_stream(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantIdentity>,
    headers: HeaderMap,
    Json(payload): Json<CompletionRequest>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<serde_json::Value>)> {
    let estimated_tokens: u64 = payload
        .messages
        .iter()
        .map(|m| m.content.len() as u64)
        .sum();

    if !state.try_consume_tokens(estimated_tokens) {
        return Err(response::err(StatusCode::PAYMENT_REQUIRED, "quota exceeded"));
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

    let events = match state.chat_service.stream_complete(payload).await {
        Ok(chunks) => {
            let mut evs: Vec<Result<Event, Infallible>> = chunks
                .into_iter()
                .map(|c| Ok(Event::default().event("delta").data(serde_json::json!({"text": c}).to_string())))
                .collect();
            evs.push(Ok(Event::default().event("done").data("{\"finish_reason\":\"stop\"}")));
            evs
        }
        Err(err) => {
            vec![
                Ok(Event::default().event("error").data(serde_json::json!({"message": err.to_string()}).to_string())),
                Ok(Event::default().event("done").data("{\"finish_reason\":\"error\"}")),
            ]
        }
    };

    Ok(Sse::new(iter(events)))
}
