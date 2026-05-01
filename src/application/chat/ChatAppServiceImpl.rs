use crate::domain::chat::CompletionRequest::CompletionRequest;
use crate::domain::chat::CompletionResult::CompletionResult;

use super::ChatAppService::ChatAppService;

impl ChatAppService {
    pub fn new(gateway: std::sync::Arc<dyn crate::domain::chat::ChatGateway::ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        self.gateway.complete(req).await
    }
}
