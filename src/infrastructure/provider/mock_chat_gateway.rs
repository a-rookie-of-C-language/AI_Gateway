use async_trait::async_trait;

use crate::domain::chat::{ChatGateway, CompletionRequest, CompletionResult};

pub struct MockChatGateway;

#[async_trait]
impl ChatGateway for MockChatGateway {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        Ok(CompletionResult {
            model: req.model.unwrap_or_else(|| "mock-model".to_string()),
            content: "AIGateway mock response from Rust".to_string(),
        })
    }
}
