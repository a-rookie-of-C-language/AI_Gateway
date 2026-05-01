use serde::{Deserialize, Serialize};

use crate::domain::core::gateway_orchestration::Message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
}
