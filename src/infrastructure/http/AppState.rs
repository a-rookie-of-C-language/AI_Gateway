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
    pub pg_pool: Option<sqlx::PgPool>,
}

impl AppState {
    pub async fn try_consume_tokens(&self, tokens: u64) -> anyhow::Result<bool> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("quota:{}", today);
        let max = self.quota_policy.max_tokens_per_day;

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        let script = redis::Script::new(r"
            local current = redis.call('INCRBY', KEYS[1], ARGV[1])
            if redis.call('TTL', KEYS[1]) == -1 then
                redis.call('EXPIRE', KEYS[1], ARGV[2])
            end
            if current > tonumber(ARGV[3]) then
                redis.call('DECRBY', KEYS[1], ARGV[1])
                return 0
            end
            return 1
        ");

        let result: i32 = script
            .key(&key)
            .arg(tokens)
            .arg(86400)
            .arg(max)
            .invoke_async(&mut conn)
            .await?;

        Ok(result == 1)
    }

    pub async fn release_tokens(&self, tokens: u64) -> anyhow::Result<()> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("quota:{}", today);
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let _: i64 = conn.decr(&key, tokens).await?;
        Ok(())
    }

    pub async fn check_redis(&self) -> bool {
        match self.redis_client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                let _: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;
                true
            }
            Err(_) => false,
        }
    }

    pub async fn check_postgres(&self) -> Option<bool> {
        match &self.pg_pool {
            Some(pool) => Some(sqlx::query("SELECT 1").execute(pool).await.is_ok()),
            None => None,
        }
    }
}
