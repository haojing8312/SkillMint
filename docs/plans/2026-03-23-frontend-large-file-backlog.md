# Frontend Large File Backlog

**Goal:** Turn the current `300 / 500` frontend runtime guardrails into an actionable backlog for `apps/runtime/src/`.

**Source:** Regenerated from `pnpm report:frontend-large-files` on 2026-03-24 using thresholds `warn=300` and `plan=500`.

## Prioritization Rules

- Prioritize giant page and app-shell files first because they are the highest-risk place for AI-assisted feature accretion.
- Prioritize files that mix view rendering, Tauri access, and orchestration ahead of files that are merely verbose.
- Prioritize user-facing runtime flows second because they affect broad behavior and become expensive to verify after every change.
- Prioritize shared hooks, view models, and helper surfaces third unless they are actively blocking a split of a larger page file.
- Large child files created during recent refactors can stay in backlog, but they should not restart the pattern of becoming the next giant file.

## Priority 1: Giant View And Shell Surfaces

These files should be the first ongoing refactor targets. New feature work in them should start with a split plan.

### P1-A: Core runtime views

- `apps/runtime/src/components/ChatView.tsx` — 2335 lines
  - Why still first: it remains the largest frontend runtime file in the repo even after the latest phase 2 root-thinning work
  - Current status: phase 1 plus the current phase 2 pass already extracted shell primitives, service wrappers, session/controller surfaces, rail presentation helpers, and group board presentation
  - Primary concerns mixed today: remaining root-level shell composition, install/risk dialog ownership, install-candidate glue, and broad cross-domain chat wiring
  - Next split direction: treat the current state as a stabilized shell rather than another urgent monolith, and only continue if root regrowth or new child giant files start appearing
  - Next safe step: watch `ChatMessageRail.tsx`, `ChatGroupRunBoard.tsx`, and the chat controllers so the latest split does not simply recreate the problem in new modules

- `apps/runtime/src/components/SettingsView.tsx` — 795 lines
  - Why still here: phase 1 and Feishu phase 2 already reduced this file substantially, but it still remains above the `PLAN` threshold and keeps root-level settings shell responsibilities
  - Current status: no longer the top giant-file risk; it has moved from crisis refactor target to follow-up thinning and stabilization work
  - Primary concerns mixed today: tab shell composition, residual cross-domain settings wiring, and ownership handoff into large child controllers or selectors
  - Next split direction: keep root ownership narrow, and only continue splitting if the remaining shell logic starts regrowing or blocks another settings-domain refactor
  - Next safe step: review whether any remaining root-only Feishu or cross-tab wiring can move into thinner domain entry helpers without rebuilding a giant coordinator

### P1-B: Shell and high-level scene hosts

- `apps/runtime/src/components/employees/EmployeeHubView.tsx` — 1648 lines
  - Why now first: `EmployeeHubScene` already exists, so this is the clearest remaining employee-center giant view and the next best candidate for the proven `thin root shell + domain sections` pattern
  - Primary concerns mixed today: local tab state, employee/team/run/settings workflows, direct Tauri access, memory/profile tools, and large tab-specific render branches
  - First split direction: keep `EmployeeHubScene` as the employee workflow container, then turn `EmployeeHubView` into a thinner tab shell with domain sections and employee-local service helpers
  - First safe step: identify what still belongs in `EmployeeHubScene`, what should move to employee view helpers/services, and what can become tab-level sections without changing UX

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

- `apps/runtime/src/components/settings/feishu/useFeishuSettingsController.ts` — 933 lines
  - Why here: the recent `SettingsView` phase 2 split intentionally moved Feishu orchestration into a controller, but that controller is now itself in the `PLAN` zone
  - First split direction: keep it as a controller boundary, then peel off polling, derived sub-state, or mutation clusters into smaller helpers without losing lifecycle clarity
  - First safe step: separate controller-only orchestration from pure state derivation and verify it does not become a hidden second `SettingsView`

- `apps/runtime/src/components/settings/feishu/feishuSelectors.ts` — 821 lines
  - Why here: selector extraction was the right move, but the derived-state surface is now large enough to deserve its own follow-up guardrail
  - First split direction: group selectors by onboarding, installer, routing, and diagnostics concerns instead of keeping every derivation in one file
  - First safe step: split purely presentational summaries from multi-input workflow selectors so the file remains a selector hub rather than a second controller

- `apps/runtime/src/components/settings/desktop/DesktopSettingsSection.tsx` — 658 lines
  - Why here: desktop settings were successfully extracted from `SettingsView`, but the resulting section is still large enough to accumulate more domain branches
  - First split direction: separate clearly independent desktop subsections or any embedded service-like logic from the main section view
  - First safe step: identify whether startup behavior, diagnostics, and system integration concerns can be rendered by focused child sections without changing UX

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

- `apps/runtime/src/components/settings/feishu/FeishuSettingsSection.tsx` — 531 lines
  - Why here: the main Feishu section is no longer in the root file, but it still sits just above the `PLAN` threshold and will regrow quickly if left unchecked
  - First split direction: keep onboarding and guided-panel UX together only where it helps comprehension, and split clearly independent diagnostics or routing detail blocks when needed
  - First safe step: watch whether future changes land in guided installer UI, routing summary, or connection overview and split only those hot spots

- `apps/runtime/src/components/NewSessionLanding.tsx` — 519 lines
  - Why here: landing flows tend to absorb onboarding, quick action, and recommendation logic quickly
  - First split direction: split reusable landing sections from recommendation or launch logic
  - First safe step: identify self-contained sections such as gallery, prompt suggestions, or entry panels

## Priority 3: Warn Queue

These files are above 300 lines and should be watched, but they are not first in line for dedicated split work.

- `apps/runtime/src/components/chat/group-run/ChatGroupRunBoard.tsx` — 413 lines
- `apps/runtime/src/scenes/chat/useChatStreamController.ts` — 435 lines
- `apps/runtime/src/scenes/chat/useChatCollaborationController.ts` — 361 lines
- `apps/runtime/src/components/chat/ChatMessageRail.tsx` — 339 lines
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

1. Start the next employee-center split by reducing `EmployeeHubView.tsx` and keeping `EmployeeHubScene` ownership crisp.
2. Keep `ChatView.tsx` in watch mode and prevent the latest rail and group-board extractions from regrowing into new giant child files.
3. Keep `types.ts`, `useFeishuSettingsController.ts`, and `feishuSelectors.ts` from becoming the next hidden giant-module cluster.
4. Keep `SettingsView.tsx` shell-focused and treat it as follow-up stabilization work rather than the current top emergency.
5. Keep `App.tsx` shell-focused so future domain work does not re-accumulate there.
6. Re-run `pnpm report:frontend-large-files` after each split milestone and update this backlog rather than treating it as static.

## Definition Of Backlog Progress

- A file leaves the `PLAN` queue only when it falls below 500 lines or when the remaining content is clearly limited to one responsibility boundary.
- A file leaves the `WARN` queue only when it falls below 300 lines or when there is a strong reason to accept its current shape.
- A file is not considered improved if code was merely moved into an equally giant child component, hook, or helper module.
- A split is successful only when scene, component, hook, service, and view boundaries are clearer after the change than before it.
- A split is not successful if Tauri access, page orchestration, and large JSX are still mixed in one place, even if the line count went down.
