use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;

use crate::domain::ratelimit::RateLimiter;

pub struct RedisRateLimiter {
    client: redis::Client,
}

impl RedisRateLimiter {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn allow(&self, key: &str, limit: u64) -> anyhow::Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let count: u64 = conn.incr(key, 1).await?;
        if count == 1 {
            let _: () = conn.expire(key, Duration::from_secs(60).as_secs() as i64).await?;
        }
        Ok(count <= limit)
    }
}
