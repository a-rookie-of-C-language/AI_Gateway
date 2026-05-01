use async_trait::async_trait;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;

pub struct MockChatGateway;

#[async_trait]
impl ChatGateway for MockChatGateway {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        Ok(CompletionResult {
            model: req.model.unwrap_or_else(|| "mock-model".to_string()),
            content: "AIGateway mock response from Rust".to_string(),
        })
    }

    async fn stream_complete(&self, _req: CompletionRequest) -> anyhow::Result<Vec<String>> {
        Ok(vec![
            "AIGateway ".to_string(),
            "mock ".to_string(),
            "stream ".to_string(),
            "response".to_string(),
        ])
    }
}
