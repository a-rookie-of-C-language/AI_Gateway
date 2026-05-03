use serde::{Deserialize, Serialize};

use crate::domain::core::gateway_orchestration::Message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub frequency_penalty: Option<f64>,
    #[serde(default)]
    pub presence_penalty: Option<f64>,
    #[serde(default)]
    pub tools: Option<serde_json::Value>,
    #[serde(default)]
    pub response_format: Option<serde_json::Value>,
}
