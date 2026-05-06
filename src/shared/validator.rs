use axum::{
    http::StatusCode,
    Json,
};

use crate::constants::*;
use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
use crate::shared::response;

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

    if let Some(temp) = payload.temperature {
        if temp < TEMPERATURE_MIN || temp > TEMPERATURE_MAX {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("temperature must be between {} and {}", TEMPERATURE_MIN, TEMPERATURE_MAX),
            ));
        }
    }

    if let Some(top_p) = payload.top_p {
        if top_p < TOP_P_MIN || top_p > TOP_P_MAX {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("top_p must be between {} and {}", TOP_P_MIN, TOP_P_MAX),
            ));
        }
    }

    if let Some(max_tokens) = payload.max_tokens {
        if max_tokens == 0 || max_tokens > MAX_TOKENS_PER_REQUEST {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("max_tokens must be between 1 and {}", MAX_TOKENS_PER_REQUEST),
            ));
        }
    }

    if let Some(fp) = payload.frequency_penalty {
        if fp < FREQUENCY_PENALTY_MIN || fp > FREQUENCY_PENALTY_MAX {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("frequency_penalty must be between {} and {}", FREQUENCY_PENALTY_MIN, FREQUENCY_PENALTY_MAX),
            ));
        }
    }

    if let Some(pp) = payload.presence_penalty {
        if pp < PRESENCE_PENALTY_MIN || pp > PRESENCE_PENALTY_MAX {
            return Err(response::err(
                StatusCode::BAD_REQUEST,
                &format!("presence_penalty must be between {} and {}", PRESENCE_PENALTY_MIN, PRESENCE_PENALTY_MAX),
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

    #[test]
    fn test_temperature_out_of_range() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.temperature = Some(3.0);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_temperature_valid() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.temperature = Some(1.5);
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_top_p_out_of_range() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.top_p = Some(1.5);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_tokens_zero() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.max_tokens = Some(0);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_tokens_too_large() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.max_tokens = Some(MAX_TOKENS_PER_REQUEST + 1);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_frequency_penalty_out_of_range() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.frequency_penalty = Some(-3.0);
        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_presence_penalty_out_of_range() {
        let mut req = make_request(vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }]);
        req.presence_penalty = Some(3.0);
        let result = validate_request(&req);
        assert!(result.is_err());
    }
}
