## Session Runs / Export Task Record Split Note

### Context
- [apps/runtime/src-tauri/src/commands/session_runs.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/session_runs.rs) is above the 800-line split threshold.
- [apps/runtime/src-tauri/src/commands/chat_session_io/session_export.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_session_io/session_export.rs) is also above the 800-line split threshold.
- This change only adds task-record read-model projection and recovered export rendering. It does not justify a rushed structural split inside the same patch.

### Why This Change Stays In Place
- `session_runs.rs` already owns run projection shaping, so adding a `task_record` read-model projection is still in-bounds for this pass.
- `session_export.rs` already owns recovered-run rendering, so adding task-record summary lines is still in-bounds for this pass.
- Keeping the logic in-place for one more iteration reduces the risk of mixing a structural split with a new runtime data contract.

### Planned Follow-Up Split
- Extract a `task_projection.rs` or similar helper from `session_runs.rs` for:
  - effective task identity lookup
  - task-record lookup
  - task-path / task-status / task-record projection
- Extract a small `recovered_task_sections.rs` helper from `session_export.rs` for:
  - recovered task graph rendering
  - recovered task identity rendering
  - recovered task record rendering

### Trigger To Split
- The trigger has now been reached once more: recovered export rendering is also carrying task-continuation lines.
- Allow this patch to stay in-place because it is a narrow read-model follow-up paired with integration verification.
- Treat the next feature on either file as the mandatory extraction point for:
  - `recovered_task_sections.rs` in `session_export.rs`
  - `task_projection.rs` in `session_runs.rs`
