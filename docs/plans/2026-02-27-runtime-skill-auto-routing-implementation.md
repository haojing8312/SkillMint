# Runtime Skill Auto-Routing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a production-ready Runtime experience for automatic child-skill routing with simple chat UX, full right-panel observability, and strict parent-to-child permission narrowing.

**Architecture:** Extend the existing `skill` tool and chat event pipeline to emit structured route-run events, then render those events in a right-panel call graph and permission trace UI. Keep main chat minimal by showing only natural assistant output plus a route summary capsule that deep-links to panel details. Enforce security by computing child permission sets as intersection of parent/session/workspace constraints.

**Tech Stack:** Tauri 2 (Rust), React 18 + TypeScript, Tailwind, framer-motion, existing Runtime tool/event architecture.

---

### Task 1: Define Route Trace Types (Frontend + Backend Contract)

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Create: `apps/runtime/src-tauri/tests/test_skill_route_events.rs`

**Step 1: Write the failing test**

Add `test_skill_route_events.rs` asserting emitted payload includes:
- `route_run_id`
- `node_id`
- `parent_node_id`
- `skill_name`
- `depth`
- `status`

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_route_events -- --nocapture`  
Expected: FAIL due to missing event type or missing fields.

**Step 3: Write minimal implementation**

Add event payload structs in `chat.rs` and wire placeholders for route-related fields.

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_route_events -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/tests/test_skill_route_events.rs
git commit -m "feat(runtime): define skill route trace event contract"
```

### Task 2: Instrument SkillInvokeTool with Route Node Lifecycle Events

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- Modify: `apps/runtime/src-tauri/src/agent/tools/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_skill_route_events.rs`

**Step 1: Write the failing test**

Extend test to assert lifecycle transitions:
- `routing` on entry
- `executing` after skill file resolved
- `completed` on success
- `failed` on error

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_route_events -- --nocapture`  
Expected: FAIL because transitions are not emitted.

**Step 3: Write minimal implementation**

Emit route-node updates from `skill_invoke.rs`, including depth and parent node id from tool context metadata.

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_route_events -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs apps/runtime/src-tauri/src/agent/tools/mod.rs apps/runtime/src-tauri/tests/test_skill_route_events.rs
git commit -m "feat(runtime): emit skill route lifecycle events from skill tool"
```

### Task 3: Enforce Parent-to-Child Permission Narrowing

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- Modify: `apps/runtime/src-tauri/src/agent/permissions.rs`
- Create: `apps/runtime/src-tauri/tests/test_skill_permission_narrowing.rs`

**Step 1: Write the failing test**

Add tests for:
- child allowed set equals intersection of parent + child + workspace policy
- forbidden tool/path is denied with stable error code `PERMISSION_DENIED`

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_permission_narrowing -- --nocapture`  
Expected: FAIL because narrowing is not fully enforced per child node.

**Step 3: Write minimal implementation**

Compute and persist narrowed permission snapshot per route node and block out-of-bound requests.

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_permission_narrowing -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs apps/runtime/src-tauri/src/agent/permissions.rs apps/runtime/src-tauri/tests/test_skill_permission_narrowing.rs
git commit -m "feat(runtime): enforce hierarchical permission narrowing for child skills"
```

### Task 4: Build Right Panel Route Overview + Call Graph UI

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/ToolIsland.tsx`
- Modify: `apps/runtime/src/types.ts`

**Step 1: Write the failing test**

If UI tests exist, add rendering tests for:
- route summary capsule visible in assistant message
- call graph nodes render in right panel

If no UI test harness exists, define manual acceptance checklist in code comments near state initialization.

**Step 2: Run test/check to verify it fails**

Run existing frontend checks (if configured): `cd apps/runtime && pnpm run build`  
Expected: FAIL due to missing types/state/props.

**Step 3: Write minimal implementation**

Add right-panel tabs:
- `Overview`
- `Call Graph`

Render per-node status and duration. Keep main chat minimal and show one summary capsule with deep-link behavior.

**Step 4: Run test/check to verify it passes**

Run: `cd apps/runtime && pnpm run build`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/ToolIsland.tsx apps/runtime/src/types.ts
git commit -m "feat(runtime-ui): add route overview and call graph panel"
```

### Task 5: Add Permissions Tab + Actionable Error Panel

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- Create: `apps/runtime/src-tauri/tests/test_skill_route_errors.rs`

**Step 1: Write the failing test**

Add backend tests asserting stable error codes for:
- `SKILL_NOT_FOUND`
- `CALL_DEPTH_EXCEEDED`
- `CALL_CYCLE_DETECTED`
- `TIMEOUT`

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_route_errors -- --nocapture`  
Expected: FAIL due to plain-text errors without stable codes.

**Step 3: Write minimal implementation**

Normalize errors to coded payloads and render right-panel remediation copy mapped by error code.

**Step 4: Run test/check to verify it passes**

Run:
- `cd apps/runtime/src-tauri && cargo test test_skill_route_errors -- --nocapture`
- `cd apps/runtime && pnpm run build`

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/types.ts apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs apps/runtime/src-tauri/tests/test_skill_route_errors.rs
git commit -m "feat(runtime): add permission trace and actionable route error UX"
```

### Task 6: Integrate Settings for Routing Controls

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Create: `apps/runtime/src-tauri/tests/test_skill_route_settings.rs`

**Step 1: Write the failing test**

Add settings persistence tests for:
- `max_call_depth` default `4`
- `node_timeout_seconds` default `60`
- `route_retry_count` default `0`

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test test_skill_route_settings -- --nocapture`  
Expected: FAIL due to missing persisted settings.

**Step 3: Write minimal implementation**

Add settings fields in DB + command layer, expose in Settings UI.

**Step 4: Run test/check to verify it passes**

Run:
- `cd apps/runtime/src-tauri && cargo test test_skill_route_settings -- --nocapture`
- `cd apps/runtime && pnpm run build`

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/tests/test_skill_route_settings.rs
git commit -m "feat(runtime): add auto-routing controls in settings"
```

### Task 7: Final Verification and Regression Sweep

**Files:**
- Modify: `docs/plans/2026-02-27-runtime-skill-auto-routing-design.md`
- Modify: `docs/plans/2026-02-27-runtime-skill-auto-routing-implementation.md`

**Step 1: Run backend test suite**

Run: `cd apps/runtime/src-tauri && cargo test`  
Expected: all tests pass, including new route/permission/error/settings tests.

**Step 2: Run frontend build**

Run: `cd apps/runtime && pnpm run build`  
Expected: build succeeds with no type errors.

**Step 3: Manual runtime smoke test**

Validate:
- normal request without child routing
- successful child routing
- cycle/depth/not-found failure with right-panel diagnostics
- permission denied path and fix guidance

**Step 4: Update docs**

Record final behavior, screenshots checklist, and known limitations in both plan/design docs.

**Step 5: Commit**

```bash
git add docs/plans/2026-02-27-runtime-skill-auto-routing-design.md docs/plans/2026-02-27-runtime-skill-auto-routing-implementation.md
git commit -m "docs(runtime): finalize skill auto-routing design and implementation plan"
```
