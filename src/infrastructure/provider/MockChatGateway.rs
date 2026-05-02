use async_trait::async_trait;
use futures_util::stream::{self};
use serde_json::json;
use tokio::sync::oneshot;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;
use crate::domain::core::quota_billing::StreamingCompletion::StreamingCompletion;

pub struct MockChatGateway;

#[async_trait]
impl ChatGateway for MockChatGateway {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        Ok(CompletionResult {
            model: req.model.unwrap_or_else(|| "mock-model".to_string()),
            content: "AIGateway mock response from Rust".to_string(),
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn stream_complete(&self, _req: CompletionRequest) -> anyhow::Result<StreamingCompletion> {
        let items = vec![
            Ok(json!({"type":"delta","text":"AIGateway "})),
            Ok(json!({"type":"delta","text":"mock "})),
            Ok(json!({"type":"delta","text":"stream "})),
            Ok(json!({"type":"delta","text":"response"})),
        ];
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(None);
        Ok(StreamingCompletion {
            stream: Box::pin(stream::iter(items)),
            usage_rx: rx,
        })
    }
}
