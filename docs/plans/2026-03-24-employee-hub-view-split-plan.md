# Employee Hub View Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src/components/employees/EmployeeHubView.tsx` into a thinner employee-center tab shell plus focused domain sections without changing current user-visible behavior.

**Architecture:** Keep `EmployeeHubScene.tsx` as the employee workflow container, keep `EmployeeHubView.tsx` as the employee-center tab shell, and move tab-heavy rendering plus employee-local utility work into focused employee domains. Preserve current tabs, Tauri command contracts, and employee-center behavior.

**Tech Stack:** React, TypeScript, Vitest, existing employee-center scene helpers, Tauri `invoke(...)` utilities, frontend large-file guardrails.

---

### Task 1: Freeze the tab shell contract

**Files:**
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Create: `apps/runtime/src/components/employees/EmployeeHubTabNav.tsx`
- Test: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx`

**Step 1: Use the overview suite as the shell guardrail**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx
```

**Step 2: Extract the employee-center tab navigation shell**

Move tab-nav presentation into `EmployeeHubTabNav.tsx` while keeping `EmployeeHubView.tsx` as the composition entry.

**Step 3: Re-run the focused suite**

Run the same command from Step 1.

**Step 4: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/EmployeeHubTabNav.tsx
git commit -m "refactor(ui): extract employee hub tab shell"
```

### Task 2: Extract overview presentation

**Files:**
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Create: `apps/runtime/src/components/employees/overview/EmployeeOverviewSection.tsx`
- Test: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx`

**Step 1: Keep overview metrics and pending cards together**

Move overview-only cards and summaries into `EmployeeOverviewSection.tsx`.

**Step 2: Verify**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx
```

**Step 3: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/overview/EmployeeOverviewSection.tsx
git commit -m "refactor(ui): extract employee hub overview section"
```

### Task 3: Extract teams domain rendering

**Files:**
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Create: `apps/runtime/src/components/employees/teams/EmployeeTeamsSection.tsx`
- Test: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx`

**Step 1: Use the team/group suite as the hard guardrail**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx
```

**Step 2: Move team creation and group rendering into `EmployeeTeamsSection.tsx`**

Keep mutation ownership in the view/scene boundary, but move the large team-oriented presentation block out of the root.

**Step 3: Re-run the suite**

Run the same command from Step 1.

**Step 4: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/teams/EmployeeTeamsSection.tsx
git commit -m "refactor(ui): extract employee teams section"
```

### Task 4: Extract runs presentation

**Files:**
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Create: `apps/runtime/src/components/employees/runs/EmployeeRunsSection.tsx`
- Test: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx`

**Step 1: Keep recent runs and run-entry actions together**

Move the runs tab rendering into `EmployeeRunsSection.tsx`.

**Step 2: Verify**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx
```

**Step 3: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/runs/EmployeeRunsSection.tsx
git commit -m "refactor(ui): extract employee runs section"
```

### Task 5: Extract memory/profile tools

**Files:**
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Create: `apps/runtime/src/components/employees/tools/EmployeeMemoryToolsSection.tsx`
- Create: `apps/runtime/src/components/employees/tools/EmployeeProfileFilesSection.tsx`
- Test: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.employee-id-flow.test.tsx`

**Step 1: Move tools-heavy UI out of the root**

Extract employee memory and profile-file presentation into focused tool sections.

**Step 2: Verify**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.employee-id-flow.test.tsx
```

**Step 3: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/tools/EmployeeMemoryToolsSection.tsx apps/runtime/src/components/employees/tools/EmployeeProfileFilesSection.tsx
git commit -m "refactor(ui): extract employee tools sections"
```

### Task 6: Reassess local service/helper extraction

**Files:**
- Modify only if needed:
  - `apps/runtime/src/components/employees/EmployeeHubView.tsx`
  - `apps/runtime/src/components/employees/services/employeeHubViewService.ts`
- Tests:
  - `src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx`
  - `src/components/employees/__tests__/EmployeeHubView.thread-binding.test.tsx`

**Step 1: Inspect remaining local `invoke(...)` clusters**

Only extract employee-local view utilities if they are still materially bloating the root.

**Step 2: Verify**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx src/components/employees/__tests__/EmployeeHubView.thread-binding.test.tsx
```

**Step 3: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/services/employeeHubViewService.ts
git commit -m "refactor(ui): stabilize employee hub local services"
```

### Verification Checkpoint

Before calling the split ready for review, run the smallest honest verification set for the touched employee-center surfaces:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx src/components/employees/__tests__/EmployeeHubView.thread-binding.test.tsx src/components/employees/__tests__/EmployeeHubView.employee-id-flow.test.tsx
pnpm report:frontend-large-files
git diff --check
```

## Expected Outcome

After these tasks:

- `EmployeeHubView.tsx` should become a thinner employee-center tab shell
- tab-heavy rendering should live in focused employee sections
- `EmployeeHubScene.tsx` should remain the stable workflow container
- employee-center UX should remain unchanged
- the repo should gain a third strong frontend split reference after `SettingsView` and `ChatView`
