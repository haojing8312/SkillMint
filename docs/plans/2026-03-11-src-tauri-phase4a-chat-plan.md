# src-tauri Phase 4A Chat Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract pre-execution chat orchestration from `apps/runtime/src-tauri/src/commands/chat.rs` into a new `packages/runtime-chat-app` crate.

**Architecture:** Create a lightweight app-layer crate that owns chat preparation, leave Tauri/runtime adapters in `src-tauri`, and migrate behavior incrementally with narrow read-only traits and focused tests.

**Tech Stack:** Rust, Tauri, SQLx, workspace path crates, Node script-based verification

---

### Task 1: Create `runtime-chat-app` skeleton

**Files:**
- Create: `packages/runtime-chat-app/Cargo.toml`
- Create: `packages/runtime-chat-app/src/lib.rs`
- Create: `packages/runtime-chat-app/src/types.rs`
- Create: `packages/runtime-chat-app/src/traits.rs`
- Create: `packages/runtime-chat-app/src/service.rs`

**Step 1: Write the failing skeleton test**

- Create a small export test file such as `packages/runtime-chat-app/tests/smoke.rs`
- Assert the crate exposes a service and one public type

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path packages/runtime-chat-app/Cargo.toml smoke -- --nocapture
```

Expected: fail because the crate and exports do not yet exist

**Step 3: Write minimal crate implementation**

- Add crate metadata and dependencies
- Export placeholder `ChatPreparationService`
- Export placeholder `PreparedChatExecution`

**Step 4: Run test to verify it passes**

Run the same command.

**Step 5: Commit**

```bash
git add packages/runtime-chat-app
git commit -m "refactor(chat): scaffold runtime chat app crate"
```

### Task 2: Move pure chat preparation helpers

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `packages/runtime-chat-app/src/types.rs`
- Modify: `packages/runtime-chat-app/src/service.rs`
- Test: `packages/runtime-chat-app/tests/modes.rs`
- Test: `packages/runtime-chat-app/tests/capability.rs`
- Test: `packages/runtime-chat-app/tests/retry.rs`
- Test: `packages/runtime-chat-app/tests/fallback.rs`

**Step 1: Write failing tests**

Cover:

- permission/session mode normalization
- capability inference
- retry budget and backoff behavior
- fallback-chain parsing

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path packages/runtime-chat-app/Cargo.toml -- --nocapture
```

Expected: failures because helper behavior is not yet implemented

**Step 3: Implement minimal helper logic**

- Copy only the pure logic from `chat.rs`
- Keep names and semantics stable where possible
- Avoid introducing new abstractions

**Step 4: Re-run crate tests**

Run the same command and confirm green

**Step 5: Commit**

```bash
git add packages/runtime-chat-app apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "refactor(chat): extract pure preparation helpers"
```

### Task 3: Define preparation traits and result model

**Files:**
- Modify: `packages/runtime-chat-app/src/types.rs`
- Modify: `packages/runtime-chat-app/src/traits.rs`
- Modify: `packages/runtime-chat-app/src/service.rs`
- Test: `packages/runtime-chat-app/tests/preparation_types.rs`

**Step 1: Write failing tests**

Add tests that assert:

- `PreparedChatExecution` contains normalized preparation outputs
- fake repositories can drive preparation behavior

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path packages/runtime-chat-app/Cargo.toml preparation -- --nocapture
```

Expected: fail because traits/result model are incomplete

**Step 3: Implement minimal contract**

Add:

- `ChatSettingsRepository`
- `ChatSessionRepository`
- `EmployeeRoutingCatalog`
- `PreparedChatExecution`

Keep them narrow and read-only

**Step 4: Re-run tests**

Run the same targeted command

**Step 5: Commit**

```bash
git add packages/runtime-chat-app
git commit -m "refactor(chat): define chat preparation contracts"
```

### Task 4: Implement `prepare_chat_execution`

**Files:**
- Modify: `packages/runtime-chat-app/src/service.rs`
- Test: `packages/runtime-chat-app/tests/prepare_chat_execution.rs`

**Step 1: Write failing tests**

Use fake repos/catalogs to verify:

- routing settings are loaded
- capability and route selection are normalized
- fallback candidates are prepared
- guidance/context fragments are included

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path packages/runtime-chat-app/Cargo.toml prepare_chat_execution -- --nocapture
```

Expected: fail because orchestration method is missing

**Step 3: Implement minimal orchestration**

- Build only the pre-execution plan
- Do not call the real executor
- Do not write to the database

**Step 4: Re-run tests**

Run the same targeted command

**Step 5: Commit**

```bash
git add packages/runtime-chat-app
git commit -m "refactor(chat): add chat preparation service"
```

### Task 5: Add `src-tauri` repo adapter

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/chat_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/Cargo.toml`
- Modify: `apps/runtime/src-tauri/Cargo.lock`

**Step 1: Write failing integration compile check**

Pick one targeted chat test or a narrow `--no-run` compile step that depends on the adapter.

Run:

```bash
node scripts/run-cargo-isolated.mjs chat-adapter -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_models --no-run
```

Expected: fail because adapter and dependency wiring do not yet exist

**Step 2: Implement minimal adapter**

- Implement `ChatSettingsRepository`
- Implement `ChatSessionRepository`
- Add a null `EmployeeRoutingCatalog` if needed

**Step 3: Re-run isolated compile verification**

Use the same command and confirm it builds

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri
git commit -m "refactor(chat): add tauri chat preparation adapter"
```

### Task 6: Route `chat.rs` through `runtime-chat-app`

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Test: `apps/runtime/src-tauri/tests/test_models.rs`
- Test: `apps/runtime/src-tauri/tests/test_skill_route_settings.rs`
- Test: add a narrow chat preparation smoke test if needed

**Step 1: Write failing test or compile check**

Prefer a targeted smoke test around preparation behavior, otherwise use a narrow compile target plus one existing routing-related test.

**Step 2: Run to verify failure**

Run:

```bash
node scripts/run-cargo-isolated.mjs chat-command -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_skill_route_settings -- --nocapture
```

Expected: fail until `chat.rs` uses the new service correctly

**Step 3: Implement minimal wiring**

- Instantiate `ChatPreparationService`
- Replace direct preparation logic with service calls
- Keep executor invocation and event emit code unchanged

**Step 4: Re-run verification**

Use the same command and confirm green

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri
git commit -m "refactor(chat): route command preparation through app layer"
```

### Task 7: Final verification and cleanup

**Files:**
- Modify: any touched file for small cleanup only

**Step 1: Run lightweight Rust suite**

Run:

```bash
pnpm test:rust-fast
```

Expected: green

**Step 2: Run focused isolated verification for chat**

Run:

```bash
node scripts/run-cargo-isolated.mjs chat-final -- test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_skill_route_settings -- --nocapture
```

Expected: green, or a clearly documented remaining blocker outside this change set

**Step 3: Review diff**

Run:

```bash
git diff --stat HEAD~1..HEAD
```

Ensure `chat.rs` shrank and no unrelated behavior was pulled into the new crate

**Step 4: Commit any cleanup**

```bash
git add -A
git commit -m "chore(chat): finalize phase 4a cleanup"
```
