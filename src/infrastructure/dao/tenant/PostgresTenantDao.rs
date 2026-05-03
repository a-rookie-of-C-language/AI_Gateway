use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::core::tenant_access_control::Tenant::Tenant;
use crate::domain::core::tenant_access_control::TenantDao::TenantDao;

pub struct PostgresTenantDao {
    pool: PgPool,
}

impl PostgresTenantDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantDao for PostgresTenantDao {
    async fn find_by_api_key_hash(&self, api_key_hash: &str) -> anyhow::Result<Option<Tenant>> {
        let row = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT tenant_id, app_id, api_key_hash, enabled, created_at, updated_at
            FROM tenants
            WHERE api_key_hash = $1 AND enabled = true
            "#,
        )
        .bind(api_key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn insert(&self, tenant: &Tenant) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tenants (tenant_id, app_id, api_key_hash, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&tenant.tenant_id)
        .bind(&tenant.app_id)
        .bind(&tenant.api_key_hash)
        .bind(tenant.enabled)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_last_login(&self, tenant_id: &str, app_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE tenants
            SET updated_at = NOW()
            WHERE tenant_id = $1 AND app_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(app_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
