# Rust Chat Session IO Split Design

**Goal:** Turn [chat_session_io.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io.rs) into the next formal Rust data-plane split after `employee_agents`, by extracting session storage, session view/rendering, export, and compaction responsibilities into focused child modules without changing the current Tauri-facing command contract.

## Result

This split has now been executed.

- the root file was reduced from `2047` lines to `757`
- the visible call surface stayed stable for [chat_session_commands.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_commands.rs) and [chat_compaction.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_compaction.rs)
- the implementation now lives in focused child modules:
  - [session_store.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io/session_store.rs)
  - [session_view.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io/session_view.rs)
  - [session_export.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io/session_export.rs)
  - [session_compaction.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io/session_compaction.rs)
- the root [chat_session_io.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io.rs) now acts as a compatibility facade plus a small residual test container

## Why This Was Next

[chat_session_io.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io.rs) is currently 2047 lines and mixes several unrelated responsibilities in one file:

- session creation, list, search, update, delete, and message reads
- legacy-schema detection for `im_thread_sessions.channel`
- IM source-channel normalization and display-title derivation
- export markdown rendering, including recovery sections and tool-call formatting
- compaction input extraction and compacted-message replacement
- file export helpers and session formatting helpers
- module-local tests for all of the above

That makes it a strong next target for the same split pattern that already worked for `employee_agents`. It is also a safe parallel target with `feishu_gateway` because it lives entirely inside the chat/session data plane and does not share external protocol, relay, or approval logic with the Feishu gateway split.

## Starting Problem

Today the file acts like a shared session I/O hub, but it is actually carrying four different kinds of work:

1. session storage queries and writes
2. session list/search shaping for the UI
3. export formatting and recovered-run reconstruction
4. compaction input/output transformation

The file also contains compatibility logic for legacy SQLite layouts, especially the optional `channel` column on `im_thread_sessions`. That is valuable, but it raises the cost of editing the file because a seemingly small change can accidentally break old databases or session list rendering.

## Final Design

### 1. Keep the root file as a thin compatibility facade

The root [chat_session_io.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io.rs) should stop being the place where new logic lands. Its end state should be:

- public re-exports for sibling modules and callers
- a minimal compatibility layer for any remaining shared helpers
- no large blocks of SQL, markdown rendering, or compaction transformation logic

That keeps the external call sites stable while letting the internals move into smaller modules.

### 2. Split by concern, not by helper count

The delivered child modules under `apps/runtime/src-tauri/src/commands/chat_session_io/` are:

- `session_store.rs`
  - `create_session_with_pool`
  - `get_messages_with_pool`
  - `list_sessions_with_pool`
  - `search_sessions_global_with_pool`
  - `update_session_workspace_with_pool`
  - `delete_session_with_pool`
  - storage-facing SQL and row shaping for session records
- `session_view.rs`
  - `resolve_im_session_source`
  - `im_thread_sessions_has_channel_column`
  - `derive_session_display_title_with_pool`
  - `normalize_stream_items`
  - `render_user_content_parts`
  - message shaping for session list and display use cases
- `session_export.rs`
  - `export_session_markdown_with_pool`
  - `write_export_file_to_path`
  - export tool-call parsing and markdown rendering helpers
  - recovered-run section rendering
- `session_compaction.rs`
  - `load_compaction_inputs_with_pool`
  - `replace_messages_with_compacted_with_pool`
  - compaction-oriented content normalization

### 3. Preserve the visible contracts

The split should keep the following behavior stable:

- session list ordering and field names
- fallback title behavior for generic sessions
- legacy `im_thread_sessions` schema support
- export markdown structure, including recovered-run sections
- compaction input normalization for assistant and user messages
- file export behavior and error strings

The goal is to shrink the file, not to redesign the chat/session user experience.

## Delivered Responsibility Split

### Session storage layer

Own the straightforward persistence operations:

- create a session row
- read messages
- update workspace
- delete session and messages
- perform session search

This layer should contain the SQL and row mapping, but not markdown rendering or compaction logic.

### Session view layer

Own the shaping logic used by the UI:

- source-channel normalization
- display title derivation
- runtime-status projection
- assistant content normalization
- user content part rendering

This keeps the UI-facing behavior in one place and makes compatibility fallbacks easier to test.

### Session export layer

Own the markdown export path:

- render messages
- reconstruct recovered runs from the journal
- format tool calls and tool outputs
- keep the export file helper near the export logic

This is the most text-heavy part of the file, so it should be isolated from storage and compaction concerns.

### Session compaction layer

Own the compaction-specific transformation path:

- read the session history into compactable messages
- resolve model configuration
- replace compacted messages back into the database

This path is separate enough from export that it should not share a large helper bucket with it.

## Why This Was Safe To Run In Parallel With `feishu_gateway`

This split can proceed alongside [feishu_gateway.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/feishu_gateway.rs) because the two efforts touch different risk surfaces:

- `chat_session_io` is internal session persistence and rendering
- `feishu_gateway` is external protocol ingestion, pairing, approval, relay, and outbound delivery
- `chat_session_io` already has thin wrappers in [chat_session_commands.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_commands.rs) and [chat_compaction.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_compaction.rs)
- the main shared dependencies are read-only helpers like `chat_runtime_io` and `session_journal`
- the main regression suites are different, so focused verification can happen independently

That means one split can be stabilized without waiting for the other one to finish.

## Risks

- changing the session list projection can break UI expectations
- altering `im_thread_sessions.channel` fallback logic can break older databases
- changing export rendering can alter recovered-run output or tool-call formatting
- moving compaction helpers too aggressively can accidentally duplicate normalization rules
- creating one new giant child file instead of several focused ones

## Verification Achieved

The split was validated with:

- `cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_sessions_with_pool_tolerates_legacy_im_thread_sessions_without_channel -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib export_session_markdown_includes_structured_run_stopped_summary -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib resolve_im_session_source_maps_wecom_and_feishu_labels -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib load_compaction_inputs_with_pool_renders_user_content_parts -- --nocapture`
- `pnpm test:rust-fast`

The targeted checks passed, and the root file is now below the `800` split-design threshold.

## Success Criteria

- [chat_session_io.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io.rs) is materially smaller and reads like a compatibility facade
- session storage logic no longer lives beside export markdown rendering
- session view/render helpers are isolated from compaction helpers
- legacy-schema behavior for session listing still works
- export markdown and compaction behavior remain unchanged from the caller’s perspective
- the same pattern can be reused for other large session-facing Rust modules
