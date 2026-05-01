use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{application::chat::ChatAppService, interfaces::http};

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatAppService>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(http::health))
        .route("/v1/chat/completions", post(http::chat_completions))
        .with_state(state)
}
