# src-tauri Phase 4A.5 Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add focused `src-tauri` integration evidence for the Phase 4A `runtime-chat-app` extraction without expanding architecture scope.

**Architecture:** Keep `runtime-chat-app` unchanged except for test-driven fixes. Add SQL-backed adapter tests for `chat_repo`, then add narrow command-level smoke tests for `chat.rs`. Finish by documenting the remaining `chat.rs` responsibilities that belong to Phase 4B.

**Tech Stack:** Rust, Tauri, SQLx, Tokio, workspace Cargo tests

---

### Task 1: Add `chat_repo` Adapter Tests

**Files:**
- Create: `apps/runtime/src-tauri/tests/test_chat_repo.rs`
- Read: `apps/runtime/src-tauri/src/commands/chat_repo.rs`
- Read: `apps/runtime/src-tauri/tests/helpers/mod.rs`
- Read: `apps/runtime/src-tauri/src/commands/models.rs`

**Step 1: Write the failing tests**

Cover these adapter methods with small DB-backed tests:

- `load_routing_settings`
- `load_chat_routing`
- `load_route_policy`
- `resolve_default_model_id`
- `resolve_default_usable_model_id`
- `load_session_model`

**Step 2: Run the new test file and verify failures**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-repo-red -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_repo -- --nocapture
```

Expected: at least one failure caused by missing test file or incomplete assertions.

**Step 3: Implement the minimal test setup**

Use the existing SQL helper patterns from `tests/helpers/mod.rs`. Seed only the rows needed by each adapter method.

**Step 4: Run the adapter tests and verify pass**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-repo-green -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_repo -- --nocapture
```

Expected: all `test_chat_repo` tests pass.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_chat_repo.rs
git commit -m "test(chat): cover chat repo adapter"
```

### Task 2: Add Narrow `chat.rs` Smoke Tests

**Files:**
- Create or Modify: `apps/runtime/src-tauri/tests/test_chat_commands.rs`
- Read: `apps/runtime/src-tauri/src/commands/chat.rs`
- Read: `apps/runtime/src-tauri/tests/helpers/mod.rs`

**Step 1: Write the failing tests**

Add one or two narrow command-level tests:

- `create_session` stores normalized session metadata
- one preparation-oriented smoke test that proves required chat configuration can be read and used without running the full agent loop

Prefer direct command invocation over UI/event-heavy flows.

**Step 2: Run the new command smoke tests and verify failures**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-commands-red -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_commands -- --nocapture
```

Expected: at least one failure until the test harness and assertions are correct.

**Step 3: Make the minimal code adjustments if tests expose wiring gaps**

Only fix defects directly required to make the new tests pass. Do not start Phase 4B extraction here.

**Step 4: Run the command smoke tests and verify pass**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-commands-green -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_commands -- --nocapture
```

Expected: all `test_chat_commands` tests pass.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_chat_commands.rs apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "test(chat): add command smoke coverage"
```

### Task 3: Re-Verify Phase 4A Boundary

**Files:**
- Read: `packages/runtime-chat-app/**`
- Read: `apps/runtime/src-tauri/src/commands/chat.rs`
- Read: `apps/runtime/src-tauri/src/commands/chat_repo.rs`

**Step 1: Re-run `runtime-chat-app` tests**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-crate-final -- test --manifest-path packages/runtime-chat-app/Cargo.toml -- --nocapture
```

Expected: all tests pass.

**Step 2: Re-run `src-tauri` library compile**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-lib-final -- check --manifest-path apps/runtime/src-tauri/Cargo.toml --lib --message-format short
```

Expected: compile succeeds.

**Step 3: Re-run the new `src-tauri` chat-focused tests**

Run:

```bash
node scripts/run-cargo-isolated.mjs phase4a5-chat-tests-final -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_repo --test test_chat_commands -- --nocapture
```

Expected: both test targets pass.

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_chat_repo.rs apps/runtime/src-tauri/tests/test_chat_commands.rs
git commit -m "test(chat): harden phase 4a integration"
```

### Task 4: Document Phase 4B Candidates

**Files:**
- Create: `docs/plans/2026-03-11-src-tauri-phase4b-chat-candidates.md`
- Read: `apps/runtime/src-tauri/src/commands/chat.rs`
- Read: `packages/runtime-chat-app/src/service.rs`

**Step 1: Write a short inventory**

Document the responsibilities still left in `chat.rs` that are likely future app-layer candidates.

**Step 2: Keep the inventory descriptive, not prescriptive**

Do not produce another large design here. Just record what remains and why it was intentionally deferred.

**Step 3: Commit**

```bash
git add docs/plans/2026-03-11-src-tauri-phase4b-chat-candidates.md
git commit -m "docs(chat): record phase 4b candidates"
```

Plan complete and saved to `docs/plans/2026-03-11-src-tauri-phase4a5-hardening-plan.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
