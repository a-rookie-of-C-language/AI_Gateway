use std::sync::Arc;

use redis::AsyncCommands;

use crate::application::chat::ChatAppService::ChatAppService;
use crate::domain::core::quota_billing::QuotaPolicy::QuotaPolicy;
use crate::domain::core::quota_billing::TokenUsageDao::TokenUsageDao;

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatAppService>,
    pub quota_policy: QuotaPolicy,
    pub redis_client: redis::Client,
    pub token_usage_dao: Option<Arc<dyn TokenUsageDao>>,
}

impl AppState {
    pub async fn try_consume_tokens(&self, tokens: u64) -> anyhow::Result<bool> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("quota:{}", today);

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let current: u64 = conn.incr(&key, tokens).await?;

        if current == tokens {
            let _: () = conn.expire(&key, 86400).await?;
        }

        if current > self.quota_policy.max_tokens_per_day {
            let _: () = conn.decr(&key, tokens).await?;
            return Ok(false);
        }

        Ok(true)
    }
}
