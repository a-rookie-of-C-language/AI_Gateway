use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
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
    match state.rate_limit_dao.allow(&key, state.rate_limit_per_min).await {
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
