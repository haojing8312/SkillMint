# Task Lineage Read Model Split Note

## Why this note exists
- [apps/runtime/src-tauri/src/commands/session_runs.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/session_runs.rs) is already above the 800-line feature-work threshold.
- [apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs) is also above the 800-line feature-work threshold.
- Task Engine Phase 1 needs one more round of lineage/task-graph read-model work, but the logic should not keep accreting inside those giant files.

## Immediate split direction
- Add a small reusable task-lineage helper module under `apps/runtime/src-tauri/src/agent/runtime/`.
- Keep `session_runs.rs` focused on SQLite + journal projection.
- Keep `trace_builder.rs` focused on event summarization and trace assembly.
- Move derived logic such as:
  - effective task identity fallback
  - task path construction
  - task graph node projection
  into the helper instead of duplicating it in multiple giant files.

## Not in this slice
- No large mechanical split of `session_runs.rs`.
- No large mechanical split of `trace_builder.rs`.
- No new Tauri command surface.

## Next split candidates
1. Extract `session_runs` read-model helpers into `src/commands/session_runs_read_model.rs` or a sibling module once task graph fields stabilize.
2. Extract trace assembly helpers from `trace_builder.rs` into `src/agent/runtime/trace_summary.rs` or `trace_projection.rs` once task graph projection is stable.
