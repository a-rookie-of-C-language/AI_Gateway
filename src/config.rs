use std::env;

#[derive(Clone)]
pub struct Config {
    pub app_name: String,
    pub http_addr: String,
    pub master_api_key: String,
    pub redis_addr: String,
    pub rate_limit_per_min: u64,
    pub rate_limit_tenant_per_min: u64,
    pub rate_limit_route_per_min: u64,
    pub rate_limit_model_per_min: u64,
    pub rate_limit_window_ms: u64,
    pub provider_base_url: String,
    pub provider_api_key: String,
    pub provider_model: String,
    pub provider_timeout_sec: u64,
}

impl Config {
    pub fn load() -> Self {
        Self {
            app_name: env_or("APP_NAME", "aigateway"),
            http_addr: env_or("HTTP_ADDR", "0.0.0.0:8080"),
            master_api_key: env_or("MASTER_API_KEY", "dev-key"),
            redis_addr: env_or("REDIS_ADDR", "redis://127.0.0.1:6379"),
            rate_limit_per_min: env_or("RATE_LIMIT_PER_MIN", "120").parse().unwrap_or(120),
            rate_limit_tenant_per_min: env_or("RATE_LIMIT_TENANT_PER_MIN", "240").parse().unwrap_or(240),
            rate_limit_route_per_min: env_or("RATE_LIMIT_ROUTE_PER_MIN", "120").parse().unwrap_or(120),
            rate_limit_model_per_min: env_or("RATE_LIMIT_MODEL_PER_MIN", "120").parse().unwrap_or(120),
            rate_limit_window_ms: env_or("RATE_LIMIT_WINDOW_MS", "60000").parse().unwrap_or(60000),
            provider_base_url: env_or("PROVIDER_BASE_URL", "https://api.openai.com/v1"),
            provider_api_key: env_or("PROVIDER_API_KEY", ""),
            provider_model: env_or("PROVIDER_MODEL", "gpt-4.1-mini"),
            provider_timeout_sec: env_or("PROVIDER_TIMEOUT_SEC", "60").parse().unwrap_or(60),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
