use async_trait::async_trait;
use futures_util::stream::BoxStream;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;

pub struct OpenAICompatibleGateway {
    pub client: Client,
    pub base_url: String,
    pub api_key: String,
    pub default_model: String,
}

impl OpenAICompatibleGateway {
    pub fn new(
        base_url: String,
        api_key: String,
        default_model: String,
        timeout_sec: u64,
    ) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_sec))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            default_model,
        })
    }
}

#[async_trait]
impl ChatGateway for OpenAICompatibleGateway {
    async fn complete(&self, req: CompletionRequest) -> anyhow::Result<CompletionResult> {
        let model = req
            .model
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.default_model.clone());

        let messages: Vec<Value> = req
            .messages
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": false
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("provider call failed: status={}, body={}", status, body);
        }

        let body: Value = response.json().await?;
        let result_model = body
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&self.default_model)
            .to_string();
        let content = body
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        Ok(CompletionResult {
            model: result_model,
            content,
        })
    }

    async fn stream_complete(&self, req: CompletionRequest) -> anyhow::Result<BoxStream<'static, anyhow::Result<Value>>> {
        let model = req
            .model
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.default_model.clone());

        let messages: Vec<Value> = req
            .messages
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": true
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("provider stream failed: status={}, body={}", status, body);
        }

        let mut upstream = response.bytes_stream();
        let (tx, rx) = mpsc::channel::<anyhow::Result<Value>>(128);

        tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut buf = String::new();
            while let Some(item) = upstream.next().await {
                match item {
                    Ok(bytes) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        while let Some(idx) = buf.find('\n') {
                            let line = buf[..idx].trim().to_string();
                            buf = buf[idx + 1..].to_string();

                            if !line.starts_with("data:") {
                                continue;
                            }
                            let payload = line.trim_start_matches("data:").trim();
                            if payload == "[DONE]" {
                                continue;
                            }
                            match serde_json::from_str::<Value>(payload) {
                                Ok(node) => {
                                    if tx.send(Ok(node)).await.is_err() {
                                        return;
                                    }
                                }
                                Err(e) => {
                                    if tx.send(Err(anyhow::anyhow!(e))).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!(e))).await;
                        return;
                    }
                }
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}
