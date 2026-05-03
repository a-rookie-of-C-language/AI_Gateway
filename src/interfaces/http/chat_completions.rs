use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::json;

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::domain::core::quota_billing::TokenUsage::TokenUsage;
use crate::domain::core::tenant_access_control::TenantIdentity::TenantIdentity;
use crate::domain::supporting::observability_audit::TraceRecord::TraceRecord;
use crate::infrastructure::http::AppState::AppState;
use crate::shared::response;

const MAX_MESSAGES: usize = 128;
const MAX_MESSAGE_CONTENT_LEN: usize = 128 * 1024;
const VALID_ROLES: &[&str] = &["system", "user", "assistant", "tool"];

fn validate_request(payload: &CompletionRequest) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if payload.messages.is_empty() {
        return Err(response::err(StatusCode::BAD_REQUEST, "messages must not be empty"));
    }
    if payload.messages.len() > MAX_MESSAGES {
        return Err(response::err(
            StatusCode::BAD_REQUEST,
            &format!("messages count exceeds limit of {}", MAX_MESSAGES),
        ));
    }
    for (i, msg) in payload.messages.iter().enumerate() {
        if !VALID_ROLES.contains(&msg.role.as_str()) {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("invalid role '{}' at message index {}", msg.role, i),
            ));
        }
        if msg.content.len() > MAX_MESSAGE_CONTENT_LEN {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("message content exceeds {} bytes at index {}", MAX_MESSAGE_CONTENT_LEN, i),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::core::gateway_orchestration::Message::Message;

    fn make_request(messages: Vec<Message>) -> CompletionRequest {
        CompletionRequest {
            model: Some("test".to_string()),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            tools: None,
            response_format: None,
        }
    }

    #[test]
    fn test_valid_request() {
        let req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_empty_messages() {
        let req = make_request(vec![]);
        let result = validate_request(&req);
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_too_many_messages() {
        let messages: Vec<Message> = (0..129)
            .map(|_| Message {
                role: "user".to_string(),
                content: "test".to_string(),
            })
            .collect();
        let req = make_request(messages);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_exactly_max_messages() {
        let messages: Vec<Message> = (0..128)
            .map(|_| Message {
                role: "user".to_string(),
                content: "test".to_string(),
            })
            .collect();
        let req = make_request(messages);
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_invalid_role() {
        let req = make_request(vec![Message {
            role: "admin".to_string(),
            content: "hello".to_string(),
        }]);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_roles() {
        for role in &["system", "user", "assistant", "tool"] {
            let req = make_request(vec![Message {
                role: role.to_string(),
                content: "hello".to_string(),
            }]);
            assert!(validate_request(&req).is_ok(), "role {} should be valid", role);
        }
    }

    #[test]
    fn test_content_too_large() {
        let large_content = "x".repeat(128 * 1024 + 1);
        let req = make_request(vec![Message {
            role: "user".to_string(),
            content: large_content,
        }]);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_content_exactly_max() {
        let max_content = "x".repeat(128 * 1024);
        let req = make_request(vec![Message {
            role: "user".to_string(),
            content: max_content,
        }]);
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_multiple_messages_valid() {
        let req = make_request(vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ]);
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_second_message_invalid_role() {
        let req = make_request(vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            Message {
                role: "invalid".to_string(),
                content: "World".to_string(),
            },
        ]);
        let result = validate_request(&req);
        assert!(result.is_err());
    }
}

pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantIdentity>,
    headers: HeaderMap,
    Json(payload): Json<CompletionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    validate_request(&payload)?;

    let estimated_tokens: u64 = payload
        .messages
        .iter()
        .map(|m| (m.content.chars().count() as u64 + 2) / 3)
        .sum();

    match state.try_consume_tokens(estimated_tokens, &tenant.tenant_id, &tenant.app_id).await {
        Ok(true) => {}
        Ok(false) => return Err(response::err(StatusCode::PAYMENT_REQUIRED, "quota exceeded")),
        Err(e) => {
            tracing::error!("quota check failed: {}", e);
            return Err(response::err(StatusCode::INTERNAL_SERVER_ERROR, "quota service unavailable"));
        }
    }

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    let trace = TraceRecord {
        request_id,
        provider: "openai-compatible".to_string(),
        span_id: None,
    };

    tracing::info!(
        tenant_id = %tenant.tenant_id,
        app_id = %tenant.app_id,
        request_id = %trace.request_id,
        provider = %trace.provider,
        estimated_tokens = estimated_tokens,
        "chat completion request"
    );

    match state.chat_service.complete(payload).await {
        Ok(data) => {
            if let Some(tt) = data.total_tokens {
                let actual = tt as u64;
                if actual > estimated_tokens {
                    if let Err(e) = state.try_consume_tokens(actual - estimated_tokens, &tenant.tenant_id, &tenant.app_id).await {
                        tracing::warn!(request_id = %trace.request_id, "quota top-up failed: {}", e);
                    }
                } else if actual < estimated_tokens {
                    if let Err(e) = state.release_tokens(estimated_tokens - actual, &tenant.tenant_id, &tenant.app_id).await {
                        tracing::warn!(request_id = %trace.request_id, "quota rollback failed: {}", e);
                    }
                }
            }
            if let (Some(pt), Some(ct), Some(tt)) = (data.prompt_tokens, data.completion_tokens, data.total_tokens) {
                if let Some(ref dao) = state.token_usage_dao {
                    let usage = TokenUsage {
                        request_id: trace.request_id.clone(),
                        tenant_id: tenant.tenant_id.clone(),
                        app_id: tenant.app_id.clone(),
                        model: data.model.clone(),
                        prompt_tokens: pt,
                        completion_tokens: ct,
                        total_tokens: tt,
                        created_at: chrono::Utc::now(),
                    };
                    if let Err(e) = dao.insert(&usage).await {
                        tracing::error!(request_id = %trace.request_id, "failed to persist token usage: {}", e);
                        if let Err(rollback_err) = state.release_tokens(estimated_tokens, &tenant.tenant_id, &tenant.app_id).await {
                            tracing::error!(request_id = %trace.request_id, "quota rollback failed: {}", rollback_err);
                        }
                    }
                }
            }
            Ok(response::ok(json!(data)))
        }
        Err(err) => {
            tracing::error!(request_id = %trace.request_id, "provider error: {:?}", err);
            if let Err(e) = state.release_tokens(estimated_tokens, &tenant.tenant_id, &tenant.app_id).await {
                tracing::error!(request_id = %trace.request_id, "quota rollback on provider error failed: {}", e);
            }
            Err(response::err(StatusCode::BAD_GATEWAY, "upstream service error"))
        }
    }
}
