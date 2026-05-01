use async_trait::async_trait;

use crate::domain::supporting::traffic_governance::RateLimitDecision::RateLimitDecision;

#[async_trait]
pub trait RateLimitDao: Send + Sync {
    async fn evaluate(&self, key: &str, limit: u64, window_ms: u64) -> anyhow::Result<RateLimitDecision>;
}
