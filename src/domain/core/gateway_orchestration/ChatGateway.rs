use async_trait::async_trait;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;
use crate::domain::core::quota_billing::StreamingCompletion::StreamingCompletion;

#[async_trait]
pub trait ChatGateway: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult>;
    async fn stream_complete(&self, req: CompletionRequest) -> anyhow::Result<StreamingCompletion>;
}
