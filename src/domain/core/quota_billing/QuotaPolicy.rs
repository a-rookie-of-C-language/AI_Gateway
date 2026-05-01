#[derive(Clone, Debug)]
pub struct QuotaPolicy {
    pub plan_code: String,
    pub max_tokens_per_day: u64,
}
