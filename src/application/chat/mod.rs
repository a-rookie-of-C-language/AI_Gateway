use std::sync::Arc;

use crate::domain::chat::{ChatGateway, CompletionRequest, CompletionResult};

#[derive(Clone)]
pub struct ChatAppService {
    gateway: Arc<dyn ChatGateway>,
}

impl ChatAppService {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        self.gateway.complete(req).await
    }
}
