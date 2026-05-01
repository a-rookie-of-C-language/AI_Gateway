use serde::{Deserialize, Serialize};

use crate::domain::chat::Message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
}
