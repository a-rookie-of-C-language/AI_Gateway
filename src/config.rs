use std::env;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    MissingMasterApiKey { env: String },
    MissingProviderApiKey { env: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingMasterApiKey { env } => {
                write!(
                    f,
                    "APP_ENV is '{}' but MASTER_API_KEY is still the default. \
                     Set a secure MASTER_API_KEY before starting in non-dev environments.",
                    env
                )
            }
            ConfigError::MissingProviderApiKey { env } => {
                write!(
                    f,
                    "APP_ENV is '{}' but PROVIDER_API_KEY is empty. \
                     Set a valid PROVIDER_API_KEY before starting in non-dev environments.",
                    env
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

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
    pub rate_limit_fail_open: bool,
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
    pub fn load() -> anyhow::Result<Self> {
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
            rate_limit_fail_open: env_or("RATE_LIMIT_FAIL_OPEN", "true").parse().unwrap_or(true),
            max_tokens_per_day: env_or("MAX_TOKENS_PER_DAY", "1000000").parse().unwrap_or(1_000_000),
            provider_base_url: env_or("PROVIDER_BASE_URL", "https://api.openai.com/v1"),
            provider_api_key: env_or("PROVIDER_API_KEY", ""),
            provider_model: env_or("PROVIDER_MODEL", "gpt-4.1-mini"),
            provider_timeout_sec: env_or("PROVIDER_TIMEOUT_SEC", "60").parse().unwrap_or(60),
            database_url: env::var("DATABASE_URL").ok(),
            db_max_connections: env_or("DB_MAX_CONNECTIONS", "5").parse().unwrap_or(5),
        };
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.app_env != "dev" && self.master_api_key == DEFAULT_MASTER_API_KEY {
            return Err(ConfigError::MissingMasterApiKey {
                env: self.app_env.clone(),
            });
        }
        if self.app_env != "dev" && self.provider_api_key.is_empty() {
            return Err(ConfigError::MissingProviderApiKey {
                env: self.app_env.clone(),
            });
        }
        Ok(())
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config {
            app_name: "test".to_string(),
            app_env: "dev".to_string(),
            http_addr: "0.0.0.0:8080".to_string(),
            master_api_key: "dev-key".to_string(),
            redis_addr: "redis://127.0.0.1:6379".to_string(),
            rate_limit_per_min: 120,
            rate_limit_tenant_per_min: 240,
            rate_limit_route_per_min: 120,
            rate_limit_model_per_min: 120,
            rate_limit_window_ms: 60000,
            rate_limit_fail_open: true,
            max_tokens_per_day: 1000000,
            provider_base_url: "https://api.openai.com/v1".to_string(),
            provider_api_key: "sk-test".to_string(),
            provider_model: "gpt-4".to_string(),
            provider_timeout_sec: 60,
            database_url: None,
            db_max_connections: 5,
        }
    }

    #[test]
    fn test_dev_env_with_default_key() {
        let cfg = default_config();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_prod_env_with_default_key() {
        let mut cfg = default_config();
        cfg.app_env = "production".to_string();
        cfg.master_api_key = "dev-key".to_string();
        let result = cfg.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingMasterApiKey { env } => assert_eq!(env, "production"),
            _ => panic!("expected MissingMasterApiKey"),
        }
    }

    #[test]
    fn test_prod_env_with_custom_key() {
        let mut cfg = default_config();
        cfg.app_env = "production".to_string();
        cfg.master_api_key = "secure-key-123".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_prod_env_without_provider_key() {
        let mut cfg = default_config();
        cfg.app_env = "production".to_string();
        cfg.master_api_key = "secure-key".to_string();
        cfg.provider_api_key = "".to_string();
        let result = cfg.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingProviderApiKey { env } => assert_eq!(env, "production"),
            _ => panic!("expected MissingProviderApiKey"),
        }
    }

    #[test]
    fn test_dev_env_without_provider_key() {
        let mut cfg = default_config();
        cfg.provider_api_key = "".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_staging_env_requires_custom_key() {
        let mut cfg = default_config();
        cfg.app_env = "staging".to_string();
        cfg.master_api_key = "dev-key".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MissingMasterApiKey { env: "prod".to_string() };
        assert!(format!("{}", err).contains("prod"));
        assert!(format!("{}", err).contains("MASTER_API_KEY"));
    }

    #[test]
    fn test_config_error_display_provider() {
        let err = ConfigError::MissingProviderApiKey { env: "staging".to_string() };
        assert!(format!("{}", err).contains("staging"));
        assert!(format!("{}", err).contains("PROVIDER_API_KEY"));
    }

    #[test]
    fn test_config_error_is_error() {
        let err: Box<dyn std::error::Error> = Box::new(ConfigError::MissingMasterApiKey {
            env: "prod".to_string(),
        });
        assert!(err.source().is_none());
    }

    #[test]
    fn test_dev_skip_all_validation() {
        let mut cfg = default_config();
        cfg.app_env = "dev".to_string();
        cfg.master_api_key = "dev-key".to_string();
        cfg.provider_api_key = "".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_env_or_default() {
        let val = env_or("NON_EXISTENT_KEY_12345", "default_value");
        assert_eq!(val, "default_value");
    }
}
