# Settings View Feishu Phase 2 Design

**Goal:** Continue reducing `apps/runtime/src/components/SettingsView.tsx` by shrinking its direct ownership of Feishu-specific business logic, while preserving all current user-visible behavior, polling semantics, onboarding flow, and Tauri command contracts.

## Strategy Summary

- Change surface: Feishu-specific selectors, service wrappers, and controller/state orchestration currently still held in `SettingsView.tsx`
- Affected modules: `SettingsView.tsx`, `apps/runtime/src/components/settings/feishu/*`, related focused `SettingsView` Feishu tests
- Main risk: moving Feishu polling, pairing, or onboarding state too aggressively and accidentally changing lifecycle or async behavior
- Recommended smallest safe path: extract pure selectors first, then `invoke(...)` service wrappers, then introduce a thin Feishu controller hook that preserves existing polling and handler semantics
- Required verification for implementation: focused Feishu settings tests, targeted routing and MCP/settings baseline tests where shared setup changes, and a rerun of `pnpm report:frontend-large-files`
- Release impact: none if phase 2 remains structure-only and preserves existing command contracts and UI behavior

## Why Phase 2 Exists

The first split converted `SettingsView.tsx` from a giant mixed page into a shell plus domain sections. That was the right first milestone, but the file still remains large because it continues to directly own most Feishu-specific business logic:

- onboarding step derivation
- connector status summaries
- diagnostics summaries
- installer-session display state
- pairing refresh and action handlers
- Feishu polling and reload orchestration
- direct `invoke(...)` command knowledge

After phase 1, the largest remaining source of complexity is no longer broad JSX sprawl. It is the root file's direct awareness of too much Feishu domain behavior.

## Phase 2 Objective

Phase 2 should not be treated as "keep slicing JSX until the file is small."

Instead, it should make `SettingsView.tsx` less Feishu-aware by moving three specific kinds of logic out of the root:

1. Pure derivation logic
2. Backend command wrappers
3. Local Feishu state orchestration

The design goal is to reduce root-file cognitive load, not merely line count.

## Non-Goals

- No redesign of the Feishu onboarding UX
- No change to polling cadence or refresh triggers
- No change to Tauri command names or payload shapes
- No replacement of existing section components
- No conversion of all settings state into one giant global hook
- No attempt to fully solve every remaining `SettingsView.tsx` concern in this phase

## Recommended Layering

### 1. Selectors Layer

Create:

```text
apps/runtime/src/components/settings/feishu/feishuSelectors.ts
```

This layer should contain pure functions only.

It is the correct home for logic such as:

- onboarding header step derivation
- onboarding display labels and status summaries
- connector status summarization
- diagnostics summary formatting
- installer display mode and hint derivation
- routing/status card labels derived from current state

Rules:

- no React state
- no `invoke(...)`
- no side effects
- input/output only

This is the lowest-risk extraction and should happen first.

### 2. Service Layer

Create or expand:

```text
apps/runtime/src/components/settings/feishu/feishuSettingsService.ts
```

This layer should centralize Feishu-specific backend calls and payload shaping.

Examples:

- load setup progress
- load pairing requests
- load runtime status
- load installer session
- start installer session
- send installer input
- stop installer session
- approve/deny pairing
- load/save advanced settings

Rules:

- wraps `invoke(...)`
- preserves existing command names and payload structures
- no React state ownership
- no JSX

The service layer reduces the root component's direct knowledge of backend command details.

### 3. Controller Layer

Create:

```text
apps/runtime/src/components/settings/feishu/useFeishuSettingsController.ts
```

This should be a thin orchestration hook, not a second giant file.

Its purpose is to own Feishu-local state and handlers that currently clutter the root file, while preserving existing lifecycle behavior.

Candidates:

- local loading and retry state
- refresh handlers
- pairing action handlers
- installer session input state
- advanced-settings save state
- polling setup and cleanup

Rules:

- preserve current polling semantics
- preserve current tab-open behavior
- do not widen scope into unrelated settings domains
- keep the hook focused on Feishu orchestration only

## Proposed Responsibility Split After Phase 2

### `SettingsView.tsx`

Should keep:

- overall modal shell
- tab selection
- cross-domain wiring
- import-and-compose responsibility
- only genuinely shared settings state

Should stop directly owning:

- most Feishu command names
- most Feishu summary derivations
- most Feishu handler orchestration

### `feishuSelectors.ts`

Should own:

- pure derived state
- label generation
- summary text
- step/status mapping

### `feishuSettingsService.ts`

Should own:

- Feishu `invoke(...)` wrappers
- request normalization
- response normalization
- null/legacy compatibility cleanup where needed

### `useFeishuSettingsController.ts`

Should own:

- Feishu-local orchestration
- async action handlers
- refresh and polling lifecycle logic
- local input and save states

### `FeishuSettingsSection.tsx` and `FeishuAdvancedSection.tsx`

Should remain mostly presentational.

They can stay prop-driven for now, as long as:

- they do not gain new backend knowledge
- they do not absorb polling/state-machine ownership

## Sequence Recommendation

Use this order:

1. Extract selectors
2. Extract service wrappers
3. Introduce a thin controller hook
4. Rewire `SettingsView.tsx` to consume controller outputs
5. Stop and evaluate file size plus readability before any further component slicing

This order matters.

If a controller hook is introduced before selectors and services are cleaned up, the project risks simply moving the same mixed complexity into a giant hook.

## Suggested Acceptance Criteria

Phase 2 should be considered successful when:

- `SettingsView.tsx` no longer directly contains most Feishu summary/label derivation logic
- `SettingsView.tsx` no longer directly contains most Feishu `invoke(...)` calls
- Feishu polling and onboarding behavior remain unchanged
- existing Feishu-focused tests still pass without broad rewrites
- root file drops again, ideally into roughly the `1600-1900` line range

## Risks To Watch

### Polling Drift

The highest risk is accidentally changing when Feishu polling starts, stops, or refreshes.

Mitigation:

- preserve existing dependency arrays and tab-gated refresh behavior
- move code in small steps
- keep tests focused on "while tab stays open" semantics

### Hidden Controller Bloat

If `useFeishuSettingsController.ts` becomes a 1000-line hook, phase 2 will fail architecturally even if tests pass.

Mitigation:

- extract selectors first
- keep service logic out of the hook
- keep JSX out of the hook

### Mixed Ownership

If some Feishu behavior remains split across root, service, controller, and section in inconsistent ways, debugging will get harder instead of easier.

Mitigation:

- define ownership explicitly before implementation
- keep a clear line between selectors, service, controller, and sections

## Recommended End State For This Phase

The intended outcome is:

- `SettingsView.tsx` becomes a thinner shell and composition root
- Feishu selectors become pure and testable
- Feishu backend access becomes explicit and centralized
- Feishu orchestration becomes local to a focused controller
- current UX and backend behavior remain stable

That is the right next step before considering any future information-architecture redesign or deeper Feishu component decomposition.
