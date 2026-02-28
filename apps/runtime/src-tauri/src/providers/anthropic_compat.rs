use super::traits::ProviderPlugin;

pub struct AnthropicCompatProvider {
    key: &'static str,
    name: &'static str,
}

impl AnthropicCompatProvider {
    pub fn new() -> Self {
        Self {
            key: "anthropic",
            name: "Anthropic",
        }
    }
}

impl ProviderPlugin for AnthropicCompatProvider {
    fn key(&self) -> &str {
        self.key
    }

    fn display_name(&self) -> &str {
        self.name
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "chat".to_string(),
            "tool_calling".to_string(),
            "vision".to_string(),
        ]
    }
}
