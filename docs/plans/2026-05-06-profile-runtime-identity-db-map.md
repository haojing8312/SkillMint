# Profile Runtime Identity / DB Dependency Map

Date: 2026-05-06

## Scope

This is Phase 0 read-only research for the Self-Improving Profile Runtime roadmap. It maps the current `employee_id`, `skill_id`, session, group run, IM binding, employee group, and real eval database dependencies before introducing `profile_id -> AI employee runtime home`.

Inspected areas:

- `apps/runtime/src-tauri/src/db/*`
- `apps/runtime/src-tauri/src/commands/*`
- `apps/runtime/src-tauri/src/im/*`
- `apps/runtime/src-tauri/src/agent/*`
- `apps/runtime/src-tauri/src/employee_runtime_adapter/*`
- `packages/*`
- `agent-evals/*`

No runtime code or schema was changed. This document intentionally does not implement Phase 1.

## Current DB Dependency Map

### Employee Identity Columns And Keys

| Table or storage | Field / key | Current meaning | Main readers / writers | Migration note |
|---|---|---|---|---|
| `agent_employees` | `id` | Stable DB row id, UUID-like for current employees | `profile_repo`, `agent_profile`, employee manage tool, IM session binding | Best candidate for one-time profile mapping seed if `profile_id` is generated separately. |
| `agent_employees` | `employee_id` | Human/routing code. Legacy fallback from `role_id` in migrations. | `profile_repo`, `profile_service`, routing, session creation, group runs, collaboration prompt | Keep as routing/display alias. Do not use as new profile runtime root. |
| `agent_employees` | `role_id` | Role alias; currently forced to `employee_id` on upsert. | Routing, prompt assembly, Feishu relay, group lookup fallback | Keep as alias. Existing code frequently matches `employee_id OR role_id OR id`. |
| `agent_employees` | `openclaw_agent_id` | Preferred IM/OpenClaw route agent id when set. | `resolve_agent_id`, Feishu binding, route session keys | Keep as external route alias. It should map to profile, not replace it. |
| `sessions` | `employee_id` | Session-level employee routing code, not DB row id. Empty for general sessions. | session list/search, `prepare_local_turn`, `load_session_execution_context`, memory bucket resolution | Must migrate or derive `profile_id` for employee sessions. Keep `employee_id` for UI/API compatibility. |
| `agent_employee_skills` | `employee_id` | Misleading name: stores `agent_employees.id`, not employee code. | `list_skill_ids_for_employee`, `replace_employee_skill_bindings`, delete employee | High rename/migration risk. In Phase 1, model as profile-skill membership and preserve legacy alias. |
| `employee_groups` | `coordinator_employee_id`, `entry_employee_id` | Employee code aliases used for team topology. | group create/clone/list/start, team entry routing | Keep as display/template aliases initially; runtime execution should resolve to profile. |
| `employee_groups` | `member_employee_ids_json`, `config_json.roles[].employee_id` | JSON arrays/objects of employee code aliases. | group management, team topology adapter | Needs JSON migration or lookup layer. This is not covered by column-only migrations. |
| `employee_group_rules` | `from_employee_id`, `to_employee_id` | Team delegation/review/report edges by employee code alias. | team topology, group run planning | Must resolve to profile at execution time; may remain as template alias for authoring. |
| `group_runs` | `main_employee_id`, `waiting_for_employee_id` | Active run coordinator/reviewer/wait target by employee code alias. | group run start, pause/review/retry/progress/snapshot | Must migrate for active runtime identity. Keep alias fields for display and old snapshots. |
| `group_run_steps` | `assignee_employee_id`, `dispatch_source_employee_id` | Step assignee/source by employee code alias. | step execution, retry/reassign, progress, events | Must bind to profile for execution; alias can remain for display/history. |
| `seeded_team_templates` | `instance_employee_ids_json` | Seeded employee code aliases. | team template seeding | Template metadata can stay alias-based, but profile mapping is needed when instantiated. |
| `im_thread_employee_bindings` | `employee_id` | Legacy thread-to-employee binding key. | schema only in inspected paths; legacy surface | Legacy-only. Needs fallback if still read by older flows/tests. |
| `im_thread_sessions` | `employee_id` | Misleading name in session binding rows: current code writes `agent_employees.id`. | `session_repo`, migrations, legacy IM fallback | Startup-critical legacy table. New profile bindings need fallback from this shape. |
| `im_conversation_sessions` | `employee_id` | Misleading name: current code writes `agent_employees.id`. | `session_repo`, migrations | Startup-critical authoritative IM binding. Migrate to profile binding or add profile column with fallback. |
| `im_message_links` | `employee_id` | Current code writes employee DB id for inbound links. | IM inbound event link | Audit/history identity should become profile-aware; keep legacy field for old traces. |
| `agent_conversation_bindings` | `agent_id` | Route identity from `employee.agent_id()`: `openclaw_agent_id` -> `employee_id` -> `role_id`. | IM authority binding store, migrations, outbound route lookup | Keep as external route alias. Needs `profile_id` beside it or mapping at lookup time. |
| `im_routing_bindings` | `agent_id` | Route target alias, not necessarily employee DB id. | IM routing commands, Feishu binding repo, startup restore | Keep as connector route alias. Must resolve to profile before session creation. |
| `im_routing_bindings` | `team_id`, `role_ids_json` | Team id plus role aliases for routing. | routing service, startup restore | Team id remains; role aliases need profile resolution. |

Current employee-related indexes / constraints:

- `agent_employees.id` primary key in current schema.
- `idx_agent_employees_employee_id_unique` and `idx_agent_employees_role_id_unique` are created in legacy migrations, not in `schema.rs`.
- `agent_employee_skills PRIMARY KEY (employee_id, skill_id)`.
- `employee_groups` has migration index `idx_employee_groups_coordinator` on `coordinator_employee_id`.
- `im_thread_employee_bindings PRIMARY KEY (thread_id, employee_id)`.
- `im_conversation_sessions PRIMARY KEY (conversation_id, employee_id)`.
- `im_thread_sessions PRIMARY KEY (thread_id, employee_id)`.
- `agent_conversation_bindings PRIMARY KEY (conversation_id, agent_id)`.

### Skill Identity Columns And Keys

| Table or storage | Field / key | Current meaning | Main readers / writers | Migration note |
|---|---|---|---|---|
| `installed_skills` | `id` | Installed skill id; primary key. | skill install/list/runtime projection, builtin seed, eval skill selection | Keep as `skill_id`, but future `profile_skills` should scope visibility/lifecycle per profile. |
| `installed_skills` | `source_type`, `pack_path`, `username` | Skill source and unpacking inputs. `builtin` self-heals to `vendored`; encrypted skillpacks use username/id. | runtime inputs, workspace skill projection, seed | `.skillpack` must remain immutable. Do not conflate preset mutation with encrypted pack mutation. |
| `sessions` | `skill_id` | Session's selected/root skill. | session create/list/search, runtime input load, group run session seed, eval runner | Keep for legacy sessions and display. New profile runtime should treat it as active skill selection, not identity root. |
| `agent_employees` | `primary_skill_id` | Default skill for employee-created sessions. | IM session creation, group session creation, employee manage tool | Move to profile default skill selection, preserve column for compatibility. |
| `agent_employee_skills` | `skill_id` | Skills visible/assigned to an employee row id. | employee list/upsert/delete, employee manage tool | Natural Phase 1 bridge to `profile_skills`. |
| Employee memory files | `<runtime>/memory/employees/<employee_bucket>/skills/<skill_id>/...` | Current long-lived memory bucket by employee alias + skill id. | chat memory tool, employee memory commands | Must migrate to profile home; this is the old model the roadmap rejects. |
| Legacy memory files | `<runtime>/memory/<skill_id>/...` | Pre-employee memory bucket for general sessions. | `build_memory_dir_for_session` fallback | Needs legacy read fallback for non-employee sessions and old installs. |
| Workspace skill projection | `.workclaw-skill-id`, projected `skills/<dir>/SKILL.md` | Runtime skill projection into work dir | `workspace_skills.rs` | Keep skill id as skill identity, but attach profile-scoped lifecycle elsewhere. |
| Agent evals | `capability_id -> entry_kind/entry_name -> skill_id` | Eval scenario selects skill by capability mapping. | `agent/evals/runner.rs`, `agent-evals/config/*.yaml` | Evals currently have no employee/profile identity; Phase 1 evals need optional profile fixture. |

Current skill-related indexes / constraints:

- `installed_skills.id` primary key.
- `agent_employee_skills PRIMARY KEY (employee_id, skill_id)`.
- No dedicated index on `sessions.skill_id`.

### Ambiguous Identity Surfaces

The largest migration risk is that `employee_id` is not semantically stable:

- In `sessions.employee_id`, group tables, and team topology it usually means an employee code alias.
- In `agent_employee_skills.employee_id`, `im_thread_sessions.employee_id`, `im_conversation_sessions.employee_id`, and `im_message_links.employee_id`, current code writes `agent_employees.id`.
- In `agent_conversation_bindings.agent_id` and `im_routing_bindings.agent_id`, the identity is a route alias from `openclaw_agent_id`, `employee_id`, or `role_id`.

Phase 1 should avoid reusing `employee_id` as a profile primary key because existing rows already mix code alias, row id, and connector route alias under similar names.

## Startup-Critical Reads

These paths are startup- or first-turn critical and need legacy fallback coverage before depending on new profile columns:

1. Database bootstrap: `db.rs` runs `apply_current_schema`, then `apply_legacy_migrations`, then `seed_runtime_defaults`. Migrations already backfill `sessions.employee_id`, `agent_employees.employee_id`, IM conversation tables, and route tables.
2. Session list and search: `chat_session_io/session_store.rs` reads `sessions.skill_id`, `sessions.employee_id`, `session_mode`, `team_id`, IM source channel from `im_thread_sessions`, and employee display names from `agent_employees`.
3. Local chat send path: `chat.rs::send_message` inserts a user message, then `SessionRuntime::run_send_message` reaches `turn_preparation.rs::prepare_local_turn`, which reads `sessions.skill_id`, `model_id`, `permission_mode`, `work_dir`, and `employee_id`.
4. Runtime context merge: `agent/runtime/repo.rs::load_session_execution_context` reads `sessions.session_mode`, `team_id`, `employee_id`, and `work_dir`; `runtime-chat-app` then uses `employee_id` for collaboration guidance and memory bucket selection.
5. Memory tool setup: `tool_registry_setup.rs` builds memory dir from `app_data_dir`, `skill_id`, and `memory_bucket_employee_id`; group step execution separately builds `<work_dir>/openclaw/<employee_id>/memory/<skill_id>`.
6. IM auto-restore: `im_host/startup_restore.rs` decides WeCom restoration by counting enabled `im_routing_bindings` by channel; Feishu restoration goes through plugin runtime state. This path must not require profile columns just to start.
7. IM inbound session bridge: `im/agent_session_runtime.rs` resolves/creates sessions through `im_conversation_sessions` first, then legacy `im_thread_sessions`, then creates a new `sessions` row seeded from employee primary skill/default work dir.
8. Team entry and group runs: `maybe_handle_team_entry_session_message_with_pool` reads `sessions.session_mode/team_id`; group run start loads `employee_groups`, rules, employees, then writes `group_runs` and `group_run_steps`.
9. Real eval harness: `agent/evals/runner.rs` resets eval state, resolves `capability_id` to a skill id, creates a general session with no employee id, sends a message, then exports session/run artifacts.

## Legacy Compatibility Risks

- Adding `profile_id NOT NULL` to startup-critical tables without defaults would break old databases. Start with nullable `profile_id` plus query fallback or backfill migration.
- `sessions` legacy rows may lack `employee_id`, `session_mode`, `team_id`, or `work_dir`; current migrations add them. Profile-aware reads must preserve the same fallback behavior.
- `agent_employees.employee_id` is added by migration and backfilled from `role_id`; any profile mapping must handle rows where `employee_id == role_id`, `openclaw_agent_id` is empty, or all three aliases collide.
- Current schema relies on migrations for unique indexes on `agent_employees.employee_id` and `role_id`. New profile uniqueness should be explicit in current schema, while preserving migration order.
- IM legacy fallback is already nuanced: `im_conversation_sessions` is authoritative, while `im_thread_sessions` is only used when no authoritative conversation row exists for a thread/employee. Profile migration must keep this lookup order.
- `im_thread_sessions.employee_id` and `im_conversation_sessions.employee_id` store employee DB ids despite the name. A naive code-alias migration would orphan existing IM sessions.
- Group/team JSON fields (`member_employee_ids_json`, `config_json`, seeded template JSON, group run event payloads) cannot be migrated by column rename. They need JSON-aware backfill or runtime alias resolution.
- Memory paths are not database rows. The old bucket shape is split between `<runtime>/memory/<skill_id>`, `<runtime>/memory/employees/<employee>/skills/<skill_id>`, and group execution's `<work_dir>/openclaw/<employee>/memory/<skill_id>`.
- Eval scenarios are capability/skill-centric and create general sessions with no employee/profile. Profile runtime evals will need explicit profile fixture setup to avoid accidentally measuring the legacy general-session path.
- `.skillpack` identity uses `skill_id` in crypto key derivation. Preset/local skill migration must not mutate encrypted pack contents or reinterpret pack ids.

## Proposed Migration Boundary

### Keep `employee_id` As Routing / Display Alias

These surfaces can retain `employee_id` as a compatibility and display alias during and after Phase 1:

- Tauri command payloads and UI fields that users already see as employee code.
- `agent_employees.employee_id`, `role_id`, and `openclaw_agent_id` as aliases attached to a profile.
- `im_routing_bindings.agent_id` and `agent_conversation_bindings.agent_id` as external connector route aliases.
- Team template authoring fields (`coordinator_employee_id`, `entry_employee_id`, `member_employee_ids_json`, rule from/to fields) as human-editable aliases, provided execution resolves them to profile ids.
- Group run event payloads and historical snapshots as immutable legacy audit/display data.
- Collaboration prompt display text where `employee_id` is a readable employee code.

### Migrate To Profile Runtime Identity

These surfaces should become profile-owned for new writes:

- New employee/profile creation: create `agent_profiles` and profile home before or with `agent_employees` compatibility row.
- Session execution context: new employee sessions should carry `profile_id`; `sessions.employee_id` remains alias only.
- Memory root selection: replace `employee_id + skill_id` bucket root with `profiles/<profile_id>/memories` and profile-scoped skill/session indexes.
- `agent_employee_skills`: bridge to `profile_skills`; old rows can be read as compatibility membership.
- IM conversation/session binding: add profile-aware binding or profile column while retaining `agent_id`/legacy employee DB id lookup fallback.
- Group run runtime execution: each step should bind a profile id for assignee/source; aliases can remain for display and old rules.
- Employee profile files: current `default_work_dir/openclaw/<employee_id>/AGENTS.md|SOUL.md|USER.md` should become profile home files or migrated into it.
- Growth, curator, session search, and future self-improving writes must hang from `profile_id`, not `employee_id + skill_id`.

### Suggested Mapping Rule

Recommended Phase 1 rule:

- Generate a stable `profile_id` for every `agent_employees.id`.
- Store aliases on the profile: `employee_db_id`, `employee_id`, `role_id`, `openclaw_agent_id`, and display name.
- For legacy lookup, resolve in this order: `profile_id` if present, `agent_employees.id`, `openclaw_agent_id`, `employee_id`, `role_id`.
- Never make new persistent memory depend on `employee_id + skill_id`; treat those values as inputs to legacy import and routing only.

## Open Questions

- Should `profile_id` be a new UUID, or can `agent_employees.id` be promoted as the profile id? A new UUID is cleaner, but promoting `id` reduces backfill complexity.
- Should `agent_employees` become a compatibility view/table over `agent_profiles`, or remain a separate UI/alias table?
- Should `sessions.profile_id` be nullable with fallback forever, or only during a migration window?
- How should group/team JSON aliases be migrated: eager JSON rewrite, profile reference side table, or lazy resolution at execution time?
- What is the canonical import path from legacy memory buckets into `profiles/<profile_id>/memories` when the same employee/skill appears in multiple roots?
- Should real evals create a synthetic profile for profile-runtime scenarios, while keeping current capability-only evals for backward compatibility?
- How much usage telemetry may curator read from `.skillpack` skills without violating the immutable commercial distribution boundary?

## Phase 0 Acceptance Items Impacted

Roadmap Phase 0 item impact:

- Dependency mapping for `employee_id`/`skill_id` across sessions, IM, group runs, memory paths, employee groups, and evals: addressed by this document.
- Legacy compatibility matrix: partially addressed by the dependency map and compatibility risks; a separate test-oriented matrix is still needed before marking Phase 0 complete.
- `profile_id` to `employee_id` mapping rule: proposed, but needs decision.
- Old memory directory to profile home migration rule: risk and boundary identified, but detailed file migration plan remains open.
- `.skillpack` immutable boundary: confirmed as a required migration boundary, with concrete risk from skillpack crypto and installed skill source handling.

Phase 0 should not be marked complete yet because the roadmap acceptance also requires old and new databases to start with a legacy schema regression test, plus code/test protection for `.skillpack` read-only behavior.

## Recommended First Implementation Slice

Start Phase 1 with the smallest profile identity slice that does not change memory behavior yet:

1. Add `agent_profiles` with `profile_id`, display/persona fields, default work dir, aliases, and timestamps.
2. Add nullable `profile_id` to `sessions` and profile mapping/backfill for existing `agent_employees`.
3. Add a profile resolver service that accepts `profile_id`, `agent_employees.id`, `openclaw_agent_id`, `employee_id`, or `role_id`.
4. Update startup-critical reads to prefer `sessions.profile_id` and fall back to `sessions.employee_id` plus alias resolver.
5. Add legacy-schema regression tests for session list/search, local chat execution context, and IM conversation binding lookup.
6. Only after this lands, move memory path selection from `employee_id + skill_id` to profile home with a read fallback/import path.
