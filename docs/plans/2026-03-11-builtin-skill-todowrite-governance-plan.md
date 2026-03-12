# Builtin Skill TodoWrite Governance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `TodoWrite` the unified task-state source for all multi-step builtin skills by codifying the rule in system prompts and updating builtin skill instructions to establish plans from the first execution turn.

**Architecture:** Introduce a platform-level governance rule in the builtin skill/system prompt layer, then update concrete builtin skills to include explicit step templates where information gathering and user confirmation are formal plan steps. Verify this through focused tests around prompt assembly and representative builtin skill behavior.

**Tech Stack:** Rust, Tauri backend, builtin skill markdown prompts, TypeScript/Vitest tests

---

### Task 1: Locate prompt assembly and add failing tests for governance rule

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/agent/system_prompts/*`
- Modify: `packages/runtime-skill-core/src/builtin_skills.rs`
- Modify: `packages/runtime-skill-core/src/lib.rs`
- Test: `packages/runtime-skill-core/src/builtin_skills.rs`
- Test: `packages/runtime-skill-core/tests/*`

**Step 1: Write the failing test**

Add tests that assert prompt assembly for builtin multi-step skills includes a rule equivalent to:

- multi-step builtin skills must establish a TodoWrite plan before execution
- information gathering and confirmation are formal plan steps

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: FAIL because the current governance rule and builtin skill contract are not yet encoded in the lightweight skill core layer.

**Step 3: Write minimal implementation**

Only add the smallest shared prompt-governance helpers required in `runtime-skill-core`, then wire `src-tauri` to consume them.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/runtime-skill-core/src/builtin_skills.rs packages/runtime-skill-core/src/lib.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/system_prompts
git commit -m "feat: add todowrite governance to builtin skill prompts"
```

### Task 2: Define multi-step builtin skill classification

**Files:**
- Modify: `packages/runtime-skill-core/src/builtin_skills.rs`
- Modify: `apps/runtime/src-tauri/src/agent/system_prompts/*`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Test: `packages/runtime-skill-core/src/builtin_skills.rs`

**Step 1: Write the failing test**

Add tests covering the classification logic so that representative builtin skills can be marked:

- multi-step builtin skill -> governance rule injected
- simple builtin skill -> governance rule not force-injected

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: FAIL because no classification logic exists yet.

**Step 3: Write minimal implementation**

Implement a minimal classifier in `runtime-skill-core` based on the builtin skill IDs already centralized in that package, then let `src-tauri` consume it.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/runtime-skill-core/src/builtin_skills.rs packages/runtime-skill-core/src/lib.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/system_prompts
git commit -m "feat: classify multi-step builtin skills for todowrite governance"
```

### Task 3: Update employee creator builtin skill as the reference implementation

**Files:**
- Modify: `apps/runtime/src-tauri/builtin-skills/employee-creator/SKILL.md`
- Test: `packages/runtime-skill-core/src/builtin_skills.rs`

**Step 1: Write the failing test**

Add or extend a lightweight builtin-skill-content test asserting the employee creator prompt requires:

- TodoWrite at the start
- “收集关键需求信息” as a formal plan step
- “等待用户确认” as a formal plan step
- execution steps not marked active before confirmation

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: FAIL because the current skill text lacks explicit TodoWrite governance.

**Step 3: Write minimal implementation**

Update the builtin employee creator `SKILL.md` with the standardized plan template and confirmation constraints.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/builtin-skills/employee-creator/SKILL.md
git commit -m "feat: require todowrite planning in employee creator skill"
```

### Task 4: Audit and update other multi-step builtin skills

**Files:**
- Modify: `apps/runtime/src-tauri/builtin-skills/*/SKILL.md`
- Test: `packages/runtime-skill-core/src/builtin_skills.rs`

**Step 1: Write the failing test**

Add a focused lightweight audit test ensuring every classified multi-step builtin skill includes:

- initial TodoWrite planning language
- explicit confirmation constraints where applicable

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: FAIL for at least the remaining multi-step builtin skills not yet updated.

**Step 3: Write minimal implementation**

Update each targeted builtin skill markdown with the smallest wording changes needed to comply with the governance template.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture`

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/builtin-skills
git commit -m "docs: align multi-step builtin skills with todowrite governance"
```

### Task 5: Verify frontend expectations remain aligned

**Files:**
- Modify: `apps/runtime/src/components/chat-side-panel/view-model.test.ts`
- Modify: `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

**Step 1: Write the failing test**

Add or tighten tests proving the UI can rely on TodoWrite as the primary task-state source for staged builtin skills, especially around guided entry and confirmation phases.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/components/chat-side-panel/view-model.test.ts src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

Expected: FAIL if current assumptions still depend on synthetic pre-plan states.

**Step 3: Write minimal implementation**

Adjust tests and any small frontend assumptions needed so the UI remains aligned with the unified TodoWrite contract.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/components/chat-side-panel/view-model.test.ts src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/chat-side-panel/view-model.test.ts apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx
git commit -m "test: align chat ui with todowrite governance"
```

### Task 6: Final verification and rollout notes

**Files:**
- Modify: `docs/plans/2026-03-11-builtin-skill-todowrite-governance-design.md`
- Modify: `docs/plans/2026-03-11-builtin-skill-todowrite-governance-plan.md`

**Step 1: Run backend verification**

Run:

```bash
cargo test --manifest-path packages/runtime-skill-core/Cargo.toml -- --nocapture
```

Expected: governance-specific backend tests pass in the lightweight skill-core layer. If a separate `src-tauri` smoke check is desired, treat it as optional integration coverage, not the primary correctness gate for prompt-governance strings.

**Step 2: Run frontend verification**

Run:

```bash
cd apps/runtime && node node_modules/vitest/vitest.mjs run src/components/chat-side-panel/view-model.test.ts src/components/__tests__/ChatView.side-panel-redesign.test.tsx src/components/__tests__/ChatView.theme.test.tsx src/__tests__/App.employee-creator-skill-flow.test.tsx src/__tests__/App.employee-assistant-update-flow.test.tsx
```

Expected: PASS. If the standard `pnpm test` wrapper cannot resolve `vitest` from `.bin`, use the direct node entry above.

**Step 3: Update rollout notes**

If classification details or enforcement wording changed during implementation, update the design and plan docs accordingly.

**Step 4: Commit**

```bash
git add docs/plans/2026-03-11-builtin-skill-todowrite-governance-design.md docs/plans/2026-03-11-builtin-skill-todowrite-governance-plan.md
git commit -m "docs: finalize builtin skill todowrite governance plan"
```
