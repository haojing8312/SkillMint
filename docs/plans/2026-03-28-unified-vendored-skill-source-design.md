# Unified Vendored Skill Source Design

**Date:** 2026-03-28

**Status:** Approved

## Summary

WorkClaw should stop treating builtin skills as a privileged runtime mechanism and instead treat them as a preinstalled distribution source. Builtin skills remain vendored in the repository, but startup/bootstrap should materialize them into the same installed-skill model used by externally sourced skills. Runtime execution should then rely on one unified projection path.

The target outcome is:

- builtin skills are still shipped with WorkClaw
- builtin skills can be upgraded by replacing vendored third-party directories and metadata
- runtime no longer branches on builtin-specific loading logic
- builtin and external directory-backed skills share one install, projection, readiness, and refresh model

## Goals

- Make builtin skills behave like preinstalled skills, not hardcoded runtime exceptions
- Reduce future vendor upgrade cost for third-party open source skills
- Unify runtime behavior for builtin and external directory-backed skills
- Preserve current stable skill ids such as `builtin-docx`, `builtin-xlsx`, `builtin-pdf`, and `builtin-pptx`
- Keep upgrade behavior deterministic and safe across app restarts

## Non-Goals

- Replacing encrypted skillpack support in this phase
- Building a remote marketplace updater for vendored skills
- Designing a full user-facing conflict-resolution UI for local modifications
- Removing repository-vendored skill assets from the source tree

## Current State

Today WorkClaw still carries builtin-specific runtime logic:

- builtin entries are seeded directly into `installed_skills`
- runtime uses builtin-only content lookup in `workspace_skills.rs`
- `runtime-skill-core` embeds builtin asset trees
- builtin readiness/status logic still needs builtin-aware root resolution

This means builtin skills are not just a distribution source. They remain a special execution path, which makes future vendor updates heavier than they should be.

## Desired Model

### 1. Installed skill is the only runtime unit

Runtime should only know how to execute an installed skill record. Each record should resolve to a runtime content model that can be projected into the workspace.

For this design, the important distinction is not "builtin vs local", but "what kind of content is behind this installed record".

### 2. Separate distribution metadata from content metadata

Current `source_type` mixes distribution origin and runtime loading behavior. The new model should separate them:

- `distribution_type`
  - `vendored`
  - `local`
  - `imported`
  - `encrypted`
- `content_type`
  - `directory`
  - `encrypted_pack`

This lets the runtime ask only one question when loading a skill:

- is this backed by a directory?
- or by an encrypted pack that must be unpacked?

Builtin becomes `distribution_type = vendored`, not a runtime branch.

### 3. Builtin becomes vendored-preinstalled

Vendored skill directories remain in the repo for development and release packaging. On startup or upgrade, WorkClaw syncs them into a controlled application data directory, for example:

- `<app-data>/skills/vendor/<skill-id>/`

Then WorkClaw writes or updates `installed_skills` so those records point at the synced directory and identify themselves as vendored directory-backed installs.

After this bootstrap:

- runtime loads vendored skills exactly like local directory skills
- readiness checks operate on the synced directory
- refresh/update logic can compare vendored metadata and content hashes without needing builtin branches

### 4. Stable ids, unified content path

The current builtin ids remain unchanged. This avoids breaking:

- saved sessions
- employee bindings
- default skill selections
- hardcoded product assumptions already using those ids

Only the implementation behind the id changes.

## Approaches Considered

### Approach A: Keep builtin runtime branch, abstract it behind a provider

This would introduce a common trait or service for loading skill content while leaving builtin as a distinct source type.

Pros:

- smaller immediate refactor
- fewer database changes

Cons:

- builtin remains a runtime special case
- vendor upgrades still involve core-code coupling
- does not fully satisfy the "builtin is just preinstalled" goal

### Approach B: Convert builtin to vendored directory-backed installs at bootstrap

This keeps vendored assets in the repo but syncs them into the same installed-skill model as external directory skills.

Pros:

- best alignment with product goal
- builtin and local directory skills share one runtime path
- future vendor upgrades become mostly asset and metadata updates

Cons:

- requires schema and migration work
- requires compatibility migration for existing builtin rows

### Approach C: Package builtin as standard skill packs and auto-install them

This would convert builtin assets into packaged artifacts and use the install pipeline to import them.

Pros:

- very standardized distribution story

Cons:

- heavier development workflow
- unnecessary packaging overhead for repository-vendored assets
- slower iteration for frequent upstream refreshes

## Recommendation

Choose Approach B.

It gives the cleanest long-term architecture with the smallest ongoing maintenance cost. It removes builtin from the runtime hot path while keeping release packaging practical.

## Architecture

### Vendored skill catalog

Keep a catalog of vendored skills in code or metadata that describes:

- stable skill id
- vendored asset root in the repo or packaged resources
- manifest seed data
- vendor version or content hash

This catalog is used only for bootstrap and upgrade sync, not for runtime projection.

### Vendored sync root

At startup, sync each vendored skill into a controlled app-data directory. The sync should:

- create missing vendored skill directories
- update directories when the vendored version changes
- preserve exact projected file layout
- avoid rewriting unchanged skills

The sync process should compute a deterministic version marker or content hash so idempotency is easy to prove in tests.

### Installed skill record

Each vendored skill should be represented in `installed_skills` the same way a directory-backed external skill is represented:

- manifest JSON
- stable id
- directory path
- `distribution_type = vendored`
- `content_type = directory`

If a full schema rename is too expensive for the first pass, WorkClaw can introduce backward-compatible new columns while still reading legacy `source_type`.

### Runtime loading

Runtime should stop branching on builtin-specific asset loading. The unified runtime path should be:

1. load installed skill record
2. inspect `content_type`
3. if `directory`, read from the directory
4. if `encrypted_pack`, unpack and project

Builtin-specific helpers in `runtime-skill-core` should no longer be required for runtime projection.

### Readiness and refresh

Skill readiness checks should work from the installed directory plus parsed skill metadata. Vendored skills then naturally participate in the same logic as local directory skills.

Refresh/update behavior should be split by distribution type:

- `vendored`: startup sync or explicit "check vendored updates"
- `local`: refresh from source directory
- `encrypted`: reinstall or unpack metadata refresh

## Compatibility Plan

### Existing builtin rows

Existing rows with `source_type = builtin` should be migrated automatically. The migration path should:

1. sync vendored assets into the app-data vendor root
2. rewrite the record as a vendored directory-backed skill
3. preserve id, manifest identity, and existing references

This migration must be idempotent.

### Existing runtime behavior

The user-visible skill list should remain stable during the transition:

- same ids
- same names
- same general descriptions

Only the backend loading path changes.

## Risks

### Dual-path drift

If builtin loading remains alive in parallel after vendored sync lands, the codebase will still pay the same complexity cost. The migration should explicitly remove builtin runtime branching instead of merely adding a second option.

### Local modification conflicts

If a vendored synced directory is user-editable, upstream sync may overwrite user changes. The first phase should treat the vendored sync root as WorkClaw-managed content. If user customization is needed later, it should happen via fork/copy flows, not in-place edits to vendored roots.

### Release packaging assumptions

Current release packaging may assume builtin assets are embedded in Rust crates. The refactor must verify that vendored assets remain available from packaged resources or app bundle contents before runtime bootstrap.

## Testing Strategy

Required test coverage:

- vendored sync is idempotent
- vendored sync upgrades when content hash or version changes
- legacy builtin rows migrate to vendored directory-backed rows
- runtime projection for vendored skills uses the unified directory path
- readiness checks work for vendored skills without builtin special-case root logic
- existing builtin ids remain resolvable

## Rollout Plan

### Phase 1

- introduce vendored catalog and sync root
- migrate builtin rows to vendored directory-backed installs
- unify runtime directory-backed projection

### Phase 2

- remove builtin runtime file-tree embedding from the hot path
- reduce `runtime-skill-core` builtin responsibilities to bootstrap/catalog helpers only

### Phase 3

- add vendored version/update reporting
- optionally expose update status in the UI

## Decision

Proceed with a unified vendored-source model where builtin skills are treated as preinstalled vendored skills and runtime execution is unified around installed directory-backed skills.
