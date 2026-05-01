#[derive(Clone, Debug)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    pub reset_after_ms: u64,
}
