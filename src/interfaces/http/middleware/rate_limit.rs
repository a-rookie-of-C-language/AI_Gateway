use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header::HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;
use crate::shared::response;

const MAX_MODEL_PARSE_BODY_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug)]
pub struct ExtractedModel(pub String);

pub async fn rate_limit(
    State(state): State<MiddlewareState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let tenant = req
        .extensions()
        .get::<TenantIdentity>()
        .cloned()
        .unwrap_or(TenantIdentity {
            tenant_id: "unknown-tenant".to_string(),
            app_id: "unknown-app".to_string(),
        });

    let route_path = req.uri().path().to_string();
    let route = route_path.replace('/', "_");

    let mut model = req
        .headers()
        .get("x-model")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if model.is_none() && (route_path == "/v1/chat/completions" || route_path == "/v1/chat/stream") {
        model = extract_model_from_body(&mut req).await;
    }

    let model = model.unwrap_or_else(|| "default".to_string());

    // Cache extracted model in extensions for downstream handlers
    req.extensions_mut().insert(ExtractedModel(model.clone()));

    let tenant_key = format!("rl:{}:{}:tenant", tenant.tenant_id, tenant.app_id);
    let route_key = format!(
        "rl:{}:{}:route:{}:model:{}",
        tenant.tenant_id, tenant.app_id, route, model
    );

    let tenant_decision = state
        .rate_limit_dao
        .evaluate(
            &tenant_key,
            state.rate_limit_tenant_per_min.max(state.rate_limit_per_min),
            state.rate_limit_window_ms,
        )
        .await;

    let route_decision = state
        .rate_limit_dao
        .evaluate(
            &route_key,
            state
                .rate_limit_route_per_min
                .min(state.rate_limit_model_per_min)
                .max(1),
            state.rate_limit_window_ms,
        )
        .await;

    match (tenant_decision, route_decision) {
        (Ok(td), Ok(rd)) => {
            let decision = if !td.allowed { td } else { rd };
            if decision.allowed {
                let mut resp = next.run(req).await;
                append_rate_limit_headers(
                    &mut resp,
                    decision.limit,
                    decision.remaining,
                    decision.reset_after_ms,
                );
                Ok(resp)
            } else {
                let (status, body) =
                    response::err(StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded");
                let mut resp = (status, body).into_response();
                let retry_after_secs = (decision.reset_after_ms + 999) / 1000;
                if let Ok(v) = HeaderValue::from_str(&retry_after_secs.to_string()) {
                    resp.headers_mut().insert("Retry-After", v);
                }
                append_rate_limit_headers(
                    &mut resp,
                    decision.limit,
                    decision.remaining,
                    decision.reset_after_ms,
                );
                Ok(resp)
            }
        }
        (Err(e), _) | (_, Err(e)) => {
            if state.rate_limit_fail_open {
                tracing::warn!("rate limiter unavailable, fail-open: {}", e);
                Ok(next.run(req).await)
            } else {
                tracing::error!("rate limiter unavailable, fail-closed: {}", e);
                Err(response::err(StatusCode::SERVICE_UNAVAILABLE, "rate limiter unavailable"))
            }
        }
    }
}

async fn extract_model_from_body(req: &mut Request<Body>) -> Option<String> {
    let (parts, body) = std::mem::replace(req, Request::new(Body::empty())).into_parts();
    let bytes = match to_bytes(body, MAX_MODEL_PARSE_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => {
            *req = Request::from_parts(parts, Body::empty());
            return None;
        }
    };

    let model = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|v| v.get("model").and_then(Value::as_str).map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty());

    *req = Request::from_parts(parts, Body::from(bytes));
    model
}

fn append_rate_limit_headers(resp: &mut Response, limit: u64, remaining: u64, reset_after_ms: u64) {
    if let Ok(v) = HeaderValue::from_str(&limit.to_string()) {
        resp.headers_mut().insert("X-RateLimit-Limit", v);
    }
    if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
        resp.headers_mut().insert("X-RateLimit-Remaining", v);
    }
    if let Ok(v) = HeaderValue::from_str(&reset_after_ms.to_string()) {
        resp.headers_mut().insert("X-RateLimit-Reset-Ms", v);
    }
}
