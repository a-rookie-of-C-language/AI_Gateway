use async_trait::async_trait;

use crate::domain::chat::CompletionRequest::CompletionRequest;
use crate::domain::chat::CompletionResult::CompletionResult;

#[async_trait]
pub trait ChatGateway: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult>;
}
