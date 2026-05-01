use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};

use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;
use crate::shared::response;

pub async fn auth(
    State(state): State<MiddlewareState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let expected = format!("Bearer {}", state.master_api_key);
    if auth_header != expected {
        return Err(response::err(StatusCode::UNAUTHORIZED, "invalid api key"));
    }

    req.extensions_mut().insert(TenantIdentity {
        tenant_id: "default-tenant".to_string(),
        app_id: "default-app".to_string(),
    });

    Ok(next.run(req).await)
}
