use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::core::quota_billing::GroupByDimension::GroupByDimension;
use crate::domain::core::quota_billing::TokenUsage::TokenUsage;
use crate::domain::core::quota_billing::TokenUsageDao::TokenUsageDao;
use crate::domain::core::quota_billing::UsageQuery::UsageQuery;
use crate::domain::core::quota_billing::UsageSummary::UsageSummary;

pub struct PostgresTokenUsageDao {
    pool: PgPool,
}

impl PostgresTokenUsageDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn build_where_clause(query: &UsageQuery) -> (String, usize) {
        let mut conditions: Vec<String> = Vec::new();
        let mut idx = 1usize;

        if query.tenant_id.is_some() {
            conditions.push(format!("tenant_id = ${}", idx));
            idx += 1;
        }
        if query.app_id.is_some() {
            conditions.push(format!("app_id = ${}", idx));
            idx += 1;
        }
        if query.model.is_some() {
            conditions.push(format!("model = ${}", idx));
            idx += 1;
        }
        if query.from.is_some() {
            conditions.push(format!("created_at >= ${}", idx));
            idx += 1;
        }
        if query.to.is_some() {
            conditions.push(format!("created_at <= ${}", idx));
            idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        (where_clause, idx)
    }
}

#[async_trait]
impl TokenUsageDao for PostgresTokenUsageDao {
    async fn insert(&self, usage: &TokenUsage) -> anyhow::Result<()> {
        sqlx::query(
            r#"INSERT INTO token_usage_records
               (request_id, tenant_id, app_id, model, prompt_tokens, completion_tokens, total_tokens, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (request_id) DO NOTHING"#,
        )
        .bind(&usage.request_id)
        .bind(&usage.tenant_id)
        .bind(&usage.app_id)
        .bind(&usage.model)
        .bind(usage.prompt_tokens)
        .bind(usage.completion_tokens)
        .bind(usage.total_tokens)
        .bind(usage.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn aggregate(&self, query: &UsageQuery) -> anyhow::Result<Vec<UsageSummary>> {
        let mut select_cols = vec![
            "COUNT(*) AS request_count".to_string(),
            "COALESCE(SUM(prompt_tokens), 0) AS total_prompt_tokens".to_string(),
            "COALESCE(SUM(completion_tokens), 0) AS total_completion_tokens".to_string(),
            "COALESCE(SUM(total_tokens), 0) AS total_tokens".to_string(),
        ];
        let mut group_cols: Vec<String> = Vec::new();

        for dim in &query.group_by {
            match dim {
                GroupByDimension::Tenant => {
                    select_cols.push("tenant_id".to_string());
                    group_cols.push("tenant_id".to_string());
                }
                GroupByDimension::Model => {
                    select_cols.push("model".to_string());
                    group_cols.push("model".to_string());
                }
                GroupByDimension::Hour => {
                    select_cols.push("date_trunc('hour', created_at) AS period_start".to_string());
                    group_cols.push("date_trunc('hour', created_at)".to_string());
                }
                GroupByDimension::Day => {
                    select_cols.push("date_trunc('day', created_at) AS period_start".to_string());
                    group_cols.push("date_trunc('day', created_at)".to_string());
                }
            }
        }

        if query.group_by.is_empty() {
            select_cols.push("NULL AS period_start".to_string());
        }

        let (where_clause, mut param_idx) = Self::build_where_clause(query);
        let mut sql = format!("SELECT {} FROM token_usage_records{}", select_cols.join(", "), where_clause);

        if !group_cols.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", group_cols.join(", ")));
        }

        sql.push_str(" ORDER BY total_tokens DESC");

        if query.limit.is_some() {
            sql.push_str(&format!(" LIMIT ${}", param_idx));
            param_idx += 1;
        }
        if query.offset.is_some() {
            sql.push_str(&format!(" OFFSET ${}", param_idx));
        }

        let mut q = sqlx::query_as::<_, UsageSummaryRow>(&sql);
        if let Some(ref v) = query.tenant_id { q = q.bind(v); }
        if let Some(ref v) = query.app_id { q = q.bind(v); }
        if let Some(ref v) = query.model { q = q.bind(v); }
        if let Some(v) = query.from { q = q.bind(v); }
        if let Some(v) = query.to { q = q.bind(v); }
        if let Some(limit) = query.limit { q = q.bind(limit); }
        if let Some(offset) = query.offset { q = q.bind(offset); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn list(&self, query: &UsageQuery) -> anyhow::Result<Vec<TokenUsage>> {
        let (where_clause, mut param_idx) = Self::build_where_clause(query);
        let mut sql = format!(
            "SELECT request_id, tenant_id, app_id, model, prompt_tokens, completion_tokens, total_tokens, created_at
             FROM token_usage_records{}",
            where_clause
        );

        sql.push_str(" ORDER BY created_at DESC");

        if query.limit.is_some() {
            sql.push_str(&format!(" LIMIT ${}", param_idx));
            param_idx += 1;
        }
        if query.offset.is_some() {
            sql.push_str(&format!(" OFFSET ${}", param_idx));
        }

        let mut q = sqlx::query_as::<_, TokenUsageRow>(&sql);
        if let Some(ref v) = query.tenant_id { q = q.bind(v); }
        if let Some(ref v) = query.app_id { q = q.bind(v); }
        if let Some(ref v) = query.model { q = q.bind(v); }
        if let Some(v) = query.from { q = q.bind(v); }
        if let Some(v) = query.to { q = q.bind(v); }
        if let Some(limit) = query.limit { q = q.bind(limit); }
        if let Some(offset) = query.offset { q = q.bind(offset); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn purge_before(&self, before: chrono::DateTime<chrono::Utc>) -> anyhow::Result<u64> {
        let result = sqlx::query("DELETE FROM token_usage_records WHERE created_at < $1")
            .bind(before)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct TokenUsageRow {
    request_id: String,
    tenant_id: String,
    app_id: String,
    model: String,
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TokenUsageRow> for TokenUsage {
    fn from(r: TokenUsageRow) -> Self {
        TokenUsage {
            request_id: r.request_id,
            tenant_id: r.tenant_id,
            app_id: r.app_id,
            model: r.model,
            prompt_tokens: r.prompt_tokens,
            completion_tokens: r.completion_tokens,
            total_tokens: r.total_tokens,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct UsageSummaryRow {
    tenant_id: Option<String>,
    app_id: Option<String>,
    model: Option<String>,
    period_start: Option<chrono::DateTime<chrono::Utc>>,
    request_count: i64,
    total_prompt_tokens: i64,
    total_completion_tokens: i64,
    total_tokens: i64,
}

impl From<UsageSummaryRow> for UsageSummary {
    fn from(r: UsageSummaryRow) -> Self {
        UsageSummary {
            tenant_id: r.tenant_id,
            app_id: r.app_id,
            model: r.model,
            period_start: r.period_start,
            request_count: r.request_count,
            total_prompt_tokens: r.total_prompt_tokens,
            total_completion_tokens: r.total_completion_tokens,
            total_tokens: r.total_tokens,
        }
    }
}
