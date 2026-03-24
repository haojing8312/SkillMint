# Rust Desktop Lifecycle Split Design

**Goal:** Turn `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs` into a thin command shell by extracting path resolution, filesystem cleanup, diagnostics/status/export helpers, and shared database snapshot helpers into focused child modules without changing the existing Tauri command contract.

## Why This File Matters

`desktop_lifecycle.rs` is smaller than the giant command files already split in this repo, but it still mixes several unrelated responsibilities:

- desktop path resolution
- cache/log cleanup
- diagnostics status and environment summary shaping
- diagnostics bundle export
- frontend diagnostic event logging
- shared database snapshot helpers used by other command modules

That makes it a good candidate for the next command-surface split once the larger employee and plugin modules are already under control.

## What The File Actually Contains Today

At the start of this effort, `desktop_lifecycle.rs` is about 606 lines and contains:

- public lifecycle DTOs
- filesystem cleanup helpers
- system-path opening helpers
- diagnostics state shaping
- diagnostics export bundle generation
- database count/storage snapshot helpers
- the `#[tauri::command]` entrypoints that expose those behaviors
- command-adjacent tests for cleanup and diagnostics export behavior

Notably, it does not currently contain a large startup/shutdown orchestration flow or tray lifecycle state machine. So the split should focus on the actual responsibilities present in the file, not on inventing layers that are not yet there.

## Recommended Split

Create a small module set under `apps/runtime/src-tauri/src/commands/desktop_lifecycle/`:

- `types.rs`
- `filesystem.rs`
- `diagnostics_service.rs`
- `database_snapshot.rs`

### `types.rs`

Own the public DTOs and payloads:

- `DesktopLifecyclePaths`
- `DesktopCleanupResult`
- `CrashSummaryInfo`
- `DesktopDiagnosticsStatus`
- `DesktopDiagnosticsExportPayload`
- `FrontendDiagnosticPayload`

### `filesystem.rs`

Own the desktop path and cleanup helpers:

- `resolve_desktop_lifecycle_paths`
- `clear_directory_contents`
- `merge_cleanup_result`
- `open_path_with_system`

### `diagnostics_service.rs`

Own the diagnostics-facing shaping and export behavior:

- `read_last_clean_exit_at`
- `build_diagnostics_status`
- `build_desktop_environment_summary`
- `list_recent_runtime_log_files`
- `list_recent_audit_log_files`
- `export_diagnostics_bundle`
- `record_frontend_diagnostic_event`

### `database_snapshot.rs`

Own the shared runtime snapshot helpers that other command modules already use:

- `collect_database_counts`
- `collect_database_storage_snapshot`

This keeps the root file from becoming a grab bag for unrelated utility code while preserving the current public helper surface for `chat.rs` and `chat_session_commands.rs`.

## Why This Shape Is Preferred

There are three possible directions:

1. Recommended: split by responsibility into `types`, `filesystem`, `diagnostics_service`, and `database_snapshot`.
2. Conservative: only move the diagnostics export path and leave shared helpers in the root.
3. Over-split: create a one-file-per-command/helper layout.

The recommended shape is the best fit because it:

- preserves stable command entrypoints
- keeps cross-file helper reuse explicit
- avoids micro-file sprawl
- gives the root file a clear thin-shell role

## Compatibility Rules

- Existing Tauri command names must stay the same.
- `chat.rs` and `chat_session_commands.rs` should continue to access the shared database snapshot helpers without behavioral changes.
- Diagnostics bundle contents must remain byte-for-byte compatible unless a test explicitly documents a change.
- File deletion and directory open behavior must remain identical from the user’s perspective.

## Testing Strategy

Preserve the existing tests and move them with the helpers they cover:

- cleanup behavior tests should live with `filesystem.rs`
- diagnostics summary and bundle export tests should live with `diagnostics_service.rs`
- shared snapshot helper tests should live with `database_snapshot.rs`
- the root file should keep only thin command-level smoke tests if any remain necessary

## Success Criteria

- `desktop_lifecycle.rs` becomes a thin entrypoint shell
- path/cleanup logic no longer lives in the root file
- diagnostics export logic no longer lives in the root file
- shared database snapshot helpers are isolated from the command shell
- existing desktop diagnostics and cleanup behavior remains unchanged
