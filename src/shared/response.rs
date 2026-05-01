use axum::{http::StatusCode, Json};
use serde_json::json;

pub fn ok(data: serde_json::Value) -> Json<serde_json::Value> {
    Json(json!({"code": 200, "data": data}))
}

pub fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(json!({"code": status.as_u16(), "error": msg})))
}
