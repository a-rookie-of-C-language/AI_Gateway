use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::middleware;

use crate::application::chat::ChatAppService::ChatAppService;
use crate::application::chat::ChatService::ChatService;
use crate::bootstrap::App::App;
use crate::config::Config;
use crate::domain::core::quota_billing::QuotaPolicy::QuotaPolicy;
use crate::domain::core::quota_billing::QuotaPolicyDao::QuotaPolicyDao;
use crate::domain::core::quota_billing::TokenUsageDao::TokenUsageDao;
use crate::domain::core::tenant_access_control::TenantDao::TenantDao;
use crate::domain::supporting::observability_audit::AuditLogDao::AuditLogDao;
use crate::domain::supporting::provider_integration::ProviderDescriptor::ProviderDescriptor;
use crate::domain::supporting::traffic_governance::RateLimitDao::RateLimitDao;
use crate::infrastructure::dao::audit::PostgresAuditLogDao::PostgresAuditLogDao;
use crate::infrastructure::dao::billing::PostgresTokenUsageDao::PostgresTokenUsageDao;
use crate::infrastructure::dao::quota::RedisQuotaPolicyDao::RedisQuotaPolicyDao;
use crate::infrastructure::dao::ratelimit::RedisRateLimitDao::RedisRateLimitDao;
use crate::infrastructure::dao::tenant::PostgresTenantDao::PostgresTenantDao;
use crate::infrastructure::http::AppState::AppState;
use crate::infrastructure::http::build_router::build_router;
use crate::infrastructure::provider::DefaultProviderRouter::DefaultProviderRouter;
use crate::infrastructure::provider::OpenAICompatibleGateway::OpenAICompatibleGateway;
use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;

pub async fn build_app() -> Result<App> {
    let cfg = Config::load().map_err(|e| {
        tracing::error!("{}", e);
        e
    })?;

    let provider = Arc::new(OpenAICompatibleGateway::new(
        cfg.provider_base_url.clone(),
        cfg.provider_api_key.clone(),
        cfg.provider_model.clone(),
        cfg.provider_timeout_sec,
    )?);

    let mut gateways: HashMap<String, Arc<dyn crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway>> = HashMap::new();
    gateways.insert("openai".to_string(), provider.clone());

    let provider_descriptor = ProviderDescriptor {
        provider_code: "openai".to_string(),
        base_url: cfg.provider_base_url.clone(),
        api_key: cfg.provider_api_key.clone(),
        enabled: true,
        models: vec![],
    };

    let router: Arc<dyn crate::domain::supporting::provider_integration::ProviderRouter::ProviderRouter> = Arc::new(
        DefaultProviderRouter::new(vec![provider_descriptor], "openai")
    );

    let chat_service: Arc<dyn ChatService> = Arc::new(ChatAppService::new(gateways, router));

    let redis_client = redis::Client::open(cfg.redis_addr.clone())?;
    let rate_limit_dao: Arc<dyn RateLimitDao> = Arc::new(RedisRateLimitDao::new(redis_client.clone()));
    let quota_policy_dao: Arc<dyn QuotaPolicyDao> = Arc::new(RedisQuotaPolicyDao::new(redis_client.clone()));

    let (token_usage_dao, pg_pool, tenant_dao, audit_log_dao): (Option<Arc<dyn TokenUsageDao>>, Option<sqlx::PgPool>, Option<Arc<dyn TenantDao>>, Option<Arc<dyn AuditLogDao>>) = match &cfg.database_url {
        Some(url) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(cfg.db_max_connections)
                .connect(url)
                .await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            tracing::info!("PostgreSQL connected, migrations applied");
            let pool_clone = pool.clone();
            let tenant_dao: Arc<dyn TenantDao> = Arc::new(PostgresTenantDao::new(pool.clone()));
            let audit_log_dao: Arc<dyn AuditLogDao> = Arc::new(PostgresAuditLogDao::new(pool.clone()));
            (Some(Arc::new(PostgresTokenUsageDao::new(pool)) as Arc<dyn TokenUsageDao>), Some(pool_clone), Some(tenant_dao), Some(audit_log_dao))
        }
        None => {
            tracing::info!("DATABASE_URL not set, token usage persistence, tenant management, and audit logs disabled");
            (None, None, None, None)
        }
    };

    let app_state = AppState {
        chat_service,
        default_quota_policy: QuotaPolicy {
            plan_code: "default".to_string(),
            max_tokens_per_day: cfg.max_tokens_per_day,
            max_tokens_per_request: None,
            rate_limit_per_min: None,
        },
        quota_policy_dao: Some(quota_policy_dao),
        redis_client,
        token_usage_dao,
        audit_log_dao,
        pg_pool,
    };

    let middleware_state = MiddlewareState {
        master_api_key: cfg.master_api_key,
        rate_limit_per_min: cfg.rate_limit_per_min,
        rate_limit_tenant_per_min: cfg.rate_limit_tenant_per_min,
        rate_limit_route_per_min: cfg.rate_limit_route_per_min,
        rate_limit_model_per_min: cfg.rate_limit_model_per_min,
        rate_limit_window_ms: cfg.rate_limit_window_ms,
        rate_limit_fail_open: cfg.rate_limit_fail_open,
        rate_limit_dao,
        tenant_dao,
    };

    let router = build_router(app_state)
        .layer(middleware::from_fn_with_state(
            middleware_state.clone(),
            crate::interfaces::http::middleware::auth::auth,
        ))
        .layer(middleware::from_fn_with_state(
            middleware_state,
            crate::interfaces::http::middleware::rate_limit::rate_limit,
        ))
        .layer(middleware::from_fn(
            crate::interfaces::http::middleware::request_id::request_id,
        ));

    Ok(App {
        addr: cfg.http_addr,
        router,
    })
}
