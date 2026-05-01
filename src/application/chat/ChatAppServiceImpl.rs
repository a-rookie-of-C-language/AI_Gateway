use futures_util::stream::BoxStream;
use serde_json::Value;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;

use super::ChatAppService::ChatAppService;

impl ChatAppService {
    pub fn new(gateway: std::sync::Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        self.gateway.complete(req).await
    }

    pub async fn stream_complete(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, anyhow::Result<Value>>> {
        self.gateway.stream_complete(req).await
    }
}
