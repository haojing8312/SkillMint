# Rust Desktop Lifecycle Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Shrink `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs` into a thin Tauri shell by moving lifecycle DTOs, filesystem helpers, diagnostics/export logic, and shared database snapshot helpers into focused child modules while preserving current behavior.

**Architecture:** Keep the root file as the stable command surface. Move cohesive helper clusters into `desktop_lifecycle/` submodules, then re-export the shared helper functions so sibling command modules can keep calling them without churn.

**Tech Stack:** Rust, Tauri commands, sqlx, SQLite, zip, WorkClaw runtime tests

---

### Task 1: Create the module skeleton and move the DTOs

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/types.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`

**Step 1: Add the module declaration**

- Add a local `types.rs` module under `desktop_lifecycle/`
- Re-export the public DTOs from the root file

**Step 2: Move the data types**

- Move `DesktopLifecyclePaths`
- Move `DesktopCleanupResult`
- Move `CrashSummaryInfo`
- Move `DesktopDiagnosticsStatus`
- Move `DesktopDiagnosticsExportPayload`
- Move `FrontendDiagnosticPayload`

**Step 3: Add a focused type smoke test if needed**

- Only add a test if moving the DTOs changes visibility or derives

### Task 2: Move filesystem and path helpers

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/filesystem.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`

**Step 1: Move helper functions**

- `resolve_desktop_lifecycle_paths`
- `clear_directory_contents`
- `merge_cleanup_result`
- `open_path_with_system`

**Step 2: Keep the root command wrappers stable**

- `get_desktop_lifecycle_paths`
- `open_desktop_path`
- `clear_desktop_cache_and_logs`

**Step 3: Add or preserve the cleanup tests**

- Keep the top-level cleanup regression coverage with the moved helper

### Task 3: Move shared database snapshot helpers

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/database_snapshot.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Possibly modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Possibly modify: `apps/runtime/src-tauri/src/commands/chat_session_commands.rs`

**Step 1: Move helper functions**

- `collect_database_counts`
- `collect_database_storage_snapshot`

**Step 2: Preserve call sites**

- Keep the existing helper names available to sibling command modules via root re-exports or direct module imports

**Step 3: Verify sibling access still compiles**

- Confirm `chat.rs` and `chat_session_commands.rs` still compile against the same helper surface

### Task 4: Move diagnostics shaping and export logic

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`

**Step 1: Move helper functions**

- `read_last_clean_exit_at`
- `build_diagnostics_status`
- `build_desktop_environment_summary`
- `list_recent_runtime_log_files`
- `list_recent_audit_log_files`
- `export_diagnostics_bundle`
- `record_frontend_diagnostic_event`

**Step 2: Keep the Tauri commands thin**

- `export_desktop_environment_summary`
- `get_desktop_diagnostics_status`
- `open_desktop_diagnostics_dir`
- `export_desktop_diagnostics_bundle`
- `record_frontend_diagnostic_event`

**Step 3: Preserve bundle contents and status shape**

- Do not alter zip entry names or diagnostics JSON unless a regression test requires it

### Task 5: Verify the split and thin the root file

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/*.rs`

**Step 1: Run focused Rust verification**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib desktop_lifecycle -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_feishu_gateway -- --nocapture
pnpm test:rust-fast
```

**Step 2: Check residual risk**

- Confirm the root file is now a thin wrapper and list any still-shared helpers that remain intentionally in the root

**Step 3: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs apps/runtime/src-tauri/src/commands/desktop_lifecycle
git commit -m "refactor(runtime): split desktop lifecycle helpers"
```
