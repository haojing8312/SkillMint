# Local Skill Fast Path And Exec Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a desktop-only explicit natural-language skill fast path and a first-class Windows-friendly `exec` tool so local chat skill execution behaves closer to Codex.

**Architecture:** Detect high-confidence explicit skill mentions before the normal model loop and, when safe, convert the turn into a direct prompt-following skill context instead of asking the model to rediscover the skill. In parallel, replace `exec -> bash -> cmd /C` with a dedicated `exec` tool that preserves the same structured result shape but uses PowerShell-friendly execution on Windows.

**Tech Stack:** Rust, Tauri runtime, SQLite-backed runtime state, projected workspace skills, structured tool dispatch, PowerShell-aware process execution, Rust integration tests.

---

### Task 1: Add focused tests for explicit skill fast-path matching

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`

**Step 1: Write the failing tests**

Add tests for:

- exact `skill_id` mention such as `feishu-pm-hub`
- explicit phrases such as `使用 feishu-pm-hub 技能`
- ambiguous multi-skill mentions returning no match
- non-explicit natural-language requests returning no match

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime_lib session_runtime`

Expected: FAIL because the explicit natural-language matcher does not exist yet.

**Step 3: Write minimal implementation**

Add a conservative explicit-skill matching helper in `session_runtime.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime_lib session_runtime`

Expected: PASS.

### Task 2: Route explicit prompt-following skills before the model loop

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/types.rs`

**Step 1: Write the failing test**

Add a runtime-focused test that proves an explicit natural-language skill target can override the default skill context without using slash-command dispatch.

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime_lib natural_language_skill`

Expected: FAIL because only slash-command input gets special handling today.

**Step 3: Write minimal implementation**

Add a small `ExplicitSkillSelection` runtime path that:

- loads workspace skill runtime entries
- matches one explicit target skill when confidence is high
- if the target is prompt-following, overrides the effective `skill_system_prompt`, `allowed_tools`, and `max_iterations`

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime_lib natural_language_skill`

Expected: PASS.

### Task 3: Add a first-class `exec` tool and keep output compatibility

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/tools/exec.rs`
- Modify: `apps/runtime/src-tauri/src/agent/tools/mod.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- Test: `apps/runtime/src-tauri/tests/test_tool_aliases.rs`

**Step 1: Write the failing test**

Add tests proving:

- `exec` is registered as a real tool, not just an alias
- `exec` remains available alongside `bash`

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime_lib test_tool_aliases`

Expected: FAIL because `exec` is only an alias today.

**Step 3: Write minimal implementation**

Create a dedicated `ExecTool` that:

- reuses the same structured result format as `bash`
- supports `command`, `timeout_ms`, and `background`
- uses PowerShell-friendly execution semantics on Windows

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime_lib test_tool_aliases`

Expected: PASS.

### Task 4: Add Windows-oriented `exec` execution tests

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_bash.rs`

**Step 1: Write the failing test**

Add tests for `exec` covering:

- success path with structured output
- execution metadata including `platform_shell`
- timeout path preserving result shape

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime_lib test_bash`

Expected: FAIL because the new `exec` tool tests are not implemented yet.

**Step 3: Write minimal implementation**

Adjust the new `ExecTool` until the tests pass without regressing `bash`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime_lib test_bash`

Expected: PASS.

### Task 5: Preserve slash-command deterministic dispatch

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_dispatch.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`

**Step 1: Write the failing test**

Add or extend tests proving existing slash-command dispatch still reaches the declared tool and preserves raw args.

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime_lib dispatch_skill_command`

Expected: FAIL only if the refactor accidentally breaks the current path.

**Step 3: Write minimal implementation**

Keep the current slash-command path unchanged except for the new real `exec` tool registration.

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime_lib dispatch_skill_command`

Expected: PASS.

### Task 6: Run WorkClaw verification for the changed runtime surface

**Files:**
- Verify touched runtime files and tests above

**Step 1: Run the fast Rust lane**

Run: `pnpm test:rust-fast`

Expected: PASS.

**Step 2: Record coverage**

Note which areas are covered:

- explicit skill matching
- prompt-following fast path
- exec tool registration
- Windows-friendly exec behavior
- slash-command stability

**Step 3: Note any remaining gaps**

Document any still-unverified area, especially true desktop e2e timing against the packaged exe.
