use axum::{
    http::StatusCode,
    Json,
};

use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::shared::response;

const MAX_MESSAGES: usize = 128;
const MAX_MESSAGE_CONTENT_LEN: usize = 128 * 1024;
const VALID_ROLES: &[&str] = &["system", "user", "assistant", "tool"];

pub fn validate_request(payload: &CompletionRequest) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
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
