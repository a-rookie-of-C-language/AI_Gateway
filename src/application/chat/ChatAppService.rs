use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::core::gateway_orchestration::ChatGateway::ChatGateway;
use crate::domain::supporting::provider_integration::ProviderRouter::ProviderRouter;

#[derive(Clone)]
pub struct ChatAppService {
    pub gateways: HashMap<String, Arc<dyn ChatGateway>>,
    pub router: Arc<dyn ProviderRouter>,
}
