# Rust Large File Backlog

**Goal:** Turn the current `500 / 800` Rust runtime guardrails into an actionable backlog for `apps/runtime/src-tauri/`.

**Source:** Generated from `node scripts/report-rust-large-files.mjs` on 2026-03-23 using thresholds `warn=500` and `plan=800`.

## Prioritization Rules

- Prioritize `commands/*.rs` files first because they are the highest-risk place for AI-assisted feature accretion.
- Prioritize startup-critical and runtime-core files second because they affect broad behavior and are expensive to verify after every change.
- Prioritize tests and tools third unless they are actively blocking refactors.
- Large child modules created during recent splits can stay in backlog, but they should not restart the pattern of becoming new giant files.

## Priority 1: Large Child Modules And Integration Boundaries

These files are now the highest-value remaining Rust runtime split targets.

- `apps/runtime/src-tauri/src/agent/tools/employee_manage.rs` — 988 lines
  - Why first: it is now the only remaining Rust production file above the `800` split-design threshold
  - First split direction: separate tool entrypoint glue, employee mutation use cases, and response-formatting helpers
  - First safe step: define a split plan before adding new tool behavior

## Priority 2: Large Child Modules And Tests

These files are above the warning or split threshold, but they no longer lead the queue ahead of the largest remaining production files and tests.

- `apps/runtime/src-tauri/src/adapters/openai.rs` — 756 lines
  - Risk: provider protocol shaping, streaming, and error normalization can keep accreting in one adapter
  - First split direction: split request building, streaming/event handling, and response normalization into cohesive helpers only if a new feature needs one of those lanes

- `apps/runtime/src-tauri/tests/helpers/mod.rs` — 658 lines
  - Risk: shared integration-test setup can quietly become the next giant test utility surface
  - First split direction: separate database fixture setup from skill/workspace helpers only when a new scenario family forces that distinction

## Completed Templates

- `apps/runtime/src-tauri/src/commands/employee_agents.rs` — now 799 lines
  - Status: completed as the first formal Rust splitting template
  - Outcome: root file is now below the `800` split-design threshold
  - Follow-up: reuse this module structure as the reference pattern for later command-file governance

- `apps/runtime/src-tauri/src/commands/employee_agents/service.rs` — now 575 lines
  - Status: completed as the current service-layer follow-up for the employee agents template
  - Outcome: orchestration now lives across focused child services instead of one giant service shell
  - Delivered structure: `group_run_progress_service.rs` and `group_run_execution_service.rs` now hold the remaining group-run progress and execution/session lanes

- `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs` — now 349 lines
  - Status: completed as the current repo-layer follow-up for the employee agents template
  - Outcome: the root repo is now a thin aggregation shell
  - Delivered structure: `group_run_repo.rs`, `session_repo.rs`, and `feishu_binding_repo.rs`

- `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs` — now 650 lines
  - Status: completed as the formal Rust template for plugin and integration-heavy command surfaces
  - Outcome: root file is now below the `800` split-design threshold
  - Delivered structure: `types`, `tauri_commands`, `settings_service`, `setup_service`, `runtime_service`, `install_repo`, `install_service`, `plugin_host_service`, `installer_session`, and `tests`
  - Follow-up: reuse this structure when shrinking other setup/runtime/integration-heavy Rust command files

- `apps/runtime/src-tauri/src/commands/feishu_gateway.rs` — now 373 lines
  - Status: completed as the formal Rust template for external gateway command surfaces
  - Outcome: root file is now well below the `500` target zone
  - Delivered structure: `types`, `payload_parser`, `gate_service`, `pairing_service`, `approval_service`, `outbound_service`, `relay_service`, `planning_service`, `ingress_service`, `repo`, `settings_service`, `tauri_commands`, and `tests`

- `apps/runtime/src-tauri/src/commands/clawhub.rs` — now 266 lines
  - Status: completed as the formal Rust template for marketplace or remote-catalog command surfaces
  - Outcome: root file is now well below the `500` target zone
  - Delivered structure: `types`, `support`, `repo`, `detail_service`, `download_service`, `install_service`, `search_service`, and `translation_service`

- `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs` — now 40 lines
  - Status: completed as the formal Rust template for chat runtime entry shells
  - Outcome: root file is now far below the `500` target zone
  - Delivered structure: `types`, `workspace_skills`, `session_titles`, `runtime_inputs`, `runtime_events`, `runtime_support`, and `message_reconstruction`

- `apps/runtime/src-tauri/src/commands/chat_session_io.rs` — now 688 lines
  - Status: completed as the formal chat/session data-plane split
  - Outcome: root file is now below the `800` split-design threshold
  - Delivered structure: `session_store`, `session_view`, `session_export`, and `session_compaction`
  - Follow-up: reuse this structure when shrinking other session-facing Rust files such as `chat_runtime_io.rs`

- `apps/runtime/src-tauri/src/agent/executor.rs` — now 96 lines
  - Status: completed as a runtime-core thinning milestone
  - Outcome: root file is now well below the `500` target zone

- `apps/runtime/src-tauri/src/db.rs` — now 147 lines
  - Status: completed as the formal Rust template for SQLite bootstrap governance
  - Outcome: root file is now well below the `500` target zone
  - Delivered structure: `schema.rs`, `migrations.rs`, and `seed.rs`
  - Follow-up: place new tables in `schema.rs`, new columns in `migrations.rs`, and repeatable startup defaults in `seed.rs`

- `apps/runtime/src-tauri/tests/test_im_employee_agents.rs` — now 93 lines
  - Status: completed as the formal Rust integration-test split template
  - Outcome: root test binary is now a thin shell that only registers scenario modules plus a small base sanity layer
  - Delivered structure: `im_routing.rs`, `group_management.rs`, `group_run.rs`, and `team_entry.rs`
  - Follow-up: reuse this scenario-family split pattern before letting another Rust integration test grow into a giant monolithic binary

## Current Warn Queue

These files are above 500 lines and should be watched, but they are not first in line for dedicated split work.

- `apps/runtime/src-tauri/src/adapters/openai.rs` — 756 lines
- `apps/runtime/src-tauri/src/commands/models_repo.rs` — 746 lines
- `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs` — 666 lines
- `apps/runtime/src-tauri/tests/helpers/mod.rs` — 658 lines
- `apps/runtime/src-tauri/src/commands/skills.rs` — 643 lines
- `apps/runtime/src-tauri/src/commands/runtime_preferences.rs` — 637 lines
- `apps/runtime/src-tauri/src/lib.rs` — 614 lines
- `apps/runtime/src-tauri/src/commands/packaging.rs` — 613 lines
- `apps/runtime/src-tauri/tests/test_session_export_recovery.rs` — 609 lines
- `apps/runtime/src-tauri/src/agent/run_guard.rs` — 597 lines
- `apps/runtime/src-tauri/tests/test_e2e_flow.rs` — 583 lines
- `apps/runtime/src-tauri/src/diagnostics.rs` — 562 lines
- `apps/runtime/src-tauri/src/commands/models.rs` — 539 lines
- `apps/runtime/src-tauri/tests/test_feishu_gateway.rs` — 529 lines
- `apps/runtime/src-tauri/src/team_templates.rs` — 514 lines
- `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs` — 513 lines
- `apps/runtime/src-tauri/src/commands/chat_policy.rs` — 506 lines
- `apps/runtime/src-tauri/tests/test_approval_bus.rs` — 505 lines

## Recommended Execution Order

1. Use `employee_agents` as the formal reference template for business-heavy command surfaces.
2. Use `openclaw_plugins` as the formal reference template for integration-heavy command surfaces.
3. Reassess the next runtime-core or tooling target from a fresh large-file report.
4. Use the `employee_agents` child-module layout as the reference when a future business-heavy Rust module starts accumulating large service or repo files.
5. Re-run `node scripts/report-rust-large-files.mjs` after each split milestone and update this backlog rather than treating it as static.

## Definition Of Backlog Progress

- A file leaves the `PLAN` queue only when it falls below 800 lines or when remaining content is clearly limited to a single responsibility.
- A file is not considered improved if code was merely moved into an equally giant child file.
- A split is successful only when command, service, repo, and gateway boundaries are clearer after the change than before it.
