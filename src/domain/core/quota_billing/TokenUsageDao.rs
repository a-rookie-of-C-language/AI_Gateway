use async_trait::async_trait;

use super::TokenUsage::TokenUsage;
use super::UsageQuery::UsageQuery;
use super::UsageSummary::UsageSummary;

#[async_trait]
pub trait TokenUsageDao: Send + Sync {
    async fn insert(&self, usage: &TokenUsage) -> anyhow::Result<()>;
    async fn aggregate(&self, query: &UsageQuery) -> anyhow::Result<Vec<UsageSummary>>;
    async fn list(&self, query: &UsageQuery) -> anyhow::Result<Vec<TokenUsage>>;
    async fn purge_before(&self, before: chrono::DateTime<chrono::Utc>) -> anyhow::Result<u64>;
}
