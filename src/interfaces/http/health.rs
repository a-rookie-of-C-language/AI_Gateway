use axum::Json;
use serde_json::json;

use crate::shared::response;

pub async fn health() -> Json<serde_json::Value> {
    response::ok(json!({"status":"ok"}))
}
