use super::ProviderDescriptor::ProviderDescriptor;

pub trait ProviderRouter: Send + Sync {
    fn route(&self, model: &str) -> Option<&ProviderDescriptor>;
    fn default_provider(&self) -> &ProviderDescriptor;
}
