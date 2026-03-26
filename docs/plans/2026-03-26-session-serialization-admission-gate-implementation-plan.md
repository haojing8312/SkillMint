# Session Serialization Admission Gate Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce same-session send-message serialization in WorkClaw by adding a runtime admission gate that rejects conflicting sends before user-message persistence.

**Architecture:** Add a dedicated `SessionAdmissionGate` under `agent/runtime/` and manage it in the Tauri app state. Keep `RunRegistry` focused on run identity and projections, while `commands/chat.rs` acquires an admission lease before inserting the user message or starting runtime execution.

**Tech Stack:** Rust, Tauri, sqlx, tokio, Cargo, pnpm

---

### Task 1: Add the Runtime Admission Gate Module

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/admission_gate.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/mod.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/admission_gate.rs`

**Step 1: Write the failing admission-gate tests**

```rust
#[test]
fn admission_gate_rejects_same_session_while_leased() {
    let gate = SessionAdmissionGate::default();
    let _first = gate.try_acquire("session-1").expect("first lease");
    let conflict = gate.try_acquire("session-1").expect_err("conflict");
    assert_eq!(conflict.code(), "SESSION_RUN_CONFLICT");
}
```

**Step 2: Run the targeted test to confirm the missing module surface**

Run: `cargo test admission_gate --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: FAIL on missing `SessionAdmissionGate`

**Step 3: Implement the minimal admission gate**

```rust
pub struct SessionAdmissionGate {
    active_sessions: Mutex<HashSet<String>>,
}

pub struct SessionAdmissionLease { ... }
pub struct SessionAdmissionConflict { ... }
```

Requirements:

- same-session acquisition conflicts
- different sessions can acquire independently
- dropping the lease releases the session
- expose a stable error code helper

**Step 4: Wire the module into runtime exports and app state**

```rust
pub(crate) mod admission_gate;
pub use admission_gate::{SessionAdmissionConflict, SessionAdmissionGate, SessionAdmissionGateState};
```

**Step 5: Re-run the targeted admission-gate tests**

Run: `cargo test admission_gate --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/admission_gate.rs apps/runtime/src-tauri/src/agent/runtime/mod.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "refactor(agent): add session admission gate"
```

### Task 2: Gate `send_message` Before Persistence

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat.rs`

**Step 1: Write the failing command-level regression test**

```rust
#[test]
fn session_run_conflict_error_is_stable() {
    let error = SessionAdmissionConflict::new("session-1").to_string();
    assert!(error.starts_with("SESSION_RUN_CONFLICT:"));
}
```

**Step 2: Run the targeted chat tests**

Run: `cargo test session_run_conflict --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: FAIL before the command-layer wiring exists

**Step 3: Acquire the admission lease at the top of `send_message`**

Implementation requirements:

- read `SessionAdmissionGateState` from app state
- acquire before `insert_session_message_with_pool`
- if acquisition fails, return the structured conflict error immediately
- keep the lease alive through the whole command scope
- do not persist the user message when admission fails

**Step 4: Re-run the targeted chat tests**

Run: `cargo test session_run_conflict --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "fix(chat): reject conflicting same-session sends before persistence"
```

### Task 3: Verify Runtime Serialization Coverage

**Files:**
- Modify: touched files only if verification reveals needed fixes
- Test: existing suites only

**Step 1: Run targeted Tauri tests**

Run: `cargo test admission_gate session_run_conflict --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: PASS

**Step 2: Run Rust fast-path verification**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 3: Run formatting and diff sanity**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add .
git commit -m "test(runtime): verify session admission gate serialization"
```
