use super::openai_compat::OpenAiCompatProvider;
use super::traits::ProviderPlugin;

pub struct MoonshotProvider {
    inner: OpenAiCompatProvider,
}

impl MoonshotProvider {
    pub fn new() -> Self {
        Self {
            inner: OpenAiCompatProvider::new(
                "moonshot",
                "Moonshot / Kimi",
                vec!["chat", "tool_calling", "long_context"],
            ),
        }
    }
}

impl ProviderPlugin for MoonshotProvider {
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
