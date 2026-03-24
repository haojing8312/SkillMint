# Settings View Feishu Phase 2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce `apps/runtime/src/components/SettingsView.tsx` further by shrinking its direct Feishu-specific business-logic ownership without changing current behavior.

**Architecture:** Extract Feishu pure selectors first, then centralize Feishu `invoke(...)` wrappers in a service module, then introduce a thin Feishu controller hook that preserves current polling and async handler semantics. Keep `SettingsView.tsx` as the shell/composition root and keep the section components primarily presentational.

**Tech Stack:** React, TypeScript, Vitest, existing Tauri `invoke(...)` APIs, frontend large-file guardrails.

---

### Task 1: Extract Feishu selectors

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/feishu/feishuSelectors.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Identify pure Feishu derivations**

Target logic for extraction:

- onboarding step derivation
- connector-status label derivation
- diagnostics summary derivation
- installer display labels and hints
- routing-status summary labels

Only extract functions that can be pure and side-effect-free.

**Step 2: Run the focused Feishu baseline**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx src/components/__tests__/SettingsView.theme.test.tsx
```

Expected: PASS before refactor

**Step 3: Create `feishuSelectors.ts`**

Move the pure logic into selector functions and rewire `SettingsView.tsx` to call them.

Do not:

- add `invoke(...)`
- move polling or handlers
- change display strings

**Step 4: Re-run the focused Feishu baseline**

Run the same command from Step 2.

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/feishu/feishuSelectors.ts
git commit -m "refactor(ui): extract feishu settings selectors"
```

### Task 2: Centralize Feishu service wrappers

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create or modify: `apps/runtime/src/components/settings/feishu/feishuSettingsService.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx`

**Step 1: Confirm touched backend surface**

Inventory the Feishu commands currently called from `SettingsView.tsx`, including:

- setup progress
- runtime status
- installer session state
- pairing list and pairing actions
- advanced settings load/save
- channel snapshot and plugin-host probes

**Step 2: Run the focused Feishu baseline**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx src/components/__tests__/SettingsView.theme.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract `invoke(...)` wrappers**

Create or expand `feishuSettingsService.ts` and move command wrappers there.

Rules:

- preserve command names
- preserve payload shapes
- normalize null/legacy responses only where current code already does so
- do not move React state

**Step 4: Rewire `SettingsView.tsx`**

Replace direct `invoke(...)` calls with service calls while preserving current handler and lifecycle behavior.

**Step 5: Re-run the focused Feishu baseline**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/feishu/feishuSettingsService.ts
git commit -m "refactor(ui): centralize feishu settings service calls"
```

### Task 3: Introduce a thin Feishu controller hook

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/feishu/useFeishuSettingsController.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx`

**Step 1: Choose only Feishu-local ownership**

Candidates:

- refresh and retry state
- installer-input local state
- pairing-action loading state
- advanced-settings save state
- Feishu polling lifecycle

Do not move:

- global tab state
- unrelated settings domains

**Step 2: Run the focused Feishu baseline**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx src/components/__tests__/SettingsView.theme.test.tsx
```

Expected: PASS before refactor

**Step 3: Create `useFeishuSettingsController.ts`**

Move Feishu-local orchestration into the hook.

Rules:

- preserve current polling behavior
- preserve current tab-open refresh behavior
- return plain state plus handlers
- do not let the hook absorb pure selectors or service code that already has a better home

**Step 4: Rewire `SettingsView.tsx`**

Make the root consume the controller outputs and continue passing them to the existing Feishu sections.

**Step 5: Re-run the focused Feishu baseline**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/feishu/useFeishuSettingsController.ts
git commit -m "refactor(ui): extract feishu settings controller"
```

### Task 4: Re-measure and verify file-budget impact

**Files:**
- Modify only if small follow-up cleanup is needed
- Report: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Measure resulting root-file size**

Run:

```bash
pnpm report:frontend-large-files
```

Expected: `SettingsView.tsx` should be materially smaller than the current post-phase-1 baseline

**Step 2: Run final verification**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx src/components/__tests__/SettingsView.theme.test.tsx src/components/__tests__/RoutingSettingsSection.test.tsx src/components/__tests__/SettingsView.mcp.test.tsx
```

Expected: PASS

**Step 3: Small cleanup only if clearly safe**

If `SettingsView.tsx` still contains trivial pure helpers that now obviously belong in the Feishu selector or service layer, move them only if the change is tiny and low-risk.

Do not start a new restructuring wave in this task.

**Step 4: Commit any final cleanup**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/feishu/feishuSelectors.ts apps/runtime/src/components/settings/feishu/feishuSettingsService.ts apps/runtime/src/components/settings/feishu/useFeishuSettingsController.ts
git commit -m "refactor(ui): finish settings feishu phase 2 cleanup"
```

---

## Verification Summary

Minimum honest verification for this phase:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx src/components/__tests__/SettingsView.theme.test.tsx src/components/__tests__/RoutingSettingsSection.test.tsx src/components/__tests__/SettingsView.mcp.test.tsx
pnpm report:frontend-large-files
```

## Exit Criteria

This phase is complete when:

- `SettingsView.tsx` no longer directly owns most Feishu pure derivations
- `SettingsView.tsx` no longer directly owns most Feishu backend command wrappers
- a thin Feishu controller hook owns Feishu-local orchestration without changing behavior
- focused Feishu and settings baseline tests pass
- the root file is materially thinner and easier to reason about
