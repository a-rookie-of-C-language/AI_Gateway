use std::sync::{
    atomic::AtomicU64,
    Arc,
};

use anyhow::Result;
use axum::middleware;

use crate::application::chat::ChatAppService::ChatAppService;
use crate::bootstrap::App::App;
use crate::config::Config;
use crate::domain::core::quota_billing::QuotaPolicy::QuotaPolicy;
use crate::domain::supporting::traffic_governance::RateLimitDao::RateLimitDao;
use crate::infrastructure::dao::ratelimit::RedisRateLimitDao::RedisRateLimitDao;
use crate::infrastructure::http::AppState::AppState;
use crate::infrastructure::http::build_router::build_router;
use crate::infrastructure::provider::OpenAICompatibleGateway::OpenAICompatibleGateway;
use crate::interfaces::http::middleware::MiddlewareState::MiddlewareState;

pub async fn build_app() -> Result<App> {
    let cfg = Config::load();

    let provider = Arc::new(OpenAICompatibleGateway::new(
        cfg.provider_base_url.clone(),
        cfg.provider_api_key.clone(),
        cfg.provider_model.clone(),
        cfg.provider_timeout_sec,
    )?);
    let chat_service = Arc::new(ChatAppService::new(provider));

    let redis_client = redis::Client::open(cfg.redis_addr.clone())?;
    let rate_limit_dao: Arc<dyn RateLimitDao> = Arc::new(RedisRateLimitDao::new(redis_client));

    let app_state = AppState {
        chat_service,
        quota_policy: QuotaPolicy {
            plan_code: "default".to_string(),
            max_tokens_per_day: 1_000_000,
        },
        used_tokens_today: Arc::new(AtomicU64::new(0)),
    };

    let middleware_state = MiddlewareState {
        master_api_key: cfg.master_api_key,
        rate_limit_per_min: cfg.rate_limit_per_min,
        rate_limit_tenant_per_min: cfg.rate_limit_tenant_per_min,
        rate_limit_route_per_min: cfg.rate_limit_route_per_min,
        rate_limit_model_per_min: cfg.rate_limit_model_per_min,
        rate_limit_window_ms: cfg.rate_limit_window_ms,
        rate_limit_dao,
    };

    let router = build_router(app_state)
        .layer(middleware::from_fn(
            crate::interfaces::http::middleware::request_id::request_id,
        ))
        .layer(middleware::from_fn_with_state(
            middleware_state.clone(),
            crate::interfaces::http::middleware::rate_limit::rate_limit,
        ))
        .layer(middleware::from_fn_with_state(
            middleware_state,
            crate::interfaces::http::middleware::auth::auth,
        ));

    Ok(App {
        addr: cfg.http_addr,
        router,
    })
}
