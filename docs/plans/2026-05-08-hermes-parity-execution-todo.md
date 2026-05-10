# Hermes Parity Execution TODO

Date: 2026-05-08

Source of truth:

- `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`
- `AGENTS.md` section `Next Stage R&D Focus: Self-Improving Profile Runtime`

Execution rule:

- Align function, capability, and user experience with Hermes Agent.
- Do not add a default manual approval queue for ordinary self-improving behavior.
- New runtime persistence hangs from `profile_id` and `profiles/<profile_id>/...`.
- Legacy `employee_id + skill_id` paths are no longer runtime memory fallback or employee UI surfaces. Treat them only as historical data that can be ignored or migrated by an explicit one-off tool.
- `.skillpack` remains immutable unless a later reviewed design explicitly changes that boundary.

## P0. Required Foundation

- [x] P0-1. Profile Session Search filters
  - Add workspace filtering.
  - Add time range filtering.
  - Add skill id/source filtering where metadata exists.
  - Keep `document_kind` and `matched_run_id` in all search results.
  - Acceptance: `memory.search` and `search_profile_sessions` can scope recall to the current profile plus workspace/time/skill filters.

- [x] P0-2. Memory version history and rollback
  - Add version snapshots for profile and project memory changes.
  - Record source action, session/run evidence when available, changed scope, and timestamp.
  - Add rollback action for a selected version.
  - Acceptance: every `memory.add`, `memory.replace`, and confirmed `memory.remove` can be restored to a previous version.

- [x] P0-3. Profile session transcript mirror
  - Write readable session transcript artifacts under `profiles/<profile_id>/sessions/<session_id>/`.
  - Include DB messages, run ids, assistant final responses, tool summaries, and compaction boundaries.
  - Acceptance: profile home contains enough evidence to understand and re-index a historical task without relying only on scattered legacy DB/journal files.

- [x] P0-4. Skill OS read-only index
  - Build source-aware skill index for preset, local, `.skillpack`, ClawHub, and industry provenance.
  - Preserve `.skillpack` immutable boundary.
  - Acceptance: agent can list/search skill summaries without loading all full skill bodies into prompt.

- [x] P0-5. Progressive skill loading
  - Route `skills_list` to the Skill OS index.
  - Route `skill_view` to load one selected skill body/details.
  - Acceptance: task execution can first inspect summaries, then load only needed skills.
  - Current: agent tool `skills` exposes `skills_list` and `skill_view`; default turn preparation now injects a summary-only `<available_skills>` block and no longer syncs/projects every installed skill into the workspace. The explicit skill command compatibility path may still project skills for execution.

## P1. Self-Improving Loop

- [~] P1-1. Preset skill versioning
  - Treat builtins as preset seeds.
  - Support patch, archive/delete, reset, version history, and rollback.
  - Acceptance: preset skills can evolve without mutating `.skillpack` and without losing reset capability.
  - Current: agent `skills` tool supports `skill_create`, `skill_patch`, `skill_archive(confirm=true)`, `skill_restore`, `skill_delete(confirm=true)`, `skill_versions`, `skill_view_version`, `skill_rollback(confirm=true)`, and `skill_reset(confirm=true)` for mutable directory-backed local/preset/agent_created skills. Snapshots are stored in `skill_versions`; lifecycle state is stored in `skill_lifecycle`; `.skillpack` mutation is blocked. Employee UI can inspect source boundary, lifecycle capabilities, `SKILL.md`, toolsets, and version history, and can run confirmed patch/reset/rollback/archive/delete plus restore for archived skills through Skill OS commands with profile growth events.

- [x] P1-2. Hermes-aligned curator
  - Add curator scans for stale memory, duplicate memory, reusable skill candidates, and low-value debris.
  - Default behavior is agent-managed with audit/history; confirmations only for dangerous operations.
  - Acceptance: curator can propose or perform safe cleanup/growth paths with source evidence and rollback where possible.
  - Current: agent `curator` tool supports `scan`, `run`, `restore`, and `history`. `scan` is dry-run and reports duplicate memory, reusable skill candidates, low-value memory debris, pinned-skill skips, stale mutable skill candidates, and active draft improvement candidates; `run` can mark unpinned, unused mutable skill drafts as `stale` while leaving pinned skills, `.skillpack`, and actively used draft skills untouched; `restore` can return stale skills to `active`. Reports are written to `curator_runs`, profile report JSON under `profiles/<profile_id>/curator/reports/`, and `growth_events.event_type='curator_scan'/'curator_restore'`. `history` keeps the raw report and also projects `mode`, `changed_targets`, `restore_candidates`, and `has_state_changes` for agent/UI consumption. `skill_lifecycle` includes real use telemetry from implicit inline/fork/direct-dispatch skill routes and explicit skill commands, so curator stale decisions distinguish unused drafts from skills that should be improved with `skills.skill_patch`. The product default is now Hermes-style automatic background maintenance: `curator_scheduler_state` is enabled by default, waits for runtime idle time, runs due profile curations on an interval, resolves blank legacy `agent_profiles.profile_home` values to the canonical runtime `profiles/<profile_id>` home, and exposes status through `get_curator_scheduler_status`. `list_employee_curator_runs` and the employee detail Curator section show recent reports, changed targets, stale skill restore actions, and automatic maintenance state.

- [~] P1-3. Growth records
  - Record learned memory, skill changes, curator runs, rollback events, and source session/run evidence.
  - Acceptance: each AI employee profile has a readable growth timeline.
  - Current: `growth_events` table is present; `memory.add`, `memory.replace`, confirmed `memory.remove`, `memory.rollback`, `source=user-correction` memory writes, `skills.skill_create`, `skills.skill_patch`, `skills.skill_archive`, `skills.skill_restore`, `skills.skill_delete`, `skills.skill_rollback`, `skills.skill_reset`, and curator scan/run/restore paths write profile/session/version/diff/report or lifecycle evidence when the runtime can identify a profile. `list_employee_growth_events` and the employee detail Growth Timeline show recent profile events with skill, memory, curator, and user-correction labels.

- [~] P1-4. Toolset Gateway projection
  - Add manifest-first toolset projection for core, memory, skills, web, browser, IM, desktop, media, and MCP.
  - Do not change approval behavior in the first slice.
  - Acceptance: profile/skill/runtime can inspect available toolsets and risk metadata.
  - Current: agent `toolsets` tool supports `list`, `view`, `profile_policy`, and `set_profile_policy`, projecting the runtime registry into core/memory/skills/web/browser/im/desktop/media/mcp groups with manifest risk fields and storing the current profile's default `allowed_toolsets` preference. Skill frontmatter can declare `requires_toolsets`, `optional_toolsets`, and `denied_toolsets`, surfaced through `skill_view`. These policies are observable configuration only in this slice and do not change approval or effective tool policy.

## P2. Productization And Verification

- [~] P2-1. Employee UI alignment
  - Show profile home, memory, sessions, skills, and growth in the employee detail workbench.
  - Do not add a marketing page or approval-center default workflow.
  - Acceptance: user can inspect what the employee remembers, which sessions it learned from, and which skills changed.
  - Current: employee detail workbench shows Profile Home artifact status, Profile Memory status, automatic Curator status, Curator reports, Growth Timeline, and a Skill OS area for the selected employee's bound skills. The Skill OS area reads `list_skill_os_index`, `get_skill_os_view`, and `list_skill_os_versions`, showing source boundary (`preset`/`agent_created`/`local` vs `.skillpack` read-only), lifecycle state, pinned state, real view/use/patch counts, toolset declarations, current `SKILL.md` content, and recent version history. Confirmed patch/reset/rollback/archive/delete controls are available for mutable skills, archived skills can be restored, and all of these actions write profile growth events. Curator reports can expand to full structured report JSON for audit, while manual Scan/Run remains an operator override for the automatic background curator. Standalone Growth Timeline export is intentionally not exposed; employee detail now offers Hermes-aligned Profile artifact export as a zip of the resolved profile home plus `PROFILE_EXPORT.json`. Session evidence drill-down is intentionally not part of the Hermes parity path.

- [~] P2-2. Hermes parity evals
  - Add real eval scenarios for memory write, session recall, skill selection, progressive loading, skill improvement, curator, and multi-turn reuse.
  - Acceptance: core self-improving flows pass repeatably.
  - Current: real-agent scenario contracts now cover profile memory write/growth, profile memory multi-turn reuse, Skill OS progressive loading, skill self-improvement create/version flow, Skill OS + Curator lifecycle parity, curator profile scan, and Toolset Gateway visibility. The headless eval runner supports scenario-level `employee_alias`, `profile_id`, `profile_display_name`, and multi-turn `user_turns`, so these runs can bind to a profile home instead of anonymous chat. Running the real scenarios still requires local `config.local.yaml` provider credentials.

- [x] P2-3. Legacy model cleanup
  - Stop adding new writes to legacy memory buckets.
  - Remove legacy-read fallback from Profile Memory runtime.
  - Acceptance: new self-improving behavior no longer depends on `employee_id + skill_id` storage.
  - Current: normal chat runtime writes and reads profile-bound memory from `profiles/<profile_id>/memories`; missing-profile sessions fall back to `profiles/_default/memories`, not legacy employee/skill buckets. Employee detail no longer shows legacy bucket stats/export/clear controls, and Tauri no longer exposes the old employee memory stats/export/clear commands.

## Current Execution Order

1. P0-1 Profile Session Search filters.
2. P0-2 Memory version history and rollback.
3. P0-3 Profile session transcript mirror.
4. P0-4/P0-5 Skill OS index and progressive loading.
5. P1/P2 slices once the P0 foundation is stable.

## Verification Matrix

- Rust runtime changes: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_profile_memory_runtime -- --nocapture`
- Memory tool changes: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_memory_tool -- --nocapture`
- Rust compile: `cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml --lib`
- Rust lib compile tests: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib --no-run`
- Fast repo regression: `pnpm test:rust-fast`
- Whitespace check: `git diff --check`
