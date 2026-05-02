use std::env;

#[derive(Clone)]
pub struct Config {
    pub app_name: String,
    pub app_env: String,
    pub http_addr: String,
    pub master_api_key: String,
    pub redis_addr: String,
    pub rate_limit_per_min: u64,
    pub rate_limit_tenant_per_min: u64,
    pub rate_limit_route_per_min: u64,
    pub rate_limit_model_per_min: u64,
    pub rate_limit_window_ms: u64,
    pub max_tokens_per_day: u64,
    pub provider_base_url: String,
    pub provider_api_key: String,
    pub provider_model: String,
    pub provider_timeout_sec: u64,
    pub database_url: Option<String>,
    pub db_max_connections: u32,
}

const DEFAULT_MASTER_API_KEY: &str = "dev-key";

impl Config {
    pub fn load() -> Self {
        let cfg = Self {
            app_name: env_or("APP_NAME", "aigateway"),
            app_env: env_or("APP_ENV", "dev"),
            http_addr: env_or("HTTP_ADDR", "0.0.0.0:8080"),
            master_api_key: env_or("MASTER_API_KEY", DEFAULT_MASTER_API_KEY),
            redis_addr: env_or("REDIS_ADDR", "redis://127.0.0.1:6379"),
            rate_limit_per_min: env_or("RATE_LIMIT_PER_MIN", "120").parse().unwrap_or(120),
            rate_limit_tenant_per_min: env_or("RATE_LIMIT_TENANT_PER_MIN", "240").parse().unwrap_or(240),
            rate_limit_route_per_min: env_or("RATE_LIMIT_ROUTE_PER_MIN", "120").parse().unwrap_or(120),
            rate_limit_model_per_min: env_or("RATE_LIMIT_MODEL_PER_MIN", "120").parse().unwrap_or(120),
            rate_limit_window_ms: env_or("RATE_LIMIT_WINDOW_MS", "60000").parse().unwrap_or(60000),
            max_tokens_per_day: env_or("MAX_TOKENS_PER_DAY", "1000000").parse().unwrap_or(1_000_000),
            provider_base_url: env_or("PROVIDER_BASE_URL", "https://api.openai.com/v1"),
            provider_api_key: env_or("PROVIDER_API_KEY", ""),
            provider_model: env_or("PROVIDER_MODEL", "gpt-4.1-mini"),
            provider_timeout_sec: env_or("PROVIDER_TIMEOUT_SEC", "60").parse().unwrap_or(60),
            database_url: env::var("DATABASE_URL").ok(),
            db_max_connections: env_or("DB_MAX_CONNECTIONS", "5").parse().unwrap_or(5),
        };
        cfg.validate();
        cfg
    }

    fn validate(&self) {
        if self.app_env != "dev" && self.master_api_key == DEFAULT_MASTER_API_KEY {
            panic!(
                "FATAL: APP_ENV is '{}' but MASTER_API_KEY is still the default '{}'. \
                 Set a secure MASTER_API_KEY before starting in non-dev environments.",
                self.app_env, DEFAULT_MASTER_API_KEY
            );
        }
        if self.app_env != "dev" && self.provider_api_key.is_empty() {
            panic!(
                "FATAL: APP_ENV is '{}' but PROVIDER_API_KEY is empty. \
                 Set a valid PROVIDER_API_KEY before starting in non-dev environments.",
                self.app_env
            );
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
