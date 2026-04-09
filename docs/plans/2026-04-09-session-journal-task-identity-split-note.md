# Session Journal Split Note

**Date:** 2026-04-09

## Why This Note Exists

[session_journal.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/session_journal.rs) is already above the Rust runtime split threshold. This Task Engine slice still needs to touch it because the change is a narrow extension of the existing journal projection contract, not a new persistence subsystem.

## Why The Change Stays In `session_journal.rs`

- The new work only extends the existing `TurnStateSnapshot -> SessionRunTurnStateSnapshot` projection.
- It does not introduce a distinct repository, schema lane, or standalone task-journal subsystem yet.
- Splitting now would mix two refactors at once:
  - Task Engine phase-1 task identity propagation
  - journal module extraction

That would increase risk on a runtime-critical path without improving the immediate contract change.

## Planned Future Split

When Task Engine grows beyond task identity into richer task-level persistence, split the journal layer into:

- `session_journal/state.rs`
- `session_journal/events.rs`
- `session_journal/projection.rs`

For this slice, keep the change local and minimal.
