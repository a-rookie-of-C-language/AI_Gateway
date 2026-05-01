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
}
