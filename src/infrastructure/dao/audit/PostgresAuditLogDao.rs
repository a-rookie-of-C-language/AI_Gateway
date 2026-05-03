use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::supporting::observability_audit::AuditLogDao::AuditLogDao;
use crate::domain::supporting::observability_audit::TraceRecord::AuditLog;

pub struct PostgresAuditLogDao {
    pool: PgPool,
}

impl PostgresAuditLogDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogDao for PostgresAuditLogDao {
    async fn insert(&self, log: &AuditLog) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (request_id, tenant_id, app_id, method, path, status_code, duration_ms, error_message, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&log.request_id)
        .bind(&log.tenant_id)
        .bind(&log.app_id)
        .bind(&log.method)
        .bind(&log.path)
        .bind(log.status_code as i16)
        .bind(log.duration_ms as i32)
        .bind(&log.error_message)
        .bind(log.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
