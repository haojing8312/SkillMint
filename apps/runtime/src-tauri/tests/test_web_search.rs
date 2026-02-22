use runtime_lib::agent::tools::WebSearchTool;
use runtime_lib::agent::Tool;
use serde_json::json;

#[test]
fn test_web_search_tool_metadata() {
    let tool = WebSearchTool::new("http://localhost:8765".to_string());
    assert_eq!(tool.name(), "web_search");
    assert!(!tool.description().is_empty());

    let schema = tool.input_schema();
    assert!(schema["properties"]["query"].is_object());
    assert!(schema["required"].as_array().unwrap().contains(&json!("query")));
}

#[test]
fn test_web_search_missing_query() {
    let tool = WebSearchTool::new("http://localhost:8765".to_string());
    let result = tool.execute(json!({}));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("query"));
}

#[test]
fn test_web_search_empty_query() {
    let tool = WebSearchTool::new("http://localhost:8765".to_string());
    let result = tool.execute(json!({"query": "  "}));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不能为空"));
}

/// 集成测试：需要 Sidecar 运行在 localhost:8765
/// 运行方式：cargo test test_web_search_integration -- --ignored
#[test]
#[ignore]
fn test_web_search_integration() {
    let tool = WebSearchTool::new("http://localhost:8765".to_string());
    let result = tool.execute(json!({"query": "Rust programming language", "count": 3}));
    assert!(result.is_ok(), "搜索失败: {:?}", result);
    let output = result.unwrap();
    assert!(!output.is_empty());
}
