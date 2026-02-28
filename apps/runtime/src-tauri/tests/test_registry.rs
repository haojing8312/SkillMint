use runtime_lib::agent::ToolRegistry;
use runtime_lib::providers::{ProviderPlugin, ProviderRegistry};
use std::sync::Arc;

#[test]
fn test_registry_with_standard_tools() {
    let registry = ToolRegistry::with_standard_tools();

    // 原有 8 个基础工具
    assert!(registry.get("read_file").is_some());
    assert!(registry.get("write_file").is_some());
    assert!(registry.get("glob").is_some());
    assert!(registry.get("grep").is_some());
    assert!(registry.get("edit").is_some());
    assert!(registry.get("todo_write").is_some());
    assert!(registry.get("web_fetch").is_some());
    assert!(registry.get("bash").is_some());
    // L2 新增 5 个文件工具
    assert!(registry.get("list_dir").is_some());
    assert!(registry.get("file_stat").is_some());
    assert!(registry.get("file_delete").is_some());
    assert!(registry.get("file_move").is_some());
    assert!(registry.get("file_copy").is_some());
    // L5 新增 2 个系统工具
    assert!(registry.get("screenshot").is_some());
    assert!(registry.get("open_in_folder").is_some());

    let defs = registry.get_tool_definitions();
    assert_eq!(defs.len(), 15);
}

struct MockProvider {
    key: &'static str,
    name: &'static str,
    capabilities: &'static [&'static str],
}

impl ProviderPlugin for MockProvider {
    fn key(&self) -> &str {
        self.key
    }

    fn display_name(&self) -> &str {
        self.name
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities
            .iter()
            .map(|capability| (*capability).to_string())
            .collect()
    }
}

#[test]
fn registry_can_register_and_lookup_provider() {
    let mut registry = ProviderRegistry::new();
    let deepseek = Arc::new(MockProvider {
        key: "deepseek",
        name: "DeepSeek",
        capabilities: &["chat", "tool_calling"],
    });
    let qwen = Arc::new(MockProvider {
        key: "qwen",
        name: "Qwen",
        capabilities: &["chat", "vision"],
    });

    registry.register(deepseek);
    registry.register(qwen);

    let provider = registry.get("deepseek").expect("deepseek should exist");
    assert_eq!(provider.display_name(), "DeepSeek");

    let vision_providers = registry.list_by_capability("vision");
    assert_eq!(vision_providers.len(), 1);
    assert_eq!(vision_providers[0].key(), "qwen");
}

#[test]
fn china_first_registry_contains_p0_plugins() {
    let registry = ProviderRegistry::with_china_first_p0();
    assert!(registry.get("deepseek").is_some());
    assert!(registry.get("qwen").is_some());
    assert!(registry.get("moonshot").is_some());
    assert!(registry.get("anthropic").is_some());

    let chat = registry.list_by_capability("chat");
    assert!(chat.len() >= 4);

    let vision = registry.list_by_capability("vision");
    assert!(vision.iter().any(|p| p.key() == "qwen"));
}
