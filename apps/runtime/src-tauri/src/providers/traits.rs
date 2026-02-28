pub trait ProviderPlugin: Send + Sync {
    fn key(&self) -> &str;
    fn display_name(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;

    fn supports(&self, capability: &str) -> bool {
        self.capabilities().iter().any(|c| c == capability)
    }
}
