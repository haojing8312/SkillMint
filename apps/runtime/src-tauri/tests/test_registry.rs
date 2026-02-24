use runtime_lib::agent::ToolRegistry;

#[test]
fn test_registry_with_file_tools() {
    let registry = ToolRegistry::with_file_tools();

    // 原有 8 个基础工具
    assert!(registry.get("read_file").is_some());
    assert!(registry.get("write_file").is_some());
    assert!(registry.get("glob").is_some());
    assert!(registry.get("grep").is_some());
    assert!(registry.get("edit").is_some());
    assert!(registry.get("todo_write").is_some());
    assert!(registry.get("web_fetch").is_some());
    assert!(registry.get("bash").is_some());
    // L2 新增 5 个文件工具
    assert!(registry.get("list_dir").is_some());
    assert!(registry.get("file_stat").is_some());
    assert!(registry.get("file_delete").is_some());
    assert!(registry.get("file_move").is_some());
    assert!(registry.get("file_copy").is_some());

    let defs = registry.get_tool_definitions();
    assert_eq!(defs.len(), 13);
}
