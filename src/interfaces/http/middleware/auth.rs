use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use subtle::ConstantTimeEq;

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
    let auth_ok: bool = auth_header.as_bytes().ct_eq(expected.as_bytes()).into();

    if !auth_ok {
        tracing::warn!(
            remote_addr = ?req.headers().get("x-forwarded-for").or(req.headers().get("x-real-ip")),
            path = %req.uri().path(),
            "authentication failed"
        );
        return Err(response::err(StatusCode::UNAUTHORIZED, "invalid api key"));
    }

    let tenant_id = req
        .headers()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default-tenant".to_string());

    let app_id = req
        .headers()
        .get("x-app-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default-app".to_string());

    req.extensions_mut().insert(TenantIdentity { tenant_id, app_id });

    Ok(next.run(req).await)
}
