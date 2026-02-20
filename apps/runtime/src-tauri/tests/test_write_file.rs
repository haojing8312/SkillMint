use runtime_lib::agent::{Tool, WriteFileTool};
use serde_json::json;
use std::fs;

#[test]
fn test_write_file_success() {
    let tool = WriteFileTool;
    let test_path = "test_write_output.txt";

    let input = json!({
        "path": test_path,
        "content": "Test content"
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("成功写入"));
    assert!(result.contains(test_path));

    // Verify file was written
    let content = fs::read_to_string(test_path).unwrap();
    assert_eq!(content, "Test content");

    // Cleanup
    fs::remove_file(test_path).unwrap();
}

#[test]
fn test_write_file_creates_parent_dirs() {
    let tool = WriteFileTool;
    let test_path = "test_write_dir/nested/file.txt";

    let input = json!({
        "path": test_path,
        "content": "Nested content"
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("成功写入"));

    // Verify file was written
    let content = fs::read_to_string(test_path).unwrap();
    assert_eq!(content, "Nested content");

    // Cleanup
    fs::remove_dir_all("test_write_dir").unwrap();
}

#[test]
fn test_write_file_missing_params() {
    let tool = WriteFileTool;

    let input = json!({"path": "test.txt"});
    let result = tool.execute(input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("缺少 content 参数"));
}
