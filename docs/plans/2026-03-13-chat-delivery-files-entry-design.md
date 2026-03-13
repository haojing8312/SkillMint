# Chat Delivery Files Entry Design

**Context**

The chat transcript currently renders a task journey timeline and a delivery summary block after assistant messages that produce deliverables. The user does not find the task progress presentation useful in the main transcript. They want a single, file-oriented entry point that appears only after a task has reached a completed state and lets them jump directly into the workspace files panel.

**Goal**

Replace the main-area task progress and delivery summary presentation with one compact card that says `查看此任务中的所有文件`. The card should appear only when the task has deliverables and the task status is `completed` or `partial`. Clicking the card should open the side panel and switch to the `文件` tab.

**Decisions**

- Treat `completed` and `partial` as finished states for this UI.
- Do not show the card for `running` or `failed`.
- Keep the existing summary anchor location: directly after the assistant message that produced the deliverables.
- Keep the existing right-side panel structure and file-highlighting behavior unchanged.
- Remove the main-area task timeline, file list, warning list, workspace shortcut, and resume-failed-work shortcut from this transcript summary area.

**Implementation Shape**

- Keep using the existing `TaskJourneyViewModel` for status and deliverable detection.
- Replace `TaskJourneySummary` content with a compact completion card component.
- Tighten the render guard in `ChatView` so transcript summary only mounts for `completed` and `partial`.
- Reuse the current `handleViewFilesFromDelivery()` behavior to open the side panel and select `files`.

**Testing**

- Update transcript summary tests to assert the new card appears for `completed`.
- Add coverage that `partial` also shows the card.
- Assert that `running`, `failed`, and empty sessions do not show the card.
- Assert that clicking the card opens the side panel and highlights the `文件` tab.
