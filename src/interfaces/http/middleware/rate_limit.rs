use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;
use crate::shared::response;

pub async fn rate_limit(
    State(state): State<MiddlewareState>,
    req: Request<Body>,
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

    let route = req.uri().path().replace('/', "_");
    let model = req
        .headers()
        .get("x-model")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("default");

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
                append_rate_limit_headers(
                    &mut resp,
                    decision.limit,
                    decision.remaining,
                    decision.reset_after_ms,
                );
                Ok(resp)
            }
        }
        _ => Err(response::err(
            StatusCode::SERVICE_UNAVAILABLE,
            "rate limiter unavailable",
        )),
    }
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
