# Rust Chat Session IO Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src-tauri/src/commands/chat_session_io.rs` into focused session storage, session view, session export, and session compaction modules while preserving the current public command surface and legacy-schema behavior.

**Architecture:** Keep `chat_session_commands.rs` and `chat_compaction.rs` as thin callers. Move the real logic out of `chat_session_io.rs` into concern-based child modules under `chat_session_io/`, and leave the root file as a compatibility facade with re-exports. Preserve the current field shapes, markdown output, and SQLite fallback behavior throughout the split.

**Tech Stack:** Rust, Tauri command wrappers, sqlx, SQLite, session journal, WorkClaw runtime tests

---

## Outcome

This plan has now been executed:

- `chat_session_io.rs` was reduced to 757 lines
- session storage, session view/rendering, session export, and session compaction were split into dedicated child modules
- `chat_session_commands.rs` and `chat_compaction.rs` kept their caller-facing behavior
- a focused compaction regression test was added in the new child-module structure

## Delivered Module Set

Delivered module set:

- `chat_session_io.rs`
- `chat_session_io/session_store.rs`
- `chat_session_io/session_view.rs`
- `chat_session_io/session_export.rs`
- `chat_session_io/session_compaction.rs`

## Verification Achieved

The split was verified with focused Rust checks:

- `list_sessions_with_pool_tolerates_null_titles`
- `list_sessions_with_pool_tolerates_legacy_im_thread_sessions_without_channel`
- `resolve_im_session_source_maps_wecom_and_feishu_labels`
- `load_compaction_inputs_with_pool_renders_user_content_parts`
- `export_session_markdown_includes_structured_run_stopped_summary`
- `cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml`
- `pnpm test:rust-fast`

## Follow-on Work

1. Treat `chat_session_io` as the next large Rust session-data split target after `employee_agents`.
2. Keep `chat_session_commands.rs` thin so the Tauri command boundary does not grow while the I/O layer is being split.
3. Keep the `feishu_gateway` split moving in parallel, because the two efforts do not share the same external behavior surface.

## Historical Task Log

### Task 1: Create the session storage module

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/chat_session_io/session_store.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat_session_io.rs` module tests
- Test: `apps/runtime/src-tauri/tests/test_chat_commands.rs`

**Step 1: Move the session storage helpers**

- Move `create_session_with_pool`
- Move `get_messages_with_pool`
- Move `list_sessions_with_pool`
- Move `search_sessions_global_with_pool`
- Move `update_session_workspace_with_pool`
- Move `delete_session_with_pool`

Keep the SQL and row shaping in the new module, but do not change output fields or ordering.

**Step 2: Keep the root file compiling**

- Re-export the moved functions from the root module
- Keep the public API visible to `chat_session_commands.rs`
- Do not change the Tauri command wrappers yet

**Step 3: Verify the storage behavior**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib create_session_with_pool -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_sessions_with_pool_tolerates_null_titles -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_sessions_with_pool_tolerates_legacy_im_thread_sessions_without_channel -- --nocapture
```

Expected: PASS

**Step 4: Verify the existing runtime-facing smoke coverage**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS

### Task 2: Extract session view and compatibility helpers

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/chat_session_io/session_view.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat_session_io.rs` module tests

**Step 1: Move the display and normalization helpers**

- Move `resolve_im_session_source`
- Move `im_thread_sessions_has_channel_column`
- Move `derive_session_display_title_with_pool`
- Move `normalize_stream_items`
- Move `render_user_content_parts`

Keep the legacy `im_thread_sessions.channel` fallback intact.

**Step 2: Keep list/search output identical**

- Preserve the session list shape
- Preserve `display_title`
- Preserve `runtime_status`
- Preserve `source_channel` and `source_label`

**Step 3: Verify the view behavior**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_sessions_with_pool_derives_display_title_for_general_sessions -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_sessions_with_pool_projects_runtime_status -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib render_user_content_parts_formats_images_and_text_files -- --nocapture
```

Expected: PASS

### Task 3: Extract session export rendering

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/chat_session_io/session_export.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat_session_io.rs` module tests
- Test: `apps/runtime/src-tauri/tests/test_session_export_recovery.rs`

**Step 1: Move export-specific helpers**

- Move `ExportToolCall`
- Move `ExportRunStopSummary`
- Move `export_session_markdown_with_pool`
- Move `write_export_file_to_path`
- Move `load_export_tool_calls_with_pool`
- Move `load_export_run_stop_summaries_with_pool`
- Move `render_export_message_content`
- Move `render_export_tool_call`
- Move `render_export_tool_call_entry`
- Move `render_export_tool_status`
- Move `render_export_tool_output`
- Move `parse_export_tool_output`
- Move `compact_export_tool_details`
- Move `render_recovered_run_sections`
- Move `export_status_label`

Do not change the markdown structure or recovery-section wording.

**Step 2: Keep export callers stable**

- Preserve the `chat_session_commands.rs` wrapper
- Preserve the `chat.rs` re-export path
- Preserve the current export file behavior and error strings

**Step 3: Verify export behavior**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib export_session_markdown_includes_structured_run_stopped_summary -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib export_session_markdown_skips_recovered_buffer_when_structured_assistant_text_matches -- --nocapture
```

Expected: PASS

Then run:

```bash
pnpm test:rust-fast
```

Expected: PASS

### Task 4: Extract session compaction helpers

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/chat_session_io/session_compaction.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat_compaction.rs`

**Step 1: Move compaction helpers**

- Move `load_compaction_inputs_with_pool`
- Move `replace_messages_with_compacted_with_pool`

Keep the message normalization rules unchanged for assistant and user content.

**Step 2: Keep `chat_compaction.rs` working without broad changes**

- Preserve the existing command entrypoint
- Make sure the compaction route still calls the same public helper names

**Step 3: Verify compaction behavior**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS

If a focused compaction regression test is added, run it directly before the fast-path suite.

### Task 5: Thin the root facade and clean up imports

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_commands.rs` if any import paths change
- Modify: `apps/runtime/src-tauri/src/commands/chat_compaction.rs` if any import paths change
- Test: `apps/runtime/src-tauri/tests/test_session_export_recovery.rs`
- Test: `apps/runtime/src-tauri/tests/test_chat_commands.rs`

**Step 1: Reduce the root file to a facade**

- Keep only the minimum compatibility glue needed by callers
- Re-export the child-module functions the existing callers already use
- Remove any remaining direct SQL or markdown rendering from the root file

**Step 2: Re-run the existing targeted tests**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_sessions_with_pool_tolerates_legacy_im_thread_sessions_without_channel -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib export_session_markdown_includes_structured_run_stopped_summary -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib create_session_with_pool_normalizes_session_fields -- --nocapture
pnpm test:rust-fast
```

Expected: PASS

**Step 3: Commit once the split is stable**

```bash
git add apps/runtime/src-tauri/src/commands/chat_session_io.rs apps/runtime/src-tauri/src/commands/chat_session_io/*.rs apps/runtime/src-tauri/src/commands/chat_session_commands.rs apps/runtime/src-tauri/src/commands/chat_compaction.rs apps/runtime/src-tauri/tests
git commit -m "refactor(runtime): split chat session io"
```
