# Skill OS Source Boundary Map

Date: 2026-05-06

Roadmap phase: Phase 0 baseline governance and compatibility boundary.

This is a read-only investigation note for the future Skill OS and preset skill work. It does not mark any roadmap acceptance item complete because it only maps the current boundary; it does not add code-level `.skillpack` protection tests or migration behavior.

## Scope

Reviewed surfaces:

- Skill core parsing and embedded builtin assets: `packages/runtime-skill-core/*`
- `.skillpack` format, encryption, pack, unpack: `packages/skillpack-rs/*`
- Runtime builtin seeding and installed skill schema: `apps/runtime/src-tauri/src/builtin_skills.rs`, `apps/runtime/src-tauri/src/db/seed.rs`, `apps/runtime/src-tauri/src/db/schema.rs`, `apps/runtime/src-tauri/src/db/migrations.rs`
- Runtime skill loading and workspace projection: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`, `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`, `apps/runtime/src-tauri/src/agent/runtime/kernel/*`
- Skill commands and ClawHub/GitHub import: `apps/runtime/src-tauri/src/commands/skills*`, `apps/runtime/src-tauri/src/commands/clawhub/*`, `apps/runtime/src-tauri/src/commands/packaging.rs`
- Expert skills UI and packaging UI: `apps/runtime/src/components/experts/*`, `apps/runtime/src/components/packaging/*`, `apps/runtime/src/components/InstallDialog.tsx`
- Builtin skill asset tests and embedded assets: `packages/builtin-skill-checks/*`, `apps/runtime/src-tauri/builtin-skills/*`

Out of scope for this phase:

- No runtime code changes.
- No `.skillpack` format changes.
- No builtin skill content changes.
- No implementation of `skill_manage`.

## Current Skill Source Model

The canonical runtime inventory is the SQLite `installed_skills` table. The current table stores:

- `id`
- `manifest`
- `installed_at`
- `last_used_at`
- `username`
- `pack_path`
- `source_type`

`source_type` is added by legacy migration with default `encrypted`; current schema creation still creates `installed_skills` without `source_type`, then migrations add it. Most readers use `COALESCE(source_type, 'encrypted')`, so a missing or unknown source defaults into the encrypted `.skillpack` path.

Current effective source types:

| Source | Current marker | Canonical content | Install path | Notes |
| --- | --- | --- | --- | --- |
| Encrypted skillpack | `source_type='encrypted'` or missing source | `.skillpack` zip at `pack_path`, username in DB | `install_skill` -> `local_skill_service::install_skill` | Manifest is stored in DB; encrypted files are decrypted by `verify_and_unpack` when needed. |
| Local directory | `source_type='local'` | Directory at `pack_path` containing `SKILL.md` / `skill.md` | `import_local_skill`, created expert skills, GitHub repo import, industry bundle import | ClawHub/GitHub/industry skills become local directory-backed skills after extraction/import. |
| Vendored builtin | `source_type='vendored'` | Runtime app data vendor directory at `pack_path`, copied from embedded builtin assets | `sync_builtin_skills_with_root` | This is the current builtin representation for new/default installs. |
| Legacy builtin | `source_type='builtin'` | Either embedded runtime assets or a legacy directory | legacy rows only | `load_installed_skill_source_with_pool` self-heals rows with existing `pack_path` to `vendored`; empty/missing `pack_path` keeps legacy builtin fallback. |
| ClawHub | DB source is usually `local`; identity is `id='clawhub-*'`, `version='clawhub'`, tag `clawhub` | Extracted downloaded zip directory | `install_clawhub_skill` / `update_clawhub_skill` | ClawHub is not a first-class `source_type` today. UI detects it by id prefix. |
| Industry bundle | DB source is `local`; identity is tags `pack:<id>`, `pack-version:<version>`, optional `industry:<tag>` | Extracted industry bundle directory | `install_industry_bundle_to_pool` | Also not a first-class source type. |

Important current normalization points:

- `apps/runtime/src-tauri/src/db/seed.rs` writes embedded builtin rows as `source_type='vendored'`, `username=''`, and a directory `pack_path`.
- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/runtime_inputs.rs` self-heals legacy `builtin` rows with a valid `pack_path` to `vendored`.
- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs` treats `local` and `vendored` as directory-backed, keeps `builtin` as a legacy embedded fallback, and treats all other source types as encrypted `.skillpack`.
- `apps/runtime/src-tauri/src/agent/tools/employee_manage/actions.rs` normalizes `builtin` to `vendored` in tool output.
- `apps/runtime/src/app-shell-utils.ts` collapses `vendored`/`builtin` to `preinstalled`, `id.startsWith("local-")` to `local-created`, and everything else to `external`.

## Skill Loading Flow

Current local chat turn flow:

1. `prepare_local_turn` loads the selected session `skill_id` from `sessions`.
2. `load_installed_skill_source_with_pool` reads `manifest`, `username`, `pack_path`, and `source_type`.
3. `load_skill_prompt` loads the selected skill's full prompt:
   - Directory-backed `local`/`vendored`: read `SKILL.md` or `skill.md` from `pack_path`.
   - Legacy `builtin`: read embedded markdown.
   - Other source types: call `skillpack_rs::verify_and_unpack(pack_path, username)` and extract `SKILL.md`; if decrypt fails, fall back to manifest description.
4. `load_workspace_skill_runtime_entries_with_pool` reads every installed skill and resolves each into a `WorkspaceSkillRuntimeEntry`.
5. `prepare_runtime_tools` calls `build_workspace_skill_context`.
6. `build_workspace_skill_context` calls `prepare_workspace_skills_prompt` unless the prompt is suppressed.
7. `prepare_workspace_skills_prompt` always calls `sync_workspace_skills_to_directory`, then creates an `<available_skills>` block with name, invoke name, description, and location.
8. `sync_workspace_skills_to_directory` deletes and recreates `<work_dir>/skills`, then:
   - Copies local/vendored directories into it.
   - Writes decrypted `.skillpack` file maps into it.
   - Writes `.workclaw-skill-id` markers.
9. `SkillInvokeTool` is registered with search roots including workspace skill directories and reads the projected `SKILL.md` on demand.

Progressive disclosure is partially present but not yet a durable Skill OS boundary:

- The system prompt does not embed all full skill bodies by default; it embeds summaries plus file locations.
- The runtime still resolves every installed skill on every turn and projects every skill's full file tree to workspace disk.
- The selected session skill is fully loaded into the system prompt up front.
- Explicit skill mentions can swap the effective system prompt to another installed skill's full prompt before model execution.
- `SkillInvokeTool` provides full on-demand reads, but it is path/search-root based rather than source/index based.

## Skillpack Read-Only Boundary

Current `.skillpack` format boundary lives in `packages/skillpack-rs`:

- `pack.rs` creates a zip with plaintext `manifest.json`.
- All skill files are encrypted into `encrypted/<relative-path>.enc`.
- `crypto.rs` derives the key from `username + manifest.id + manifest.name` and uses AES-256-GCM.
- `manifest.encrypted_verify` is an encrypted verification token checked before unpacking.
- `unpack.rs::verify_and_unpack` opens the zip, reads plaintext manifest, verifies username/key, decrypts encrypted entries, and returns an in-memory `HashMap<String, Vec<u8>>`.

Runtime read/install boundary:

- `install_skill` verifies and unpacks once, then writes only DB metadata: manifest, username, pack path, source type `encrypted`.
- `load_skill_prompt` unpacks encrypted packs to read `SKILL.md` content or fall back to manifest description.
- `resolve_workspace_skill_runtime_entry` unpacks encrypted packs into an in-memory file tree for workspace projection.
- `sync_workspace_skills_to_directory` writes decrypted file tree copies into `<work_dir>/skills/<projected-dir>`.
- `SkillInvokeTool` reads the projected plaintext `SKILL.md` from search roots, not from the `.skillpack` file directly.

Current protection properties:

- There is no runtime code path that modifies an original `.skillpack` file after installation.
- `refresh_local_skill` rejects source types other than `local` and `vendored`, so it does not refresh encrypted pack metadata.
- ClawHub update checks require `source_type='local'`, so encrypted packs are not updated by ClawHub paths.

Current gaps:

- The read-only boundary is convention-based, not modeled as a first-class immutable source policy.
- Any non-`local`/`vendored`/legacy-`builtin` source is treated as encrypted by default. A future source type could accidentally enter `.skillpack` decrypt logic unless source types become an enum.
- Decrypted encrypted skill content is written to workspace disk as a projected copy. That copy is not the canonical `.skillpack`, but an agent or future skill patch tool could mistakenly treat it as editable canonical content.
- `delete_skill` deletes any installed skill DB row with no backend source guard. The current UI hides deletion for preinstalled skills, but backend protection is not source-aware.
- `update_skill_dir_tags` writes arbitrary directory `SKILL.md` tags and is not connected to installed skill source policy. It is packaging-oriented, but a future caller could point it at a preset/local skill directory without lifecycle audit.

## Builtin-To-Preset Migration Impact

Renaming/reframing builtin/vendored to preset touches both persistence and runtime behavior.

Backend modules affected:

- `packages/runtime-skill-core/src/builtin_skills.rs`
  - Embedded builtin asset registry and constants.
  - `is_multistep_builtin_skill` and `apply_builtin_todowrite_governance` currently match `source_type` `builtin | vendored`.
- `apps/runtime/src-tauri/src/builtin_skills.rs`
  - Runtime re-export layer for embedded assets.
- `apps/runtime/src-tauri/src/db/seed.rs`
  - Seeds embedded builtin assets into app data vendor directories.
  - Currently overwrites vendor directories from embedded assets and upserts source type `vendored`.
- `apps/runtime/src-tauri/src/db/schema.rs` and `migrations.rs`
  - `installed_skills` lacks lifecycle/version/source metadata needed for preset reset/archive/patch.
- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/runtime_inputs.rs`
  - Legacy `builtin` self-heal target is `vendored`.
- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`
  - Directory-backed source check is `local | vendored`; legacy embedded path is `builtin`.
  - Builtin governance injection depends on source labels.
- `apps/runtime/src-tauri/src/commands/skills/local_skill_service.rs`
  - `refresh_local_skill` allows `vendored`, which means preinstalled assets can refresh manifest from disk.
- `apps/runtime/src-tauri/src/commands/skills/runtime_status_service.rs`
  - Office builtin dependency checks use builtin IDs and directory-backed roots.
- `apps/runtime/src-tauri/src/agent/tools/employee_manage/actions.rs`
  - Tool output normalizes `builtin` to `vendored`.
- `packages/builtin-skill-checks/tests/builtin_skill_assets.rs`
  - Tests lock the embedded asset and re-export expectations.

Frontend modules affected:

- `apps/runtime/src/app-shell-utils.ts`
  - Maps `vendored`/`builtin` to `preinstalled`.
- `apps/runtime/src/components/experts/ExpertsView.tsx`
  - Counts and labels preinstalled skills.
  - Allows refresh for preinstalled but hides deletion.
  - Does not model reset/archive/patch.
- `apps/runtime/src/components/experts/__tests__/ExpertsView.test.tsx`
  - Expects source-specific actions for `vendored`.

Target preset migration implication:

- `vendored` should become a compatibility alias for `preset`, not the product term.
- Preset skills need a separate immutable seed version and mutable user/profile overlay.
- Seed sync must stop blindly overwriting user-modified preset directories. Reset should be explicit and audited.
- `builtin` can remain a legacy read alias until old DBs migrate.

## Progressive Disclosure Insertion Points

Current all-skill work happens in these places:

- `load_workspace_skill_runtime_entries_with_pool`
  - Reads and resolves every installed skill each turn.
  - For encrypted packs, this can decrypt every installed `.skillpack`.
- `resolve_workspace_skill_runtime_entry`
  - Converts the installed source into either `LocalDir` or decrypted `FileTree`.
  - Parses full `SKILL.md` to build config, invocation, metadata, command dispatch.
- `sync_workspace_skills_to_directory`
  - Writes every skill's full file tree into `<work_dir>/skills`.
- `build_workspace_skills_prompt`
  - Injects every eligible skill summary and location into system prompt.
- `SkillInvokeTool`
  - Reads full `SKILL.md` from projected directories and returns full instructions.
- `resolve_explicit_prompt_following_skill`
  - Matches user messages against every loaded entry and can select a full skill prompt before execution.
- Skill routing index and adjudicator paths
  - Use `WorkspaceSkillRuntimeEntry` collections for route decisions.

Recommended insertion model:

- Add a Skill OS index layer before `WorkspaceSkillRuntimeEntry` that can list source/lifecycle/summary/trigger/toolset without unpacking or copying full content.
- Replace "resolve every skill into full runtime entry" with "resolve candidate skill entries by profile, query, route decision, or explicit `skill_view` request".
- Keep `SkillInvokeTool` as compatibility, but introduce source-aware `skills_list` and `skill_view` tools backed by the Skill OS index/service.
- Defer workspace projection until a skill is selected or viewed. For encrypted `.skillpack`, projection should be read-only, ephemeral, and marked as derived, not canonical.

## Skill OS Target Boundary

Suggested conceptual boundaries:

1. `skill_sources`
   - Source identity and immutability policy.
   - Values should converge to `preset`, `local`, `agent_created`, `skillpack`.
   - Compatibility aliases: `vendored -> preset`, `builtin -> preset`, `encrypted -> skillpack`, `clawhub local -> local` plus optional `origin=clawhub`.

2. `profile_skills`
   - Which skills are visible to a profile.
   - Lifecycle: active, archived, deleted/hidden, pinned.
   - Overlay location for mutable content.

3. `skill_usage`
   - `view_count`, `use_count`, `last_viewed_at`, `last_used_at`, curator score, pinned state.

4. Preset seed metadata
   - `source_version` / WorkClaw build version that provided the seed.
   - `seed_hash` and maybe per-file hashes.
   - `seed_path` or embedded asset id.
   - `current_overlay_path` for user-modified content.
   - `last_reset_at`, `last_patched_at`, `modified_from_seed` flag.
   - Upgrade policy: keep user overlay, store new seed, show diff; never silently overwrite modified preset.

5. Patch and rollback metadata
   - `change_id`, source session/run/tool evidence, diff, version state, risk confirmation state when applicable, applied diff, rollback path.
   - Applies to preset/local/agent_created.
   - Must reject skillpack as target unless future reviewed design changes it.

6. Derived workspace projection metadata
   - Projection path, source skill id, source revision/hash, read-only/derived flag, expiry/session id.
   - Prevents agents and future patch tools from confusing `<work_dir>/skills` copies with canonical skill content.

Tool boundaries:

- `skills_list`
  - Should live at Skill OS index/read boundary.
  - Read-only.
  - Returns profile-visible summaries, source type, lifecycle, trigger/description, and whether full view/manage is allowed.
  - Must not unpack full `.skillpack` file trees by default.

- `skill_view`
  - Should live at Skill OS content-read boundary.
  - Read-only.
  - For `skillpack`, it may decrypt and return content only for authorized installed packs, but must label output as read-only and derived.
  - Should record usage telemetry if allowed.

- `skill_manage`
  - Should live at mutable lifecycle/write boundary.
  - Must refuse `skillpack`.
  - Should only operate on preset overlays, local, and agent_created with diff/version/audit metadata; high-risk changes require confirmation.
  - Reset only applies to preset overlays and restores from seed metadata.

## Risks And Open Questions

Risks:

- Encrypted skill projection writes decrypted copies to workspace disk. That is not canonical mutation, but it weakens the practical read-only boundary and creates a place future patch tools could edit by mistake.
- `delete_skill` has no backend source guard. UI currently blocks preinstalled deletion, but backend commands can remove any source row.
- `refresh_local_skill` allows `vendored`, which is acceptable for current preinstalled manifest refresh, but conflicts with future preset semantics if preset seeds become immutable plus overlay.
- `sync_builtin_skill_directory` removes and recreates vendored builtin directories from embedded assets. Future preset migration must avoid overwriting user-modified preset overlays.
- `source_type` is free text with permissive fallback to encrypted. A typo or new source can enter wrong decrypt/read behavior.
- ClawHub and industry bundle provenance is encoded indirectly through IDs/tags instead of explicit source metadata.
- `<work_dir>/skills` is fully recreated each turn. If a user intentionally keeps files there, runtime projection can remove them.
- Current UI source buckets do not distinguish encrypted skillpack from other external sources.

Open questions:

- Should ClawHub become `source_type='local'` with `origin='clawhub'`, or a first-class source?
- Should installed encrypted skillpacks expose usage telemetry to curator while remaining immutable?
- Where should decrypted `skillpack` projected files live: workspace, profile temp, session temp, or in-memory only?
- Should preset reset restore the WorkClaw version currently installed, or allow choosing an older seed version?
- Should local skills be patched in place, or should Skill OS create managed overlays to preserve user-owned directories?

## Recommended First Implementation Slice

First slice: create a source-policy and read-only index layer before adding mutation.

Recommended sequence:

1. Introduce a Rust enum/source policy helper for installed skill sources.
   - Normalize aliases: `builtin`/`vendored` -> preset-compatible preinstalled; `encrypted` -> skillpack.
   - Add explicit capability flags: `can_view`, `can_patch`, `can_archive`, `can_delete`, `can_reset`, `requires_unpack_for_view`, `is_immutable_source`.

2. Add a read-only Skill OS index service.
   - Reads `installed_skills`.
   - Returns summaries and source policy.
   - Does not project or unpack every skill by default.
   - Keeps ClawHub/industry provenance as metadata, even if DB migration waits.

3. Add tests that enforce `.skillpack` immutability at the policy boundary.
   - `skillpack` cannot patch, archive, delete through Skill OS, reset, or curator mutate.
   - Legacy `encrypted` rows normalize to `skillpack`.

4. Wire future `skills_list` to this read-only index first.
   - This advances Phase 3 without implementing `skill_manage`.
   - It provides the target boundary for progressive disclosure.

5. Only after the read boundary is stable, design preset overlay/reset metadata.
   - Do not rename `vendored` storage in one step.
   - Keep compatibility aliases until legacy DB and UI tests are updated.

This slice gives WorkClaw a source-aware Skill OS boundary without touching `.skillpack` format, builtin skill files, or runtime mutation behavior.

## 2026-05-08 Execution Update

- Added Skill OS read-only index via `list_skill_os_index_with_pool`.
  - Reads `installed_skills` manifest rows.
  - Handles legacy schemas without `source_type` by treating them as `encrypted`/`skillpack`.
  - Returns manifest summary, tags, source policy, and capability flags.
- Added agent tool `skills`.
  - `skills_list` returns the Skill OS index summary.
  - `skill_view` loads one requested skill only.
  - local/preset sources read the requested `SKILL.md`; `.skillpack` rows return manifest/read-only/derived metadata and do not unpack or mutate content.
- `.skillpack` is exposed as immutable in this boundary: `immutable_content=true`, `read_only=true`, `can_agent_delete=false`.
- Remaining progressive-loading work: default turn preparation still syncs/projects installed skills; next slice should switch the default prompt/projection path to Skill OS summary-first.
