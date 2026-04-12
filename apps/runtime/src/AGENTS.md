# Frontend Runtime AGENTS.md

## Scope
- This file applies to work in `apps/runtime/src/`.
- Use it as the local frontend runtime guidance layer on top of the root `AGENTS.md`.

## Primary Goal
- Keep React and TypeScript runtime changes maintainable during AI-native development.
- Prefer clear scene, component, hook, and service boundaries over continuing to grow giant view files.
- Avoid replacing giant files with many trivial micro-files.

## Default Landing Zones
- `App.tsx`: app shell, top-level dependency wiring, and high-level main-view switching
- `scenes/<domain>/`: page-level or domain-level orchestration, state ownership, and cross-component coordination
- `components/<domain>/`: presentation components, reusable UI blocks, and local interaction logic
- `hooks/`: reusable stateful logic, subscriptions, async loading, and view-model hooks that return data and actions but not JSX
- `api/` or `services/`: Tauri `invoke(...)` wrappers, request shaping, response normalization, and backend-specific error handling
- `lib/` or `utils/`: pure helpers, formatting, projections, and non-React utilities
- `types.ts` or `types/<domain>.ts`: shared frontend-facing types

When a task does not naturally fit these landing zones, explain the chosen placement before editing code.

## Current Reference Direction
- Treat the current `App.tsx -> scenes/* -> components/*` direction as the default frontend split pattern.
- Use the employee-hub scene extraction design in `docs/plans/2026-03-21-app-employee-hub-scene-design.md` as the first repository example to copy before inventing a new split pattern.
- Prefer introducing or extending a focused scene boundary before letting `App.tsx`, `SettingsView.tsx`, or `ChatView.tsx` absorb more orchestration.

### Current ChatView Reference Split
- Treat `ChatView.tsx` as a composition root, not a landing zone for every new concern.
- Prefer adding new chat UI sections under `components/chat/` when the code primarily owns JSX and local presentation interaction.
- Prefer adding reusable chat state orchestration under `components/chat/use*.ts` when the code owns subscriptions, scroll/focus behavior, or derived view models without JSX.
- Prefer adding pure chat projection helpers under `components/chat/*Helpers.ts` when the code formats, groups, labels, or derives display data from existing state.
- Current reference examples:
  - `components/chat/useChatViewportController.ts`
  - `components/chat/useChatDerivedViewModels.ts`
  - `components/chat/chatGroupRunHelpers.ts`
  - `components/chat/chatViewHelpers.tsx`

Use this split before creating another giant child component or pushing more orchestration back into `ChatView.tsx`.

### Current Settings Reference Split
- Treat large settings controllers such as `components/settings/feishu/useFeishuSettingsController.ts` as orchestration roots, not as landing zones for every async action, default object, and derived section prop.
- Prefer extracting async connector actions into `components/settings/<domain>/*ControllerActions.ts` when the file is accumulating load/save/install/retry/start handlers.
- Prefer extracting default settings objects into `components/settings/<domain>/*ControllerDefaults.ts` when large literal state shapes start to dominate the controller.
- Prefer extracting pure section prop assembly into `components/settings/<domain>/*ControllerViewModel.ts` when one controller is mostly deriving labels, flags, and grouped props for child sections.
- Prefer keeping large settings section files as composition roots and moving stable UI blocks into focused files such as environment panels, authorization panels, connection detail panels, and advanced-settings forms.

Use this split before adding more view-model derivation or backend action code directly into a large settings controller or one giant section component.

## Responsibility Split
- `App.tsx` owns app-shell concerns and only the cross-scene dependencies that truly belong at the shell.
- `scene` modules own page-level or domain-level state, orchestration, and cross-component workflow coordination.
- `component` modules own presentation and local interaction that is easiest to understand close to JSX.
- `hooks` own reusable state logic, subscriptions, and derived view models; they should return state and actions, not JSX.
- `api` or `services` own Tauri `invoke(...)` wrappers, backend access details, and error mapping; they should not own React state.
- `lib` or `utils` own pure helper logic only.

## File Budget Policy
- `<= 300` lines: target zone for runtime frontend files
- `301-500` lines: warning zone; avoid adding net-new page state, Tauri I/O, or major render branches until module placement is reconsidered
- `501+` lines: split-design zone; write or update a short split plan before adding feature work

These thresholds are governance triggers, not blanket failure rules. Do not split files mechanically just to get under a number.

## Responsibility Trigger Rule
- If a frontend file mixes two or more of these concerns, consider split planning even before it reaches the warning threshold:
  - page-level or scene-level orchestration
  - backend or Tauri data access
  - large JSX presentation blocks
  - reusable subscription or state-machine logic

## What Should Move Out Of Root Components
- large clusters of `invoke(...)` calls and backend error mapping
- reusable form state and validation logic
- event listener setup and teardown flows
- pure projection, filtering, grouping, and formatting logic
- distinct screen sections that can stand alone as a reusable UI block
- scroll-follow, focus, highlight, virtualization, and viewport synchronization logic
- side-panel projection and failed-run aggregation logic
- group-run or delegation display-state derivation

## What Can Stay Close To JSX
- final page or scene composition
- small local interaction handlers that are meaningful only in one view
- small derived render values that are clearer in place
- compatibility glue that is easier to read at the view entrypoint

## Avoid Micro-File Sprawl
- Create a new file only when it owns a real orchestration concern, reusable UI block, reusable state behavior, backend access concern, or meaningful complexity removal.
- Do not create one-file-per-helper or one-file-per-render-function directories.
- Do not move a giant root component into an equally giant child component or hook and call that a successful split.
- Prefer one hook or helper per meaningful concern cluster. For example, "viewport control" is a good file boundary; "scrollToBottom" alone is not.

## Stability Rules
- Preserve existing user-visible behavior unless the task explicitly calls for a behavior change.
- Keep view-level contracts and prop flows stable unless the task intentionally changes them.

## Working Style For AI Agents
- Name the intended target layer before writing new frontend runtime logic.
- If touching a file above 300 lines, explain why the change belongs there instead of a new scene, hook, service, or component.
- If touching a file above 500 lines for feature work, create or update a split plan in `docs/plans/` first.
- Prefer scene or service extraction over adding more orchestration directly to large view files.
- When doing repo hygiene or large-file cleanup on frontend runtime surfaces, run `pnpm report:frontend-large-files` to confirm the next highest-risk files instead of relying only on manual inspection.
