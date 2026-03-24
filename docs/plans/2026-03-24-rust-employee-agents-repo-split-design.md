# Rust Employee Agents Repo Split Design

**Goal:** Turn [repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/repo.rs) into a maintainable storage aggregation layer by splitting the remaining giant persistence surface into focused repository lanes for group-run persistence, session/thread persistence, and Feishu binding persistence. The split must preserve the current SQL behavior and keep [repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/repo.rs) as a thin compatibility and re-export shell.

## Why `repo.rs` Is The Next Employee Agents Target

The root [employee_agents.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents.rs) is already healthy, but the giant-file risk has shifted downward into [repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/repo.rs), which is still around 1800 lines.

That file currently mixes several different persistence concerns:

- employee association and scope reads
- thread/session linking and inbound-event persistence
- group-run state transitions, retries, reassignment, and snapshots
- Feishu routing/binding persistence

If we leave those mixed together, later employee-domain work will keep growing `repo.rs` the same way the old root command file used to grow.

## Current Boundary Map

### What `repo.rs` Owns Today

- a small `profile_repo` child slice that already owns profile CRUD and skill bindings
- session seed and thread/session link persistence
- group-run rows, state transitions, step transitions, and event persistence
- snapshot reads for group run detail views
- Feishu routing binding insert/delete/replace flows
- row structs for all of those concerns
- a test module still attached to the root repo

### Why The Current Shape Is Hard To Extend

- `group run` writes and reads dominate the file and form a natural giant child inside the root repo
- session/thread persistence has a different lifecycle from group-run persistence, but they are interleaved
- Feishu binding persistence is integration-oriented storage logic, not core employee storage logic, yet it lives in the same file
- row structs for unrelated concerns crowd each other and make the repo harder to scan

## Recommended Design

### 1. Keep `repo.rs` As A Thin Aggregation Layer

The root [repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/repo.rs) should keep only:

- child module declarations
- `pub(crate)` or `pub(super)` re-exports for child repo functions and row types
- a very small amount of compatibility glue if multiple services still import through the root

It should stop being the place where large SQL blocks live directly.

### 2. Split By Persistence Concern

This split should not use one-file-per-function or generic helper extraction. The correct boundaries are storage lanes:

- `profile_repo.rs`
  - already exists and remains the employee profile and skill-binding persistence lane
- `group_run_repo.rs`
  - all `group_runs`, `group_run_steps`, and `group_run_events` persistence
  - retry, reassign, review, snapshot, and finalize-related queries
- `session_repo.rs`
  - `sessions`, `im_thread_sessions`, `im_message_links`, and linked session-message writes
  - thread/session lookup, seed, and route-session linkage
- `feishu_binding_repo.rs`
  - `im_routing_bindings` rows related to Feishu employee routing
  - binding insert, displaced-binding cleanup, count, and scope queries

These are the smallest real concern boundaries that already exist in the SQL.

### 3. Prioritize The Largest Lane First

The execution order should be:

1. `group_run_repo.rs`
2. `session_repo.rs`
3. `feishu_binding_repo.rs`
4. final thin `repo.rs` cleanup

That order is recommended because:

- `group_run` is currently the largest persistence lane and the biggest source of future accretion
- `session_repo` is a clean middle-sized lane that is easy to separate after `group_run`
- `feishu_binding_repo` is smaller and can be moved once the core storage lanes are already clear

## Suggested Module Layout

Under `apps/runtime/src-tauri/src/commands/employee_agents/`:

- [repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/repo.rs)
  - thin aggregation layer and re-exports
- [profile_repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/profile_repo.rs)
  - employee profile and skill-binding persistence
- `group_run_repo.rs`
  - group run writes, read-model rows, snapshots, and state transitions
- `session_repo.rs`
  - session seed, thread/session links, inbound event links, and message persistence
- `feishu_binding_repo.rs`
  - Feishu routing binding persistence and cleanup

## Responsibility Split

### `group_run_repo.rs`

This module should own:

- `GroupRunStateRow`, `GroupRunSnapshotRow`, `GroupRunStepSnapshotRow`, `GroupRunEventSnapshotRow`
- failed-step queries
- review-state queries and updates
- execute/reassign/retry/finalize persistence
- `group_runs`, `group_run_steps`, `group_run_events` indexes and row lifecycle concerns already expressed in SQL

This is the most important extraction because it reduces the biggest future-growth surface.

### `session_repo.rs`

This module should own:

- `ThreadSessionRecord`
- `SessionSeedInput`, `ThreadSessionLinkInput`, `InboundEventLinkInput`
- `GroupStepSessionRow`, `EmployeeSessionSeedRow`, `SessionMessageRow`
- latest-session lookup
- session seed and update
- thread/session link upsert
- inbound event link persistence
- session message insert/list helpers

This keeps the thread/session lane readable and separate from group-run storage.

### `feishu_binding_repo.rs`

This module should own:

- `EmployeeAssociationRow`
- `EmployeeGroupEntryRow`
- `InsertFeishuBindingInput`
- scope and binding list/count helpers
- displaced-binding cleanup and insert flows
- `im_routing_bindings` persistence specific to Feishu employee routing

This keeps integration-facing routing storage from being mixed with session or group-run storage.

## Row Type Placement Rule

A row struct should live with the repo module that owns the SQL producing it.

That means:

- group-run rows belong in `group_run_repo.rs`
- session/thread rows belong in `session_repo.rs`
- Feishu-binding rows belong in `feishu_binding_repo.rs`
- profile rows remain in `profile_repo.rs`

The root repo should stop accumulating unrelated row structs over time.

## Risks

- breaking service imports if re-exports move too aggressively
- moving one giant repo file into one equally giant child file
- changing ordering or null/default handling in snapshot rows
- changing thread/session linking behavior by accident

## Smallest Safe Split Order

1. Create `group_run_repo.rs` and move only group-run rows plus queries.
2. Re-export those functions from root `repo.rs`.
3. Verify group-run targeted tests.
4. Create `session_repo.rs` and move session/thread/inbound-link persistence.
5. Re-export those functions from root `repo.rs`.
6. Verify session-routing targeted tests.
7. Create `feishu_binding_repo.rs` and move Feishu binding persistence.
8. Re-export those functions from root `repo.rs`.
9. Remove any leftover row structs or SQL from root `repo.rs` that now belong in child modules.
10. Move or keep the current root repo tests depending on whether they still fit the aggregation shell.

## Success Criteria

- [repo.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/repo.rs) becomes a thin re-export shell
- `group_run` persistence is no longer mixed with session or Feishu binding persistence
- row structs live next to the SQL that owns them
- no new giant replacement child file is created
- targeted Rust verification remains green during the split
