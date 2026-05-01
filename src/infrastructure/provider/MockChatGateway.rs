use async_trait::async_trait;
use futures_util::stream::{self, BoxStream};
use serde_json::json;

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

    async fn stream_complete(&self, _req: CompletionRequest) -> anyhow::Result<BoxStream<'static, anyhow::Result<serde_json::Value>>> {
        let items = vec![
            Ok(json!({"type":"delta","text":"AIGateway "})),
            Ok(json!({"type":"delta","text":"mock "})),
            Ok(json!({"type":"delta","text":"stream "})),
            Ok(json!({"type":"delta","text":"response"})),
        ];
        Ok(Box::pin(stream::iter(items)))
    }
}
