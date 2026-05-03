use async_trait::async_trait;
use redis::AsyncCommands;

use crate::domain::core::quota_billing::QuotaPolicy::QuotaPolicy;
use crate::domain::core::quota_billing::QuotaPolicyDao::QuotaPolicyDao;

pub struct RedisQuotaPolicyDao {
    redis_client: redis::Client,
}

impl RedisQuotaPolicyDao {
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    fn policy_key(tenant_id: &str, app_id: &str) -> String {
        format!("quota_policy:{}:{}", tenant_id, app_id)
    }
}

#[async_trait]
impl QuotaPolicyDao for RedisQuotaPolicyDao {
    async fn get_policy(&self, tenant_id: &str, app_id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = Self::policy_key(tenant_id, app_id);

        let data: Option<String> = conn.get(&key).await?;
        match data {
            Some(json_str) => {
                let policy: QuotaPolicy = serde_json::from_str(&json_str)?;
                Ok(Some(policy))
            }
            None => Ok(None),
        }
    }

    async fn set_policy(&self, tenant_id: &str, app_id: &str, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = Self::policy_key(tenant_id, app_id);

        let json_str = serde_json::to_string(policy)?;
        let _: () = conn.set(&key, &json_str).await?;

        Ok(())
    }
}
