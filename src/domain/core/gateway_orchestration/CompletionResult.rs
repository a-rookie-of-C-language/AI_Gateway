use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CompletionResult {
    pub model: String,
    pub content: String,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub finish_reason: Option<String>,
}
