use async_trait::async_trait;

use super::Tenant::Tenant;

#[async_trait]
pub trait TenantDao: Send + Sync {
    async fn find_by_api_key_hash(&self, api_key_hash: &str) -> anyhow::Result<Option<Tenant>>;
    async fn insert(&self, tenant: &Tenant) -> anyhow::Result<()>;
    async fn update_last_login(&self, tenant_id: &str, app_id: &str) -> anyhow::Result<()>;
}
