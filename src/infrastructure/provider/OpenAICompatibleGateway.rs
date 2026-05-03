use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::gateway_orchestration::CompletionResult::CompletionResult;
use crate::domain::core::quota_billing::StreamingCompletion::StreamingCompletion;
use crate::domain::core::quota_billing::TokenUsage::TokenUsage;

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

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false
        });

        if let Some(v) = req.temperature { body["temperature"] = serde_json::json!(v); }
        if let Some(v) = req.max_tokens { body["max_tokens"] = serde_json::json!(v); }
        if let Some(v) = req.top_p { body["top_p"] = serde_json::json!(v); }
        if let Some(v) = req.frequency_penalty { body["frequency_penalty"] = serde_json::json!(v); }
        if let Some(v) = req.presence_penalty { body["presence_penalty"] = serde_json::json!(v); }
        if let Some(ref v) = req.tools { body["tools"] = v.clone(); }
        if let Some(ref v) = req.response_format { body["response_format"] = v.clone(); }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
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

        let usage = body.get("usage");
        let prompt_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(Value::as_i64);
        let completion_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(Value::as_i64);
        let total_tokens = usage.and_then(|u| u.get("total_tokens")).and_then(Value::as_i64);
        let finish_reason = body
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(Value::as_str)
            .map(|s| s.to_string());

        Ok(CompletionResult {
            model: result_model,
            content,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            finish_reason,
        })
    }

    async fn stream_complete(&self, req: CompletionRequest) -> anyhow::Result<StreamingCompletion> {
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

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "stream_options": {"include_usage": true}
        });

        if let Some(v) = req.temperature { body["temperature"] = serde_json::json!(v); }
        if let Some(v) = req.max_tokens { body["max_tokens"] = serde_json::json!(v); }
        if let Some(v) = req.top_p { body["top_p"] = serde_json::json!(v); }
        if let Some(v) = req.frequency_penalty { body["frequency_penalty"] = serde_json::json!(v); }
        if let Some(v) = req.presence_penalty { body["presence_penalty"] = serde_json::json!(v); }
        if let Some(ref v) = req.tools { body["tools"] = v.clone(); }
        if let Some(ref v) = req.response_format { body["response_format"] = v.clone(); }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("provider stream failed: status={}, body={}", status, body);
        }

        let mut upstream = response.bytes_stream();
        let (tx, rx) = mpsc::channel::<anyhow::Result<Value>>(128);
        let (usage_tx, usage_rx) = oneshot::channel::<Option<TokenUsage>>();

        let chunk_timeout = Duration::from_secs(30);
        let max_consecutive_timeouts = 3;

        tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut buf: Vec<u8> = Vec::new();
            let mut last_usage: Option<TokenUsage> = None;
            let mut consecutive_timeouts = 0u32;

            loop {
                match tokio::time::timeout(chunk_timeout, upstream.next()).await {
                    Ok(Some(item)) => {
                        consecutive_timeouts = 0;
                        match item {
                            Ok(bytes) => {
                                buf.extend_from_slice(&bytes);
                                while let Some(idx) = buf.iter().position(|&b| b == b'\n') {
                                    let line_bytes = buf[..idx].to_vec();
                                    buf.drain(..idx + 1);

                                    let line = match std::str::from_utf8(&line_bytes) {
                                        Ok(s) => s.trim().to_string(),
                                        Err(_) => continue,
                                    };

                                    if !line.starts_with("data:") {
                                        continue;
                                    }
                                    let payload = line.trim_start_matches("data:").trim();
                                    if payload == "[DONE]" {
                                        break;
                                    }
                                    match serde_json::from_str::<Value>(payload) {
                                        Ok(node) => {
                                            if let Some(usage_obj) = node.get("usage") {
                                                let prompt_tokens = usage_obj.get("prompt_tokens").and_then(Value::as_i64).unwrap_or(0);
                                                let completion_tokens = usage_obj.get("completion_tokens").and_then(Value::as_i64).unwrap_or(0);
                                                let total_tokens = usage_obj.get("total_tokens").and_then(Value::as_i64).unwrap_or(0);
                                                let stream_model = node.get("model").and_then(Value::as_str).unwrap_or("").to_string();
                                                last_usage = Some(TokenUsage {
                                                    request_id: String::new(),
                                                    tenant_id: String::new(),
                                                    app_id: String::new(),
                                                    model: stream_model,
                                                    prompt_tokens,
                                                    completion_tokens,
                                                    total_tokens,
                                                    created_at: chrono::Utc::now(),
                                                });
                                                continue;
                                            }
                                            if tx.send(Ok(node)).await.is_err() {
                                                let _ = usage_tx.send(last_usage);
                                                return;
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!("failed to parse SSE payload: {}, raw: {}", e, payload);
                                            continue;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow::anyhow!(e))).await;
                                let _ = usage_tx.send(last_usage);
                                return;
                            }
                        }
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        consecutive_timeouts += 1;
                        tracing::warn!(
                            consecutive_timeouts,
                            max_consecutive_timeouts,
                            "upstream SSE chunk timeout"
                        );
                        if consecutive_timeouts >= max_consecutive_timeouts {
                            tracing::error!("upstream SSE too many consecutive timeouts, aborting");
                            let _ = tx.send(Err(anyhow::anyhow!("upstream stream timeout"))).await;
                            let _ = usage_tx.send(last_usage);
                            return;
                        }
                        if tx.send(Ok(serde_json::json!({"type": "keepalive"}))).await.is_err() {
                            let _ = usage_tx.send(last_usage);
                            return;
                        }
                    }
                }
            }
            let _ = usage_tx.send(last_usage);
        });

        Ok(StreamingCompletion {
            stream: Box::pin(ReceiverStream::new(rx)),
            usage_rx,
        })
    }
}
