# Parallel Task Tabs Design

## Summary

WorkClaw should support lightweight parallel task tabs so users can start a new task without interrupting a currently running session. Tabs should represent active working contexts, not the full historical session list. The existing left sidebar remains the system of record for session history, while the new tab strip provides a small set of actively open work surfaces.

The first version should support two tab kinds only:

- Start-task tab: an empty task composer surface
- Session tab: an opened chat session

This design intentionally avoids turning every historical session into a permanent browser-style tab model. That would duplicate the sidebar, add heavy state management, and make session navigation harder to reason about.

## Goals

- Let users start a new task while an existing task continues to run
- Preserve the current left sidebar as the canonical session history
- Avoid interrupting running sessions when users click `开始任务`
- Keep the first version small enough to land safely in the current App architecture

## Non-Goals

- Full browser-style session tab persistence across restarts
- Drag-and-drop tab reordering
- Right-click tab menus
- Employee page, experts page, or settings page as first-class tabs in v1
- Multi-window support

## User Experience Rules

### Start Task Behavior

When the user clicks `开始任务`:

- If the active tab is a running session tab, create and focus a new empty start-task tab
- If the active tab is an ended session tab, reuse that tab and switch it to an empty start-task tab
- If the active tab is already a start-task tab, keep using it

This matches the desired behavior: running work is protected, finished work can be replaced in place.

### Plus Button Behavior

The `+` button always creates a new empty start-task tab and focuses it.

### Sidebar Session Click Behavior

In v1, clicking a session in the left sidebar opens that session in the currently active tab:

- If the active tab is a start-task tab, convert it into that session tab
- If the active tab is a session tab, replace that tab’s session target with the selected session

This keeps the first version simple. We can add “open in new tab” later if needed.

### Closing Tabs

Closing a tab only closes the front-end working context. It must never delete the underlying session.

When closing:

- If multiple tabs remain, focus the nearest neighbor
- If the last tab is closed, immediately create a fresh start-task tab so the app never ends up tabless

## Architecture

## Change Surface
- Runtime app shell state in `apps/runtime/src/App.tsx`
- Likely new top tab-strip UI component under `apps/runtime/src/components/`
- Existing session history sidebar in `apps/runtime/src/components/Sidebar.tsx`
- Existing App session and navigation tests in `apps/runtime/src/__tests__/`
- Smoke navigation E2E in `apps/runtime/e2e/smoke.navigation.spec.ts`

## Proposed State Model

Add a front-end-only tab model:

```ts
type WorkTab =
  | {
      id: string;
      kind: "start-task";
      title: string;
      draftInput?: string;
    }
  | {
      id: string;
      kind: "session";
      sessionId: string;
      title: string;
    };
```

Add:

- `tabs: WorkTab[]`
- `activeTabId: string`

Derive:

- `activeTab`
- `activeSessionId` when the active tab is a session tab
- `activeStartTaskTab` when the active tab is a start-task tab

The existing `sessions` array remains the real session list loaded from backend and local recovery.

## Why This Is Safer Than Full Session Tabs

Today the app assumes one global active session in many places:

- `selectedSessionId`
- pending initial messages
- group run focus state
- employee-assistant context
- chat view mount logic

Replacing the entire model with “many selected sessions at once” would be high risk. A lightweight work-tab layer lets us isolate active working context without changing the backend session model.

## Runtime Status Rule

The “is current session running?” decision should be based on existing session runtime metadata:

- `running`
- `waiting_approval`

These should count as active/running tabs for start-task auto-split logic.

Ended tabs include:

- `completed`
- `failed`
- missing runtime status / idle

## Rendering Model

Main content should render from the active tab:

- `start-task` tab -> existing `NewSessionLanding`
- `session` tab -> existing `ChatView`

The global view modes `experts`, `employees`, and `settings` should remain outside the tab system in v1. That keeps scope controlled and avoids mixing entity management pages with task-parallel surfaces.

## Navigation Interaction

In v1:

- The tab strip appears only inside the start-task/chat area
- Switching to experts/employees/settings temporarily leaves the tabbed work surface intact
- Returning to `开始任务` restores the previously active work tab

This gives users parallel task continuity without forcing tabs onto every major screen.

## Persistence

V1 should persist only the last active tab context, not the entire open tab set.

Persist:

- active tab kind
- active session id if applicable
- active start-task tab marker

Do not persist:

- the full tab array
- tab order
- multiple open tabs

Reason: this keeps restart recovery understandable and aligns with the current single-session recovery model.

## Edge Cases

### Running Session + Start Task

Must create a new start-task tab rather than replacing the running session tab.

### Finished Session + Start Task

May safely reuse the existing tab as a fresh start-task tab.

### Deleted Session While Tab Is Open

If a session tab points to a deleted session:

- convert that tab into a start-task tab
- keep focus in place
- do not crash or leave a blank state

### Session Selected From Sidebar While Running Session Tab Is Active

V1 should still replace the current tab. This is the simplest rule.

If later user research shows this is surprising, we can add:

- modified click opens in new tab
- explicit “open in new tab” affordance

## Testing Strategy

Add coverage for:

- Clicking `开始任务` from a running session opens a new tab
- Clicking `开始任务` from a finished session reuses the current tab
- Clicking `+` creates a new start-task tab
- Closing tabs never deletes sessions
- Sidebar session selection opens into the current tab
- Refresh still restores the last active tab/session safely
- Experts/employees/settings navigation does not destroy open task tabs

## Recommendation

Implement the lightweight parallel task tab model in phases:

1. Add front-end tab state and tab strip UI
2. Route start-task/chat rendering through active tab
3. Implement running-session auto-split rules
4. Add recovery and regression tests

This is the smallest safe path that delivers real parallel-task value without destabilizing WorkClaw’s current session system.
