use runtime_lib::agent::{ListDirTool, Tool, ToolContext};
use serde_json::json;
use std::fs;
use std::path::Path;

#[test]
fn test_list_dir_basic() {
    // 创建临时目录结构
    let dir = "test_list_dir_basic_tmp";
    fs::create_dir_all(format!("{}/subdir", dir)).unwrap();
    fs::write(format!("{}/hello.txt", dir), "Hello!").unwrap();
    fs::write(format!("{}/data.bin", dir), vec![0u8; 2048]).unwrap();

    let tool = ListDirTool;
    let ctx = ToolContext::default();
    let input = json!({"path": dir});
    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    let summary = parsed["summary"].as_str().unwrap();
    let details = &parsed["details"];

    // 应包含文件和子目录标记
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["tool"], "list_dir");
    assert!(summary.contains("[FILE]"), "应包含 [FILE] 标记");
    assert!(summary.contains("[DIR]"), "应包含 [DIR] 标记");
    assert!(summary.contains("hello.txt"), "应包含 hello.txt");
    assert!(summary.contains("data.bin"), "应包含 data.bin");
    assert!(summary.contains("subdir"), "应包含 subdir");

    // 文件大小应以人类可读格式显示
    assert!(
        summary.contains("KB") || summary.contains("B"),
        "应显示文件大小"
    );
    assert_eq!(details["entry_count"], 3);
    assert_eq!(details["entries"].as_array().unwrap().len(), 3);

    // 清理
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn test_list_dir_empty() {
    // 创建空目录
    let dir = "test_list_dir_empty_tmp";
    fs::create_dir_all(dir).unwrap();

    let tool = ListDirTool;
    let ctx = ToolContext::default();
    let input = json!({"path": dir});
    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["summary"], "空目录", "空目录应返回 '空目录'");
    assert_eq!(parsed["details"]["entry_count"], 0);
    assert_eq!(parsed["details"]["entries"], json!([]));

    // 清理
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn test_list_dir_nonexistent() {
    let tool = ListDirTool;
    let ctx = ToolContext::default();
    let input = json!({"path": "nonexistent_dir_xyz_12345"});
    let result = tool.execute(input, &ctx);

    assert!(result.is_err(), "不存在的目录应返回错误");
}

#[test]
fn test_list_dir_sorted() {
    // 创建包含多个文件的目录，验证排序
    let dir = "test_list_dir_sorted_tmp";
    fs::create_dir_all(dir).unwrap();
    fs::write(format!("{}/charlie.txt", dir), "c").unwrap();
    fs::write(format!("{}/alpha.txt", dir), "a").unwrap();
    fs::write(format!("{}/bravo.txt", dir), "b").unwrap();

    let tool = ListDirTool;
    let ctx = ToolContext::default();
    let input = json!({"path": dir});
    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    let summary = parsed["summary"].as_str().unwrap();

    // 验证字母序排列：alpha < bravo < charlie
    let alpha_pos = summary.find("alpha.txt").unwrap();
    let bravo_pos = summary.find("bravo.txt").unwrap();
    let charlie_pos = summary.find("charlie.txt").unwrap();
    assert!(alpha_pos < bravo_pos, "alpha 应排在 bravo 前面");
    assert!(bravo_pos < charlie_pos, "bravo 应排在 charlie 前面");

    // 清理
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn test_list_dir_missing_path_param() {
    let tool = ListDirTool;
    let ctx = ToolContext::default();
    let input = json!({});
    let result = tool.execute(input, &ctx);
    assert!(result.is_err(), "缺少 path 参数应返回错误");
    assert!(result.unwrap_err().to_string().contains("缺少 path 参数"));
}

#[test]
fn test_list_dir_appends_structured_entries_with_exact_paths() {
    let dir = "test_list_dir_structured_tmp";
    fs::create_dir_all(dir).unwrap();
    let file_name = "测试记录26.3.16.docx";
    let file_path = format!("{}/{}", dir, file_name);
    fs::write(&file_path, "hello").unwrap();

    let tool = ListDirTool;
    let ctx = ToolContext::default();
    let input = json!({"path": dir});
    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["tool"], "list_dir");
    assert!(
        parsed["summary"]
            .as_str()
            .is_some_and(|summary| summary.contains(file_name)),
        "摘要应保留原始中文文件名"
    );

    let entries = parsed["details"]["entries"]
        .as_array()
        .expect("entries should be an array");
    let first = entries.first().expect("expected one entry");
    assert_eq!(first["name"], file_name);
    assert_eq!(first["kind"], "file");
    assert_eq!(first["size_bytes"], 5);
    assert!(
        first["path"]
            .as_str()
            .is_some_and(|path| Path::new(path).is_absolute() && path.ends_with(file_name)),
        "path should preserve the exact absolute file path"
    );

    fs::remove_dir_all(dir).unwrap();
}
