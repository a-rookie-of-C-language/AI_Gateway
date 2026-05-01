use axum::{
    body::Body,
    http::{header::HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

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
