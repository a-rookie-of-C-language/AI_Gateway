use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::constants::REDIS_POOL_TIMEOUT_SECS;

pub type RedisConnection = ConnectionManager;

#[derive(Clone)]
pub struct RedisPool {
    manager: ConnectionManager,
}

impl RedisPool {
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let manager = tokio::time::timeout(
            std::time::Duration::from_secs(REDIS_POOL_TIMEOUT_SECS),
            ConnectionManager::new(client),
        )
        .await
        .map_err(|_| anyhow::anyhow!("redis connection timeout"))?
        .map_err(|e| anyhow::anyhow!("redis connection error: {}", e))?;

        Ok(Self { manager })
    }

    pub async fn get_connection(&self) -> anyhow::Result<RedisConnection> {
        Ok(self.manager.clone())
    }

    pub async fn ping(&self) -> bool {
        let mut conn = self.manager.clone();
        let result: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;
        result.is_ok()
    }
}
