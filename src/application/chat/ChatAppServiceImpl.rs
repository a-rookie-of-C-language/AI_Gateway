use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;
use crate::domain::core::quota_billing::StreamingCompletion::StreamingCompletion;
use crate::domain::supporting::provider_integration::ProviderRouter::ProviderRouter;

use super::ChatAppService::ChatAppService;
use super::ChatService::ChatService;

const MAX_RETRIES: u32 = 3;

impl ChatAppService {
    pub fn new(
        gateways: HashMap<String, Arc<dyn ChatGateway>>,
        router: Arc<dyn ProviderRouter>,
    ) -> Self {
        Self { gateways, router }
    }

    fn get_gateway(&self, model: &str) -> Option<&Arc<dyn ChatGateway>> {
        let provider = self.router.route(model).or_else(|| Some(self.router.default_provider()));
        provider.and_then(|p| self.gateways.get(&p.provider_code))
    }

    async fn retry_complete(&self, req: &CompletionRequest) -> anyhow::Result<CompletionResult> {
        let model = req.model.as_deref().unwrap_or("");
        let gateway = match self.get_gateway(model) {
            Some(g) => g,
            None => anyhow::bail!("no provider available for model: {}", model),
        };

        let mut last_err = None;
        for attempt in 0..MAX_RETRIES {
            match gateway.complete(req.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(attempt, error = %e, "provider call failed, retrying");
                    last_err = Some(e);
                    if attempt < MAX_RETRIES - 1 {
                        let backoff = std::time::Duration::from_millis(100 * 2u64.pow(attempt));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("all retries exhausted")))
    }
}

#[async_trait]
impl ChatService for ChatAppService {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        self.retry_complete(&req).await
    }

    async fn stream_complete(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<StreamingCompletion> {
        let model = req.model.as_deref().unwrap_or("");
        let gateway = match self.get_gateway(model) {
            Some(g) => g,
            None => anyhow::bail!("no provider available for model: {}", model),
        };

        gateway.stream_complete(req).await
    }
}
