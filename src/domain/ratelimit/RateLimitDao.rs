use async_trait::async_trait;

#[async_trait]
pub trait RateLimitDao: Send + Sync {
    async fn allow(&self, key: &str, limit: u64) -> anyhow::Result<bool>;
}
