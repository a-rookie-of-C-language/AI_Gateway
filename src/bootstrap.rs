use std::sync::Arc;

use anyhow::Result;
use axum::{middleware, Router};
use tokio::net::TcpListener;

use crate::{
    application::chat::ChatAppService,
    config::Config,
    domain::ratelimit::RateLimiter,
    infrastructure::{
        http::router::{build_router, AppState},
        provider::mock_chat_gateway::MockChatGateway,
        ratelimit::redis_rate_limiter::RedisRateLimiter,
    },
    interfaces::http::middleware::MiddlewareState,
};

pub struct App {
    addr: String,
    router: Router,
}

impl App {
    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        tracing::info!("aigateway listening on {}", self.addr);
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

pub async fn build_app() -> Result<App> {
    let cfg = Config::load();

    let provider = Arc::new(MockChatGateway);
    let chat_service = Arc::new(ChatAppService::new(provider));

    let redis_client = redis::Client::open(cfg.redis_addr.clone())?;
    let limiter: Arc<dyn RateLimiter> = Arc::new(RedisRateLimiter::new(redis_client));

    let app_state = AppState { chat_service };
    let middleware_state = MiddlewareState {
        master_api_key: cfg.master_api_key,
        rate_limit_per_min: cfg.rate_limit_per_min,
        limiter,
    };

    let router = build_router(app_state)
        .layer(middleware::from_fn(crate::interfaces::http::middleware::request_id))
        .layer(middleware::from_fn_with_state(
            middleware_state.clone(),
            crate::interfaces::http::middleware::rate_limit,
        ))
        .layer(middleware::from_fn_with_state(
            middleware_state,
            crate::interfaces::http::middleware::auth,
        ));

    Ok(App {
        addr: cfg.http_addr,
        router,
    })
}
