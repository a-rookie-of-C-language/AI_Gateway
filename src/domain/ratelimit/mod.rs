use async_trait::async_trait;

#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn allow(&self, key: &str, limit: u64) -> anyhow::Result<bool>;
}
