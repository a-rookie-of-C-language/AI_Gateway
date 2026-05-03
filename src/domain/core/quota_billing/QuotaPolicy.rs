use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub plan_code: String,
    pub max_tokens_per_day: u64,
    pub max_tokens_per_request: Option<u64>,
    pub rate_limit_per_min: Option<u64>,
}

impl Default for QuotaPolicy {
    fn default() -> Self {
        Self {
            plan_code: "default".to_string(),
            max_tokens_per_day: 1_000_000,
            max_tokens_per_request: None,
            rate_limit_per_min: None,
        }
    }
}
