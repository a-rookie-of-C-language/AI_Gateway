use std::sync::Arc;

use crate::domain::chat::ChatGateway::ChatGateway;

#[derive(Clone)]
pub struct ChatAppService {
    pub gateway: Arc<dyn ChatGateway>,
}
