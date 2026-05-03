use async_trait::async_trait;

use super::QuotaPolicy::QuotaPolicy;

#[async_trait]
pub trait QuotaPolicyDao: Send + Sync {
    async fn get_policy(&self, tenant_id: &str, app_id: &str) -> anyhow::Result<Option<QuotaPolicy>>;
    async fn set_policy(&self, tenant_id: &str, app_id: &str, policy: &QuotaPolicy) -> anyhow::Result<()>;
}
