# Chat View Phase 2 Design

**Goal:** Finish the next safe thinning pass on `apps/runtime/src/components/ChatView.tsx` after phase 1 by moving remaining root-only render helpers and orchestration-heavy presentation into focused chat modules while preserving the current chat UX and runtime behavior.

## Strategy Summary

- Change surface: `apps/runtime/src/components/ChatView.tsx` root composition, root-only render helper placement, orchestration-board presentation placement, and chat-local presentation helper boundaries
- Affected modules: `ChatView.tsx`, existing `components/chat/*` surfaces, possible new chat presentation helper modules, and the already-extracted chat controllers if they need narrower helper boundaries
- Main risk: letting a “phase 2 cleanup” drift into UX redesign or broad orchestration rewrites, especially around streaming message rendering and group-run coordination
- Recommended smallest safe path: keep current chat layout, props, event names, and controller contracts stable, then move remaining heavy presentation/helper surfaces out of the root one domain at a time
- Required verification for implementation: focused `ChatView` suites for stream output, IM/group-run behavior, and session resilience, plus `pnpm report:frontend-large-files`
- Release impact: none if the phase remains structure-only and does not change visible chat behavior, event semantics, or Tauri command contracts

## Scope

- Reduce the remaining root weight in `ChatView.tsx` after phase 1
- Keep chat UX, message order, composer behavior, and side-panel behavior unchanged
- Keep controller ownership stable unless a controller needs a small helper split to avoid silent regrowth
- Continue the repo-local `shell -> controller -> presentation surface` split pattern for chat
- Leave backend contracts, event names, and schemas untouched

## Non-Goals

- No chat UX redesign in phase 2
- No provider, routing, sidecar, or backend behavior changes
- No migration to a different state model
- No new giant `useChatPhase2Controller`
- No mechanical “split every helper into a file” cleanup that creates micro-files without clear responsibility

## Current State After Phase 1

Phase 1 already moved major domains out of the root:

- shell primitives
- chat-local service wrappers
- session/runtime controller
- stream controller
- collaboration controller
- message rail
- composer domain

That work materially improved the architecture, but `ChatView.tsx` still sits around 2817 lines and remains the largest frontend runtime file. The root is no longer a single giant mixed-concern page, but it still knows too much about:

- large markdown/render helper surfaces
- failed-run card presentation glue
- group orchestration board composition
- remaining cross-domain chat shell wiring

This means phase 2 should not repeat phase 1. It should focus on the remaining root-only presentation and helper clusters.

## Approach Options Considered

### Option 1: Root-thinning by helper and board extraction

Keep controllers and services mostly stable, and move remaining root-only heavy view logic into focused presentation modules.

Pros:

- lowest behavior risk
- preserves the new phase 1 architecture
- keeps phase 2 tightly scoped to the actual remaining root weight
- easiest path to reducing `ChatView.tsx` without destabilizing the chat runtime

Cons:

- root will still remain a composition file rather than a tiny wrapper
- requires discipline to avoid turning helper files into vague dumping grounds

### Option 2: Deep controller refactor

Push more of the remaining root behavior into stream or collaboration controllers.

Pros:

- could shrink the root faster
- might simplify some root wiring

Cons:

- high risk of creating hidden giant controllers
- increases lifecycle and event-order drift risk
- not necessary for the current root-thinning goal

### Option 3: Layout or UX refresh at the same time

Rework chat layout, group board placement, or markdown rendering while continuing the split.

Pros:

- could improve product ergonomics

Cons:

- mixes architecture work with product change
- greatly expands regression surface
- poor fit for a safe phase 2

## Recommended Approach

Use **Option 1: root-thinning by helper and board extraction**.

The right second pass is to leave the controller boundaries mostly intact and shrink the root by moving the remaining heavy presentation/helper concerns into explicit chat modules. The theme is:

- keep behavior where it is proven stable
- move rendering and view-only composition closer to the surface that owns it
- avoid hiding new complexity inside giant controllers

## Proposed Phase 2 Targets

### 1. Markdown render surface

Likely extraction targets:

- `markdownComponents`
- markdown-local helper mapping
- render-only message helpers that are easier to understand close to the rail

Recommended landing shape:

- `apps/runtime/src/components/chat/chatMarkdownComponents.tsx`
- or a narrowly named child helper under `components/chat/`

Rules:

- keep render-only logic here
- no React side effects
- no backend access

### 2. Stream item and failed-run presentation helpers

Likely extraction targets:

- `renderStreamItems`
- `renderRunFailureCard`
- other root-only helpers that exist mainly to feed the message rail

Recommended landing shape:

- fold into `ChatMessageRail.tsx` if the logic is truly rail-local
- otherwise create a narrowly named rail helper file such as `chatMessageRailHelpers.tsx`

Rules:

- keep helper ownership near the rail
- do not move session or stream orchestration into the rail

### 3. Group orchestration board presentation

Likely extraction targets:

- root-level board layout glue
- board-specific status presentation
- board-only action-row rendering

Recommended landing shape:

- `apps/runtime/src/components/chat/group-run/ChatGroupRunBoard.tsx`
- optionally add a small board-specific helper file only if the JSX remains hard to read

Rules:

- presentation in the board
- orchestration state stays in `useChatCollaborationController.ts`
- no direct backend calls from the board

### 4. Pending dialog and transient action surfaces

If still materially present in the root, continue shrinking by moving:

- confirm surfaces
- pending action dialogs
- local action-modal composition that is not needed elsewhere

Recommended landing shape:

- `ChatPendingActionDialogs.tsx`
- or another narrowly scoped chat dialog surface

Rules:

- keep dialog rendering near the chat surface
- do not create a generic modal utility with chat-specific assumptions baked in

## Controller Follow-Up Guardrails

Phase 2 should also watch the new controller boundaries so the split does not simply relocate the giant-file problem.

### `useChatStreamController.ts`

Current size is in the `WARN` zone. If phase 2 touches it, only do so to peel off clearly pure helpers such as:

- event normalization helpers
- derived stream-only summaries
- stable no-state selector functions

Do not split it purely for line count.

### `useChatCollaborationController.ts`

Also in the `WARN` zone. If it needs follow-up, prefer:

- board-specific view-model shaping
- pure summary helpers

Keep lifecycle, polling, and mutation ownership together unless there is a very obvious boundary.

## Proposed Target Structure

```text
apps/runtime/src/components/chat/
  ChatMessageRail.tsx
  ChatComposer.tsx
  ChatPendingActionDialogs.tsx
  chatMarkdownComponents.tsx
  chatMessageRailHelpers.tsx
  group-run/
    ChatGroupRunBoard.tsx
    groupRunBoardHelpers.tsx

apps/runtime/src/scenes/chat/
  useChatSessionController.ts
  useChatStreamController.ts
  useChatCollaborationController.ts
```

This is a direction, not a mandatory file list. The main rule is that new files must have clear ownership.

## Migration Strategy

Use a conservative order:

1. extract markdown/render-only helper surfaces
2. extract failed-run and stream-item presentation helpers closer to the rail
3. extract the group orchestration board presentation
4. extract any remaining pending action dialogs if they are still root-heavy
5. re-run the frontend large-file report and reassess whether the root now reflects a healthy shell/composition boundary

This order keeps the highest-risk runtime logic stable while still reducing the remaining root weight.

## Acceptance Criteria

Phase 2 should be considered successful when:

- `ChatView.tsx` no longer directly carries most large render-only helper clusters
- `ChatView.tsx` no longer directly carries most orchestration-board presentation
- the visible chat UX, event semantics, and backend contracts remain unchanged
- focused chat tests still pass
- `pnpm report:frontend-large-files` shows a meaningful additional drop in `ChatView.tsx`
- no new child module becomes an obvious hidden giant file without being explicitly called out

## Risks To Watch

### Hidden helper dumping-ground regression

The easiest failure mode is moving a pile of unrelated helpers into one vague file. New helper modules should stay close to either the rail or the board and remain narrow.

### Message rendering drift

Markdown, failed-run cards, and streaming item rendering all affect the main chat path. Keep render order and empty/loading behavior exactly the same.

### Collaboration board drift

Board presentation is tightly coupled to collaboration state. Keep data ownership in the controller and only move view work into the board.

## Recommended Next Step

Write a short implementation plan that keeps phase 2 structure-only, starts with render/helper extraction, and verifies each step against the existing focused chat suites plus the frontend large-file report.
