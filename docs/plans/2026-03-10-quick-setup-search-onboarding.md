# Quick Setup Search Onboarding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn quick setup into a two-step onboarding wizard that includes search engine configuration before initial setup is considered complete.

**Architecture:** Keep the modal orchestration in `App.tsx`, but extract search-specific presets and form rendering into shared UI/helpers so Settings and onboarding stay aligned. Use TDD: add failing UI tests first, then implement the minimal shared search configuration flow.

**Tech Stack:** React 18, TypeScript, Vitest, Testing Library, Tauri invoke bridge

---

### Task 1: Add failing onboarding tests

**Files:**
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`

**Step 1: Write the failing tests**

- Add a test that saves the quick model form and expects the dialog to remain open on a search step.
- Add a first-launch test that expects the gate/dialog to remain until a search config is saved.
- Add a settings-triggered test that expects step 2 to allow skipping.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- App.model-setup-hint.test.tsx`

Expected: FAIL because the dialog currently closes immediately after model save and has no search step.

### Task 2: Extract reusable search form

**Files:**
- Create: `apps/runtime/src/components/SearchConfigForm.tsx`
- Create: `apps/runtime/src/lib/search-config.ts`
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Move presets/helpers**

- Extract search presets and any small helper types into `search-config.ts`.

**Step 2: Build shared form**

- Render the same fields used in Settings:
  - preset
  - name
  - api key with show/hide
  - base url
  - SerpApi engine field when needed
  - test/save actions

**Step 3: Switch SettingsView to the shared form**

- Keep current behavior intact while delegating rendering and local field updates to the shared component.

### Task 3: Implement quick setup wizard step 2

**Files:**
- Modify: `apps/runtime/src/App.tsx`

**Step 1: Add search state/loading**

- Load `list_search_configs` in App.
- Track wizard step, search form, search test/save state, and search key visibility.

**Step 2: Change completion logic**

- Initial setup completion requires both model and search configs.
- Blocking gate/cancel logic should respect the new requirement.

**Step 3: Advance model step to search step**

- After successful model save, reload models and move the wizard to the search step.
- Close only after the search step is saved, or skipped in non-blocking mode.

### Task 4: Verify and polish

**Files:**
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`
- Modify: `apps/runtime/src/components/__tests__/SettingsView.theme.test.tsx` if needed

**Step 1: Run targeted tests**

Run: `pnpm --dir apps/runtime test -- App.model-setup-hint.test.tsx SettingsView.theme.test.tsx`

Expected: PASS

**Step 2: Run broader runtime tests if needed**

Run: `pnpm --dir apps/runtime test`

Expected: PASS or identify unrelated failures separately.
