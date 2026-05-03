use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::infrastructure::http::AppState::AppState;
use crate::shared::response;

pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let redis_ok = state.check_redis().await;
    let pg_ok = state.check_postgres().await;
    let provider_ok = state.check_provider().await;

    let mut components = serde_json::Map::new();
    components.insert("redis".to_string(), json!(if redis_ok { "ok" } else { "error" }));
    match pg_ok {
        Some(ok) => { components.insert("postgres".to_string(), json!(if ok { "ok" } else { "error" })); }
        None => { components.insert("postgres".to_string(), json!("not_configured")); }
    }
    components.insert("provider".to_string(), json!(if provider_ok { "ok" } else { "error" }));

    let all_ok = redis_ok && pg_ok.unwrap_or(true) && provider_ok;
    let status = if all_ok { "ok" } else { "degraded" };
    let code = if all_ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (code, response::ok(json!({"status": status, "components": components})))
}
