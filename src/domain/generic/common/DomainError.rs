use std::fmt;

#[derive(Debug, Clone)]
pub struct DomainError {
    pub code: String,
    pub message: String,
}

impl DomainError {
    pub fn not_found(message: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: message.to_string(),
        }
    }

    pub fn validation(message: &str) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: message.to_string(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.to_string(),
        }
    }

    pub fn quota_exceeded() -> Self {
        Self {
            code: "QUOTA_EXCEEDED".to_string(),
            message: "daily token quota exceeded".to_string(),
        }
    }

    pub fn upstream(message: &str) -> Self {
        Self {
            code: "UPSTREAM_ERROR".to_string(),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

impl From<DomainError> for (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    fn from(err: DomainError) -> Self {
        use axum::http::StatusCode;
        let status = match err.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
            "QUOTA_EXCEEDED" => StatusCode::PAYMENT_REQUIRED,
            "UPSTREAM_ERROR" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status,
            axum::Json(serde_json::json!({
                "error": {
                    "code": err.code,
                    "message": err.message
                }
            })),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found() {
        let err = DomainError::not_found("user not found");
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "user not found");
    }

    #[test]
    fn test_validation() {
        let err = DomainError::validation("invalid input");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.message, "invalid input");
    }

    #[test]
    fn test_internal() {
        let err = DomainError::internal("something went wrong");
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_quota_exceeded() {
        let err = DomainError::quota_exceeded();
        assert_eq!(err.code, "QUOTA_EXCEEDED");
    }

    #[test]
    fn test_upstream() {
        let err = DomainError::upstream("provider timeout");
        assert_eq!(err.code, "UPSTREAM_ERROR");
    }

    #[test]
    fn test_display() {
        let err = DomainError::not_found("resource");
        assert_eq!(format!("{}", err), "[NOT_FOUND] resource");
    }

    #[test]
    fn test_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(DomainError::internal("test"));
        assert!(err.source().is_none());
    }

    #[test]
    fn test_into_axum_response() {
        let err = DomainError::validation("bad request");
        let (status, json) = err.into();
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(json.0["error"]["code"], "VALIDATION_ERROR");
    }

    #[test]
    fn test_not_found_status() {
        let err = DomainError::not_found("missing");
        let (status, _) = err.into();
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_quota_exceeded_status() {
        let err = DomainError::quota_exceeded();
        let (status, _) = err.into();
        assert_eq!(status, axum::http::StatusCode::PAYMENT_REQUIRED);
    }

    #[test]
    fn test_upstream_status() {
        let err = DomainError::upstream("error");
        let (status, _) = err.into();
        assert_eq!(status, axum::http::StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_internal_status() {
        let err = DomainError::internal("error");
        let (status, _) = err.into();
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_unknown_code_status() {
        let err = DomainError {
            code: "UNKNOWN".to_string(),
            message: "unknown error".to_string(),
        };
        let (status, _) = err.into();
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}
