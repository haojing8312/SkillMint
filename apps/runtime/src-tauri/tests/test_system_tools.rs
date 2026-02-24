use runtime_lib::agent::{OpenInFolderTool, ScreenshotTool, Tool, ToolContext, ToolRegistry};
use serde_json::json;
use tempfile::TempDir;

// ===== ScreenshotTool 测试 =====

#[test]
fn test_screenshot_tool_name() {
    let tool = ScreenshotTool;
    assert_eq!(tool.name(), "screenshot");
}

#[test]
fn test_screenshot_tool_schema() {
    let tool = ScreenshotTool;
    let schema = tool.input_schema();

    // 验证 schema 包含 path 参数
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["path"].is_object());
    assert_eq!(schema["properties"]["path"]["type"], "string");

    // 验证 path 是必填参数
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("path")));
}

#[test]
fn test_screenshot_tool_missing_path() {
    let tool = ScreenshotTool;
    let ctx = ToolContext::default();
    let input = json!({});
    let result = tool.execute(input, &ctx);

    assert!(result.is_err(), "缺少 path 参数应该返回错误");
    assert!(result.unwrap_err().to_string().contains("缺少 path 参数"));
}

// ===== OpenInFolderTool 测试 =====

#[test]
fn test_open_in_folder_tool_name() {
    let tool = OpenInFolderTool;
    assert_eq!(tool.name(), "open_in_folder");
}

#[test]
fn test_open_in_folder_tool_schema() {
    let tool = OpenInFolderTool;
    let schema = tool.input_schema();

    // 验证 schema 包含 path 参数
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["path"].is_object());
    assert_eq!(schema["properties"]["path"]["type"], "string");

    // 验证 path 是必填参数
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("path")));
}

#[test]
fn test_open_in_folder_missing_path() {
    let tool = OpenInFolderTool;
    let ctx = ToolContext::default();
    let input = json!({});
    let result = tool.execute(input, &ctx);

    assert!(result.is_err(), "缺少 path 参数应该返回错误");
    assert!(result.unwrap_err().to_string().contains("缺少 path 参数"));
}

#[test]
fn test_open_in_folder_nonexistent_path() {
    let tool = OpenInFolderTool;
    let ctx = ToolContext::default();
    let input = json!({ "path": "/tmp/nonexistent_path_system_tools_test_xyz123" });
    let result = tool.execute(input, &ctx);

    assert!(result.is_err(), "不存在的路径应该返回错误");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("路径不存在"),
        "错误消息应包含'路径不存在'，实际: {}",
        err_msg
    );
}

// ===== 注册表集成测试 =====

#[test]
fn test_system_tools_registered_in_registry() {
    let registry = ToolRegistry::with_file_tools();

    // 验证 screenshot 工具已注册
    assert!(
        registry.get("screenshot").is_some(),
        "screenshot 工具应该已注册"
    );

    // 验证 open_in_folder 工具已注册
    assert!(
        registry.get("open_in_folder").is_some(),
        "open_in_folder 工具应该已注册"
    );
}

#[test]
fn test_open_in_folder_with_existing_dir() {
    // 创建临时目录，验证对真实存在的目录不会报错
    // 注意：此测试不会真正打开文件管理器窗口，
    // 只验证 spawn 调用不会 panic
    let tmp = TempDir::new().unwrap();
    let tool = OpenInFolderTool;
    let ctx = ToolContext::default();
    let input = json!({ "path": tmp.path().to_str().unwrap() });

    // 在 CI 环境中 explorer/open/xdg-open 可能会失败，
    // 但不应该 panic，只可能返回 Err
    let result = tool.execute(input, &ctx);
    // 仅验证返回了某种结果（成功或错误），不 panic
    let _ = result;
}
