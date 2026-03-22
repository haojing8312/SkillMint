# Feishu Guided Onboarding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a skippable guided Feishu onboarding flow that handles first-time setup separately from the existing settings console.

**Architecture:** Keep the existing Feishu settings page as the long-term console, but add a dedicated onboarding state machine and guided flow UI. Reuse the existing Tauri commands for environment, plugin install, robot validation, runtime start, and routing status, while reshaping frontend state and entry points around an explicit step-by-step journey.

**Tech Stack:** React, TypeScript, Tauri commands, Vitest, Rust command layer.

---

### Task 1: Define onboarding state and derived step helpers

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add tests that describe:
- an unfinished Feishu setup can expose a guided step order
- the flow distinguishes `existing robot` from `create robot`
- skipped setup does not block the rest of the app

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: FAIL because onboarding helpers and new rendering paths do not exist yet

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:
- add a small onboarding state model
- add helpers for current step, can-continue, and skipped state
- keep existing status loading calls but derive onboarding progress from them

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS for the new state tests

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: add feishu onboarding state helpers"
```

### Task 2: Add a guided onboarding panel inside the Feishu settings area

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add tests that assert:
- the settings page shows a “continue onboarding” entry when setup is incomplete
- the onboarding flow renders one primary step at a time
- the existing console remains available as an advanced section

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: FAIL because the guided panel is not rendered yet

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:
- add a guided onboarding shell above the current console blocks
- render only the active step content
- add a clear “re-open onboarding” / “skip for now” entry
- collapse the existing settings controls under an advanced or console section

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: add guided feishu onboarding panel"
```

### Task 3: Split existing-robot and create-robot paths cleanly

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add tests that assert:
- selecting `绑定已有机器人` requires App ID and App Secret
- selecting `新建机器人` does not require pre-filled credentials
- the primary action label and validation rules differ by path

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: FAIL because both paths are still mixed together

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:
- add an onboarding path selector
- gate validation rules by selected path
- hide or defer existing robot fields for the create path
- keep the advanced create installer controls but route them through the guided step

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: separate feishu onboarding paths"
```

### Task 4: Make action feedback explicit per step

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add tests that assert:
- install failures appear inside the install step
- runtime start failures appear inside the authorization step
- “button did nothing” cases become visible inline validation messages

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: FAIL because errors are currently only weak top-level notices

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:
- add step-local loading and error containers
- surface blocked conditions before invoking commands
- map backend failures to the current step

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "fix: show explicit feishu onboarding step feedback"
```

### Task 5: Add skippable first-use entry from quick setup

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/__tests__/App.*.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add or extend app-level tests that assert:
- quick setup can show a Feishu optional entry
- the user can skip Feishu setup without blocking app usage
- the user can later reopen onboarding from settings

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run src/__tests__/App*.test.tsx src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: FAIL because no quick-setup Feishu entry exists yet

**Step 3: Write minimal implementation**

In `App.tsx`:
- add a lightweight Feishu optional step into the quick setup experience
- persist skipped state locally
- wire “configure now” into the Feishu onboarding entry

In `SettingsView.tsx`:
- expose a “continue onboarding” opener regardless of whether the user skipped before

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run src/__tests__/App*.test.tsx src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/__tests__ apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: add skippable feishu quick setup entry"
```

### Task 6: Preserve settings console and diagnostics after onboarding

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add tests that assert:
- advanced settings remain reachable after onboarding
- connection details and diagnostics still render
- completed onboarding no longer dominates the page but can be reopened

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: FAIL because the post-onboarding console layout has not been adjusted yet

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:
- downgrade onboarding to a compact “completed” banner once setup is done
- preserve diagnostics, advanced settings, and manual controls below it

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: preserve feishu console after onboarding"
```

### Task 7: Full verification and packaging sanity

**Files:**
- Verify only

**Step 1: Run focused frontend tests**

Run: `pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx`
Expected: PASS

**Step 2: Run quick-setup related tests**

Run: `pnpm exec vitest run src/__tests__/App*.test.tsx`
Expected: PASS for touched quick setup flows

**Step 3: Run Rust regression coverage**

Run: `cargo test --test test_packaged_plugin_host_resources`
Expected: PASS

**Step 4: Run desktop fast-path verification**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 5: Run packaging sanity**

Run: `pnpm build:runtime`
Expected: PASS and produce Windows installer artifacts

**Step 6: Commit**

```bash
git add .
git commit -m "feat: add guided feishu onboarding flow"
```
