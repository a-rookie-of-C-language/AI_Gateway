use futures_util::stream::BoxStream;
use serde_json::Value;
use tokio::sync::oneshot;

use super::TokenUsage::TokenUsage;

pub struct StreamingCompletion {
    pub stream: BoxStream<'static, anyhow::Result<Value>>,
    pub usage_rx: oneshot::Receiver<Option<TokenUsage>>,
}
