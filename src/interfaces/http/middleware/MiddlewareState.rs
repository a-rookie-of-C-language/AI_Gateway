use std::sync::Arc;

use crate::domain::supporting::traffic_governance::RateLimitDao::RateLimitDao;

#[derive(Clone)]
pub struct MiddlewareState {
    pub master_api_key: String,
    pub rate_limit_per_min: u64,
    pub rate_limit_tenant_per_min: u64,
    pub rate_limit_route_per_min: u64,
    pub rate_limit_model_per_min: u64,
    pub rate_limit_window_ms: u64,
    pub rate_limit_dao: Arc<dyn RateLimitDao>,
}
