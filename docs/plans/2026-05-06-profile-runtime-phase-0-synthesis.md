# Profile Runtime Phase 0 Synthesis

Date: 2026-05-06

Status: Phase 0 synthesis and implementation boundary. This document consolidates the five parallel Phase 0 research maps and fixes the first architectural decisions before code changes.

Related roadmap: `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`

## 1. Inputs

This synthesis is based on:

- Identity / DB dependency map: `docs/plans/2026-05-06-profile-runtime-identity-db-map.md`
- Memory path / runtime context map: `docs/plans/2026-05-06-profile-runtime-memory-map.md`
- Skill OS source boundary map: `docs/plans/2026-05-06-skill-os-source-boundary-map.md`
- Toolset gateway map: `docs/plans/2026-05-06-toolset-gateway-map.md`
- Self-improving employee UX design: `docs/plans/2026-05-06-self-improving-employee-ux.md`

The research batch completes the read-only dependency mapping portion of Phase 0. Profile Runtime Foundation Slice 1 has now added the first implementation: `agent_profiles`, nullable `sessions.profile_id`, alias resolution, source policy guards, and compile-time regression coverage. After the 2026-05-07 Hermes pivot, compatibility means database/session migration only; it does not mean preserving OpenClaw-shaped runtime directories as a product target. Phase 0 remains in progress only because this Windows environment currently blocks direct Tauri lib test execution with `STATUS_ENTRYPOINT_NOT_FOUND`; the tests compile and must be executed after that environment issue is fixed.

## 2. Executive Decision

WorkClaw should proceed to a small Phase 1 foundation slice, but only after the Phase 0 migration tests and source guards are planned into the implementation batch.

The first code slice should not migrate memory files or rewrite skill loading behavior. It should establish the new identity and source-policy foundations:

1. Add `agent_profiles`.
2. Add nullable `sessions.profile_id`.
3. Add profile alias resolution for legacy employee identities.
4. Add legacy-schema regression tests around sessions, IM bindings, and profile fallback.
5. Add a source policy enum/index for skills with `.skillpack` immutable semantics.
6. Add read-only toolset/skill/profile projections where useful, without changing approval behavior or prompt output.

This is the lowest-risk path because it stops identity drift before Memory OS, Skill OS, Growth Loop, or Curator start writing durable state.

## 3. Fixed Architecture Decisions

### 3.1 `profile_id` Identity

Decision: use a stable profile UUID as the target identity root.

For existing rows, seed or map `profile_id` from `agent_employees.id` when possible. Do not use `agent_employees.employee_id` as the profile primary key.

Rationale:

- `employee_id` is semantically polluted. It can mean employee code, DB row id, or external route alias depending on table and code path.
- Several IM/session tables already store `agent_employees.id` in columns named `employee_id`.
- A stable UUID root gives Memory OS, Skill OS, Growth Loop, and Curator a clean durable boundary.

Migration rule:

- Keep `employee_id`, `role_id`, `openclaw_agent_id`, and route `agent_id` as migration/routing aliases.
- New self-improving persistence hangs from `profile_id`.
- Old APIs can keep exposing `employee_id` where the UI or IM user model expects it.

### 3.2 `agent_employees` Future Role

Decision: keep `agent_employees` as the UI-facing employee table during the transition; make `agent_profiles` the new runtime identity table.

Long-term direction:

- `agent_profiles` becomes source of truth for runtime identity, profile home, profile memory, profile skills, growth state, and toolset defaults.
- `agent_employees` can become a projection surface for employee management, route aliases, and older APIs.

Do not remove or rename `agent_employees` in the first slice.

### 3.3 Team And Group JSON

Decision: do not eagerly rewrite team/group JSON in the first slice.

Use a runtime alias resolver:

```text
employee alias / role alias / openclaw agent id / agent employee row id
  -> profile_id
  -> profile runtime context
```

Rationale:

- Team templates, group rules, group run steps, active run state, seeded templates, and event payloads contain employee aliases in JSON.
- Column-only migration cannot fix JSON identity.
- Active run history should preserve old aliases for display and audit.

### 3.4 Memory Migration

Decision: do not move old memory files in the first slice.

First implement a migration-aware locator:

```text
ProfileMemoryLocator
  profile home target:
    profiles/<profile_id>/memories/
  legacy read candidates:
    memory/<skill_id>/
    memory/employees/<employee_bucket>/skills/<skill_id>/
    <work_dir>/openclaw/<employee_id>/memory/<skill_id>/   # legacy import source only
    IM roles/<role_id>/MEMORY.md and org/CASEBOOK.md as optional source namespaces
```

The first implementation should preserve current prompt output unless explicitly testing the new path. Later migration should copy/import with provenance, not move/delete old buckets.

### 3.5 Instruction Assets And Memory Boundary

Decision: do not use `USER.md` as a new Memory OS file. Existing employee `AGENTS.md / SOUL.md / USER.md` content should migrate into profile instructions, not OpenClaw compatibility files.

Canonical target:

```text
profiles/<profile_id>/instructions/
  RULES.md
  PERSONA.md
  USER_CONTEXT.md
```

`memories/MEMORY.md` is for learned stable facts and experience. Instruction files are behavior/persona/context assets. Do not add OpenClaw-style instruction file injection in the first slice because it changes prompt injection and user expectations.

### 3.6 IM Memory

Decision: treat IM memory as a source namespace first, not as the canonical profile memory store.

IM memory such as `roles/<role_id>/MEMORY.md`, `sessions/<thread_id>.md`, and `org/CASEBOOK.md` can later be imported into profile memory, but must keep provenance:

- source surface: IM
- channel: Feishu / WeCom / future
- thread or conversation id
- role/agent alias
- confidence / confirmed flag where available

Do not silently merge IM memory into desktop chat prompt during the first slice.

### 3.7 `.skillpack` Boundary

Decision: `.skillpack` content is immutable to self-improving flows.

Allowed:

- Install / uninstall through explicit user action.
- Read/decrypt for execution.
- Project decrypted copies as ephemeral runtime artifacts.
- Record read-only usage telemetry for curator scoring.

Forbidden:

- `skill_manage` patch, archive, reset, or delete of `.skillpack` canonical content.
- Curator mutation of `.skillpack` content.
- Treating projected plaintext workspace copies as canonical editable skill source.

Implementation implication:

- Introduce source policy enum before adding mutation tools.
- Add backend guards; do not rely only on UI hiding actions.

### 3.8 Preset Skill Behavior

Decision: builtin/vendored should evolve into preset.

Rules:

- Preset skills are seed content, not sacred immutable assets.
- Users and Hermes-aligned agent flows may patch, archive, delete, reset, and restore preset skills, with version history and high-risk confirmations where needed.
- Reset creates a new version/history entry rather than silently overwriting audit history.
- If a user deletes a preset skill, a future app upgrade should not silently reinstall it. It may show a restore/update suggestion.

### 3.9 Curator Default

Decision: Curator should be Hermes-aligned, not manual-approval-first by default.

Initial Curator behavior:

- Dry-run report is useful for observability, but it is not the long-term default product model.
- Low-risk cleanup may become automatic when provenance, versioning, and rollback exist.
- Destructive automatic execution remains blocked behind explicit confirmation.
- Pinned content is protected.
- `.skillpack` is read-only telemetry only.
- All curator actions require source evidence and rollback path.

Future employee-level automation should preserve Hermes-like flow: smooth agent growth, clear history, and confirmations only for high-risk changes.

### 3.10 Toolset Gateway First Slice

Decision: start with manifest-first read-only projection.

Do not change approval behavior in the first slice. First normalize observability:

- Map existing tools into `core`, `memory`, `skills`, `web`, `browser`, `im`, `desktop`, `media`, `mcp`.
- Fill missing metadata for sidecar/browser/MCP and runtime tools.
- Add tests that prove projection and metadata completeness.

Approval policy changes come later after projections are visible and tested.

## 4. Cross-Map Risk Register

### Risk 1. Identity Semantic Pollution

Highest risk.

`employee_id` can mean:

- employee code alias
- `agent_employees.id`
- route alias
- IM/OpenClaw agent id
- JSON topology label

Mitigation:

- Introduce `profile_id` as a separate root.
- Add alias resolver.
- Add nullable profile columns with fallback.
- Preserve display aliases.

### Risk 2. Startup-Critical SQLite Breakage

Affected paths:

- session list/search
- chat send
- IM inbound session binding
- group run start/resume
- employee list/detail

Mitigation:

- Nullable new columns.
- Backfill where safe.
- Query fallback to legacy fields.
- Legacy-schema tests before claiming Phase 0 complete.

### Risk 3. Memory File Migration Data Loss

Memory is file-backed and split across desktop, employee, group-run, and IM surfaces.

Mitigation:

- Locator first.
- Copy/import later.
- Keep provenance.
- Do not delete legacy buckets in the first migration.

### Risk 4. `.skillpack` Mutation Leak

Projected decrypted skill files can look editable to future tools.

Mitigation:

- Source policy enum.
- Canonical source lookup.
- Backend mutation guards.
- UI read-only label.
- Tests for skillpack immutability.

### Risk 5. Tool Metadata / Approval Mismatch

Tool metadata exists but actual approval comes from runtime-policy classification.

Mitigation:

- First add read-only projection.
- Do not change approval behavior until toolset policies are reviewed.
- Add tests for metadata and projection completeness.

### Risk 6. Product Surface Outruns Runtime Safety

UX can show growth/curator controls before backend guarantees are in place.

Mitigation:

- First UX slice is read-only Employee Growth Workbench.
- Mutation buttons remain disabled or route through existing risk confirmation until backend support exists.

## 5. Phase 0 Completion Gap

Current completed items:

- Identity / DB research map.
- Memory path research map.
- Skill source boundary research map.
- Toolset gateway research map.
- Employee growth UX research/design.
- Main synthesis decisions in this document.

Remaining before Phase 0 can be marked `[x]`:

- Legacy-schema regression tests for old DBs with no profile columns.
- Code-level `.skillpack` immutable source policy or guard.
- Compatibility matrix encoded in tests or implementation comments.
- First alias resolver design implemented or at least specified in Phase 1 plan.
- Roadmap Phase 0 checkboxes updated with verification evidence.

## 6. Recommended First Implementation Slice

### Slice Name

Profile Runtime Foundation Slice 1

### Scope

Implement the smallest code path that makes profile identity real without changing user-visible behavior.

Included:

- `agent_profiles` table.
- Nullable `sessions.profile_id`.
- Profile seed/backfill from `agent_employees.id`.
- Profile alias resolver service.
- Read-only profile home path resolver.
- Legacy session and IM binding fallback tests.
- Source policy enum for skills.
- `.skillpack` mutation guard in backend delete/mutation boundary.
- No memory file migration.
- No skill_manage.
- No curator mutation.
- No approval policy change.

Excluded:

- Moving memory directories.
- Creating `MEMORY.md` or profile instruction injection.
- Rewriting team JSON.
- Changing prompt output.
- Changing `.skillpack` format.
- Changing approval risk classification.

### Proposed Write Areas

- `apps/runtime/src-tauri/src/db/schema.rs`
- `apps/runtime/src-tauri/src/db/migrations.rs`
- `apps/runtime/src-tauri/src/commands/chat_session_io/*`
- `apps/runtime/src-tauri/src/commands/employee_agents/*` only where profile alias lookup is needed
- `apps/runtime/src-tauri/src/agent/runtime/*` only for read-only profile resolution, not memory migration
- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs` or adjacent new source policy module
- `packages/runtime-skill-core` only if source policy belongs in shared skill core

### Recommended Tests

- `pnpm test:rust-fast`
- Focused legacy SQLite tests:
  - old `sessions` table without `profile_id`
  - old `im_conversation_sessions` / `im_thread_sessions` shape
  - `agent_employee_skills.employee_id` storing `agent_employees.id`
- Skill source policy tests:
  - encrypted skillpack is immutable
  - local/preset mutable policy is distinguishable
  - unknown source type is rejected or normalized explicitly, not silently treated as encrypted

## 7. Parallelization Plan After Synthesis

After this synthesis, implementation can be split, but not all tasks can proceed at once.

Can run in parallel after Phase 1 implementation plan:

1. Profile DB/schema and legacy tests.
2. Skill source policy enum and `.skillpack` guard.
3. Read-only toolset projection tests.
4. Read-only Employee Growth Workbench UI shell.

Must remain coordinated/mostly serial:

- Session creation and send-message path profile wiring.
- IM inbound profile resolution.
- Group run profile binding.
- Memory locator integration into prompt/runtime registry.

Reason: these share startup-critical runtime identity state.

## 8. Open Questions Still Needing Implementation-Time Decisions

These do not block Slice 1 if the slice stays narrow:

- Whether `agent_profiles.id` should equal legacy `agent_employees.id` for all migrated employees or only store it as `legacy_employee_row_id`.
- Whether profile home should be created eagerly on DB migration or lazily on first runtime access.
- Whether general sessions get a synthetic default profile or remain profile-less until Memory OS.
- Whether `.skillpack` uninstall should be blocked for active profile skill memberships or allowed with cleanup warnings.
- Whether ClawHub and industry bundles become first-class source types immediately or remain provenance tags on `local`.

## 9. Recommended Next Action

Phase 1 implementation plan has been written:

- `docs/plans/2026-05-06-profile-runtime-foundation-slice-1-plan.md`

Execute it with small, reviewed batches:

1. DB/schema + tests.
2. Alias resolver + session fallback.
3. Skill source policy + `.skillpack` guard.
4. Read-only projections/UI shell if the previous batches remain stable.

Do not start Memory OS file migration or Skill OS mutation until this foundation passes verification.
