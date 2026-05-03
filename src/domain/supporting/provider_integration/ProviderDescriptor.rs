use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub provider_code: String,
    pub base_url: String,
    pub api_key: String,
    pub enabled: bool,
    pub models: Vec<String>,
}

impl ProviderDescriptor {
    pub fn supports_model(&self, model: &str) -> bool {
        self.enabled && (self.models.is_empty() || self.models.iter().any(|m| m == model))
    }
}
