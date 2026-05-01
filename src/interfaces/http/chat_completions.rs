use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

use crate::domain::chat::CompletionRequest::CompletionRequest;
use crate::infrastructure::http::AppState::AppState;
use crate::shared::response;

pub async fn chat_completions(
    State(state): State<AppState>,
    Json(req): Json<CompletionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chat_service.complete(req).await {
        Ok(data) => Ok(response::ok(json!(data))),
        Err(err) => Err(response::err(StatusCode::BAD_GATEWAY, &err.to_string())),
    }
}
