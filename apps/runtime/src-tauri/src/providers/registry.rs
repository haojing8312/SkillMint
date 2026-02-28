use std::collections::HashMap;
use std::sync::Arc;

use super::anthropic_compat::AnthropicCompatProvider;
use super::deepseek::DeepSeekProvider;
use super::moonshot::MoonshotProvider;
use super::qwen::QwenProvider;
use super::traits::ProviderPlugin;

#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn ProviderPlugin>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Arc<dyn ProviderPlugin>) {
        self.providers.insert(provider.key().to_string(), provider);
    }

    pub fn get(&self, key: &str) -> Option<Arc<dyn ProviderPlugin>> {
        self.providers.get(key).map(Arc::clone)
    }

    pub fn list(&self) -> Vec<Arc<dyn ProviderPlugin>> {
        self.providers.values().map(Arc::clone).collect()
    }

    pub fn list_by_capability(&self, capability: &str) -> Vec<Arc<dyn ProviderPlugin>> {
        self.providers
            .values()
            .filter(|provider| provider.supports(capability))
            .map(Arc::clone)
            .collect()
    }

    pub fn with_china_first_p0() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(DeepSeekProvider::new()));
        registry.register(Arc::new(QwenProvider::new()));
        registry.register(Arc::new(MoonshotProvider::new()));
        registry.register(Arc::new(AnthropicCompatProvider::new()));
        registry
    }
}
