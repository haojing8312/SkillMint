# Quick Setup Titlebar Safety Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Keep desktop window controls reachable while the quick setup dialog is open.

**Architecture:** Preserve the existing frameless Tauri window and custom titlebar. Adjust the quick setup overlay to reserve the titlebar height as a safe area instead of covering the full viewport, then lock that behavior with a focused runtime test.

**Tech Stack:** React, Tauri window chrome, Vitest, Testing Library, Tailwind utility classes

---

### Task 1: Lock the expected behavior with a failing test

**Files:**
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`

**Step 1: Write the failing test**

Add assertions that the quick setup dialog keeps the app titlebar visible and uses a top offset that preserves the titlebar safety area.

**Step 2: Run test to verify it fails**

Run: `pnpm vitest apps/runtime/src/__tests__/App.model-setup-hint.test.tsx --run`

Expected: FAIL because the dialog still uses a full-screen `inset-0` overlay.

### Task 2: Implement the smallest layout fix

**Files:**
- Modify: `apps/runtime/src/components/ModelSetupOverlays.tsx`

**Step 1: Reserve titlebar space**

Replace the full-screen quick setup overlay positioning with a layout that starts below the desktop titlebar and shrink the panel height calculation to match.

**Step 2: Keep behavior unchanged otherwise**

Do not change dismissal logic, scrolling, or onboarding step behavior.

### Task 3: Verify the runtime surface

**Files:**
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`
- Modify: `apps/runtime/src/components/ModelSetupOverlays.tsx`

**Step 1: Run focused runtime tests**

Run: `pnpm vitest apps/runtime/src/__tests__/App.model-setup-hint.test.tsx apps/runtime/src/__tests__/App.window-chrome.test.tsx --run`

Expected: PASS

**Step 2: Run WorkClaw verification command for user-facing runtime flow**

Run: `pnpm test:e2e:runtime`

Expected: PASS, or report any pre-existing failures clearly if they block full verification.
