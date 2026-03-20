# Audit Log Diagnostics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add low-noise, append-only audit logs for key runtime/session lifecycle events so support can diagnose session-loss and user-action disputes from exported diagnostics.

**Architecture:** Reuse the existing diagnostics directory and JSONL logging pattern. Add a dedicated audit log writer under diagnostics, instrument a narrow set of Tauri command and lifecycle boundaries, and extend the diagnostics bundle export to include the new audit files and relevant database file metadata.

**Tech Stack:** Tauri, Rust, SQLite, existing diagnostics JSONL export flow, Vitest/ Rust tests.

---

### Task 1: Add Dedicated Audit Log Support

**Files:**
- Modify: `apps/runtime/src-tauri/src/diagnostics.rs`
- Test: `apps/runtime/src-tauri/src/diagnostics.rs`

**Step 1: Write the failing test**

Add a Rust unit test that verifies diagnostics initialization creates an audit directory and that a new audit writer appends JSONL records there.

**Step 2: Run test to verify it fails**

Run: `pnpm test:rust-fast`
Expected: FAIL because the audit directory/writer does not exist yet.

**Step 3: Write minimal implementation**

Add `audit_dir` to `DiagnosticsPaths`, ensure it is created, and add a small append-only audit writer API that mirrors the existing runtime log writer but uses a separate daily `audit-YYYY-MM-DD.jsonl` file.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast`
Expected: PASS for the new diagnostics tests.

### Task 2: Instrument Key Audit Events

**Files:**
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_commands.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs` if helper snapshots are needed
- Test: `apps/runtime/src-tauri/tests/` or focused module tests near touched files

**Step 1: Write the failing tests**

Add focused tests around the new helper functions or command-level behavior for these audit events:
- startup recovery snapshot
- clean shutdown snapshot
- create session
- delete session
- export session

**Step 2: Run test to verify it fails**

Run: `pnpm test:rust-fast`
Expected: FAIL because audit events are not emitted yet.

**Step 3: Write minimal implementation**

Log only append-only audit records with small, stable fields:
- event name
- run id when available
- session id when available
- session/message counts when cheaply available
- db path and db/wal/shm existence + sizes for startup/shutdown/export snapshots

Keep instrumentation side-effect free: log failures must never break the primary command path.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast`
Expected: PASS with the new audit coverage.

### Task 3: Include Audit Logs in Diagnostics Export

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Test: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs` or existing related tests

**Step 1: Write the failing test**

Add a test that exports the diagnostics bundle and verifies audit log files are included when present.

**Step 2: Run test to verify it fails**

Run: `pnpm test:rust-fast`
Expected: FAIL because audit logs are not included yet.

**Step 3: Write minimal implementation**

Extend the existing diagnostics export payload to collect recent audit JSONL files and include them in the zip alongside runtime logs.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast`
Expected: PASS for diagnostics bundle export tests.

### Task 4: Verify User-Facing Diagnostics Flow Still Works

**Files:**
- Modify: `apps/runtime/src/components/__tests__/SettingsView.data-retention.test.tsx` if the existing export action needs stronger assertions
- Test: `apps/runtime/src/components/__tests__/SettingsView.data-retention.test.tsx`

**Step 1: Write or update the failing test**

Assert the diagnostics export action still works without any new user-visible settings or prompts.

**Step 2: Run test to verify it fails if behavior drifted**

Run: `pnpm test:rust-fast`
Expected: No frontend failures are required yet unless the export surface changes.

**Step 3: Write minimal implementation**

Only update tests if needed; avoid UI changes.

**Step 4: Run targeted verification**

Run: `pnpm test:rust-fast`
Run: `pnpm build:runtime`
Expected: PASS, proving the Tauri + runtime integration still builds.
