use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct TraceRecord {
    pub request_id: String,
    pub provider: String,
    pub span_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub request_id: String,
    pub tenant_id: String,
    pub app_id: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}
