use runtime_lib::agent::{tools::SidecarBridgeTool, Tool};
use serde_json::json;

#[test]
fn test_sidecar_bridge_tool_exists() {
    let tool = SidecarBridgeTool::new(
        "http://localhost:8765".to_string(),
        "/api/browser/navigate".to_string(),
        "browser_navigate".to_string(),
        "Navigate browser to URL".to_string(),
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" }
            },
            "required": ["url"]
        }),
    );

    assert_eq!(tool.name(), "browser_navigate");
    assert_eq!(tool.description(), "Navigate browser to URL");

    // Verify schema is correct
    let schema = tool.input_schema();
    assert_eq!(schema["type"], "object");
}
