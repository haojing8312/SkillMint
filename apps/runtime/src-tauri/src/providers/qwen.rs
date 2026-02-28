use super::openai_compat::OpenAiCompatProvider;
use super::traits::ProviderPlugin;

pub struct QwenProvider {
    inner: OpenAiCompatProvider,
}

impl QwenProvider {
    pub fn new() -> Self {
        Self {
            inner: OpenAiCompatProvider::new(
                "qwen",
                "Qwen",
                vec!["chat", "tool_calling", "vision", "image_gen"],
            ),
        }
    }
}

impl ProviderPlugin for QwenProvider {
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
