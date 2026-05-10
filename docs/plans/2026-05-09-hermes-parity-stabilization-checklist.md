# Hermes Parity Stabilization Checklist

Date: 2026-05-09

Purpose: freeze the current Hermes-aligned self-improving runtime work into reviewable slices, acceptance checks, and explicit no-go boundaries before adding more features.

Primary references:

- `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`
- `docs/plans/2026-05-08-hermes-parity-execution-todo.md`
- `AGENTS.md` section `Next Stage R&D Focus: Self-Improving Profile Runtime`

## Stabilization Rule

- Do not add new self-improving features until this checklist is either complete or intentionally deferred.
- Prefer fixes that make existing Hermes-aligned flows reliable over new UI or runtime surfaces.
- Keep ordinary memory/skill growth agent-managed with source evidence, version history, audit records, and rollback where possible.
- Do not reintroduce a default approval queue, standalone Growth Timeline export, OpenClaw directory mirror, or `.skillpack` mutation.

## Change Slices

### Slice A. Profile Runtime Identity

Status: `[~]`

Scope:

- `agent_profiles`
- profile alias resolution
- `sessions.profile_id`
- profile home path creation
- legacy schema migration and fallback

Acceptance:

- `[x]` New eval profiles bind sessions to `profile_id`.
- `[x]` Alias resolver prefers the canonical profile row with non-empty `profile_home`.
- `[x]` Legacy databases without `sessions.profile_id` can initialize before profile index creation.
- `[ ]` Desktop-created employees always get canonical profile home without relying on profile wizard completion.

Notes:

- `employee_id` remains an alias and UI label, not the new memory identity center.
- Real eval reset now clears eval profiles and profile homes to avoid cross-scenario contamination.

### Slice B. Memory OS

Status: `[~]`

Scope:

- profile `memories/MEMORY.md`
- project memory
- memory versions and rollback
- profile session search and transcript mirror
- growth events for memory changes

Acceptance:

- `[x]` Agent can write profile memory through the `memory` tool.
- `[x]` User preference memory real eval passes on MiniMax-M2.7.
- `[x]` Multi-turn profile memory reuse real eval passes on MiniMax-M2.7.
- `[x]` Memory changes create version history and rollback metadata.
- `[~]` UI exposes memory status; full version rollback UI is still pending.

### Slice C. Skill OS

Status: `[~]`

Scope:

- source-aware Skill OS index
- `skills_list` and `skill_view`
- agent-created skill creation
- patch, archive, restore, delete, versions, rollback, reset
- `.skillpack` immutable boundary

Acceptance:

- `[x]` Runtime no longer injects all installed skill bodies by default.
- `[x]` Skill OS progressive loading real eval passes on MiniMax-M2.7.
- `[x]` Skill self-improvement create/version real eval passes on MiniMax-M2.7 through `builtin-skill-creator`.
- `[x]` `.skillpack` mutation is blocked in tests.
- `[~]` Employee Skill OS UI shows current content and version history; full diff expansion remains pending.

### Slice D. Curator

Status: `[~]`

Scope:

- curator scan/run/restore/history
- stale skill lifecycle
- pinned skill protection
- curator reports in profile home
- growth events for curator actions

Acceptance:

- `[x]` Curator profile scan real eval passes on MiniMax-M2.7.
- `[x]` Skill OS + Curator lifecycle parity real eval passes on MiniMax-M2.7.
- `[x]` Curator does not mutate `.skillpack`.
- `[x]` Curator reports can expand to structured JSON in employee detail.
- `[~]` Curator can restore stale skills; report-level bulk rollback is pending.

### Slice E. Toolset Gateway

Status: `[~]`

Scope:

- manifest-first toolset projection
- core/memory/skills/web/browser/im/desktop/media/mcp groups
- profile default allowed toolsets preference
- skill frontmatter toolset declarations for observability

Acceptance:

- `[x]` Toolset Gateway visibility real eval passes on MiniMax-M2.7.
- `[x]` Toolset policies remain observable configuration only in this slice.
- `[x]` Existing approval/risk behavior is not silently changed by toolset preferences.
- `[ ]` Future enforcement requires a separate roadmap slice before implementation.

### Slice F. Employee Workbench UI

Status: `[~]`

Scope:

- Profile Home artifact status
- Profile artifact export zip
- memory status
- Skill OS panel
- Curator report panel
- Growth Timeline panel

Acceptance:

- `[x]` Employee detail exposes Profile Home artifacts.
- `[x]` Profile artifact export zips the resolved profile home plus `PROFILE_EXPORT.json`.
- `[x]` Standalone Growth Timeline export is removed.
- `[x]` Curator report full JSON expansion is available.
- `[ ]` Manual desktop smoke test must verify the full workbench flow.

### Slice G. Real Agent Eval Harness

Status: `[~]`

Scope:

- scenario config
- provider profiles
- eval DB/profile/workspace cleanup
- scenario outputs and reports

Acceptance:

- `[x]` MiniMax-M2.7 Anthropic-compatible endpoint is configured through `MINIMAX_API_KEY`.
- `[x]` Eval runner clears profile/session/growth/curator/toolset/agent-created skill state between scenarios.
- `[x]` All seven Hermes parity real eval scenarios have passed at least once on MiniMax-M2.7.
- `[~]` Repeatability is good enough for a manual gate; nightly automation is pending.

Latest passing scenarios:

- `toolset_gateway_visibility_2026_05_08`
- `profile_memory_write_growth_2026_05_08`
- `profile_memory_multi_turn_reuse_2026_05_08`
- `skill_os_progressive_loading_2026_05_08`
- `skill_self_improvement_create_version_2026_05_08`
- `curator_profile_scan_2026_05_08`
- `skill_curator_lifecycle_parity_2026_05_09`

## Explicit No-Go List

- `[x]` No default manual approval queue for ordinary self-improving behavior.
- `[x]` No `memory_patch_proposals` or growth review inbox.
- `[x]` No standalone Growth Timeline export button.
- `[x]` No OpenClaw-shaped profile mirror as a next-generation runtime target.
- `[x]` No `.skillpack` patch/archive/delete/reset by ordinary Skill OS or Curator flows.
- `[x]` No toolset preference enforcement until the roadmap explicitly approves enforcement semantics.

## Manual Desktop Smoke Test

Run:

```bash
pnpm app
```

Smoke path:

1. Open the AI employee workbench.
2. Select an employee with a profile home.
3. Verify Profile Home artifacts show instructions, memories, sessions, skills, growth, and curator.
4. Trigger or inspect Profile Memory write/read.
5. Create an agent skill and verify it appears in Skill OS with version history.
6. Run curator scan/history and expand the structured report JSON.
7. Export Profile artifact zip and verify `PROFILE_EXPORT.json` plus profile files are present.
8. Confirm `.skillpack` entries remain read-only if any are installed.

Pass criteria:

- No blank panes.
- No runtime command errors in normal workbench navigation.
- Profile artifact export succeeds.
- Memory write uses `memory` tool in real agent flow.
- Skill creation uses Skill OS instead of raw workspace file writes.

## Verification Commands

Already used for this checkpoint:

```bash
pnpm --dir apps/runtime exec tsc --noEmit
pnpm --dir apps/runtime test -- src/components/employees/__tests__
pnpm test:builtin-skills
pnpm test:rust-fast
cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml --lib
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_agent_eval_scenarios --test test_agent_profile_docs --test test_profile_memory_runtime --test test_curator_tool --test test_skill_os --test test_toolsets_tool --test test_employee_growth -- --nocapture
git diff --check
```

Known caveat:

- Direct `cargo test --lib ...` on this Windows setup can compile but fail at binary startup with `STATUS_ENTRYPOINT_NOT_FOUND`. Prefer integration tests and `cargo check --lib` until that environment issue is fixed.

## Next Stabilization Tasks

1. Run the manual desktop smoke test above.
2. Split or document oversized new runtime/UI files flagged by hygiene review.
3. Update roadmap markers only after manual smoke test confirms the employee workbench path.
4. Prepare logical commit batches by slice A-G.

## Repo Hygiene Review - 2026-05-09

Command:

```bash
pnpm review:repo-hygiene
```

Result:

- Initial findings: 138.
- `stale-doc-or-skill-reference`: 1, fixed by adding the existing frontend file-growth report command to `AGENTS.md`.
- `oversized-file`: 137.
- After the docs fix, rerunning `pnpm review:repo-hygiene` reports 137 findings, all `oversized-file`.

Classification:

- Safe docs fix: `AGENTS.md` missed the existing `pnpm report:frontend-large-files` command reference. Action: add the command reference only.
- Current Hermes-parity split candidates: `apps/runtime/src-tauri/src/agent/evals/runner.rs`, `apps/runtime/src-tauri/src/agent/runtime/runtime_io/profile_session_index.rs`, `apps/runtime/src-tauri/src/agent/runtime/runtime_io/skill_os_index.rs`, `apps/runtime/src-tauri/src/agent/tools/memory_tool.rs`, `apps/runtime/src-tauri/src/commands/agent_profile.rs`, `apps/runtime/src-tauri/src/commands/skills.rs`, `apps/runtime/src/components/employees/EmployeeHubView.tsx`, `apps/runtime/src/components/employees/hooks/useEmployeeHubTools.ts`, and `apps/runtime/src/components/employees/tools/EmployeeSkillOsSection.tsx`.
- Test growth candidates: `apps/runtime/src-tauri/tests/test_memory_tool.rs`, `apps/runtime/src-tauri/tests/test_profile_memory_runtime.rs`, and `apps/runtime/src-tauri/tests/test_skill_os.rs`; keep coverage intact before considering helper extraction.
- Pre-existing large surfaces: runtime adapters, session journal, chat/session commands, OpenClaw plugin code, IM routing, and broad UI shells. These are deferred governance items, not blockers for Hermes parity stabilization.

Smallest safe cleanup batch:

- No deletion.
- No behavior refactor.
- Keep this review as the baseline for later split planning.
