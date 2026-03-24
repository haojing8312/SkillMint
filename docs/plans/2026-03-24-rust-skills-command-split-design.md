# Rust Skills Command Split Design

**Goal:** Turn `apps/runtime/src-tauri/src/commands/skills.rs` into a thin command shell by extracting local skill flows, industry bundle flows, shared helpers, and DTOs into focused child modules without changing the public Tauri command contract.

## Why This Split

`skills.rs` is currently a mixed command surface and utility module. It holds:

- local skill preview, creation, import, and refresh flows
- industry bundle install and update-check flows
- skill catalog list/delete commands
- shared slug, semver, tag, and markdown helpers
- small DTO definitions used by the command layer

That combination makes the file harder to evolve safely and encourages future command logic to grow in the same place.

## Recommended Smallest Safe Split

Create four child modules under `apps/runtime/src-tauri/src/commands/skills/`:

- `types.rs`
- `helpers.rs`
- `local_skill_service.rs`
- `industry_bundle_service.rs`

Keep the root `skills.rs` as a thin shell that:

- exposes the existing Tauri command names
- keeps `DbState` visible where current callers expect it
- re-exports child module DTOs and helper-facing functions as needed
- retains only small command wrappers and the simplest query/delete commands if they are not yet worth moving

## Module Responsibilities

### `types.rs`

Own the current DTOs and shared state wrapper:

- `DbState`
- `ImportResult`
- `LocalSkillPreview`
- `InstalledSkillSummary`
- `IndustryInstallResult`
- `IndustryBundleUpdateCheck`

### `helpers.rs`

Own pure helpers and formatting/normalization logic:

- slug sanitization
- markdown rendering
- tag merging
- semver parsing/comparison
- tag value extraction
- display-name normalization
- file-reading fallback for `SKILL.md` / `skill.md`

### `local_skill_service.rs`

Own local-skill command flows:

- display-name availability checks
- preview rendering
- local skill creation
- local skill import
- local skill refresh
- local-to-pool import helpers

### `industry_bundle_service.rs`

Own bundle install and update-check flows:

- install industry bundle to pool
- install industry bundle via `DbState`
- check bundle update against installed local skills
- update-check wrapper that uses `DbState`

## Boundary Rules

- Do not change command names or response shapes.
- Do not move unrelated SQLite schema logic into this split.
- Keep `list_skills` and `delete_skill` behavior unchanged.
- Avoid creating one-file-per-helper sprawl; helper extraction should stay cohesive.

## Expected End State

- root `skills.rs` becomes a thin entry layer plus a few small wrappers
- command-specific logic is isolated by use case
- helper logic is reusable without bloating the root file
- future additions to `skills.rs` have obvious landing zones

## Risks

- Accidentally changing the order of import/update side effects
- Breaking local skill path selection or markdown rendering
- Changing bundle version comparison semantics
- Introducing too many child files for one-off helper code

## Verification Strategy

- Add focused Rust tests for helper behavior that must stay stable
- Run `pnpm test:rust-fast`
- If command-level behavior changes are touched, keep the existing `skills` command tests green
