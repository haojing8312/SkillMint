# Chat View Split Design

**Goal:** Turn `apps/runtime/src/components/ChatView.tsx` into the next formal frontend large-file split target by separating chat runtime orchestration, IM/group-run orchestration, and large chat presentation surfaces while preserving current user-visible behavior.

## Strategy Summary

- Change surface: `apps/runtime/src/components/ChatView.tsx` structure, state ownership, event-subscription placement, `invoke(...)` call placement, and major chat presentation boundaries
- Affected modules: `ChatView.tsx`, new chat-specific child components, new chat runtime hooks or scene helpers, and any new chat-local service modules
- Main risk: changing chat behavior while trying to clean up structure, especially streaming state, approvals, group-run controls, session focus behavior, and composer flows
- Recommended smallest safe path: keep `ChatView` as the page entry shell, preserve all visible behavior and existing props, then split by runtime domain rather than by arbitrary JSX chunks
- Required verification for implementation: focused `ChatView` test suites, `pnpm report:frontend-large-files`, and at least one app-shell smoke test that still mounts the chat surface
- Release impact: none if the first phase remains structure-only and keeps the existing runtime contracts, event names, and UX unchanged

## Scope

- Split `ChatView.tsx` without redesigning the chat UX in phase 1
- Keep current props, message rendering order, composer behavior, and session focus behavior stable
- Keep current Tauri command names, event names, and payload contracts unchanged
- Introduce clearer boundaries for chat runtime orchestration, IM/group-run orchestration, and large chat presentation blocks
- Establish a reusable split pattern for future large chat-adjacent runtime files

## Non-Goals

- No redesign of the chat information architecture in phase 1
- No backend command, event, or schema changes
- No state-management library migration
- No attempt to solve every chat-side tech-debt issue in one pass
- No giant `useChatViewController` that simply hides all the same complexity in one hook

## Current Problem

`apps/runtime/src/components/ChatView.tsx` is about 3984 lines and currently mixes too many concerns:

- session-scoped runtime state hydration and persistence
- message loading, run loading, and approval loading
- stream event subscriptions and runtime-state mutation
- IM delegation and group-run orchestration
- large message-rail JSX and markdown rendering
- side-panel state and derived task/web-search/file models
- composer state, draft persistence, file attachments, and workdir actions
- scroll-follow and jump-arrow behavior
- risk-confirm, install-confirm, and ask-user interaction surfaces

This makes `ChatView.tsx` the highest-risk frontend file for continued feature accretion. It is the main user path, so every extra concern added to the root file increases regression cost for the whole runtime.

## Approach Options Considered

### Option 1: Thin root shell plus domain controllers and presentation surfaces

Keep `ChatView.tsx` as the page entry shell, but move orchestration into focused hooks or scene helpers and move large UI blocks into dedicated child components.

Pros:

- lowest behavior risk
- aligns with the repo-local `scene -> component -> service` guidance
- creates durable landing zones for future chat work
- lets us split by responsibility instead of by arbitrary render sections

Cons:

- requires discipline so the root file actually becomes thinner rather than a prop-plumbing hub
- some wiring remains in the root until later passes

### Option 2: One giant `useChatController` plus a thin presentational view

Move nearly all state, effects, handlers, and async calls into one giant hook.

Pros:

- root component would shrink quickly
- many tests could keep rendering the same top-level view

Cons:

- simply relocates the giant-file problem into a giant hook
- makes orchestration harder to review because complexity becomes less visible
- weak fit for the current guardrail goal

### Option 3: Split by visible layout blocks only

Extract header, message list, side panel, and composer as child components while leaving most state and effects in the root.

Pros:

- easiest first mechanical extraction
- small review surface for JSX-only changes

Cons:

- does not solve the main problem because the root would still own the same runtime complexity
- tends to create heavy prop chains and “dumb wrappers” without real boundary improvement

## Recommended Approach

Use **Option 1: thin root shell plus domain controllers and presentation surfaces**.

The first split should target responsibility boundaries, not just line-count reduction. `ChatView.tsx` should remain the visible page entry, but it should stop directly owning most of the following:

- session runtime orchestration
- stream event plumbing
- group-run control handlers
- large message-rail helper clusters
- composer attachment and workdir behavior

The root should become the composition point for a few clear domains rather than the direct owner of all chat behavior.

## Proposed Target Structure

Use a chat module area plus scene-level orchestration:

```text
apps/runtime/src/components/chat/
  ChatShell.tsx
  ChatHeader.tsx
  ChatExecutionContextBar.tsx
  ChatMessageRail.tsx
  ChatComposer.tsx
  ChatPendingActionDialogs.tsx
  group-run/
    ChatGroupRunBoard.tsx

apps/runtime/src/scenes/chat/
  useChatSessionController.ts
  useChatStreamController.ts
  useChatGroupRunController.ts
  useChatComposerController.ts

apps/runtime/src/services/chat/
  chatSessionService.ts
  chatGroupRunService.ts
  chatApprovalService.ts
```

This is the intended direction, not a mandatory file checklist. The number of files can shift, but the responsibilities should stay stable.

## Responsibility Split

### `ChatView.tsx`

Should keep:

- top-level prop contract
- page-level shell composition
- composition of header, orchestration board, message rail, side panel, and composer
- only the cross-domain wiring that truly spans multiple chat surfaces

Should stop directly owning:

- most `invoke(...)` calls
- most stream-event subscription logic
- most group-run control handlers
- most composer attachment/workdir behavior
- large render-helper clusters that belong closer to message or orchestration surfaces

### `useChatSessionController.ts`

Should own:

- message loading
- run loading
- pending approval loading
- persisted runtime-state hydration and publication
- session-focus and selected-session lifecycle handling

Rules:

- owns session-scoped orchestration
- does not return JSX
- should call service wrappers instead of raw `invoke(...)` where practical

### `useChatStreamController.ts`

Should own:

- chat stream event subscriptions
- streaming items and reasoning state mutation
- agent-state updates
- ask-user prompt handling
- tool-call and approval bus event synchronization

Rules:

- preserve existing event names and semantics
- preserve current buffering and dedupe behavior
- stay focused on stream/runtime events only

### `useChatGroupRunController.ts`

Should own:

- group-run snapshot loading and refresh
- group-run action handlers
- reassign/retry/pause/resume/review flows
- delegation-card state and derived orchestration summaries

Rules:

- preserve current IM and group-run behavior
- stay out of generic chat rendering concerns
- can expose derived view models instead of raw snapshot-only plumbing

### `useChatComposerController.ts`

Should own:

- local draft persistence
- attachment selection and removal
- workdir selection and persistence
- send/cancel flows and composer-local error state

Rules:

- preserve current send behavior and optimistic rendering semantics
- keep attachment shaping and workdir logic out of the root
- avoid owning stream-event state

### `ChatMessageRail.tsx`

Should own:

- historical message rendering
- streaming assistant bubble rendering
- failed-run cards
- thinking/tool/render composition close to the message UI

Rules:

- may keep JSX-local helper logic that is easier to read near rendering
- should not own backend access or broad session orchestration

### `ChatGroupRunBoard.tsx`

Should own:

- the orchestration-board UI
- review, retry, pause, resume, and reassign button presentation
- step-detail rendering and focus-highlighting UI

Rules:

- prop-driven or view-model-driven
- no direct backend calls inside the component

### Chat service modules

Should own:

- `invoke(...)` wrappers
- payload shaping
- response normalization
- shared error extraction for chat-local commands

They should not own React state or JSX.

## Domain Boundaries

### Session/runtime domain

Own:

- loading messages and runs
- pending approvals
- persisted runtime snapshot handling
- session focus/highlight lifecycle

This is the first orchestration layer because it touches nearly everything else.

### Stream/events domain

Own:

- stream items
- reasoning state
- agent state
- ask-user events
- tool-call bus integration

This should be extracted early because the stream effect cluster is large and tightly coupled.

### Group-run and delegation domain

Own:

- orchestration board
- delegation cards
- group member state summaries
- group-run step focus and lifecycle controls

This is a distinct runtime domain and one of the largest chunks still mixed into the root.

### Message-rail domain

Own:

- message list rendering
- recovered assistant message display
- streaming bubble UI
- run-failure cards
- install-candidate UI

This domain should stay presentation-heavy but become independent of session and stream orchestration details.

### Composer domain

Own:

- draft persistence
- attachments
- workdir button behavior
- send/stop actions
- quick prompts

This should be split after session and stream domains so the send path remains stable during earlier refactors.

## Migration Strategy

Use a conservative staged extraction:

1. Extract shell-only surfaces that do not change behavior
2. Extract chat-local `invoke(...)` wrappers
3. Extract session/runtime orchestration
4. Extract stream-event orchestration
5. Extract group-run orchestration board and controller
6. Extract composer controller and composer UI
7. Thin the remaining root and rerun the frontend large-file report

This sequence is important. If we extract visual subcomponents first without moving orchestration, the root file will still remain the same architectural bottleneck.

## Suggested Verification Surface For Implementation

The split should be guarded by existing focused chat tests rather than by one giant suite run:

- `src/components/__tests__/ChatView.session-resilience.test.tsx`
- `src/components/__tests__/ChatView.run-guardrails.test.tsx`
- `src/components/__tests__/ChatView.im-routing-panel.test.tsx`
- `src/components/__tests__/ChatView.side-panel-redesign.test.tsx`
- `src/components/__tests__/ChatView.find-skills-install.test.tsx`
- `src/components/__tests__/ChatView.thinking-block.test.tsx`
- `src/components/__tests__/ChatView.theme.test.tsx`

Re-run:

```bash
pnpm report:frontend-large-files
```

after each major milestone so the refactor does not simply move the giant-file problem into a new hidden cluster.

## Acceptance Criteria

Phase 1 should be considered successful when:

- `ChatView.tsx` no longer directly contains most chat-local `invoke(...)` usage
- `ChatView.tsx` no longer directly owns most stream-event effect clusters
- `ChatView.tsx` no longer directly owns most group-run action orchestration
- the visible chat UX, props, event semantics, and backend contracts remain unchanged
- focused chat tests still pass
- the root file falls materially, ideally toward the `1500-2200` line range in the first honest split

## Risks To Watch

### Hidden giant-hook regression

The biggest failure mode is replacing a 3984-line component with a 1500-line `useChatController`. Each controller must stay scoped to one domain.

### Stream behavior drift

The stream domain is stateful and event-driven. Any subscription move must preserve event ordering, dedupe behavior, and session scoping.

### Group-run lifecycle drift

The IM/group-run board mixes snapshot polling, step focus, approvals, and retry controls. This area should move as a domain, not as scattered helpers.

### Prop-drilling explosion

If extraction creates giant props objects that just mirror the old root state one-to-one, the split is not yet successful. Controllers should expose narrower view models and action bundles where useful.

## Recommended Next Step

Write an implementation plan that starts with service and controller extraction rather than a pure JSX split, and sequence the work so the most critical orchestration domains gain boundaries before the root file is asked to shrink.
