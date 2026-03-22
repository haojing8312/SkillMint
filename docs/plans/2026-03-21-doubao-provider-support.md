# Doubao Provider Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Doubao support to WorkClaw quick setup and model connection presets, with the default model set to the strongest general-purpose Doubao model.

**Architecture:** Reuse the existing shared provider catalog so one preset addition updates both the first-run quick setup and the Settings model connection form. Keep internal recommendation helpers aligned by adding a minimal `doubao` recommendation entry in the routing core.

**Tech Stack:** React, TypeScript, Vitest, Rust

---

### Task 1: Add failing catalog and routing tests

**Files:**
- Modify: `apps/runtime/src/__tests__/model-provider-catalog.test.ts`
- Modify: `packages/runtime-routing-core/tests/routing.rs`

**Step 1: Write the failing tests**
- Add assertions that the model provider catalog contains a `doubao` entry.
- Add assertions that the Doubao default model is `doubao-seed-1.6`.
- Add assertions that routing recommendations for provider key `doubao` include `doubao-seed-1.6`.

**Step 2: Run tests to verify they fail**

Run: `pnpm --filter runtime test -- --run apps/runtime/src/__tests__/model-provider-catalog.test.ts`
Expected: FAIL because the Doubao preset does not exist yet.

Run: `cargo test -p runtime-routing-core recommended_models_cover_known_providers --test routing`
Expected: FAIL because the `doubao` provider is not covered yet.

**Step 3: Commit**
- Skip commit for this small local task unless requested.

### Task 2: Implement the Doubao preset

**Files:**
- Modify: `apps/runtime/src/model-provider-catalog.ts`

**Step 1: Write minimal implementation**
- Add an official provider preset for Doubao / Volcano Ark.
- Use OpenAI-compatible protocol.
- Set `baseUrl` to `https://ark.cn-beijing.volces.com/api/v3`.
- Set `defaultModel` to `doubao-seed-1.6`.
- Keep `models` limited to only `doubao-seed-1.6` per approved scope.
- Add helper text warning that some Ark accounts may need to replace the model with their own endpoint ID.

**Step 2: Run the targeted front-end test**

Run: `pnpm --filter runtime test -- --run apps/runtime/src/__tests__/model-provider-catalog.test.ts`
Expected: PASS

### Task 3: Align routing recommendations

**Files:**
- Modify: `packages/runtime-routing-core/src/lib.rs`

**Step 1: Write minimal implementation**
- Add `recommended_models_for_provider("doubao")`.
- Keep the returned list limited to `doubao-seed-1.6`.

**Step 2: Run the targeted Rust test**

Run: `cargo test -p runtime-routing-core recommended_models_cover_known_providers --test routing`
Expected: PASS

### Task 4: Run final verification

**Files:**
- No code changes

**Step 1: Run final verification commands**

Run: `pnpm --filter runtime test -- --run apps/runtime/src/__tests__/model-provider-catalog.test.ts`
Expected: PASS

Run: `cargo test -p runtime-routing-core --test routing`
Expected: PASS

**Step 2: Summarize verification**
- Report the exact commands run, what they covered, and any remaining unverified UI behavior.
