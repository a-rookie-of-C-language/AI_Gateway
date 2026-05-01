use std::sync::Arc;

use crate::application::chat::ChatAppService::ChatAppService;

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatAppService>,
}
