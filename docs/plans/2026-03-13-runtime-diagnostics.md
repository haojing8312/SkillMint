# Runtime Diagnostics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add local runtime diagnostics to WorkClaw so desktop crashes, abnormal exits, frontend exceptions, and key runtime failures produce exportable evidence on the user's machine.

**Architecture:** Introduce a small diagnostics backend inside `apps/runtime/src-tauri` that owns diagnostics directories, structured log writes, crash markers, run-state tracking, and zip export. Expose typed Tauri commands to the React settings UI, then add a diagnostics section and export action in the existing desktop/system settings area.

**Tech Stack:** Rust, Tauri 2, sqlx/SQLite, React, TypeScript, Vitest, Rust unit tests

---

### Task 1: Define diagnostics backend contract

**Files:**
- Create: `apps/runtime/src-tauri/src/diagnostics.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/src/diagnostics.rs`

**Step 1: Write the failing Rust tests**

Add tests for:

- diagnostics directory creation
- writing a JSONL log record
- detecting abnormal previous run from active state file
- listing latest crash summary metadata

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml diagnostics -- --nocapture`

Expected: FAIL because `diagnostics.rs` and tested functions do not exist yet.

**Step 3: Write minimal implementation**

Implement:

- diagnostics root resolution under `app_data_dir/diagnostics`
- `ensure_diagnostics_dirs`
- `write_log_record`
- active run state write/clear helpers
- crash summary file read helper

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml diagnostics -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/diagnostics.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat: add runtime diagnostics backend foundation"
```

### Task 2: Add panic and abnormal-exit capture

**Files:**
- Modify: `apps/runtime/src-tauri/src/diagnostics.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/src/diagnostics.rs`

**Step 1: Write the failing Rust tests**

Add tests for:

- recording a crash summary payload
- startup detection of stale active-run state
- clean shutdown clearing active-run state

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml diagnostics::tests -- --nocapture`

Expected: FAIL because crash recording and stale-run handling are incomplete.

**Step 3: Write minimal implementation**

Implement:

- global diagnostics state setup in app startup
- panic hook installation
- crash summary file write
- startup abnormal-exit detection metadata
- cleanup on normal exit / exit requested path

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml diagnostics::tests -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/diagnostics.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat: capture panic and abnormal desktop exits"
```

### Task 3: Expose diagnostics commands to Tauri

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`

**Step 1: Write the failing Rust tests**

Add tests for:

- diagnostics path summary payload
- export summary text including diagnostics paths/status
- export bundle creation using temp fixtures

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml desktop_lifecycle -- --nocapture`

Expected: FAIL because new diagnostics commands and fields do not exist.

**Step 3: Write minimal implementation**

Add commands:

- `get_desktop_diagnostics_status`
- `open_desktop_diagnostics_dir`
- `export_desktop_diagnostics_bundle`

Also extend environment summary output to include diagnostics paths and abnormal-exit status.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml desktop_lifecycle -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat: expose desktop diagnostics commands"
```

### Task 4: Capture frontend global exceptions

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/diagnostics.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.data-retention.test.tsx`

**Step 1: Write the failing frontend tests**

Add tests for:

- diagnostics status loading in settings
- export diagnostics button invoking the new command
- frontend exception reporter invoking backend command

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.data-retention.test.tsx`

Expected: FAIL because new diagnostics UI and invoke calls do not exist.

**Step 3: Write minimal implementation**

Implement:

- a small diagnostics bootstrap module registering `window.onerror` and `window.onunhandledrejection`
- a backend invoke for recording frontend exception events
- settings diagnostics section with export/open actions and latest status display

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.data-retention.test.tsx`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/diagnostics.ts apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.data-retention.test.tsx
git commit -m "feat: add frontend diagnostics capture and settings actions"
```

### Task 5: Add key runtime event logging

**Files:**
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/sidecar.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Test: `apps/runtime/src-tauri/src/diagnostics.rs`

**Step 1: Write the failing Rust tests**

Add tests for:

- structured log event shape
- sidecar start failure logging
- route attempt / run failure log entry writing helpers

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml diagnostics -- --nocapture`

Expected: FAIL because event logging coverage is incomplete.

**Step 3: Write minimal implementation**

Add explicit diagnostics writes at these points:

- app startup
- db init result
- sidecar start failure / timeout / stop
- send_message entry
- run finalize failure path
- diagnostics export success/failure

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml diagnostics -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/src/sidecar.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/src/diagnostics.rs
git commit -m "feat: log key desktop runtime events"
```

### Task 6: Verify end-to-end and document support workflow

**Files:**
- Modify: `docs/user-manual/08-security.md`
- Create: `docs/troubleshooting/runtime-diagnostics.md`

**Step 1: Write the documentation updates**

Document:

- where diagnostics live
- what export bundle contains
- how support should ask users to export diagnostics
- privacy boundary of the local bundle

**Step 2: Run verification commands**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`

Expected: PASS

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.data-retention.test.tsx`

Expected: PASS

**Step 3: Manual verification**

Run the desktop app, open settings, and verify:

- diagnostics paths show up
- export diagnostics bundle succeeds
- exported zip lands under diagnostics exports directory

**Step 4: Commit**

```bash
git add docs/user-manual/08-security.md docs/troubleshooting/runtime-diagnostics.md
git commit -m "docs: add runtime diagnostics support workflow"
```
