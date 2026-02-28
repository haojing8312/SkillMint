use super::openai_compat::OpenAiCompatProvider;
use super::traits::ProviderPlugin;

pub struct DeepSeekProvider {
    inner: OpenAiCompatProvider,
}

impl DeepSeekProvider {
    pub fn new() -> Self {
        Self {
            inner: OpenAiCompatProvider::new(
                "deepseek",
                "DeepSeek",
                vec!["chat", "tool_calling", "reasoning"],
            ),
        }
    }
}

impl ProviderPlugin for DeepSeekProvider {
    fn key(&self) -> &str {
        self.inner.key()
    }

    fn display_name(&self) -> &str {
        self.inner.display_name()
    }

    fn capabilities(&self) -> Vec<String> {
        self.inner.capabilities()
    }
}
