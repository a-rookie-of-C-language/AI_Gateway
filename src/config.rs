use std::env;

#[derive(Clone)]
pub struct Config {
    pub app_name: String,
    pub http_addr: String,
    pub master_api_key: String,
    pub redis_addr: String,
    pub rate_limit_per_min: u64,
}

impl Config {
    pub fn load() -> Self {
        Self {
            app_name: env_or("APP_NAME", "aigateway"),
            http_addr: env_or("HTTP_ADDR", "0.0.0.0:8080"),
            master_api_key: env_or("MASTER_API_KEY", "dev-key"),
            redis_addr: env_or("REDIS_ADDR", "redis://127.0.0.1:6379"),
            rate_limit_per_min: env_or("RATE_LIMIT_PER_MIN", "120").parse().unwrap_or(120),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
