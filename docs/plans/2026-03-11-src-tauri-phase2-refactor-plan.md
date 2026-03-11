# src-tauri Phase 2 Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract pure routing/model logic and executor helpers from `apps/runtime/src-tauri` into lightweight crates so more Rust changes avoid the full desktop runtime build graph.

**Architecture:** Introduce `runtime-routing-core` for static route/model rules and `runtime-executor-core` for pure executor helpers. Keep SQLx, provider calls, Tauri commands, and execution orchestration inside `src-tauri`.

**Tech Stack:** Rust workspace crates, Tauri runtime crate, focused cargo tests

---

### Task 1: Create `runtime-routing-core`

**Files:**
- Create: `packages/runtime-routing-core/Cargo.toml`
- Create: `packages/runtime-routing-core/src/lib.rs`
- Create: `packages/runtime-routing-core/tests/...`
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`
- Modify: `apps/runtime/src-tauri/Cargo.toml`

**Step 1: Write failing tests**

Add lightweight tests for:
- capability route template listing
- provider recommended model selection
- capability-based model filtering
- cache freshness helper

**Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path packages/runtime-routing-core/Cargo.toml -- --nocapture`

Expected: FAIL because logic has not been extracted yet.

**Step 3: Implement minimal extraction**

Move pure routing/model helpers into the new crate and make `commands/models.rs` consume them.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test --manifest-path packages/runtime-routing-core/Cargo.toml -- --nocapture`

### Task 2: Create `runtime-executor-core`

**Files:**
- Create: `packages/runtime-executor-core/Cargo.toml`
- Create: `packages/runtime-executor-core/src/lib.rs`
- Create: `packages/runtime-executor-core/tests/...`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/Cargo.toml`

**Step 1: Write failing tests**

Add lightweight tests for:
- output truncation
- token estimation
- micro compaction
- message trimming
- error code/message splitting
- repeated failure streak behavior

**Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path packages/runtime-executor-core/Cargo.toml -- --nocapture`

Expected: FAIL because helpers have not been extracted yet.

**Step 3: Implement minimal extraction**

Move pure helper logic into the new crate and make `executor.rs` call through to it.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test --manifest-path packages/runtime-executor-core/Cargo.toml -- --nocapture`

### Task 3: Targeted verification

**Files:**
- Modify: none

**Step 1: Run all lightweight crate tests**

Run:
- `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`
- `cargo test --manifest-path packages/runtime-policy/Cargo.toml -- --nocapture`
- `cargo test --manifest-path packages/runtime-routing-core/Cargo.toml -- --nocapture`
- `cargo test --manifest-path packages/runtime-executor-core/Cargo.toml -- --nocapture`
- `cargo test --manifest-path packages/builtin-skill-checks/Cargo.toml -- --nocapture`

**Step 2: Run minimal app-level verification**

Only run narrowly-targeted `src-tauri` tests affected by wiring changes if the environment permits. Do not block on a full Tauri suite.

**Step 3: Inspect final scope**

Ensure Phase 2 still limits itself to pure logic extraction and does not drift into DB/application-service redesign.
