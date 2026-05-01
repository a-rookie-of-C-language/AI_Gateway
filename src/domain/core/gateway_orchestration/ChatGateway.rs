use async_trait::async_trait;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;

#[async_trait]
pub trait ChatGateway: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult>;
}
