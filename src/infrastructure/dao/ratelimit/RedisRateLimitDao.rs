use redis::AsyncCommands;

use crate::domain::ratelimit::RateLimitDao::RateLimitDao;

pub struct RedisRateLimitDao {
    pub client: redis::Client,
}

impl RedisRateLimitDao {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl RateLimitDao for RedisRateLimitDao {
    async fn allow(&self, key: &str, limit: u64) -> anyhow::Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let count: u64 = conn.incr(key, 1).await?;
        if count == 1 {
            let _: () = conn.expire(key, 60).await?;
        }
        Ok(count <= limit)
    }
}
