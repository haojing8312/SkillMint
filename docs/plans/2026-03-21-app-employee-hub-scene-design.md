# App Employee Hub Scene Design

**Goal:** Reduce the size and responsibility of `apps/runtime/src/App.tsx` by extracting the employee-center workflow into an isolated scene container without changing user-visible behavior.

## Scope

- Extract employee-center orchestration out of `App.tsx`
- Keep existing Tauri commands, data models, and UI behavior unchanged
- Reuse the existing `EmployeeHubView` as the primary presentation component
- Avoid global state library changes, router changes, or backend protocol changes

## Current Problem

`apps/runtime/src/App.tsx` currently mixes:

- app-shell concerns
- cross-view navigation
- employee-center data loading
- employee-center mutations and side effects
- employee-entered task launch flow

This makes the file hard to modify safely and causes unrelated changes to compete inside the same module.

## Proposed Design

### 1. Introduce an employee scene container

Create a dedicated scene module, e.g. `apps/runtime/src/scenes/employees/EmployeeHubScene.tsx`.

This scene will own:

- employee-center state
- employee-center data refresh logic
- employee-center mutation handlers
- mapping between Tauri command results and `EmployeeHubView` props

`App.tsx` will only decide when to render the employee scene and pass a small set of top-level dependencies.

### 2. Add an employee API wrapper

Create a small API wrapper, e.g. `apps/runtime/src/scenes/employees/employeeHubApi.ts`, to centralize employee-scene `invoke(...)` calls.

This wrapper will:

- keep command names unchanged
- convert raw invoke calls into named functions
- reduce direct Tauri coupling inside React view orchestration

### 3. Preserve existing UI composition

Keep `apps/runtime/src/components/employees/EmployeeHubView.tsx` as the main presentation layer for now.

This refactor is not a redesign of employee-center internals. It is a responsibility move:

- `App.tsx` becomes the shell
- `EmployeeHubScene` becomes the employee workflow container
- `EmployeeHubView` remains the UI surface

## Responsibility Split

### `App.tsx`

- active main view switching
- global dialogs and app-shell layout
- top-level session navigation
- cross-scene callbacks that truly belong to the shell

### `EmployeeHubScene`

- load employees, groups, and employee-scene supporting data
- handle save/delete/set-main/start-task actions
- hold employee-scene initial tab / highlight state
- call shell-level navigation callbacks when needed

### `EmployeeHubView`

- render tabs, forms, lists, summaries, and local UI interactions
- remain presentation-oriented

## Migration Strategy

Use a conservative staged extraction:

1. Add `employeeHubApi.ts` with no behavior changes
2. Add `EmployeeHubScene.tsx` that initially receives props from `App.tsx`
3. Move employee-specific state and handlers from `App.tsx` into `EmployeeHubScene`
4. Keep only shell-level dependencies in `App.tsx`
5. Verify behavior through existing employee-related UI tests

This minimizes blast radius and allows the refactor to stop safely after each stage.

## Risks

- Some employee actions currently depend on session-launch helpers in `App.tsx`
- Some highlight / initial-tab logic may still be partially shell-owned
- Over-extraction into tiny hooks would create indirection without improving boundaries

## Non-Goals

- No backend refactor
- No new state management library
- No redesign of `EmployeeHubView`
- No broad scene architecture rollout beyond employee center in this phase

## Success Criteria

- `App.tsx` is materially smaller and more shell-focused
- employee-center logic is isolated in one scene module
- existing employee-center behavior and tests continue to pass
- the same extraction pattern can later be reused for settings or experts
