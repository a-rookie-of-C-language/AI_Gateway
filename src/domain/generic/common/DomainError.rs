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
