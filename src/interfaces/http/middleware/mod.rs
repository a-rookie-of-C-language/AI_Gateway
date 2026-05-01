use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{domain::ratelimit::RateLimiter, shared::response};

#[derive(Clone)]
pub struct MiddlewareState {
    pub master_api_key: String,
    pub rate_limit_per_min: u64,
    pub limiter: Arc<dyn RateLimiter>,
}

pub async fn request_id(mut req: Request<Body>, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        req.headers_mut().insert("x-request-id", value);
    }

    let mut resp = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        resp.headers_mut().insert("x-request-id", value);
    }
    resp
}

pub async fn auth(
    State(state): State<MiddlewareState>,
    req: Request<Body>,
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

    Ok(next.run(req).await)
}

pub async fn rate_limit(
    State(state): State<MiddlewareState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anon");

    let key = format!("rl:{}", auth_header);
    match state.limiter.allow(&key, state.rate_limit_per_min).await {
        Ok(true) => Ok(next.run(req).await),
        Ok(false) => Err(response::err(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
        )),
        Err(_) => Err(response::err(
            StatusCode::SERVICE_UNAVAILABLE,
            "rate limiter unavailable",
        )),
    }
}

pub fn _sample() -> Json<serde_json::Value> {
    Json(json!({"ok":true}))
}
