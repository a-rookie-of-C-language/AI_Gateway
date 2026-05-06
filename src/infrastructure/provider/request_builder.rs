use serde_json::Value;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;

pub fn build_request_body(req: &CompletionRequest, default_model: &str, stream: bool) -> Value {
    let model = req
        .model
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_model.to_string());

    let messages: Vec<Value> = req
        .messages
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": stream
    });

    if stream {
        body["stream_options"] = serde_json::json!({"include_usage": true});
    }

    if let Some(v) = req.temperature { body["temperature"] = serde_json::json!(v); }
    if let Some(v) = req.max_tokens { body["max_tokens"] = serde_json::json!(v); }
    if let Some(v) = req.top_p { body["top_p"] = serde_json::json!(v); }
    if let Some(v) = req.frequency_penalty { body["frequency_penalty"] = serde_json::json!(v); }
    if let Some(v) = req.presence_penalty { body["presence_penalty"] = serde_json::json!(v); }
    if let Some(ref v) = req.tools { body["tools"] = v.clone(); }
    if let Some(ref v) = req.response_format { body["response_format"] = v.clone(); }

    body
}
