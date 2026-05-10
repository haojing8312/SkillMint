use anyhow::Result;
use runtime_lib::agent::tool_manifest::{ToolCategory, ToolMetadata, ToolSource};
use runtime_lib::agent::tools::ToolsetsTool;
use runtime_lib::agent::{Tool, ToolContext, ToolRegistry};
use serde_json::{json, Value};
use std::sync::Arc;

struct FakeTool {
    name: &'static str,
    category: ToolCategory,
    source: ToolSource,
}

impl Tool for FakeTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "fake test tool"
    }

    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {}})
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            category: self.category,
            source: self.source,
            read_only: true,
            ..ToolMetadata::default()
        }
    }

    fn execute(&self, _input: Value, _ctx: &ToolContext) -> Result<String> {
        Ok("ok".to_string())
    }
}

#[test]
fn toolsets_tool_projects_registry_into_manifest_first_toolsets() {
    let registry = Arc::new(ToolRegistry::new());
    registry.register(Arc::new(FakeTool {
        name: "memory",
        category: ToolCategory::Memory,
        source: ToolSource::Runtime,
    }));
    registry.register(Arc::new(FakeTool {
        name: "skills",
        category: ToolCategory::Agent,
        source: ToolSource::Runtime,
    }));
    registry.register(Arc::new(FakeTool {
        name: "browser_navigate",
        category: ToolCategory::Other,
        source: ToolSource::Native,
    }));
    registry.register(Arc::new(FakeTool {
        name: "mcp_docs_search",
        category: ToolCategory::Other,
        source: ToolSource::Native,
    }));
    registry.register(Arc::new(FakeTool {
        name: "web_search",
        category: ToolCategory::Search,
        source: ToolSource::Native,
    }));

    let tool = ToolsetsTool::new(registry);
    let raw = tool
        .execute(json!({"action": "list"}), &ToolContext::default())
        .expect("list toolsets");
    let output: Value = serde_json::from_str(&raw).expect("toolsets json");

    assert_eq!(output["action"], "list");
    assert!(raw.contains("\"memory\""));
    assert!(raw.contains("\"skills\""));
    assert!(raw.contains("\"browser\""));
    assert!(raw.contains("\"mcp\""));
    assert!(raw.contains("\"web\""));
    assert!(raw.contains("browser_navigate"));
    assert!(raw.contains("mcp_docs_search"));

    let browser = tool
        .execute(
            json!({"action": "view", "toolset": "browser"}),
            &ToolContext::default(),
        )
        .expect("view browser toolset");
    assert!(browser.contains("browser_navigate"));
    assert!(!browser.contains("mcp_docs_search"));
}

#[test]
fn toolsets_tool_persists_profile_default_allowed_toolsets_without_enforcement() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime
        .block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
        })
        .expect("create sqlite pool");
    let registry = Arc::new(ToolRegistry::new());
    let tool = ToolsetsTool::new(registry).with_profile_policy(pool, "profile-toolsets");

    let saved = tool
        .execute(
            json!({
                "action": "set_profile_policy",
                "allowed_toolsets": ["memory", "skills", "web"]
            }),
            &ToolContext::default(),
        )
        .expect("save profile policy");
    let saved_json: Value = serde_json::from_str(&saved).expect("saved json");
    assert_eq!(
        saved_json["profile_policy"]["profile_id"],
        "profile-toolsets"
    );
    assert_eq!(saved_json["profile_policy"]["enforced"], false);
    assert_eq!(
        saved_json["profile_policy"]["allowed_toolsets"],
        json!(["memory", "skills", "web"])
    );

    let loaded = tool
        .execute(json!({"action": "profile_policy"}), &ToolContext::default())
        .expect("load profile policy");
    let loaded_json: Value = serde_json::from_str(&loaded).expect("loaded json");
    assert_eq!(
        loaded_json["profile_policy"]["allowed_toolsets"],
        json!(["memory", "skills", "web"])
    );

    let err = tool
        .execute(
            json!({
                "action": "set_profile_policy",
                "allowed_toolsets": ["memory", "unknown"]
            }),
            &ToolContext::default(),
        )
        .expect_err("unknown toolset is rejected");
    assert!(err.to_string().contains("未知 toolset"));
}
