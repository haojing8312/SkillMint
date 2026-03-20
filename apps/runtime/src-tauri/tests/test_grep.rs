use runtime_lib::agent::{GrepTool, Tool, ToolContext};
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
    let ctx = ToolContext::default();
    let input = json!({
        "pattern": "hello",
        "path": test_file
    });

    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid json payload");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["tool"], "grep");
    assert_eq!(parsed["details"]["total_matches"], 2);
    assert_eq!(parsed["details"]["files_searched"], 1);
    let matches = parsed["details"]["matches"]
        .as_array()
        .expect("matches array");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0]["line"], 1);
    assert_eq!(matches[0]["text"], "line 1: hello");
    assert_eq!(matches[1]["line"], 3);
    assert_eq!(matches[1]["text"], "line 3: hello world");

    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_grep_case_insensitive() {
    let test_file = "test_grep_ci.txt";
    fs::write(test_file, "Hello\nHELLO\nhello\n").unwrap();

    let tool = GrepTool;
    let ctx = ToolContext::default();
    let input = json!({
        "pattern": "hello",
        "path": test_file,
        "case_insensitive": true
    });

    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid json payload");
    assert_eq!(parsed["details"]["total_matches"], 3);

    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_grep_no_matches() {
    let test_file = "test_grep_none.txt";
    fs::write(test_file, "foo\nbar\nbaz\n").unwrap();

    let tool = GrepTool;
    let ctx = ToolContext::default();
    let input = json!({
        "pattern": "notfound",
        "path": test_file
    });

    let result = tool.execute(input, &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid json payload");
    assert_eq!(parsed["details"]["total_matches"], 0);
    assert_eq!(parsed["details"]["files_searched"], 1);

    fs::remove_file(test_file).unwrap();
}
