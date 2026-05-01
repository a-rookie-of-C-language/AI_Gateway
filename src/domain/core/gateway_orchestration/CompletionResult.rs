use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CompletionResult {
    pub model: String,
    pub content: String,
}
