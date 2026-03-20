use runtime_lib::agent::{GlobTool, Tool, ToolContext};
use serde_json::json;
use std::fs;

#[test]
fn test_glob_find_files() {
    // Setup test files
    fs::create_dir_all("test_glob_dir/subdir").unwrap();
    fs::write("test_glob_dir/file1.txt", "").unwrap();
    fs::write("test_glob_dir/file2.txt", "").unwrap();
    fs::write("test_glob_dir/subdir/file3.txt", "").unwrap();
    fs::write("test_glob_dir/file.rs", "").unwrap();

    let tool = GlobTool;
    let ctx = ToolContext::default();
    let input = json!({
        "pattern": "**/*.txt",
        "base_dir": "test_glob_dir"
    });

    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["tool"], "glob");
    assert!(parsed["summary"]
        .as_str()
        .unwrap()
        .contains("找到 3 个文件"));
    let matches = parsed["details"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 3);
    let joined = matches
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("file1.txt"));
    assert!(joined.contains("file2.txt"));
    assert!(joined.contains("file3.txt"));
    assert!(!joined.contains("file.rs"));

    // Cleanup
    fs::remove_dir_all("test_glob_dir").unwrap();
}

#[test]
fn test_glob_no_matches() {
    let tool = GlobTool;
    let ctx = ToolContext::default();
    let input = json!({
        "pattern": "**/*.nonexistent_ext_xyz"
    });

    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["details"]["match_count"], 0);
    assert_eq!(parsed["details"]["matches"], json!([]));
}
