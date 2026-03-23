# Frontend Large File Backlog

**Goal:** Turn the current `300 / 500` frontend runtime guardrails into an actionable backlog for `apps/runtime/src/`.

**Source:** Generated from `pnpm report:frontend-large-files` on 2026-03-23 using thresholds `warn=300` and `plan=500`.

## Prioritization Rules

- Prioritize giant page and app-shell files first because they are the highest-risk place for AI-assisted feature accretion.
- Prioritize files that mix view rendering, Tauri access, and orchestration ahead of files that are merely verbose.
- Prioritize user-facing runtime flows second because they affect broad behavior and become expensive to verify after every change.
- Prioritize shared hooks, view models, and helper surfaces third unless they are actively blocking a split of a larger page file.
- Large child files created during recent refactors can stay in backlog, but they should not restart the pattern of becoming the next giant file.

## Priority 1: Giant View And Shell Surfaces

These files should be the first ongoing refactor targets. New feature work in them should start with a split plan.

### P1-A: Core runtime views

- `apps/runtime/src/components/SettingsView.tsx` — 4998 lines
  - Why first: largest frontend runtime file in the repo and an obvious magnet for continued settings, onboarding, connector, and diagnostics growth
  - Primary concerns mixed today: large JSX sections, page-level orchestration, and likely clusters of backend or Tauri access logic
  - First split direction: keep the view entry thin, move tab-level or feature-level orchestration into scene or coordinator modules, and isolate backend access behind service or API wrappers
  - First safe step: inventory major sections and split candidates by settings domain such as models, health, search, routing, and connector setup

- `apps/runtime/src/components/ChatView.tsx` — 3984 lines
  - Why first: central user-facing surface that historically attracts rendering, event handling, tool output, attachments, and session-flow logic
  - Primary concerns mixed today: large message rendering, session interaction logic, side-panel flow logic, and runtime event integration
  - First split direction: keep final chat composition in the view, move orchestration into scene or coordinator hooks, and isolate major sub-surfaces such as tool cards, side panels, or composer behavior into focused modules
  - First safe step: separate data and event orchestration from large presentational message and panel sections

### P1-B: Shell and high-level scene hosts

- `apps/runtime/src/components/employees/EmployeeHubView.tsx` — 1648 lines
  - Why first: employee center is already acknowledged as a mixed-concern area and likely still carries orchestration that belongs in the scene layer
  - Primary concerns mixed today: broad view composition, employee workflow interaction, and domain-specific conditional branches
  - First split direction: continue the scene-oriented split so the view becomes primarily presentational while employee workflows, mutations, and coordination live in scene or service helpers
  - First safe step: audit what still belongs in `EmployeeHubScene` or employee-specific coordinators instead of the view

- `apps/runtime/src/App.tsx` — 702 lines
  - Why first: app shell files silently regrow when every cross-cutting feature lands there
  - Primary concerns mixed today: shell wiring, startup state, selected cross-scene coordination, and historical domain leakage
  - First split direction: keep only true shell concerns in `App.tsx`, continue pushing domain orchestration into `scenes/*`, and reduce shell-only props to what must actually cross scene boundaries
  - First safe step: identify any remaining domain-specific state or mutation handlers that still belong in focused coordinators or scene containers

## Priority 2: Large Shared Runtime Modules

These files are not the first shell or page targets, but they are large enough to deserve planned thinning.

- `apps/runtime/src/types.ts` — 954 lines
  - Why here: shared type hubs often become dumping grounds and create wide coupling across the runtime
  - First split direction: break out stable domain-specific type groups into `types/<domain>.ts` or co-located domain modules
  - First safe step: group types by session, employee, packaging, models, and chat concerns before moving definitions

- `apps/runtime/src/components/ModelSetupOverlays.tsx` — 632 lines
  - Why here: overlay flows tend to accumulate conditional branches and duplicated state shaping
  - First split direction: separate overlay-specific panels and extract any reusable setup state or validation logic
  - First safe step: identify repeated or clearly separable overlay blocks

- `apps/runtime/src/scenes/useQuickSetupCoordinator.ts` — 559 lines
  - Why here: coordinator hooks can quietly become giant hidden scene containers
  - First split direction: split by workflow stage, backend access helper, or derived state boundary rather than by arbitrary helper functions
  - First safe step: isolate backend access and pure derivation logic from orchestration state

- `apps/runtime/src/components/experts/SkillLibraryView.tsx` — 538 lines
  - Why here: likely mixes search, filtering, loading state, and large rendering blocks
  - First split direction: separate list or card presentation from library-specific orchestration and filtering behavior
  - First safe step: move filtering and data-preparation logic away from large JSX sections

- `apps/runtime/src/components/NewSessionLanding.tsx` — 519 lines
  - Why here: landing flows tend to absorb onboarding, quick action, and recommendation logic quickly
  - First split direction: split reusable landing sections from recommendation or launch logic
  - First safe step: identify self-contained sections such as gallery, prompt suggestions, or entry panels

## Priority 3: Warn Queue

These files are above 300 lines and should be watched, but they are not first in line for dedicated split work.

- `apps/runtime/src/scenes/useImBridgeIntegration.ts` — 425 lines
- `apps/runtime/src/components/InstallDialog.tsx` — 415 lines
- `apps/runtime/src/scenes/buildAppShellRenderProps.ts` — 415 lines
- `apps/runtime/src/components/employees/FeishuRoutingWizard.tsx` — 396 lines
- `apps/runtime/src/model-provider-catalog.ts` — 393 lines
- `apps/runtime/src/components/AppMainContent.tsx` — 392 lines
- `apps/runtime/src/scenes/employees/useEmployeeSessionLaunchCoordinator.ts` — 388 lines
- `apps/runtime/src/components/experts/ExpertCreateView.tsx` — 380 lines
- `apps/runtime/src/components/chat-side-panel/view-model.ts` — 375 lines
- `apps/runtime/src/scenes/useExpertSkillCoordinator.ts` — 374 lines
- `apps/runtime/src/scenes/useRuntimeSessionCoordinator.ts` — 367 lines
- `apps/runtime/src/components/packaging/IndustryPackView.tsx` — 348 lines
- `apps/runtime/src/components/chat-side-panel/WorkspaceFilesPanel.tsx` — 347 lines
- `apps/runtime/src/components/Sidebar.tsx` — 343 lines
- `apps/runtime/src/scenes/useSessionViewCoordinator.ts` — 311 lines

## Recommended Execution Order

1. Tackle `SettingsView.tsx` first because it is by far the largest frontend runtime file and likely the easiest place for new feature growth to keep landing.
2. Tackle `ChatView.tsx` next because it is the core user-facing surface and has the highest long-term product blast radius.
3. Continue the employee-center split by reducing `EmployeeHubView.tsx` and keeping scene ownership crisp.
4. Keep `App.tsx` shell-focused so future domain work does not re-accumulate there.
5. Then move to the shared `types.ts` and large coordinator or overlay files that still hide broad responsibility.
6. Re-run `pnpm report:frontend-large-files` after each split milestone and update this backlog rather than treating it as static.

## Definition Of Backlog Progress

- A file leaves the `PLAN` queue only when it falls below 500 lines or when the remaining content is clearly limited to one responsibility boundary.
- A file leaves the `WARN` queue only when it falls below 300 lines or when there is a strong reason to accept its current shape.
- A file is not considered improved if code was merely moved into an equally giant child component, hook, or helper module.
- A split is successful only when scene, component, hook, service, and view boundaries are clearer after the change than before it.
- A split is not successful if Tauri access, page orchestration, and large JSX are still mixed in one place, even if the line count went down.
