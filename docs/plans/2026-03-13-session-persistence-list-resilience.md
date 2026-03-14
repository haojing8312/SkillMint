# Session Persistence List Resilience Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prevent newly created sessions from disappearing from the UI after refresh/restart by making session list loading observable and resilient.

**Architecture:** Keep the existing SQLite-backed session creation flow, but harden the read path so one malformed session row cannot blank the entire list. On the frontend, optimistically retain newly created sessions and report list-loading failures through the diagnostics channel instead of silently replacing the sidebar with an empty state.

**Tech Stack:** React, TypeScript, Vitest, Tauri, Rust, sqlx, SQLite

---

### Task 1: Cover frontend session list failure handling

**Files:**
- Modify: `apps/runtime/src/__tests__/App.session-create-flow.test.tsx`

**Step 1: Write the failing tests**

- Add a sidebar mock that exposes the current session count and first session id.
- Add a test where `create_session` succeeds but `list_sessions` rejects; assert the chat still opens, the sidebar still contains the created session, and `record_frontend_diagnostic_event` is invoked.

**Step 2: Run test to verify it fails**

Run: `pnpm --filter runtime test -- --run src/__tests__/App.session-create-flow.test.tsx`
Expected: FAIL because the app currently clears the session list and does not report the error.

### Task 2: Cover backend `list_sessions` row resilience

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_chat_commands.rs`

**Step 1: Write the failing test**

- Seed one session row with `NULL` title and one normal row.
- Call `list_sessions_with_pool(...)`.
- Assert the result succeeds and returns both rows, normalizing the null-title row to a usable fallback title.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_chat_commands -- --nocapture`
Expected: FAIL because the current typed query requires non-null strings for every row.

### Task 3: Implement minimal frontend resilience

**Files:**
- Modify: `apps/runtime/src/App.tsx`

**Step 1: Add diagnostics helper usage**

- Import the frontend diagnostics reporter or add a local helper.
- On `loadSessions` failure, send a diagnostic event with the command name and error message.

**Step 2: Add optimistic session retention**

- After `create_session` returns, insert a minimal `SessionInfo` into local state before or regardless of the next `list_sessions` refresh failure.
- Preserve existing selection behavior.

**Step 3: Run the targeted frontend tests**

Run: `pnpm --filter runtime test -- --run src/__tests__/App.session-create-flow.test.tsx`
Expected: PASS

### Task 4: Implement minimal backend list normalization

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`

**Step 1: Replace brittle typed row decoding**

- Read nullable fields as `Option<String>` where appropriate.
- Normalize missing/blank titles to a fallback like `New Chat`.
- Preserve existing payload fields and sort order.

**Step 2: Run targeted backend tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_chat_commands -- --nocapture`
Expected: PASS

### Task 5: Verify end-to-end targeted behavior

**Files:**
- Verify only

**Step 1: Run both targeted suites**

Run: `pnpm --filter runtime test -- --run src/__tests__/App.session-create-flow.test.tsx`
Expected: PASS

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_chat_commands -- --nocapture`
Expected: PASS
