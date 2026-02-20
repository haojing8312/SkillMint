use runtime_lib::agent::{GrepTool, Tool};
use serde_json::json;
use std::fs;

#[test]
fn test_grep_find_matches() {
    let test_file = "test_grep_matches.txt";
    fs::write(
        test_file,
        "line 1: hello\nline 2: world\nline 3: hello world\n",
    )
    .unwrap();

    let tool = GrepTool;
    let input = json!({
        "pattern": "hello",
        "path": test_file
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("找到 2 处匹配"));
    assert!(result.contains("1:line 1: hello"));
    assert!(result.contains("3:line 3: hello world"));

    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_grep_case_insensitive() {
    let test_file = "test_grep_ci.txt";
    fs::write(test_file, "Hello\nHELLO\nhello\n").unwrap();

    let tool = GrepTool;
    let input = json!({
        "pattern": "hello",
        "path": test_file,
        "case_insensitive": true
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("找到 3 处匹配"));

    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_grep_no_matches() {
    let test_file = "test_grep_none.txt";
    fs::write(test_file, "foo\nbar\nbaz\n").unwrap();

    let tool = GrepTool;
    let input = json!({
        "pattern": "notfound",
        "path": test_file
    });

    let result = tool.execute(input).unwrap();
    assert!(result.contains("找到 0 处匹配"));

    fs::remove_file(test_file).unwrap();
}
