use runtime_lib::agent::permissions::PermissionMode;

// ── Default 模式 ─────────────────────────────────────────────────────────────

#[test]
fn test_default_write_file_needs_confirmation() {
    // Default 模式下 write_file 需要确认
    assert!(PermissionMode::Default.needs_confirmation("write_file"));
}

#[test]
fn test_default_edit_needs_confirmation() {
    // Default 模式下 edit 需要确认
    assert!(PermissionMode::Default.needs_confirmation("edit"));
}

#[test]
fn test_default_bash_needs_confirmation() {
    // Default 模式下 bash 需要确认
    assert!(PermissionMode::Default.needs_confirmation("bash"));
}

#[test]
fn test_default_read_file_no_confirmation() {
    // Default 模式下 read_file 不需要确认
    assert!(!PermissionMode::Default.needs_confirmation("read_file"));
}

#[test]
fn test_default_glob_no_confirmation() {
    // Default 模式下 glob 不需要确认
    assert!(!PermissionMode::Default.needs_confirmation("glob"));
}

#[test]
fn test_default_grep_no_confirmation() {
    // Default 模式下 grep 不需要确认
    assert!(!PermissionMode::Default.needs_confirmation("grep"));
}

// ── AcceptEdits 模式 ──────────────────────────────────────────────────────────

#[test]
fn test_accept_edits_bash_needs_confirmation() {
    // AcceptEdits 模式下 bash 仍需确认
    assert!(PermissionMode::AcceptEdits.needs_confirmation("bash"));
}

#[test]
fn test_accept_edits_write_file_no_confirmation() {
    // AcceptEdits 模式下 write_file 自动通过
    assert!(!PermissionMode::AcceptEdits.needs_confirmation("write_file"));
}

#[test]
fn test_accept_edits_edit_no_confirmation() {
    // AcceptEdits 模式下 edit 自动通过
    assert!(!PermissionMode::AcceptEdits.needs_confirmation("edit"));
}

#[test]
fn test_accept_edits_read_file_no_confirmation() {
    // AcceptEdits 模式下 read_file 不需要确认
    assert!(!PermissionMode::AcceptEdits.needs_confirmation("read_file"));
}

#[test]
fn test_accept_edits_glob_no_confirmation() {
    // AcceptEdits 模式下 glob 不需要确认
    assert!(!PermissionMode::AcceptEdits.needs_confirmation("glob"));
}

#[test]
fn test_accept_edits_grep_no_confirmation() {
    // AcceptEdits 模式下 grep 不需要确认
    assert!(!PermissionMode::AcceptEdits.needs_confirmation("grep"));
}

// ── Unrestricted 模式 ─────────────────────────────────────────────────────────

#[test]
fn test_unrestricted_write_file_no_confirmation() {
    // Unrestricted 模式下 write_file 自动通过
    assert!(!PermissionMode::Unrestricted.needs_confirmation("write_file"));
}

#[test]
fn test_unrestricted_edit_no_confirmation() {
    // Unrestricted 模式下 edit 自动通过
    assert!(!PermissionMode::Unrestricted.needs_confirmation("edit"));
}

#[test]
fn test_unrestricted_bash_no_confirmation() {
    // Unrestricted 模式下 bash 自动通过
    assert!(!PermissionMode::Unrestricted.needs_confirmation("bash"));
}

#[test]
fn test_unrestricted_read_file_no_confirmation() {
    // Unrestricted 模式下 read_file 不需要确认
    assert!(!PermissionMode::Unrestricted.needs_confirmation("read_file"));
}

#[test]
fn test_unrestricted_glob_no_confirmation() {
    // Unrestricted 模式下 glob 不需要确认
    assert!(!PermissionMode::Unrestricted.needs_confirmation("glob"));
}

#[test]
fn test_unrestricted_grep_no_confirmation() {
    // Unrestricted 模式下 grep 不需要确认
    assert!(!PermissionMode::Unrestricted.needs_confirmation("grep"));
}

// ── Default trait ─────────────────────────────────────────────────────────────

#[test]
fn test_default_trait_returns_default_variant() {
    // Default::default() 应返回 Default 变体
    assert_eq!(PermissionMode::default(), PermissionMode::Default);
}
