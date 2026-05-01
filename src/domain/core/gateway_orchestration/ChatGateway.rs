use async_trait::async_trait;
use futures_util::stream::BoxStream;
use serde_json::Value;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;

#[async_trait]
pub trait ChatGateway: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult>;
    async fn stream_complete(&self, req: CompletionRequest) -> anyhow::Result<BoxStream<'static, anyhow::Result<Value>>>;
}
