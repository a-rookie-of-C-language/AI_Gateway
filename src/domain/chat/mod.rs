use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionResult {
    pub model: String,
    pub content: String,
}

#[async_trait]
pub trait ChatGateway: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult>;
}
