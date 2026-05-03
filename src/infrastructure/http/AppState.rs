use std::sync::Arc;

use redis::AsyncCommands;

use crate::application::chat::ChatService::ChatService;
use crate::domain::core::quota_billing::QuotaPolicy::QuotaPolicy;
use crate::domain::core::quota_billing::QuotaPolicyDao::QuotaPolicyDao;
use crate::domain::core::quota_billing::TokenUsageDao::TokenUsageDao;
use crate::domain::supporting::observability_audit::AuditLogDao::AuditLogDao;

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<dyn ChatService>,
    pub default_quota_policy: QuotaPolicy,
    pub quota_policy_dao: Option<Arc<dyn QuotaPolicyDao>>,
    pub redis_client: redis::Client,
    pub token_usage_dao: Option<Arc<dyn TokenUsageDao>>,
    pub audit_log_dao: Option<Arc<dyn AuditLogDao>>,
    pub pg_pool: Option<sqlx::PgPool>,
}

impl AppState {
    pub async fn get_quota_policy(&self, tenant_id: &str, app_id: &str) -> QuotaPolicy {
        if let Some(ref dao) = self.quota_policy_dao {
            match dao.get_policy(tenant_id, app_id).await {
                Ok(Some(policy)) => return policy,
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("failed to get tenant quota policy: {}, using default", e);
                }
            }
        }
        self.default_quota_policy.clone()
    }

    pub async fn try_consume_tokens(&self, tokens: u64, tenant_id: &str, app_id: &str) -> anyhow::Result<bool> {
        let policy = self.get_quota_policy(tenant_id, app_id).await;
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("quota:{}:{}:{}", tenant_id, app_id, today);
        let max = policy.max_tokens_per_day;

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

    pub async fn release_tokens(&self, tokens: u64, tenant_id: &str, app_id: &str) -> anyhow::Result<()> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("quota:{}:{}:{}", tenant_id, app_id, today);
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

    pub async fn check_provider(&self) -> bool {
        use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
        use crate::domain::core::gateway_orchestration::CompletionRequest::CompletionRequest;
        use crate::domain::core::gateway_orchestration::Message::Message;

        let req = CompletionRequest {
            model: Some("health-check".to_string()),
            messages: vec![Message {
                role: "user".to_string(),
                content: "ping".to_string(),
            }],
            temperature: None,
            max_tokens: Some(1),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            tools: None,
            response_format: None,
        };

        match tokio::time::timeout(std::time::Duration::from_secs(5), self.chat_service.complete(req)).await {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                tracing::warn!("provider health check failed: {}", e);
                false
            }
            Err(_) => {
                tracing::warn!("provider health check timed out");
                false
            }
        }
    }
}
