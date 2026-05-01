use std::sync::Arc;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;

#[derive(Clone)]
pub struct ChatAppService {
    pub gateway: Arc<dyn ChatGateway>,
}
