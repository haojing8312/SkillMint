# Frontend AI Guardrails Design

**Goal:** Establish repository guidance for AI-native frontend development in `apps/runtime/src/` so large React and TypeScript files stop growing by default.

## Why This Comes First

WorkClaw's frontend runtime already has several files that are far beyond healthy page or component size. As of 2026-03-23, `SettingsView.tsx` is about 4998 lines, `ChatView.tsx` is about 3984 lines, `EmployeeHubView.tsx` is about 1648 lines, and `App.tsx` is about 702 lines.

The problem is not only historical code shape. In AI-assisted development, large frontend files keep absorbing new state, new `invoke(...)` calls, and new render branches unless the repository gives the coding agent a clear module map, a file-budget policy, and preferred landing zones for new logic.

This design treats guidance files and simple reporting as the first control layer. The goal is to improve file placement before adding stronger automation.

## Research-Based Policy

The proposed thresholds are governance triggers, not absolute quality scores.

- `<= 300` lines: normal target zone for runtime frontend files
- `301-500` lines: warning zone; avoid adding net-new page state, Tauri I/O, or major render branches until module placement is reconsidered
- `501+` lines: split-design zone; write or update a short split plan before implementing new feature work in that file

These thresholds should apply to production runtime `ts` and `tsx` files under `apps/runtime/src/`, not to generated files or test files.

The frontend thresholds are intentionally stricter than the Rust runtime thresholds because JSX, hooks, conditional rendering, and event orchestration become hard to reason about earlier than comparable Rust code.

## Anti-Goals

- Do not turn this into CI failure policy in the first phase
- Do not block narrow bug fixes in large files when fast repair is safer
- Do not force one-file-per-hook or one-file-per-render-helper micro-splits
- Do not mechanically split a file just to get under a number if responsibilities are still mixed
- Do not require immediate migration to a strict framework architecture beyond current repo direction

## Recommended Frontend Layering

For `apps/runtime/src/`, the default landing zones should be:

- `App.tsx`: app shell, top-level dependency wiring, and high-level view switching
- `scenes/<domain>/`: page-level or domain-level orchestration, state ownership, and cross-component coordination
- `components/<domain>/`: presentation components, local UI interaction, and reusable visual blocks
- `hooks/`: reusable stateful logic, subscriptions, async loading logic, and view-model hooks that return state and actions but not JSX
- `api/` or `services/`: Tauri `invoke(...)` wrappers, frontend data access helpers, and error mapping for remote or backend calls
- `lib/` or `utils/`: pure functions, formatting, projections, and non-React helpers
- `types.ts` or `types/<domain>.ts`: shared frontend-facing types

This is intentionally pragmatic rather than framework-pure. It matches the current repository direction where `App.tsx` is already delegating to `scenes/*` coordinators and selected view flows are moving toward scene containers.

## Responsibility Rules

### App Shell

- owns app-shell concerns, top-level render routing, and only the cross-scene dependencies that truly belong at the shell
- should not keep accumulating domain-specific orchestration once a scene boundary is clear

### Scene Layer

- owns page-level or domain-level state, orchestration, and cross-component workflow coordination
- is the default home for flows that would otherwise make `App.tsx`, `ChatView.tsx`, or `SettingsView.tsx` absorb more logic
- may call hooks and services, but should keep view assembly readable

### Component Layer

- owns presentation and local interaction that is easiest to understand close to JSX
- should not become the long-term home for large clusters of `invoke(...)` calls, multi-step workflow logic, or reusable state machines
- should not directly own page-level state unless there is a clear reason not to introduce a scene or coordinator

### Hooks Layer

- owns reusable state logic, subscriptions, and derived view models
- should return data, derived state, and actions, but not JSX
- should not hide major domain boundaries that belong in services or scenes

### API Or Service Layer

- owns Tauri `invoke(...)` wrappers, request shaping, response normalization, and backend-specific error handling
- keeps backend access details out of scenes and components
- should not own React state

### Lib Or Utils Layer

- owns pure helper logic only
- should not become a fallback junk drawer for poorly placed orchestration

## Responsibility Trigger Rule

Line count is the only automatic reporting metric in phase 1, but it should not be the only signal used by humans or coding agents.

A frontend file should be considered for split planning even below the warning threshold when it mixes two or more of these concerns:

- page-level or scene-level orchestration
- backend or Tauri data access
- large JSX presentation blocks
- reusable subscription or state-machine logic

This rule exists because frontend complexity often becomes painful before file size crosses a hard threshold.

## What Should Move Out Of Root Components

- large clusters of `invoke(...)` calls and backend error mapping
- reusable form state and validation logic
- event listener setup and teardown flows
- pure projection, filtering, grouping, and formatting logic
- distinct screen sections that can be understood as self-contained UI blocks
- reusable overlay, wizard, card, or panel logic

## What Can Stay Close To JSX

- the final composition of scene or page sections
- small local interaction handlers that are only meaningful in one view
- small derived render values that are easier to read in place
- compatibility glue that is clearer at the view entrypoint than in a helper file

## File Count Guardrail

To avoid replacing giant files with noisy micro-files, a new frontend file should meet at least one of these conditions:

- it owns a separate page or domain orchestration concern
- it owns a distinct reusable UI block
- it owns a reusable stateful behavior or subscription concern
- it owns backend access logic or protocol shaping
- extracting it removes meaningful branching, state ownership, or render complexity from a larger file

Avoid one-file-per-helper and one-file-per-render-function decomposition.

## Reference Template

The frontend needs a concrete reference split, not only abstract principles.

The current reference direction should be:

- `App.tsx` as the shell
- `scenes/<domain>/...` as orchestration boundaries
- `components/...` as presentation boundaries
- narrow service or API wrappers for backend access

The employee hub extraction work is the current best repository example of this direction. The design in `docs/plans/2026-03-21-app-employee-hub-scene-design.md` shows the intended move from shell-heavy composition toward scene-oriented ownership.

## Guidance File Layout

The repository should use two levels of guidance for frontend runtime work:

1. Root `AGENTS.md`
   - short cross-repo rule that points frontend runtime work to the closer guidance
   - high-level threshold summary only

2. `apps/runtime/src/AGENTS.md`
   - frontend-runtime-specific layering rules
   - file-budget rules
   - scene/component/hook/service boundaries
   - reminders to avoid both giant files and micro-file sprawl

Detailed rationale belongs in docs, not in the short local guidance file.

## Reporting And Backlog

Phase 1 should add a simple reporting script, not CI enforcement.

- add `scripts/report-frontend-large-files.mjs`
- add root script `report:frontend-large-files`
- scope it to `apps/runtime/src/`
- include production `ts` and `tsx` files
- exclude `__tests__`, `*.test.*`, `*.spec.*`, and obvious generated or build outputs

The script should classify files as:

- `WARN` for `301-500`
- `PLAN` for `501+`

The repo should also maintain a generated or curated backlog document, for example:

- `docs/plans/2026-03-23-frontend-large-file-backlog.md`

## Initial Backlog Priorities

The first governance targets should be:

- `apps/runtime/src/components/SettingsView.tsx`
- `apps/runtime/src/components/ChatView.tsx`
- `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- `apps/runtime/src/App.tsx`

Secondary watchlist candidates include:

- `apps/runtime/src/components/ModelSetupOverlays.tsx`
- `apps/runtime/src/scenes/useQuickSetupCoordinator.ts`
- `apps/runtime/src/components/experts/SkillLibraryView.tsx`
- `apps/runtime/src/components/NewSessionLanding.tsx`

The backlog should prioritize files that combine view rendering, backend access, and page orchestration in one place.

## Definition Of Improvement

A frontend file is not considered improved only because it falls under the threshold.

A split is successful when:

- state ownership is clearer
- data access is more isolated
- view composition is easier to read
- scene, component, hook, and service boundaries are more obvious after the change

A split is not successful if a giant root component simply becomes a giant child component or a hook that hides the same amount of orchestration.

## Future Automation, But Not In Phase 1

The first phase is documentation and reporting only. Future phases may add:

- CI comment warnings for new files that exceed thresholds
- lint rules for forbidden direct `invoke(...)` usage in selected view folders
- dependency-direction checks between scenes, components, and service layers
- review checklist automation

## Success Criteria

- Root guidance stays concise and points frontend runtime work to local guidance
- `apps/runtime/src/` has a local `AGENTS.md` with explicit layering and file-budget rules
- The repository documents the `300 / 500` thresholds as governance triggers
- A simple report script shows which runtime frontend files are in `WARN` or `PLAN`
- The repo maintains a first-class backlog of large frontend files instead of treating them as invisible debt
