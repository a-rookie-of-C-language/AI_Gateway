use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsage {
    pub request_id: String,
    pub tenant_id: String,
    pub app_id: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
