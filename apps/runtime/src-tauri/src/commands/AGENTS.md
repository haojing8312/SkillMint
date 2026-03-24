# Rust Commands AGENTS.md

## Scope
- This file applies to `apps/runtime/src-tauri/src/commands/`.
- Follow the closer rules here when editing command modules or adding new command-adjacent submodules.

## Goal
- Keep Tauri command files as entrypoints, not as long-term homes for business logic, SQL, or provider protocols.

## What Belongs In Root Command Files
- Tauri command function signatures
- command input parsing
- response shaping for the frontend contract
- obvious orchestration handoff to `service`, `repo`, `gateway`, or sibling helper modules
- small compatibility glue that must stay visible at the entrypoint

## What Should Move Out
- long SQL blocks
- branching business policy
- normalization and validation flows shared by multiple commands
- external platform protocol handling
- large helper type clusters once they crowd out the command entrypoint

## Preferred Submodules
- `<domain>/service.rs`: business rules and orchestration
- `<domain>/repo.rs`: SQLite reads and writes
- `<domain>/gateway.rs` or `adapter.rs`: external integration logic
- `<domain>/types.rs`: internal DTOs and helper types

Use these as defaults, not dogma. Explain deviations before editing.

## Reference Split
- `employee_agents` is the current reference split for giant Rust command files.
- Before designing a new command split, inspect:
  - `src/commands/employee_agents.rs`
  - `src/commands/employee_agents/types.rs`
  - `src/commands/employee_agents/group_management.rs`
  - `src/commands/employee_agents/group_run_entry.rs`
  - `src/commands/employee_agents/memory_commands.rs`
  - `src/commands/employee_agents/tauri_commands.rs`
- Reuse that pattern of:
  - thin root command file
  - focused child modules by use case or responsibility
  - explicit `service` / `repo` / command-helper boundaries
  - no giant replacement child file without a clear concern boundary

- `openclaw_plugins` is the current reference split for giant plugin or integration command files.
- Before designing a split for a command that mixes runtime state, setup, installation, and external host probing, inspect:
  - `src/commands/openclaw_plugins.rs`
  - `src/commands/openclaw_plugins/tauri_commands.rs`
  - `src/commands/openclaw_plugins/runtime_service.rs`
  - `src/commands/openclaw_plugins/settings_service.rs`
  - `src/commands/openclaw_plugins/setup_service.rs`
  - `src/commands/openclaw_plugins/install_repo.rs`
  - `src/commands/openclaw_plugins/install_service.rs`
  - `src/commands/openclaw_plugins/plugin_host_service.rs`
  - `src/commands/openclaw_plugins/installer_session.rs`
  - `src/commands/openclaw_plugins/tests.rs`
- Reuse that pattern of:
  - thin root command file
  - command wrappers separated from setup, runtime, install, and host probing concerns
  - explicit `service` / `repo` / command-helper boundaries
  - tests moved out of the root file once the root starts crowding out entrypoint logic

## Giant File Rules
- If a command file is above 500 lines, avoid placing net-new business logic directly in that file.
- If a command file is above 800 lines, add or update a split plan in `docs/plans/` before feature work.
- For bug fixes in large files, prefer the smallest safe repair, but do not let the repair become an excuse for adding unrelated logic nearby.

## Layer Responsibilities
- `command`: Tauri entrypoint, input parsing, response shaping, handoff
- `service`: business rules, validation, normalization, orchestration
- `repo`: SQLite queries, writes, transactions, row mapping
- `gateway` or `adapter`: external API calls and protocol translation

## Avoid Over-Splitting
- Do not create a new file for a one-off helper unless it removes meaningful complexity.
- Prefer submodules that represent a use case, persistence concern, or integration boundary.

## Stability Rules
- Preserve existing Tauri command names and response shapes unless the task explicitly changes the contract.
- Keep user-visible side effects in the same order unless the task intentionally changes behavior.

## Next Target Reminder
- `feishu_gateway.rs`, `clawhub.rs`, `chat_runtime_io.rs`, and `openclaw_plugins.rs` are now completed command-governance examples.
- The next preferred Rust command-governance target should come from the current large-file backlog rather than from those completed samples.
- Use `employee_agents` as the baseline for business-heavy command surfaces and `openclaw_plugins` as the baseline for integration-heavy command surfaces.
