# OpenClaw-Style Tool Results Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce an OpenClaw-style structured result contract for WorkClaw's highest-priority built-in tools, starting with `read_file`, `grep`, and `bash`, while preserving compatibility with the current runtime.

**Architecture:** Keep the existing `Tool::execute(...) -> Result<String>` trait for now, but standardize returned strings as serialized JSON payloads containing `ok`, `summary`, and `details`. Migrate the three highest-frequency tools first, update tests to assert parsed structure, and keep existing runtime behavior stable enough for current executor and UI flows.

**Tech Stack:** Rust, Tauri runtime, serde_json, existing runtime-chat-app prompt assembly tests, WorkClaw Rust fast test suite.

---

### Task 1: Add Shared Result Helpers

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/mod.rs`
- Create: `apps/runtime/src-tauri/src/agent/tools/tool_result.rs`
- Test: `apps/runtime/src-tauri/tests/` existing tool tests

**Step 1: Write the failing test**

Add a focused helper-level test or first tool-level assertion expecting:

- success payload contains `ok`, `tool`, `summary`, `details`
- JSON is parseable and stable

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test test_read_file -- --nocapture
```

Expected: FAIL because `read_file` still returns raw content.

**Step 3: Write minimal implementation**

Create helper functions that build:

- success payload JSON string
- optional structured error message helpers or error code string builders

Keep helpers tiny and tool-agnostic.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test test_read_file -- --nocapture
```

Expected: PASS for the first updated assertion.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/mod.rs apps/runtime/src-tauri/src/agent/tools/tool_result.rs apps/runtime/src-tauri/tests/test_read_file.rs
git commit -m "refactor: add structured tool result helpers"
```

### Task 2: Migrate `read_file`

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/read_file.rs`
- Test: `apps/runtime/src-tauri/tests/test_read_file.rs`

**Step 1: Write the failing test**

Add assertions that `read_file` returns parseable JSON with:

- `ok == true`
- `tool == "read_file"`
- `summary`
- `details.path`
- `details.content`
- `details.line_count`

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test test_read_file -- --nocapture
```

Expected: FAIL because tool still returns raw text.

**Step 3: Write minimal implementation**

Update `read_file` to:

- keep exact file content in `details.content`
- include path metadata
- summarize the read operation

Keep error behavior unchanged except for clearer recovery hints if a targeted test requires it.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test test_read_file -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/read_file.rs apps/runtime/src-tauri/tests/test_read_file.rs
git commit -m "refactor: migrate read_file to structured results"
```

### Task 3: Migrate `grep`

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/grep_tool.rs`
- Test: `apps/runtime/src-tauri/tests/test_grep.rs`

**Step 1: Write the failing test**

Add assertions that `grep` returns parseable JSON with:

- `ok == true`
- `tool == "grep"`
- `summary`
- `details.files_searched`
- `details.total_matches`
- `details.matches[]` entries with `path`, `line`, `text`

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test test_grep -- --nocapture
```

Expected: FAIL because output is still formatted plain text.

**Step 3: Write minimal implementation**

Refactor `grep` internals so directory and file search both collect structured matches first, then serialize one unified success result.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test test_grep -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/grep_tool.rs apps/runtime/src-tauri/tests/test_grep.rs
git commit -m "refactor: migrate grep to structured results"
```

### Task 4: Migrate `bash`

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/bash.rs`
- Test: `apps/runtime/src-tauri/tests/test_bash.rs`
- Test: `apps/runtime/src-tauri/tests/test_bash_background.rs`

**Step 1: Write the failing test**

Add assertions that foreground results return parseable JSON with:

- `ok`
- `tool == "bash"`
- `summary`
- `details.command`
- `details.exit_code`
- `details.stdout`
- `details.stderr`
- `details.timed_out`

Add assertions that background mode returns:

- `details.background == true`
- `details.process_id`

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test test_bash -- --nocapture
cargo test --test test_bash_background -- --nocapture
```

Expected: FAIL because outputs are currently prose strings.

**Step 3: Write minimal implementation**

Update `bash` result construction to emit structured JSON strings for:

- success
- non-zero exit
- timeout
- background launch

Keep dangerous-command blocking behavior intact.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test test_bash -- --nocapture
cargo test --test test_bash_background -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/bash.rs apps/runtime/src-tauri/tests/test_bash.rs apps/runtime/src-tauri/tests/test_bash_background.rs
git commit -m "refactor: migrate bash to structured results"
```

### Task 5: Add Prompt Guidance For Structured Tool Results

**Files:**
- Modify: `packages/runtime-chat-app/src/service.rs`
- Test: `packages/runtime-chat-app/tests/prompt_assembly.rs`

**Step 1: Write the failing test**

Add a prompt assembly test expecting guidance that:

- read/grep/bash results include structured details
- downstream reasoning should prefer `details`
- file paths and command outcomes should be reused from tool results instead of rewritten

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test prompt_assembly -- --nocapture
```

Expected: FAIL until prompt text is added.

**Step 3: Write minimal implementation**

Append a compact structured-tool guidance block when relevant tool names are present.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test prompt_assembly -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/runtime-chat-app/src/service.rs packages/runtime-chat-app/tests/prompt_assembly.rs
git commit -m "feat: add structured tool result prompt guidance"
```

### Task 6: Verify Compatibility And Guardrails

**Files:**
- Modify if needed: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify if needed: `apps/runtime/src-tauri/tests/test_react_loop.rs`
- Test: relevant Rust fast suites

**Step 1: Write the failing test**

Only if needed, add regression assertions that:

- repeated successful loops still stop
- repeated failed retries still stop
- structured JSON outputs do not accidentally defeat progress detection

**Step 2: Run test to verify it fails**

Run the focused regression tests if any new one is added.

**Step 3: Write minimal implementation**

Adjust executor hashing/parsing only if structured payloads change guard behavior.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test test_react_loop -- --nocapture
```

Expected: PASS for touched regressions.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/tests/test_react_loop.rs
git commit -m "fix: preserve run guard behavior with structured tool results"
```

### Task 7: Final Verification

**Files:**
- No new files expected

**Step 1: Run focused tests**

Run:

```bash
cargo test --test test_read_file -- --nocapture
cargo test --test test_grep -- --nocapture
cargo test --test test_bash -- --nocapture
cargo test --test test_bash_background -- --nocapture
cargo test --test prompt_assembly -- --nocapture
cargo test --test test_react_loop -- --nocapture
```

Expected: PASS.

**Step 2: Run repo verification**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS.

**Step 3: Review remaining unverified areas**

Check whether any UI tool-card rendering relies on the previous plain-text shape and record that as follow-up if not handled in this batch.

**Step 4: Commit**

```bash
git add .
git commit -m "refactor: adopt structured tool results for core runtime tools"
```
