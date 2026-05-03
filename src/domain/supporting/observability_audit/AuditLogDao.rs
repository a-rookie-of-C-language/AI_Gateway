use async_trait::async_trait;

use super::TraceRecord::AuditLog;

#[async_trait]
pub trait AuditLogDao: Send + Sync {
    async fn insert(&self, log: &AuditLog) -> anyhow::Result<()>;
}
