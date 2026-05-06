use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use subtle::ConstantTimeEq;

use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;
use crate::shared::response;

fn extract_api_key(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ").map(|s| s.trim())
}

fn parse_tenant_api_key(api_key: &str) -> Option<(&str, &str)> {
    let body = api_key.strip_prefix("ak_")?;
    let (key_id, secret) = body.split_once('.')?;
    if key_id.is_empty() || secret.is_empty() {
        return None;
    }
    Some((key_id, secret))
}

fn verify_secret(secret: &str, encoded_hash: &str) -> bool {
    let parsed = match PasswordHash::new(encoded_hash) {
        Ok(v) => v,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed)
        .is_ok()
}

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

    let api_key = match extract_api_key(auth_header) {
        Some(key) if !key.is_empty() => key,
        _ => {
            tracing::warn!(
                remote_addr = ?req.headers().get("x-forwarded-for").or(req.headers().get("x-real-ip")),
                path = %req.uri().path(),
                "missing or invalid authorization header"
            );
            return Err(response::err(StatusCode::UNAUTHORIZED, "missing api key"));
        }
    };

    let mut tenant_id = String::new();
    let mut app_id = String::new();
    let mut authenticated = false;

    if let Some(ref tenant_dao) = state.tenant_dao {
        let tenant_key = parse_tenant_api_key(api_key);
        match tenant_key {
            Some((key_id, secret)) => match tenant_dao.find_by_api_key_id(key_id).await {
            Ok(Some(tenant)) => {
                if verify_secret(secret, &tenant.api_key_hash) {
                    tenant_id = tenant.tenant_id;
                    app_id = tenant.app_id;
                    authenticated = true;
                    if let Err(e) = tenant_dao.update_last_login(&tenant_id, &app_id).await {
                        tracing::warn!("failed to update tenant last login: {}", e);
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!("tenant dao error: {}", e);
            }
            },
            None => {}
        };
    }

    if !authenticated {
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

        tenant_id = req
            .headers()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default-tenant".to_string());

        app_id = req
            .headers()
            .get("x-app-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default-app".to_string());
    }

    req.extensions_mut().insert(TenantIdentity { tenant_id, app_id });

    Ok(next.run(req).await)
}
