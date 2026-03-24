# Settings View Split Design

**Goal:** Turn `apps/runtime/src/components/SettingsView.tsx` into the first formal frontend large-file splitting target by extracting stable tab and domain boundaries while preserving current user-visible behavior.

## Strategy Summary

- Change surface: `apps/runtime/src/components/SettingsView.tsx` structure, state ownership, `invoke(...)` call placement, and tab-level module boundaries
- Affected modules: `SettingsView.tsx`, settings-related child components, new settings domain modules under `apps/runtime/src/components/settings/`, and any new domain-local service or hook modules
- Main risk: changing settings behavior while attempting structural cleanup, especially in Feishu onboarding, connector management, model setup, and desktop/runtime preferences
- Recommended smallest safe path: preserve all current tabs, props, command names, and interaction order while splitting by tab domain and isolating backend access behind domain-local service modules
- Required verification for implementation: focused `SettingsView` tests, a report rerun via `pnpm report:frontend-large-files`, and at least one app-shell smoke test that still mounts the settings entry
- Release impact: no direct release-sensitive impact if the first phase is structure-only and does not alter visible behavior or backend contracts

## Scope

- Split `SettingsView.tsx` without redesigning the settings UX in phase 1
- Keep current tab names, initial-tab behavior, and parent usage stable
- Keep Tauri command names and payload contracts unchanged
- Introduce clearer domain boundaries for settings-specific logic and presentation
- Create a reusable split pattern that can later be applied to other giant frontend files

## Non-Goals

- No information-architecture redesign in this phase
- No backend command or schema changes
- No global state library adoption
- No conversion of the entire settings surface into a single giant coordinator hook
- No attempt to solve all settings domain debt in one pass

## Current Problem

`apps/runtime/src/components/SettingsView.tsx` is now roughly 4998 lines and mixes too many responsibilities:

- modal shell and tab navigation
- model configuration form state and testing
- search provider configuration and presets
- runtime preferences and desktop lifecycle actions
- MCP server management
- routing and capability policy state
- provider health and route logs
- Feishu connector setup, onboarding, installer session handling, advanced settings, and pairing approvals

The file currently holds both large JSX blocks and a very large amount of stateful orchestration. That makes it too easy for every new settings feature to land in the same file.

## Approach Options Considered

### Option 1: Tab-domain split with a thin root shell

Keep `SettingsView.tsx` as the modal and tab-entry shell, but split each major settings domain into dedicated child modules. Move `invoke(...)` clusters and domain-specific orchestration into domain-local service and helper modules.

Pros:

- lowest behavior risk
- easiest to review incrementally
- aligns with the frontend guardrails already established for `App.tsx -> scenes/* -> components/*`
- creates clear future landing zones for new settings work

Cons:

- requires discipline so the root shell does not remain too heavy
- may still leave some cross-tab coordination in the root until later cleanup

### Option 2: Giant settings coordinator plus thin presentational tabs

Move nearly all state and behavior into `useSettingsCoordinator` or `SettingsScene`, then make tab components mostly presentational.

Pros:

- root component would shrink quickly
- could centralize cross-tab behavior in one place

Cons:

- high risk of simply moving giant-file complexity into a giant hook
- makes review harder because orchestration becomes hidden rather than clarified
- weaker fit for the immediate goal of creating durable domain boundaries

### Option 3: Capability-flow split by use case rather than tabs

Split by user workflows such as model setup flow, Feishu onboarding flow, desktop diagnostics flow, and connector management flow, regardless of current tab boundaries.

Pros:

- potentially best long-term architecture
- can align code with user journeys instead of existing visual buckets

Cons:

- highest risk for a first phase
- likely to reshape information architecture while trying to refactor
- too broad for a zero-UX-change first split

## Recommended Approach

Use **Option 1: tab-domain split with a thin root shell**.

This is the smallest safe path because it keeps the current settings UX intact while establishing durable module boundaries. The root `SettingsView.tsx` should stay as the modal entry and tab switcher, but each settings domain should become a dedicated home for its JSX, local state, and backend access helpers.

The design borrows the same principle already used in the employee hub split:

- root shell stays visible and stable
- domain orchestration moves downward
- child modules own clearly bounded concerns

## Proposed Target Structure

Create a settings module area under `apps/runtime/src/components/settings/`:

```text
apps/runtime/src/components/settings/
  SettingsShell.tsx
  SettingsTabNav.tsx
  shared/
    settingsTypes.ts
    settingsError.ts
  models/
    ModelsSettingsSection.tsx
    modelSettingsService.ts
  desktop/
    DesktopSettingsSection.tsx
    desktopSettingsService.ts
  search/
    SearchSettingsSection.tsx
    searchSettingsService.ts
  mcp/
    McpSettingsSection.tsx
    mcpSettingsService.ts
  routing/
    RoutingSettingsSection.tsx
    routingSettingsService.ts
  feishu/
    FeishuSettingsSection.tsx
    FeishuOnboardingSection.tsx
    feishuSettingsService.ts
```

This should be treated as the reference direction, not an inflexible checklist. The exact number of files can change, but the responsibilities should remain consistent.

## Responsibility Split

### `SettingsView.tsx`

- owns modal entry props and close behavior
- owns active-tab selection and initial-tab synchronization
- renders the shell and delegates tab content to domain sections
- keeps only cross-tab state that genuinely must stay shared

### `SettingsShell.tsx`

- owns consistent settings-page frame structure
- owns high-level title, close affordance, and shared surface layout
- should not absorb domain logic

### `SettingsTabNav.tsx`

- owns the tab button list and tab-selection UI only
- should stay dumb and data-driven

### Domain section components

- `ModelsSettingsSection.tsx`
- `DesktopSettingsSection.tsx`
- `SearchSettingsSection.tsx`
- `McpSettingsSection.tsx`
- `RoutingSettingsSection.tsx`
- `FeishuSettingsSection.tsx`

Each section owns:

- the JSX and interaction flow for that settings domain
- domain-local state that does not need to be shared across all tabs
- calls to domain-local service helpers

### Domain service modules

- wrap `invoke(...)` calls for the domain
- normalize request and response shaping
- centralize error mapping where useful
- must not hold React state

### Optional domain hooks

Only introduce a hook when it represents reusable stateful behavior that would otherwise make the section hard to read.

Do not create a `useSettingsCoordinator` that simply hides the same mixed responsibilities in one giant hook.

## Domain Boundaries

### Models domain

Own:

- model list loading
- model form state
- test connection behavior
- save, edit, delete model actions
- provider preset and suggestion handling

Move out early because it is relatively self-contained and lower-risk than Feishu.

### Desktop domain

Own:

- runtime preferences
- desktop preference save state
- lifecycle path display
- diagnostics and cleanup actions
- permission-mode confirmation flow if it remains local to desktop settings

This domain is also a good early target because it is isolated and mostly procedural.

### Search domain

Own:

- search provider config list and form state
- presets and validation
- search test action

This should mirror the models domain split pattern closely.

### MCP domain

Own:

- MCP server list
- MCP add/remove form state
- MCP env parsing and validation

### Routing domain

Own:

- route settings
- capability policy state
- route logs and route stats
- route-template selection and save state

This is more orchestration-heavy and should come after models and desktop.

### Feishu domain

Own:

- connector credentials and advanced settings
- environment probe and runtime status
- installer session state
- pairing approval queue
- onboarding state and derived step logic

This is the highest-risk domain and should be split last in phase 1.

## Migration Strategy

Use a conservative staged extraction:

1. Extract shell-only structure from `SettingsView.tsx`
2. Extract the tab-navigation surface into a dumb child component
3. Extract the models domain and its service helper
4. Extract the desktop domain and its service helper
5. Extract search and MCP domains
6. Extract routing domain
7. Extract Feishu domain last, including onboarding and installer-related helpers
8. Trim root state and helper leftovers until `SettingsView.tsx` becomes a genuine shell

This sequence intentionally starts with lower-risk domains so the split pattern can stabilize before touching the most behavior-dense part of the page.

## What Must Stay Stable In Phase 1

- `SettingsView` public props
- current initial-tab behavior
- current tab names and presence
- current Tauri command names and payload shapes
- current user-visible action ordering
- current success/error behavior semantics
- current `App.tsx` integration path

## Risks

- Feishu onboarding and connector management currently appear to have the densest state interactions and test surface
- some helpers may look generic but are actually domain-specific and should stay close to the owning domain
- route and health surfaces may share state in ways that tempt over-centralization
- `types.ts` currently contains many shared frontend types, so the split should avoid a premature type-shuffle unless a domain boundary clearly benefits

## Testing Strategy For Implementation

Phase-1 implementation should rely on focused settings tests first, not broad redesign validation.

Expected verification shape:

- targeted `SettingsView` tests for the domains touched in each extraction stage
- one app-level smoke test that still mounts the settings entry
- `pnpm report:frontend-large-files` after each milestone to confirm the root file is shrinking and no giant child replacement is emerging

Candidate existing tests include:

- `apps/runtime/src/components/__tests__/SettingsView.model-connection.test.tsx`
- `apps/runtime/src/components/__tests__/SettingsView.desktop-system-tab.test.tsx`
- `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- `apps/runtime/src/components/__tests__/SettingsView.feishu.test.tsx`

## Success Criteria

- `SettingsView.tsx` becomes materially smaller and more shell-focused
- new settings work has a clear default landing zone by domain
- backend access is more isolated from large JSX blocks
- no domain split simply recreates another giant root-sized child module
- user-visible settings behavior stays unchanged in phase 1
- the split pattern is clear enough to reuse for other giant frontend runtime files
