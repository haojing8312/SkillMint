use runtime_lib::agent::{GlobTool, Tool};
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
    let input = json!({
        "pattern": "**/*.txt",
        "base_dir": "test_glob_dir"
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("找到 3 个文件"));
    assert!(result.contains("file1.txt"));
    assert!(result.contains("file2.txt"));
    assert!(result.contains("file3.txt"));
    assert!(!result.contains("file.rs"));

    // Cleanup
    fs::remove_dir_all("test_glob_dir").unwrap();
}

#[test]
fn test_glob_no_matches() {
    let tool = GlobTool;
    let input = json!({
        "pattern": "**/*.nonexistent_ext_xyz"
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("找到 0 个文件"));
}
