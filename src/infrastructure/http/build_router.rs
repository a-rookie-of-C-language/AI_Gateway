use axum::{
    routing::{get, post},
    Router,
};

use crate::infrastructure::http::AppState::AppState;
use crate::interfaces::http::{chat_completions::chat_completions, health::health};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}
