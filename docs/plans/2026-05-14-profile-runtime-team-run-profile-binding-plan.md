# Profile Runtime Team Run Profile Binding Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. For this repository, run implementation through Codex with `codex`, `test-driven-development`, and `systematic-debugging` skills enabled.

**Goal:** Complete the Phase 1 minimum safe slice that lets every newly-created team/group run step carry a resolved `profile_id` while preserving legacy `employee_id` UI/API behavior and old database compatibility.

**Architecture:** Add nullable profile identity to the group run step read/write path, resolve profile bindings from the existing profile runtime alias layer at step creation/execution boundaries, and keep employee ids as display/routing aliases. Legacy rows without `profile_id` must still load; new rows should persist or return a profile id whenever `agent_profiles` can resolve the assignee or dispatch source.

**Tech Stack:** Rust, Tauri, sqlx, SQLite, in-memory Rust integration tests, WorkClaw `employee_agents` group-run runtime, existing `profile_runtime` alias resolver.

---

## Strategy Summary

- Change surface: SQLite schema/current schema guardrails for `group_run_steps`, group run repo DTO/read-model projection, group run session creation/execution service, Rust group-run regression tests, and optional frontend TypeScript DTO typing.
- Affected modules: `apps/runtime/src-tauri/src/db/schema.rs`, `apps/runtime/src-tauri/src/db/migrations.rs` if legacy migrations need a historical table patch, `apps/runtime/src-tauri/src/profile_runtime/*`, `apps/runtime/src-tauri/src/commands/employee_agents/types.rs`, `group_run_repo.rs`, `group_run_execution_service.rs`, `session_repo.rs`, adjacent `employee_agents` tests, and `apps/runtime/src/types/employees.ts` if DTO shape changes.
- Main risk: breaking legacy SQLite databases or changing team run UI/API contracts while adding profile identity; secondary risk is treating a legacy employee row id as a real profile id when no `agent_profiles` row exists.
- Smallest safe path: add nullable columns and best-effort resolver first; never require `profile_id` for loading old rows; preserve `assignee_employee_id` and `dispatch_source_employee_id`; write `profile_id` only when a real profile row resolves; expose profile ids as optional fields.
- Verification: focused Rust tests for schema, start-run persistence, snapshot compatibility, and reassignment/session paths; then `pnpm test:rust-fast`; if frontend DTO/components change, run a focused Vitest/TypeScript check; always finish with `git diff --check`.
- Release impact: runtime DB/schema behavior is migration-sensitive but not packaging/release-metadata sensitive; no new sidecar/OpenClaw dependency and no intentional user-visible behavior change beyond optional profile id evidence.

## Roadmap Alignment

- Roadmap: `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`
- Phase: Phase 1 Profile Runtime 重构
- Target acceptance item: `[ ] 团队运行中的每个步骤绑定 profile，而不是只绑定文本 employee id。`
- Related but not completed by this slice:
  - `[ ] 旧员工启动后能迁移或映射到 profile，不丢会话。`
  - `[ ] IM 路由能定位目标 profile。`
  - `[ ] Profile home 删除、重置、导出有明确交互和风险确认。`

Do not mark the roadmap checkbox complete until implementation proves new group run steps persist/return profile ids and legacy rows still load.

## Goals

- New group/team run steps persist an optional assignee `profile_id` resolved from the assignee employee alias.
- New group/team run step snapshots/DTOs return optional profile identity without removing employee aliases.
- Step-created events may include optional profile identity as evidence, but must keep old payload keys.
- Existing rows with no profile column or empty profile values still load and continue/retry without crashing.
- Step execution/session creation can reuse the same resolution path to bind `sessions.profile_id` when possible.

## Non-Goals

- Do not implement full old-employee startup migration/backfill.
- Do not rewrite employee group JSON templates or group rules.
- Do not remove or rename `employee_id`, `assignee_employee_id`, `dispatch_source_employee_id`, or `waiting_for_employee_id`.
- Do not change memory injection, prompt output, approval behavior, `.skillpack` format, Toolset policy, or Curator behavior.
- Do not add sidecar endpoints, sidecar-only dependencies, or OpenClaw compatibility features.
- Do not complete IM route profile targeting in this slice.

## Current Findings

- `sessions.profile_id` and `agent_profiles` already exist in current schema, with `sessions.profile_id` indexed.
- `SessionSeedInput` and `insert_session_seed` still only write `employee_id`; new group run sessions therefore do not persist `profile_id` yet.
- `EmployeeGroupRunStep` currently exposes only `assignee_employee_id`, `dispatch_source_employee_id`, and `session_id`.
- `GroupRunStepSnapshotRow` and step execution context rows are still employee-id-only.
- `profile_runtime::alias_resolver` can resolve aliases across `profile_id`, `legacy_employee_row_id`, `employee_id`, `role_id`, and `openclaw_agent_id`.
- `profile_runtime::repo::load_profile_alias_candidates_with_pool` currently coalesces missing profile rows to the employee row id; implementation must avoid falsely treating that synthetic id as proof of a real `agent_profiles` row when persisting profile bindings.
- `resolve_group_step_memory_binding` already prefers real `agent_profiles` rows and falls back to legacy memory dir when missing; use this as the safety model.

## Legacy Mapping Decision

Legacy `agent_profiles` mapping is not a hard prerequisite for this minimum safe team-run binding slice.

Reasoning:
- The slice can add nullable profile columns and optional DTO fields now.
- New/current employees that already have `agent_profiles` can bind profile ids immediately.
- Old rows or old employees without a real profile row can keep `profile_id = NULL` and still load.
- The separate acceptance item for old employee startup migration/mapping remains incomplete and should not be folded into this slice.

Do not create a new blocking legacy-mapping task for this implementation. A child implementation task already exists under this plan: `t_a8b9f59d`.

## Minimum Safe Sequence

1. Add schema support for optional group-run step profile identity.
2. Add read helpers that tolerate missing columns and empty values.
3. Resolve real profile rows for assignee/dispatch source during new step creation.
4. Project optional profile ids in Rust DTOs and frontend type definitions.
5. Bind newly-created group run sessions to `sessions.profile_id` when a real profile row resolves.
6. Add regression tests before implementation and keep legacy no-profile rows green.
7. Only after tests pass, consider updating the Phase 1 roadmap checkbox.

## Non-Breakable Compatibility Boundaries

- Existing Tauri command names and request payloads stay unchanged.
- Existing response fields stay unchanged; new `profile_id` fields must be optional/additive.
- `employee_id` and `assignee_employee_id` remain valid route/display aliases.
- Legacy `sessions.employee_id`, `group_runs.main_employee_id`, and `group_runs.waiting_for_employee_id` remain readable and writable.
- Existing `group_run_events.payload_json` keys remain present.
- Existing SQLite databases without new columns must load through fallback or migration.
- No new sidecar/OpenClaw runtime dependency is allowed.

## Default Path vs Legacy Fallback

- Default path for new data: resolve assignee/dispatch aliases to real `agent_profiles.id`, persist optional profile columns, and return them in snapshots.
- Legacy fallback: if `agent_profiles` is absent, the column is absent, or no real profile row exists, leave profile fields `NULL`/empty and continue using employee aliases.
- Never require profile id to continue, retry, reassign, approve, or list a group run.

## Planned File Changes

### Backend Rust

- Modify: `apps/runtime/src-tauri/src/db/schema.rs`
  - Add `assignee_profile_id TEXT` and `dispatch_source_profile_id TEXT` to `group_run_steps` current schema.
  - Add best-effort `ALTER TABLE` guards for current-schema initialization if appropriate.
  - Add indexes only if they improve lookup and do not add migration risk.

- Modify: `apps/runtime/src-tauri/src/db/migrations.rs`
  - If historical migrations own `group_run_steps`, add backward-compatible `ALTER TABLE` steps for the new nullable columns.
  - If current schema bootstrap covers all practical paths, keep migration change minimal but prove old DB compatibility in tests.

- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/types.rs`
  - Add optional fields to `EmployeeGroupRunStep`:
    - `pub assignee_profile_id: Option<String>`
    - `pub dispatch_source_profile_id: Option<String>`
  - Consider optional aliases only; do not remove existing employee fields.

- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_repo.rs`
  - Extend `GroupRunStepSnapshotRow` with optional profile fields.
  - Add a helper that checks whether `group_run_steps` has profile columns before selecting them, or use a schema-safe projection strategy.
  - Extend `insert_group_run_step_seed` parameters to accept optional profile ids and insert them when columns exist.
  - Extend reassignment reset to clear/rewrite assignee profile id when the assignee changes.
  - Keep all queries safe for legacy test tables that only have a subset of group-run columns.

- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`
  - Resolve assignee and dispatch source profile ids before inserting step seeds.
  - Include optional profile ids in `step_created` event payloads while preserving old keys.
  - In `ensure_group_run_session_with_pool` and `ensure_group_step_session_with_pool`, pass resolved profile id into session seed creation.
  - Do not change prompt text or memory injection.

- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
  - Extend `SessionSeedInput` with `profile_id: Option<&str>`.
  - Make `insert_session_seed` write `profile_id` only when the column exists, or use a safe fallback path if the column is missing.
  - Preserve existing `employee_id` writes.

- Reuse or extend: `apps/runtime/src-tauri/src/profile_runtime/*`
  - Add a repo helper for resolving only real `agent_profiles` rows by alias.
  - Avoid using coalesced employee row ids as persisted `profile_id` unless an actual profile row exists.

### Frontend Type Surface

- Modify only if backend DTO changes are surfaced to TypeScript:
  - `apps/runtime/src/types/employees.ts`
    - Add `assignee_profile_id?: string | null` and `dispatch_source_profile_id?: string | null` to `EmployeeGroupRunStep`.
- Do not add new UI behavior unless required for type safety.

## Data Migration and Legacy Fallback

- New columns must be nullable and default to `NULL` or empty string.
- No destructive migration or table rebuild is allowed for this slice.
- Existing step rows should not be backfilled in-place unless a simple, deterministic real-profile join exists and tests prove it.
- Prefer runtime resolution for old rows:
  - If row has `assignee_profile_id`, return it.
  - Else optionally resolve `assignee_employee_id` to a real `agent_profiles.id` for read-model display.
  - If no real profile row exists, return `None`.
- Session seed migration must preserve `employee_id`; `profile_id` is additive.

## Negative Tests

Add tests proving forbidden or risky paths do not happen:

- Legacy `group_run_steps` rows without profile ids still produce snapshots.
- Missing `agent_profiles` table or no matching profile row does not fail start/continue/retry.
- Persisted profile id is not populated from a synthetic employee row id when no real `agent_profiles` row exists.
- No test requires `apps/runtime/sidecar` or OpenClaw directory compatibility for the new profile binding.
- Existing employee-id fields remain present in returned DTOs.

## Testing Strategy

Minimum required commands for the implementation worker:

```bash
pnpm test:rust-fast
```

Focused commands to run while developing, adjusted to the actual test names added:

```bash
cd apps/runtime/src-tauri
cargo test --test test_employee_groups_db group_run_steps -- --nocapture
cargo test --test test_im_employee_agents group_run -- --nocapture
cargo test profile_runtime -- --nocapture
```

If frontend types/components are touched:

```bash
pnpm --filter runtime test -- --run <focused-test-name>
pnpm --filter runtime typecheck
```

Final hygiene:

```bash
git diff --check
git status --short
```

## Codex Execution Batches

### Batch 1: Schema and DTO Contract

**Objective:** Add optional profile columns and response fields without changing behavior.

**Files:**
- Modify: `apps/runtime/src-tauri/src/db/schema.rs`
- Modify if needed: `apps/runtime/src-tauri/src/db/migrations.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/types.rs`
- Modify if needed: `apps/runtime/src/types/employees.ts`
- Test: `apps/runtime/src-tauri/tests/test_employee_groups_db.rs`

**Steps:**
1. Write failing schema test that `group_run_steps` current schema contains nullable profile columns.
2. Write failing DTO/snapshot compatibility test that old rows without profile ids still serialize/load.
3. Implement nullable columns and optional DTO fields.
4. Run focused Rust tests for the new schema assertions.
5. Commit with message: `feat(profile-runtime): add group run step profile contract`.

### Batch 2: Real Profile Resolution Helper

**Objective:** Resolve aliases to real `agent_profiles.id` only when a profile row exists.

**Files:**
- Modify: `apps/runtime/src-tauri/src/profile_runtime/repo.rs`
- Modify if needed: `apps/runtime/src-tauri/src/profile_runtime/alias_resolver.rs`
- Test: colocated profile runtime tests or an adjacent Rust integration test

**Steps:**
1. Write failing test for resolving `employee_id`, `role_id`, and `openclaw_agent_id` to a real profile row.
2. Write failing test proving missing profile rows do not return synthetic employee row ids for persistence.
3. Implement `resolve_real_profile_id_for_alias_with_pool` or equivalent helper.
4. Run focused profile runtime tests.
5. Commit with message: `feat(profile-runtime): resolve real profile aliases for persistence`.

### Batch 3: Persist Profile Binding on New Group Run Steps

**Objective:** Store optional profile ids when creating plan/execute/review/revision steps.

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/group_run.rs`

**Steps:**
1. Write failing test that creates employees with real profile rows, starts a group run, and asserts step rows have `assignee_profile_id`.
2. Write failing test that old/no-profile employees still create and load steps with `NULL` profile ids.
3. Resolve and pass optional assignee/dispatch profile ids into step seed insertion.
4. Include optional profile ids in `step_created` event payloads.
5. Run focused group run tests.
6. Commit with message: `feat(profile-runtime): bind group run steps to profiles`.

### Batch 4: Session Profile Binding for Group Run Sessions

**Objective:** Ensure group run coordinator and employee step sessions also carry `sessions.profile_id` when resolvable.

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/group_run.rs` or a focused session repo test

**Steps:**
1. Write failing test asserting coordinator group-run session stores `sessions.profile_id` for a mapped coordinator.
2. Write failing test asserting employee step session stores `sessions.profile_id` for a mapped assignee.
3. Extend `SessionSeedInput` and safe insert path.
4. Pass optional profile ids at group-run session creation points.
5. Run focused session/group-run tests.
6. Commit with message: `feat(profile-runtime): bind group run sessions to profiles`.

### Batch 5: Reassign/Retry/Continue Compatibility

**Objective:** Keep lifecycle controls working and update/clear profile binding on reassignment.

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_lifecycle_service.rs` if reassignment orchestration lives there
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/group_run.rs`

**Steps:**
1. Write failing test for reassigning a failed step to another mapped employee and expecting the assignee profile id to change.
2. Write failing test for retry/continue on an old row with no profile id.
3. Implement reassignment profile update or clear behavior.
4. Run focused lifecycle tests.
5. Commit with message: `fix(profile-runtime): preserve group run profile binding through lifecycle`.

### Batch 6: Verification and Roadmap Update

**Objective:** Prove the slice and update docs only after tests pass.

**Files:**
- Modify after green tests: `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`
- Keep this plan as implementation evidence.

**Steps:**
1. Run `pnpm test:rust-fast`.
2. Run frontend checks only if frontend files changed.
3. Run `git diff --check`.
4. If all profile-binding acceptance conditions are met, update the roadmap checkbox for team-run step profile binding to `[x]` or `[~]` with dated evidence.
5. Record remaining risks in the kanban handoff.

## Acceptance Criteria for Implementation

- New group run step rows have nullable profile columns in current and migrated schemas.
- New group run steps for mapped employees persist a real `agent_profiles.id`.
- Group run snapshots/results expose optional profile ids while preserving employee-id fields.
- Legacy databases/rows without profile columns or values still load.
- Group run coordinator and step sessions can persist `sessions.profile_id` when resolvable.
- Reassign/retry/continue flows do not lose compatibility.
- `pnpm test:rust-fast` and `git diff --check` pass.
- No new sidecar/OpenClaw dependency is introduced.

## Handoff Prompt for Codex

Use this prompt skeleton for the implementation worker:

```text
You are implementing WorkClaw roadmap Phase 1: team/group run step profile binding.
Read AGENTS.md, .agents/skills/workclaw-implementation-strategy/SKILL.md, and docs/plans/2026-05-14-profile-runtime-team-run-profile-binding-plan.md first.
Use TDD. Do not change memory injection, prompt output, approval behavior, .skillpack behavior, sidecar endpoints, or OpenClaw compatibility.
Goal: add optional profile_id binding to group_run_steps and group run sessions while keeping employee_id aliases and legacy rows working.
Required verification: focused Rust tests, pnpm test:rust-fast, git diff --check. If frontend files change, run focused frontend type/test checks.
Return changed files, tests run, remaining risks, and whether the roadmap checkbox can be updated.
```
