// Rate limiting constants
pub const DEFAULT_RATE_LIMIT_PER_MIN: u64 = 120;
pub const DEFAULT_RATE_LIMIT_TENANT_PER_MIN: u64 = 240;
pub const DEFAULT_RATE_LIMIT_ROUTE_PER_MIN: u64 = 120;
pub const DEFAULT_RATE_LIMIT_MODEL_PER_MIN: u64 = 120;
pub const DEFAULT_RATE_LIMIT_WINDOW_MS: u64 = 60000;

// Quota constants
pub const DEFAULT_MAX_TOKENS_PER_DAY: u64 = 1_000_000;
pub const MAX_TOKENS_PER_REQUEST: u64 = 1_000_000;

// Retry constants
pub const MAX_PROVIDER_RETRIES: u32 = 3;
pub const MAX_ROLLBACK_RETRIES: u32 = 3;
pub const RETRY_BACKOFF_BASE_MS: u64 = 100;

// Buffer limits
pub const MAX_SSE_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_MODEL_PARSE_BODY_BYTES: usize = 1024 * 1024; // 1MB

// Validation constants
pub const MAX_MESSAGES: usize = 128;
pub const MAX_MESSAGE_CONTENT_LEN: usize = 128 * 1024; // 128KB
pub const VALID_ROLES: &[&str] = &["system", "user", "assistant", "tool"];

// Parameter ranges
pub const TEMPERATURE_MIN: f64 = 0.0;
pub const TEMPERATURE_MAX: f64 = 2.0;
pub const TOP_P_MIN: f64 = 0.0;
pub const TOP_P_MAX: f64 = 1.0;
pub const FREQUENCY_PENALTY_MIN: f64 = -2.0;
pub const FREQUENCY_PENALTY_MAX: f64 = 2.0;
pub const PRESENCE_PENALTY_MIN: f64 = -2.0;
pub const PRESENCE_PENALTY_MAX: f64 = 2.0;

// Redis pool constants
pub const REDIS_POOL_MAX_SIZE: usize = 15;
pub const REDIS_POOL_TIMEOUT_SECS: u64 = 5;

// SSE chunk timeout
pub const SSE_CHUNK_TIMEOUT_SECS: u64 = 30;
pub const MAX_CONSECUTIVE_TIMEOUTS: u32 = 3;

// Streaming usage persistence timeout
pub const STREAMING_USAGE_TIMEOUT_SECS: u64 = 10;
