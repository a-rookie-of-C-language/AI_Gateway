use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub tenant_id: Option<String>,
    pub app_id: Option<String>,
    pub model: Option<String>,
    pub period_start: Option<chrono::DateTime<chrono::Utc>>,
    pub request_count: i64,
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_tokens: i64,
}
