use runtime_lib::agent::{FileDeleteTool, Tool, ToolContext};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// 辅助函数：创建临时工作目录
fn setup_work_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("test_file_delete_{}", name));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_delete_file() {
    // 测试删除单个文件
    let work_dir = setup_work_dir("file");
    let file_path = work_dir.join("test.txt");
    fs::write(&file_path, "hello").unwrap();
    assert!(file_path.exists());

    let tool = FileDeleteTool;
    let ctx = ToolContext {
        work_dir: Some(work_dir.clone()),
    };

    let input = json!({
        "path": file_path.to_str().unwrap()
    });

    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("成功删除"));
    assert!(!file_path.exists());

    // 清理
    fs::remove_dir_all(&work_dir).ok();
}

#[test]
fn test_delete_empty_directory() {
    // 测试删除空目录
    let work_dir = setup_work_dir("empty_dir");
    let empty_dir = work_dir.join("empty");
    fs::create_dir_all(&empty_dir).unwrap();
    assert!(empty_dir.exists());

    let tool = FileDeleteTool;
    let ctx = ToolContext {
        work_dir: Some(work_dir.clone()),
    };

    let input = json!({
        "path": empty_dir.to_str().unwrap()
    });

    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("成功删除"));
    assert!(!empty_dir.exists());

    // 清理
    fs::remove_dir_all(&work_dir).ok();
}

#[test]
fn test_delete_nonempty_dir_without_recursive() {
    // 测试删除非空目录时没有设置 recursive 应该失败
    let work_dir = setup_work_dir("nonempty_no_recursive");
    let dir = work_dir.join("nonempty");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("file.txt"), "content").unwrap();

    let tool = FileDeleteTool;
    let ctx = ToolContext {
        work_dir: Some(work_dir.clone()),
    };

    let input = json!({
        "path": dir.to_str().unwrap()
    });

    let result = tool.execute(input, &ctx);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("非空目录"),
        "错误消息应包含 '非空目录'，实际: {}",
        err_msg
    );

    // 目录应该仍然存在
    assert!(dir.exists());

    // 清理
    fs::remove_dir_all(&work_dir).ok();
}

#[test]
fn test_delete_nonempty_dir_with_recursive() {
    // 测试使用 recursive=true 删除非空目录应该成功
    let work_dir = setup_work_dir("nonempty_recursive");
    let dir = work_dir.join("nonempty");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("file.txt"), "content").unwrap();
    fs::write(dir.join("sub").join("nested.txt"), "nested").unwrap();

    let tool = FileDeleteTool;
    let ctx = ToolContext {
        work_dir: Some(work_dir.clone()),
    };

    let input = json!({
        "path": dir.to_str().unwrap(),
        "recursive": true
    });

    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("成功删除"));
    assert!(!dir.exists());

    // 清理
    fs::remove_dir_all(&work_dir).ok();
}

#[test]
fn test_delete_nonexistent_path() {
    // 测试删除不存在的路径应该报错
    let work_dir = setup_work_dir("nonexistent");

    let tool = FileDeleteTool;
    let ctx = ToolContext {
        work_dir: Some(work_dir.clone()),
    };

    let input = json!({
        "path": work_dir.join("does_not_exist.txt").to_str().unwrap()
    });

    let result = tool.execute(input, &ctx);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("不存在"),
        "错误消息应包含 '不存在'，实际: {}",
        err_msg
    );

    // 清理
    fs::remove_dir_all(&work_dir).ok();
}
