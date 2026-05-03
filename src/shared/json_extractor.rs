use axum::{
    extract::FromRequest,
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;

use crate::shared::response;

/// Custom JSON extractor that returns unified error format.
pub struct UnifiedJson<T>(pub T);

impl<T> UnifiedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[axum::async_trait]
impl<S, T> FromRequest<S, axum::body::Body> for UnifiedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(
        req: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                let message = format!("invalid request body: {}", rejection.body_text());
                response::err(StatusCode::BAD_REQUEST, &message)
            })?;
        Ok(UnifiedJson(value))
    }
}
