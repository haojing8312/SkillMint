# Skill Source Model

**Status:** Active  
**Last updated:** 2026-03-28

## Summary

WorkClaw now treats builtin skills as a **preinstalled distribution source**, not a privileged runtime execution path.

The important rule is:

- runtime executes installed skills
- installed skills are loaded by content shape
- builtin is no longer the primary runtime model

In current code, the formal source label for preinstalled skills is `vendored`.

## Source Types

### `vendored`

Use `vendored` for skills that ship with WorkClaw and are synced into the app-data vendor directory at bootstrap.

Characteristics:

- shipped in the repository or release assets
- materialized into a managed directory such as `<app-data>/skills/vendor/<skill-id>`
- treated as directory-backed installed skills
- user-visible meaning: `预装`
- refreshable
- not removable from product UI

Typical examples:

- `builtin-general`
- `builtin-docx`
- `builtin-xlsx`
- `builtin-pdf`
- `builtin-pptx`

### `local`

Use `local` for directory-backed skills created or imported by the user.

Characteristics:

- stored as a directory path in `installed_skills.pack_path`
- treated as directory-backed installed skills
- user-visible meaning: `本地`
- refreshable
- removable

### `encrypted`

Use `encrypted` for packaged skills that must still be unpacked or verified before runtime projection.

Characteristics:

- backed by a packaged artifact rather than a plain directory
- runtime still uses the encrypted unpack path
- user-visible meaning: usually `已安装`
- removable

### Legacy `builtin`

`builtin` now exists only as a compatibility shell for old database rows.

Rules:

- do not create new rows with `source_type = builtin`
- if an old builtin row has a valid directory `pack_path`, self-heal it to `vendored`
- if an old builtin row has no usable directory yet, runtime may still fall back to embedded assets as a last resort

This fallback exists only to avoid breaking existing installations during migration.

## Runtime Rules

Runtime should think in terms of **content shape**, not distribution branding.

Current effective rule:

- if a skill is directory-backed, load it through the unified directory path
- if a skill is encrypted, unpack it
- only if a legacy builtin row has no usable directory should builtin embedded fallback apply

That means new runtime work should avoid branching on:

- “is this builtin?”

and prefer:

- “is this directory-backed?”

## UI Rules

Frontend should not guess source semantics only from id prefixes when source metadata is available.

Current presentation contract:

- `vendored` or legacy `builtin` => `预装`
- `local` => `本地`
- everything else => `已安装`

The shared helpers live in:

- `apps/runtime/src/app-shell-utils.ts`

Use those helpers instead of re-implementing source classification in each view.

## Data Rules

For `installed_skills` rows:

- new preinstalled rows should be written as `vendored`
- bootstrap sync should keep vendored rows pointed at the managed vendor directory
- read paths may self-heal legacy builtin rows into vendored when a valid directory already exists

Stable skill ids such as `builtin-general` remain valid and should not be renamed just because the runtime source model changed.

## Maintainer Guidance

When adding a new preinstalled third-party skill:

1. Vendor the full skill directory into the repo.
2. Ensure bootstrap sync materializes it into the app-data vendor root.
3. Seed or update the installed row as `vendored`.
4. Make UI and tool surfaces rely on source metadata, not id-prefix guesses.
5. Only add compatibility fallback if there is a real migration need.

When refactoring future code:

- prefer “preinstalled/vendored vs local vs encrypted” wording
- avoid introducing fresh `builtin` execution branches
- keep legacy builtin handling isolated and easy to delete later

## What Still Remains Legacy

The project still contains some compatibility code for:

- old `builtin` rows already stored in SQLite
- embedded builtin asset fallback for rows without a usable vendored directory yet

Those paths should continue shrinking over time. They are compatibility mechanisms, not the target architecture.
