use redis::Script;

use crate::domain::supporting::traffic_governance::RateLimitDao::RateLimitDao;
use crate::domain::supporting::traffic_governance::RateLimitDecision::RateLimitDecision;

const SLIDING_WINDOW_LUA: &str = r#"
local key = KEYS[1]
local now_ms = tonumber(ARGV[1])
local window_ms = tonumber(ARGV[2])
local limit = tonumber(ARGV[3])

redis.call('ZREMRANGEBYSCORE', key, 0, now_ms - window_ms)
local current = redis.call('ZCARD', key)

if current < limit then
  local member = tostring(now_ms) .. '-' .. tostring(math.random(100000,999999))
  redis.call('ZADD', key, now_ms, member)
  redis.call('PEXPIRE', key, window_ms)
  local used = current + 1
  local remaining = limit - used
  return {1, remaining, window_ms}
else
  local oldest = redis.call('ZRANGE', key, 0, 0, 'WITHSCORES')
  local reset_after = window_ms
  if oldest ~= nil and #oldest >= 2 then
    local oldest_ts = tonumber(oldest[2])
    reset_after = (oldest_ts + window_ms) - now_ms
    if reset_after < 0 then
      reset_after = 0
    end
  end
  return {0, 0, reset_after}
end
"#;

pub struct RedisRateLimitDao {
    pub client: redis::Client,
}

impl RedisRateLimitDao {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl RateLimitDao for RedisRateLimitDao {
    async fn evaluate(
        &self,
        key: &str,
        limit: u64,
        window_ms: u64,
    ) -> anyhow::Result<RateLimitDecision> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64;

        let script = Script::new(SLIDING_WINDOW_LUA);
        let result: Vec<i64> = script
            .key(key)
            .arg(now_ms as i64)
            .arg(window_ms as i64)
            .arg(limit as i64)
            .invoke_async(&mut conn)
            .await?;

        if result.len() < 3 {
            anyhow::bail!("invalid limiter response");
        }

        Ok(RateLimitDecision {
            allowed: result[0] == 1,
            limit,
            remaining: result[1].max(0) as u64,
            reset_after_ms: result[2].max(0) as u64,
        })
    }
}
