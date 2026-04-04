# Implicit Skill Routing Control Plane Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a dedicated control plane for natural-language implicit skill routing so WorkClaw can route skill-shaped requests into specialized execution lanes before they enter the generic agent loop.

**Architecture:** Build a cached `SkillRouteIndex`, a conservative `InvocationRouter`, and dedicated runners for direct-dispatch and prompt-skill lanes. Keep `OpenTask` as the only path that reaches the current generic turn loop, and emit route-level observability for every turn.

**Tech Stack:** Rust, Tauri runtime, SQLite-backed runtime state, projected workspace skills, structured routing, runtime telemetry, Rust unit and integration tests.

---

### Task 1: Define route intent and decision types

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/intent.rs`
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/mod.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/intent.rs`

**Step 1: Write the failing test**

Add tests for route decision types covering:

- `OpenTask`
- `PromptSkillInline`
- `PromptSkillFork`
- `DirectDispatchSkill`

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skill_routing::intent -- --nocapture`
Expected: FAIL because the new routing module and enums do not exist yet.

**Step 3: Write minimal implementation**

Create `InvocationIntent`, `RouteDecision`, and `RouteConfidence` types with only the fields needed for routing and execution handoff.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skill_routing::intent -- --nocapture`
Expected: PASS.

### Task 2: Build a cached skill route index from workspace skill entries

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/index.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/types.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/index.rs`

**Step 1: Write the failing test**

Add tests proving the index captures:

- `skill_id`
- display name
- aliases
- description
- `when_to_use`
- execution mode
- command dispatch metadata

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skill_routing::index -- --nocapture`
Expected: FAIL because no route index exists yet.

**Step 3: Write minimal implementation**

Create `SkillRouteIndex` and a builder that projects route metadata from workspace skill runtime entries without reading raw skill files at route time.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skill_routing::index -- --nocapture`
Expected: PASS.

### Task 3: Implement local candidate recall for implicit skill routing

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/recall.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/recall.rs`

**Step 1: Write the failing test**

Add recall tests for:

- domain-word match to one skill family
- alias match
- multiple candidate recall ordering
- empty recall result

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skill_routing::recall -- --nocapture`
Expected: FAIL because local recall does not exist.

**Step 3: Write minimal implementation**

Implement a local scorer that ranks candidates using:

- skill id and alias overlap
- description overlap
- `when_to_use` overlap
- domain / family tag boosts

Keep the output small and deterministic.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skill_routing::recall -- --nocapture`
Expected: PASS.

### Task 4: Add a conservative route adjudicator

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/adjudicator.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/adjudicator.rs`

**Step 1: Write the failing test**

Add adjudicator tests for:

- clear single winner -> route to skill
- close tie -> `OpenTask`
- no candidate -> `OpenTask`
- prompt skill versus direct dispatch selection

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skill_routing::adjudicator -- --nocapture`
Expected: FAIL because no adjudicator exists.

**Step 3: Write minimal implementation**

Implement a conservative adjudicator that only chooses a skill when one candidate clearly wins. Otherwise return `OpenTask` with a reason code.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skill_routing::adjudicator -- --nocapture`
Expected: PASS.

### Task 5: Add dedicated route runners and keep the generic loop isolated

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/runner.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`

**Step 1: Write the failing test**

Add runtime tests proving:

- routed direct-dispatch skills bypass the generic loop
- routed prompt-inline skills receive narrowed runtime context
- routed fork skills choose the fork runner
- `OpenTask` still reaches the generic loop

**Step 2: Run test to verify it fails**

Run: `cargo test --lib session_runtime -- --nocapture`
Expected: FAIL because route-specific runners do not exist yet.

**Step 3: Write minimal implementation**

Add a `RouteRunner` layer that maps `RouteDecision` to:

- direct dispatch
- prompt skill inline
- prompt skill fork
- generic open task

Refactor `session_runtime.rs` so it orchestrates rather than performs route logic inline.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib session_runtime -- --nocapture`
Expected: PASS.

### Task 6: Add route-level observability

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/observability.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/skill_routing/observability.rs`

**Step 1: Write the failing test**

Add tests for emitted route metadata including:

- route latency
- candidate count
- selected runner
- selected skill
- fallback reason

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skill_routing::observability -- --nocapture`
Expected: FAIL because route observability does not exist.

**Step 3: Write minimal implementation**

Emit structured route outcomes from the new routing layer and persist enough information to diagnose route quality and performance regressions.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skill_routing::observability -- --nocapture`
Expected: PASS.

### Task 7: Verify benchmark request behavior against the new control plane

**Files:**
- Verify touched runtime files and tests above

**Step 1: Run focused Rust verification**

Run:
- `cargo test --lib skill_routing::intent -- --nocapture`
- `cargo test --lib skill_routing::index -- --nocapture`
- `cargo test --lib skill_routing::recall -- --nocapture`
- `cargo test --lib skill_routing::adjudicator -- --nocapture`
- `cargo test --lib session_runtime -- --nocapture`

Expected: PASS.

**Step 2: Run the fast runtime lane**

Run: `pnpm test:rust-fast`
Expected: PASS.

**Step 3: Record benchmark observations**

Compare at least one natural-language implicit skill request before and after the change. Capture:

- total latency
- route latency
- selected runner
- turn count
- tool count

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/skill_routing apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs apps/runtime/src-tauri/src/agent/runtime/runtime_io docs/plans/2026-04-04-implicit-skill-routing-control-plane-design.md docs/plans/2026-04-04-implicit-skill-routing-control-plane.md
git commit -m "feat(runtime): add implicit skill routing control plane"
```
