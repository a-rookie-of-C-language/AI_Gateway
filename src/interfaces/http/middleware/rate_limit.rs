use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;
use crate::shared::response;

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
    match state
        .rate_limit_dao
        .evaluate(&key, state.rate_limit_per_min, state.rate_limit_window_ms)
        .await
    {
        Ok(decision) => {
            if decision.allowed {
                let mut resp = next.run(req).await;
                append_rate_limit_headers(&mut resp, decision.limit, decision.remaining, decision.reset_after_ms);
                Ok(resp)
            } else {
                let (status, body) = response::err(StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded");
                let mut resp = (status, body).into_response();
                append_rate_limit_headers(&mut resp, decision.limit, decision.remaining, decision.reset_after_ms);
                Ok(resp)
            }
        }
        Err(_) => Err(response::err(
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
