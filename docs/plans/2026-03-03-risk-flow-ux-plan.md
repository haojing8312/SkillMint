# High-Risk Flow UX Standardization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Standardize medium/high-risk operations across runtime UI using a shared frontend risk policy and confirmation dialog, while preserving backend command semantics.

**Architecture:** Introduce a shared risk metadata module and a single reusable confirmation dialog component. Integrate view-by-view via thin adapters so destructive/sensitive actions consistently require confirmation, prevent duplicate execution, and show visible feedback. Keep all invoke payloads and command names unchanged.

**Tech Stack:** React 18, TypeScript, Tailwind v4 semantic classes (`sm-*`), Vitest, Testing Library, Tauri API mocks

---

Related skills: `@test-driven-development`, `@verification-before-completion`

### Task 1: Add Shared Risk Policy and Confirmation Dialog Contract Tests

**Files:**
- Create: `apps/runtime/src/components/risk-action.ts`
- Create: `apps/runtime/src/components/RiskConfirmDialog.tsx`
- Create: `apps/runtime/src/components/__tests__/RiskConfirmDialog.test.tsx`

**Step 1: Write the failing test**

```tsx
import { render, screen } from "@testing-library/react";
import { RiskConfirmDialog } from "../RiskConfirmDialog";

test("renders high-risk irreversible warning and confirm button", () => {
  render(
    <RiskConfirmDialog
      open
      level="high"
      title="删除技能"
      summary="将永久删除本地技能"
      impact="会移除该技能及相关会话入口"
      irreversible
      confirmLabel="确认删除"
      cancelLabel="取消"
      loading={false}
      onConfirm={() => {}}
      onCancel={() => {}}
    />
  );
  expect(screen.getByText("删除技能")).toBeInTheDocument();
  expect(screen.getByText(/不可逆/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "确认删除" })).toBeInTheDocument();
});
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/RiskConfirmDialog.test.tsx`
Expected: FAIL (component/module missing)

**Step 3: Write minimal implementation**

Implement `RiskConfirmDialog` and `risk-action` types:

```ts
export type RiskLevel = "low" | "medium" | "high";
export interface RiskActionMeta {
  level: RiskLevel;
  title: string;
  summary: string;
  impact?: string;
  irreversible?: boolean;
  confirmLabel?: string;
}
```

```tsx
export function RiskConfirmDialog({ open, ...props }: Props) {
  if (!open) return null;
  return <div role="dialog">...</div>;
}
```

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/RiskConfirmDialog.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/risk-action.ts apps/runtime/src/components/RiskConfirmDialog.tsx apps/runtime/src/components/__tests__/RiskConfirmDialog.test.tsx
git commit -m "feat(ui): add shared risk action policy and confirm dialog"
```

### Task 2: Enforce High-Risk Confirmation on Sidebar Permission Mode Switch

**Files:**
- Modify: `apps/runtime/src/components/Sidebar.tsx`
- Modify: `apps/runtime/src/components/risk-action.ts`
- Create: `apps/runtime/src/components/__tests__/Sidebar.risk-flow.test.tsx`

**Step 1: Write the failing test**

```tsx
test("switching to unrestricted requires explicit confirmation", async () => {
  // render Sidebar with selectedSkillId
  // change select to unrestricted
  // expect risk dialog shown
  // click cancel -> onChangeNewSessionPermissionMode not called with unrestricted
});
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/Sidebar.risk-flow.test.tsx`
Expected: FAIL (no confirmation gating yet)

**Step 3: Write minimal implementation**

Add local pending mode state + `RiskConfirmDialog`:

```tsx
if (nextMode === "unrestricted") {
  setPendingPermissionMode(nextMode);
  setConfirmOpen(true);
  return;
}
onChangeNewSessionPermissionMode(nextMode);
```

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/Sidebar.risk-flow.test.tsx src/components/__tests__/Sidebar.theme.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/Sidebar.tsx apps/runtime/src/components/risk-action.ts apps/runtime/src/components/__tests__/Sidebar.risk-flow.test.tsx
git commit -m "feat(ui): guard unrestricted permission mode with high-risk confirm"
```

### Task 3: Standardize ChatView High-Risk/Medium-Risk Flows

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/risk-action.ts`
- Modify: `apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx`
- Create: `apps/runtime/src/components/__tests__/ChatView.risk-flow.test.tsx`

**Step 1: Write the failing test**

```tsx
test("canceling install confirmation does not call install_clawhub_skill", async () => {
  // trigger install suggestion
  // open confirm dialog
  // cancel
  // assert invoke not called for install
});

test("confirming install calls install once and locks duplicate clicks", async () => {
  // click confirm twice while loading
  // expect single invoke call
});
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/ChatView.risk-flow.test.tsx`
Expected: FAIL (duplicate lock/contract not fully enforced)

**Step 3: Write minimal implementation**

Normalize install/stop/tool-confirm via shared risk metadata and in-flight locks.

```tsx
if (installingSlug) return;
setInstallingSlug(slug);
```

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/ChatView.risk-flow.test.tsx src/components/__tests__/ChatView.find-skills-install.test.tsx src/components/__tests__/ChatView.theme.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx apps/runtime/src/components/__tests__/ChatView.risk-flow.test.tsx
git commit -m "refactor(ui): standardize chat risk confirmations and duplicate-action locks"
```

### Task 4: Standardize Experts and InstallDialog Risk Flows

**Files:**
- Modify: `apps/runtime/src/components/experts/ExpertsView.tsx`
- Modify: `apps/runtime/src/components/experts/FindSkillsView.tsx`
- Modify: `apps/runtime/src/components/experts/SkillLibraryView.tsx`
- Modify: `apps/runtime/src/components/InstallDialog.tsx`
- Modify: `apps/runtime/src/components/risk-action.ts`
- Modify: `apps/runtime/src/components/experts/__tests__/ExpertsView.test.tsx`
- Create: `apps/runtime/src/components/experts/__tests__/ExpertsRiskFlow.test.tsx`
- Modify: `apps/runtime/src/components/__tests__/InstallDialog.industry-pack.test.tsx`

**Step 1: Write the failing test**

```tsx
test("remove skill requires high-risk confirmation before callback", () => {
  // click remove -> dialog opens
  // cancel -> onDeleteSkill not called
  // confirm -> onDeleteSkill called once
});
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime && pnpm test -- src/components/experts/__tests__/ExpertsRiskFlow.test.tsx`
Expected: FAIL

**Step 3: Write minimal implementation**

Wrap remove/update/install triggers through `RiskConfirmDialog` with level mapping:
- remove: `high`
- update/install: `medium`

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime && pnpm test -- src/components/experts/__tests__/ExpertsView.test.tsx src/components/experts/__tests__/ExpertsRiskFlow.test.tsx src/components/__tests__/InstallDialog.industry-pack.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/experts/ExpertsView.tsx apps/runtime/src/components/experts/FindSkillsView.tsx apps/runtime/src/components/experts/SkillLibraryView.tsx apps/runtime/src/components/InstallDialog.tsx apps/runtime/src/components/experts/__tests__/ExpertsView.test.tsx apps/runtime/src/components/experts/__tests__/ExpertsRiskFlow.test.tsx apps/runtime/src/components/__tests__/InstallDialog.industry-pack.test.tsx
git commit -m "feat(ui): unify experts and install risk confirmation flows"
```

### Task 5: Standardize Settings and EmployeeHub Destructive Operations

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Modify: `apps/runtime/src/components/risk-action.ts`
- Modify: `apps/runtime/src/components/__tests__/SettingsView.theme.test.tsx`
- Modify: `apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx`
- Create: `apps/runtime/src/components/__tests__/SettingsView.risk-flow.test.tsx`
- Create: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.risk-flow.test.tsx`

**Step 1: Write the failing test**

```tsx
test("delete employee requires high-risk confirmation", async () => {
  // trigger delete
  // dialog visible
  // cancel -> delete invoke not called
  // confirm -> delete invoke called once
});
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/SettingsView.risk-flow.test.tsx`
Expected: FAIL

**Step 3: Write minimal implementation**

Integrate shared risk dialog in settings and employee hub destructive operations, unify success/failure feedback messages.

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime && pnpm test -- src/components/__tests__/SettingsView.risk-flow.test.tsx src/components/employees/__tests__/EmployeeHubView.risk-flow.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.theme.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/__tests__/SettingsView.theme.test.tsx apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx apps/runtime/src/components/__tests__/SettingsView.risk-flow.test.tsx apps/runtime/src/components/employees/__tests__/EmployeeHubView.risk-flow.test.tsx
git commit -m "feat(ui): apply unified high-risk confirmation in settings and employee hub"
```

### Task 6: Standardize Packaging Export Feedback and Action Locks

**Files:**
- Modify: `apps/runtime/src/components/packaging/PackForm.tsx`
- Modify: `apps/runtime/src/components/packaging/IndustryPackView.tsx`
- Modify: `apps/runtime/src/components/packaging/__tests__/IndustryPackView.test.tsx`
- Create: `apps/runtime/src/components/packaging/__tests__/PackForm.risk-flow.test.tsx`

**Step 1: Write the failing test**

```tsx
test("export action locks button while packing and shows completion message", async () => {
  // trigger export
  // expect button disabled/loading text
  // expect success message after completion
});
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime && pnpm test -- src/components/packaging/__tests__/PackForm.risk-flow.test.tsx`
Expected: FAIL

**Step 3: Write minimal implementation**

Ensure export actions use consistent lock/message contract aligned with low-risk action standards.

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime && pnpm test -- src/components/packaging/__tests__/PackForm.risk-flow.test.tsx src/components/packaging/__tests__/IndustryPackView.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/packaging/PackForm.tsx apps/runtime/src/components/packaging/IndustryPackView.tsx apps/runtime/src/components/packaging/__tests__/PackForm.risk-flow.test.tsx apps/runtime/src/components/packaging/__tests__/IndustryPackView.test.tsx
git commit -m "refactor(ui): normalize packaging action feedback and in-flight locking"
```

### Task 7: Full Verification and Final Consistency Pass

**Files:**
- Modify: `apps/runtime/src/components/risk-action.ts` (if minor mapping gaps found)
- Modify: `apps/runtime/src/index.css` (only if dialog/token style adjustments needed)
- Test: all runtime frontend tests

**Step 1: Write final guard assertion (if missing)**

Add one guard in risk-flow tests ensuring all high-risk actions use dialog path.

```tsx
expect(screen.getByRole("dialog")).toBeInTheDocument();
```

**Step 2: Run full tests**

Run: `cd apps/runtime && pnpm test`
Expected: PASS

**Step 3: Build verification**

Run: `cd apps/runtime && pnpm build`
Expected: PASS (`tsc` + `vite build` success)

**Step 4: Minimal cleanup**

Resolve any test warnings causing flaky behavior (act warnings, async timing).

**Step 5: Commit**

```bash
git add apps/runtime/src/components apps/runtime/src/index.css
git commit -m "feat(ui): complete high-risk flow UX standardization across runtime"
```
