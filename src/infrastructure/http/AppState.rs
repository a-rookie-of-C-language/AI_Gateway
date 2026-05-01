use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::application::chat::ChatAppService::ChatAppService;
use crate::domain::core::quota_billing::QuotaPolicy::QuotaPolicy;

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatAppService>,
    pub quota_policy: QuotaPolicy,
    pub used_tokens_today: Arc<AtomicU64>,
}

impl AppState {
    pub fn try_consume_tokens(&self, tokens: u64) -> bool {
        loop {
            let current = self.used_tokens_today.load(Ordering::Relaxed);
            let next = current.saturating_add(tokens);
            if next > self.quota_policy.max_tokens_per_day {
                return false;
            }
            if self
                .used_tokens_today
                .compare_exchange(current, next, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }
}
