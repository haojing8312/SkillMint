use runtime_lib::agent::tools::{
    register_tool_alias, BashTool, ExecTool, GlobTool, ListDirTool, ReadFileTool,
};
use runtime_lib::agent::ToolRegistry;
use std::sync::Arc;

#[test]
fn test_openclaw_style_aliases_delegate_to_existing_tools() {
    let registry = ToolRegistry::new();
    let read = Arc::new(ReadFileTool);
    let find = Arc::new(GlobTool);
    let ls = Arc::new(ListDirTool);
    let exec = Arc::new(ExecTool::new());
    let bash = Arc::new(BashTool::new());

    registry.register(read.clone());
    registry.register(find.clone());
    registry.register(ls.clone());
    registry.register(bash.clone());
    registry.register(exec.clone());

    register_tool_alias(&registry, "read", read);
    register_tool_alias(&registry, "find", find);
    register_tool_alias(&registry, "ls", ls);

    assert!(registry.get("read").is_some());
    assert!(registry.get("find").is_some());
    assert!(registry.get("ls").is_some());
    assert!(registry.get("exec").is_some());
    assert!(registry.get("bash").is_some());
}
