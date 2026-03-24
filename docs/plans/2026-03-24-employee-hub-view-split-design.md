# Employee Hub View Split Design

**Goal:** Turn `apps/runtime/src/components/employees/EmployeeHubView.tsx` into the next formal frontend large-file split target by moving employee-center tab-specific workflows, local Tauri access, and bulky render branches into focused employee modules while preserving the current employee-center UX.

## Strategy Summary

- Change surface: `apps/runtime/src/components/employees/EmployeeHubView.tsx` structure, local state placement, tab-specific Tauri access placement, and major employee-center presentation boundaries
- Affected modules: `EmployeeHubView.tsx`, existing employee components such as `EmployeeFeishuAssociationSection`, possible new employee tab sections or helpers, and the already-existing `EmployeeHubScene`
- Main risk: mixing a structural split with employee-center UX changes, especially around teams, runs, memory tools, Feishu association, and settings workflows
- Recommended smallest safe path: keep `EmployeeHubScene` as the workflow container, preserve current tabs and visible behavior, then split `EmployeeHubView` by employee-center domain rather than by arbitrary JSX chunks
- Required verification for implementation: focused `EmployeeHubView` suites for overview, Feishu association, thread binding, and group orchestration behavior, plus `pnpm report:frontend-large-files`
- Release impact: none if the first pass stays structure-only and keeps current employee-center contracts and UX unchanged

## Current State

`EmployeeHubScene` already exists and has removed shell-level employee workflow ownership from `App.tsx`. That was the right first move, but `EmployeeHubView.tsx` is still about 1596 lines and remains a large mixed-concern runtime view.

Today the view still mixes:

- local tab shell ownership
- employee selection and highlight display
- employee memory/profile tooling
- Feishu association saving
- team creation and group mutation forms
- recent run reporting and launch actions
- direct `invoke(...)` usage for several employee-center utilities
- large tab-specific render branches

So the next split should not repeat the old scene extraction. The real issue now is that the view itself still owns too many employee-center domains.

## Approach Options Considered

### Option 1: Thin tab shell plus domain sections and employee-local helpers

Keep `EmployeeHubView.tsx` as the tab shell, but move tab-heavy rendering and local Tauri utility work into focused employee modules.

Pros:

- lowest behavior risk
- builds directly on the existing `EmployeeHubScene`
- keeps the successful WorkClaw split pattern consistent with `SettingsView` and `ChatView`
- creates stable landing zones for future employee-center work

Cons:

- root view still remains a composition shell rather than a tiny wrapper
- requires discipline so the split improves boundaries instead of creating many prop-heavy wrappers

### Option 2: One giant employee hub controller hook

Move nearly all local view state, effects, and handlers into one `useEmployeeHubController`.

Pros:

- root view would shrink quickly

Cons:

- simply hides the giant-file problem in a hook
- poor fit for the current frontend guardrail goal
- likely to blur view-only state, Tauri access, and employee workflow orchestration

### Option 3: Pure JSX tab slicing

Extract big tab panels but leave the same Tauri access, derived state, and local tab logic in the root.

Pros:

- easiest mechanical first step

Cons:

- does not solve the real boundary problem
- tends to create prop-drilling shells rather than true domain ownership

## Recommended Approach

Use **Option 1: thin tab shell plus domain sections and employee-local helpers**.

The split should assume:

- `EmployeeHubScene` already owns top-level employee workflow entry
- `EmployeeHubView` should become a tab shell and composition surface
- employee-center tab domains should own their own render-heavy and utility-heavy behavior

This is the employee-center equivalent of the `SettingsView` and `ChatView` direction:

- root shell stays visible
- domain sections own presentation
- small helper/service modules absorb local I/O and formatting work

## Proposed Target Structure

```text
apps/runtime/src/components/employees/
  EmployeeHubView.tsx
  EmployeeHubTabNav.tsx
  overview/
    EmployeeOverviewSection.tsx
  directory/
    EmployeeDirectorySection.tsx
    employeeDirectoryHelpers.ts
  teams/
    EmployeeTeamsSection.tsx
    employeeTeamForms.ts
  runs/
    EmployeeRunsSection.tsx
  settings/
    EmployeeSettingsSection.tsx
  tools/
    EmployeeMemoryToolsSection.tsx
    EmployeeProfileFilesSection.tsx
  services/
    employeeHubViewService.ts
```

This is a direction, not a required exact file list. The main rule is that each new file must own a real employee-center domain.

## Responsibility Split

### `EmployeeHubScene.tsx`

Should keep:

- selected employee orchestration
- shell-level navigation callbacks
- scene-level delete / set-main / launch flows
- open-request and highlight semantics

Should not absorb:

- tab-specific local UI state that only matters inside the employee center view
- large tab-specific rendering

### `EmployeeHubView.tsx`

Should keep:

- employee-center tab shell
- tab switch state
- composition of tab sections
- only the cross-tab view glue that truly spans multiple employee-center surfaces

Should stop directly owning:

- most tab-specific `invoke(...)` calls
- most tab-specific forms and validation clusters
- large settings and tools render branches
- deeply nested team and run presentation blocks

### Overview section

Should own:

- overview metrics cards
- pending item cards
- overview-only summaries and navigation affordances

### Directory section

Should own:

- employee list and employee detail presentation
- employee-specific actions close to the selected employee surface
- directory-local render helpers

### Teams section

Should own:

- group creation/editing form rendering
- team roster presentation
- group rule and member visuals

### Runs section

Should own:

- recent run reporting UI
- run summaries and run-oriented actions

### Settings / tools sections

Should own:

- employee memory tools
- profile file tools
- Feishu association settings presentation
- any employee-center settings that are not scene-wide

### Employee-local service/helper modules

Should own:

- local `invoke(...)` wrappers that are specific to the view layer
- payload shaping for memory/profile/export utilities
- formatting and tab-specific derived helpers

They should not own React state or shell navigation semantics.

## Split Priorities

### First extraction targets

The safest first cuts are:

- memory/profile tools
- teams tab form and list rendering
- runs tab rendering
- overview metrics and pending cards

These are large, visually distinct, and easier to move without disturbing the outer employee-center shell.

### Later extraction targets

More careful follow-ups can include:

- selected employee directory details
- employee settings domain helpers
- any remaining employee-center local service wrappers

## Non-Goals

- No employee-center redesign in the first split pass
- No backend command or schema changes
- No rewrite of `EmployeeHubScene`
- No migration to a different state model
- No giant `useEmployeeHubController`

## Acceptance Criteria

The first split should be considered successful when:

- `EmployeeHubView.tsx` no longer directly carries most tab-specific `invoke(...)` clusters
- `EmployeeHubView.tsx` no longer directly carries the largest teams/tools/render branches
- `EmployeeHubScene` remains the scene container and does not regrow into a giant shell
- employee-center UX, tabs, and flows remain unchanged
- focused employee-center tests still pass
- `pnpm report:frontend-large-files` shows a meaningful drop in `EmployeeHubView.tsx`

## Risks To Watch

### Hidden tab-helper regression

The easiest failure mode is pushing too much into one `employeeHubHelpers.ts` file. Helpers should stay domain-local.

### Scene/view boundary drift

If tab-specific view state gets moved back into `EmployeeHubScene`, the split will just relocate complexity instead of reducing it.

### New child giant regression

A split is not successful if `EmployeeHubView.tsx` shrinks but one new `EmployeeTeamsSection.tsx` immediately becomes another 900-line giant file.

## Recommended Next Step

Write an implementation plan that starts with tab/domain extraction rather than another scene rewrite, uses the existing employee-center tests as guardrails, and treats `EmployeeHubScene` as the stable container boundary.
