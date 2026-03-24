# Settings View Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src/components/SettingsView.tsx` into stable shell and domain boundaries without changing current user-visible behavior.

**Architecture:** Keep `SettingsView.tsx` as the modal entry and tab shell while extracting tab navigation, domain sections, and domain-local `invoke(...)` wrappers into focused child modules under `apps/runtime/src/components/settings/`. Start with the lowest-risk domains, keep props and Tauri contracts stable, and leave the Feishu domain for the final phase of the first split.

**Tech Stack:** React, TypeScript, Vitest, existing Tauri `invoke(...)` APIs, frontend large-file guardrails.

---

### Task 1: Freeze the shell contract and tab surface

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/SettingsShell.tsx`
- Create: `apps/runtime/src/components/settings/SettingsTabNav.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.theme.test.tsx`

**Step 1: Write a failing shell-level test when needed**

If no existing test already proves the shell renders the expected tab container and honors `initialTab`, add or extend a focused test in:

```tsx
apps/runtime/src/components/__tests__/SettingsView.theme.test.tsx
```

Target assertions:
- `SettingsView` still renders
- `initialTab` still controls the selected tab
- the close entry still exists

**Step 2: Run the focused test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.theme.test.tsx
```

Expected: PASS if existing coverage is already enough, or FAIL only for the newly added missing assertion

**Step 3: Extract shell-only components**

Create:
- `SettingsShell.tsx` for page frame layout
- `SettingsTabNav.tsx` for tab buttons only

Keep:
- `SettingsView` props unchanged
- tab names unchanged
- no domain logic moved yet beyond render structure

**Step 4: Re-run the focused test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.theme.test.tsx
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/SettingsShell.tsx apps/runtime/src/components/settings/SettingsTabNav.tsx apps/runtime/src/components/__tests__/SettingsView.theme.test.tsx
git commit -m "refactor(ui): extract settings shell and tab nav"
```

### Task 2: Extract the models domain

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/models/ModelsSettingsSection.tsx`
- Create: `apps/runtime/src/components/settings/models/modelSettingsService.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.model-connection.test.tsx`

**Step 1: Extend or confirm focused model coverage**

Ensure the focused model settings test still proves:
- model connection feedback renders correctly
- model interactions still use the same visible flow

**Step 2: Run the focused model test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.model-connection.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract model service helpers**

Create `modelSettingsService.ts` and move model-domain `invoke(...)` wrappers and related request shaping there. Keep command names and payloads identical.

**Step 4: Extract `ModelsSettingsSection.tsx`**

Move model-domain JSX and local behavior into the section component. Keep only shell wiring in `SettingsView.tsx`.

**Step 5: Re-run the focused model test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.model-connection.test.tsx
```

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/models/ModelsSettingsSection.tsx apps/runtime/src/components/settings/models/modelSettingsService.ts apps/runtime/src/components/__tests__/SettingsView.model-connection.test.tsx
git commit -m "refactor(ui): split settings models domain"
```

### Task 3: Extract the desktop domain

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/desktop/DesktopSettingsSection.tsx`
- Create: `apps/runtime/src/components/settings/desktop/desktopSettingsService.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.desktop-system-tab.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.data-retention.test.tsx`

**Step 1: Confirm desktop-focused coverage**

Use the existing desktop and retention tests as the guardrail for desktop extraction.

**Step 2: Run the focused desktop tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.desktop-system-tab.test.tsx src/components/__tests__/SettingsView.data-retention.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract desktop service helpers**

Create `desktopSettingsService.ts` and move desktop-runtime `invoke(...)` wrappers there.

**Step 4: Extract `DesktopSettingsSection.tsx`**

Move desktop/runtime-preference JSX and local interaction logic into the section component.

**Step 5: Re-run the focused desktop tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.desktop-system-tab.test.tsx src/components/__tests__/SettingsView.data-retention.test.tsx
```

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/desktop/DesktopSettingsSection.tsx apps/runtime/src/components/settings/desktop/desktopSettingsService.ts apps/runtime/src/components/__tests__/SettingsView.desktop-system-tab.test.tsx apps/runtime/src/components/__tests__/SettingsView.data-retention.test.tsx
git commit -m "refactor(ui): split settings desktop domain"
```

### Task 4: Extract the search and MCP domains

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/search/SearchSettingsSection.tsx`
- Create: `apps/runtime/src/components/settings/search/searchSettingsService.ts`
- Create: `apps/runtime/src/components/settings/mcp/McpSettingsSection.tsx`
- Create: `apps/runtime/src/components/settings/mcp/mcpSettingsService.ts`

**Step 1: Add or confirm focused tests if missing**

If there is no existing focused test for the search or MCP area, add a narrow test that proves:
- the tab still renders
- a representative action still behaves correctly

Prefer adding tests only for the exact surfaces being extracted.

**Step 2: Run the focused tests**

Run the narrow vitest command for any tests added or reused in this step.

Expected: PASS before refactor

**Step 3: Extract search and MCP service helpers**

Move domain-local `invoke(...)`, env parsing, preset application, and error shaping into service helpers where appropriate.

**Step 4: Extract the two section components**

Move search-domain and MCP-domain JSX plus local logic into their own section components.

**Step 5: Re-run the focused tests**

Run the same narrow vitest command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/search/SearchSettingsSection.tsx apps/runtime/src/components/settings/search/searchSettingsService.ts apps/runtime/src/components/settings/mcp/McpSettingsSection.tsx apps/runtime/src/components/settings/mcp/mcpSettingsService.ts
git commit -m "refactor(ui): split settings search and mcp domains"
```

### Task 5: Extract the routing domain

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/routing/RoutingSettingsSection.tsx`
- Create: `apps/runtime/src/components/settings/routing/routingSettingsService.ts`

**Step 1: Add or confirm focused routing coverage**

If routing-specific tests are missing, add a narrow test that protects:
- routing tab rendering
- one representative save or load flow

**Step 2: Run the focused routing test**

Run the narrow vitest command for the reused or newly added routing-focused test.

Expected: PASS before refactor

**Step 3: Extract routing helpers and section**

Move route settings, policy handling, route logs, and stats rendering into the routing domain section and service helper.

**Step 4: Re-run the focused routing test**

Run the same narrow vitest command from Step 2.

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/routing/RoutingSettingsSection.tsx apps/runtime/src/components/settings/routing/routingSettingsService.ts
git commit -m "refactor(ui): split settings routing domain"
```

### Task 6: Extract the Feishu domain last

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Create: `apps/runtime/src/components/settings/feishu/FeishuSettingsSection.tsx`
- Create: `apps/runtime/src/components/settings/feishu/FeishuOnboardingSection.tsx`
- Create: `apps/runtime/src/components/settings/feishu/feishuSettingsService.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx`

**Step 1: Run the existing focused Feishu tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx
```

Expected: PASS before refactor

**Step 2: Extract Feishu service helpers**

Move Feishu-domain `invoke(...)` wrappers, installer polling helpers, and response shaping into `feishuSettingsService.ts`.

**Step 3: Extract Feishu sections**

Move connector management, onboarding, installer output rendering, advanced settings, and pairing UI into focused section components. Keep public behavior and tab contracts unchanged.

**Step 4: Re-run the focused Feishu tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx src/components/__tests__/SettingsView.feishu.test.tsx src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings/feishu/FeishuSettingsSection.tsx apps/runtime/src/components/settings/feishu/FeishuOnboardingSection.tsx apps/runtime/src/components/settings/feishu/feishuSettingsService.ts apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx apps/runtime/src/components/__tests__/SettingsView.feishu-routing-wizard.test.tsx
git commit -m "refactor(ui): split settings feishu domain"
```

### Task 7: Trim and verify the root shell

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Verify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Remove stale root-only leftovers**

Trim:
- unused imports
- leftover helper functions that now belong in domain services
- stale state variables that moved to sections

Keep only:
- shell props
- active tab selection
- shared tab-to-section assembly

**Step 2: Run the focused settings suite**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/SettingsView.*
```

Expected: PASS

**Step 3: Run one app-shell smoke test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.model-setup-hint.test.tsx
```

Expected: PASS

**Step 4: Re-run the frontend large-file report**

Run:

```bash
pnpm report:frontend-large-files
```

Expected:
- `SettingsView.tsx` is materially smaller
- no newly created child file becomes a giant replacement without clear responsibility

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/settings
git commit -m "refactor(ui): thin settings view shell"
```
