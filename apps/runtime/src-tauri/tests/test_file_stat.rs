use runtime_lib::agent::{FileStatTool, Tool, ToolContext};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_stat_regular_file() {
    // 创建临时文件用于测试
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("test_stat.txt");
    fs::write(&file_path, "hello world").unwrap();

    let tool = FileStatTool;
    let ctx = ToolContext::default();
    let input = json!({ "path": file_path.to_str().unwrap() });
    let result = tool.execute(input, &ctx).unwrap();

    // 解析 JSON 输出
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["type"], "file");
    assert_eq!(parsed["size"], 11); // "hello world" = 11 字节
    assert_eq!(parsed["readonly"], false);
    // 验证 modified 字段存在且为字符串
    assert!(parsed["modified"].is_string(), "modified 应该是字符串");
}

#[test]
fn test_file_stat_directory() {
    // 创建临时目录用于测试
    let tmp = TempDir::new().unwrap();
    let dir_path = tmp.path().join("test_dir");
    fs::create_dir(&dir_path).unwrap();

    let tool = FileStatTool;
    let ctx = ToolContext::default();
    let input = json!({ "path": dir_path.to_str().unwrap() });
    let result = tool.execute(input, &ctx).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["type"], "directory");
    assert_eq!(parsed["readonly"], false);
    assert!(parsed["modified"].is_string(), "modified 应该是字符串");
}

#[test]
fn test_file_stat_nonexistent_path() {
    // 不存在的路径应返回错误
    let tool = FileStatTool;
    let ctx = ToolContext::default();
    let input = json!({ "path": "/tmp/nonexistent_path_abc123xyz" });
    let result = tool.execute(input, &ctx);

    assert!(result.is_err(), "不存在的路径应该返回错误");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("获取文件元信息失败"),
        "错误消息应包含'获取文件元信息失败'，实际: {}",
        err_msg
    );
}

#[test]
fn test_file_stat_missing_path_param() {
    // 缺少 path 参数应返回错误
    let tool = FileStatTool;
    let ctx = ToolContext::default();
    let input = json!({});
    let result = tool.execute(input, &ctx);

    assert!(result.is_err(), "缺少参数应该返回错误");
    assert!(result.unwrap_err().to_string().contains("缺少 path 参数"));
}
