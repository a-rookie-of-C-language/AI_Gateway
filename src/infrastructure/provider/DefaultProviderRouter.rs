use crate::domain::supporting::provider_integration::ProviderDescriptor::ProviderDescriptor;
use crate::domain::supporting::provider_integration::ProviderRouter::ProviderRouter;

pub struct DefaultProviderRouter {
    providers: Vec<ProviderDescriptor>,
    default_index: usize,
}

impl DefaultProviderRouter {
    pub fn new(providers: Vec<ProviderDescriptor>, default_code: &str) -> Self {
        let default_index = providers
            .iter()
            .position(|p| p.provider_code == default_code)
            .unwrap_or(0);

        Self {
            providers,
            default_index,
        }
    }
}

impl ProviderRouter for DefaultProviderRouter {
    fn route(&self, model: &str) -> Option<&ProviderDescriptor> {
        self.providers.iter().find(|p| p.supports_model(model))
    }

    fn default_provider(&self) -> &ProviderDescriptor {
        &self.providers[self.default_index]
    }
}
