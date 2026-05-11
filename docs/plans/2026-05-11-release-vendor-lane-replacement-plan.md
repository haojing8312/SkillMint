# Release Vendor Lane Replacement Plan

**Date:** 2026-05-11

**Roadmap phase:** Hermes-aligned sidecar removal Batch 3C.

**Source of truth:** [Hermes-Aligned Sidecar Removal Roadmap](2026-05-11-hermes-aligned-sidecar-removal-roadmap.md) and [OpenClaw Remnant Classification](2026-05-11-openclaw-remnant-classification.md).

## Scope

This is a planning-only batch. It documents how existing OpenClaw release/vendor lanes will be retained, replaced, or deprecated while WorkClaw migrates toward Hermes-aligned runtime boundaries.

This batch does not change runtime code, sidecar code, frontend code, package manager files, database schema, root `package.json` scripts, sync scripts, check scripts, or vendored sidecar files.

OpenClaw remains legacy migration input only. It must not become the default architecture for new routing, IM, browser, MCP, toolset, profile runtime, or release flows.

## Batch 3C Decision

Keep current OpenClaw-named vendor lanes as temporary legacy migration guards until replacement checks exist and downstream references have moved to neutral commands.

Future implementation batches should add neutral/Hermes-aligned release checks first, then migrate docs and guidance, then remove legacy command names only after grep checks prove no active guidance depends on them. Vendored sidecar folders are deleted last, after all route, browser, plugin, and IM consumers are gone.

## Artifact Inventory and Disposition

| Artifact | Current role | Batch 3C mapping | Replacement or deprecation target | Required future validation |
| --- | --- | --- | --- | --- |
| `package.json` `test:openclaw-vendor-lane` | Root release-sensitive check for legacy IM vendor metadata. | Retain temporarily as a legacy migration guard. | Replace with a neutral `test:runtime-vendor-lanes` or `test:legacy-vendor-lanes` command while the old name remains an alias during Stage 1. | `pnpm test:release-docs`; new neutral vendor-lane test; old alias still passes until Stage 3; grep proves active guidance uses neutral command. |
| `package.json` `sync:openclaw-core` | Manual sync entrypoint for the sidecar OpenClaw routing subset. | Retain temporarily; do not run as product-default release flow. | Deprecate after native route/gateway consumers no longer need `apps/runtime/sidecar/vendor/openclaw-core/**`. No Hermes-native upstream sync is planned for this source. | Grep proves no runtime, sidecar, docs, or release scripts require the sync command except historical plans/runbooks; route/browser/plugin/IM replacement checks pass before deletion. |
| `package.json` `sync:openclaw-im-core` | Manual sync entrypoint for the reserved second-channel IM vendor lane. | Retain temporarily; treat as an unused legacy reserve lane. | Deprecate after native WeCom/IM platform adapter checks replace any need for OpenClaw IM vendoring. No new OpenClaw-backed channel adoption should be added. | WeCom and channel connector tests pass through native runtime gateway; grep proves no docs recommend OpenClaw IM vendoring for new connectors. |
| `scripts/sync-openclaw-core.mjs` | Copies a small routing subset into `apps/runtime/sidecar/vendor/openclaw-core`. | Retain temporarily as a historical recovery tool. | Deprecate at Stage 3 after neutral checks are established; delete only with the sidecar vendor folder in Stage 4. | `rg -n "sync-openclaw-core|openclaw-core" package.json AGENTS.md README.md README.en.md docs scripts apps/runtime` has only historical or legacy references before removal. |
| `scripts/sync-openclaw-im-core.mjs` | Initializes/syncs metadata for reserved OpenClaw IM vendor lane. | Retain temporarily as a historical recovery tool. | Deprecate at Stage 3; delete only when `openclaw-im-core` metadata is no longer needed and no second-channel migration path depends on it. | Native IM/WeCom checks pass; grep proves no active docs or scripts tell maintainers to add new OpenClaw IM adapter code. |
| `scripts/check-openclaw-vendor-lane.test.mjs` | Verifies `openclaw-im-core` metadata and sync script existence. | Replace with a neutral vendor-lane policy check, then keep this test behind the old alias until Stage 3. | New test should validate that legacy vendor folders are marked historical, that neutral commands exist, and that legacy OpenClaw lanes are not product-default architecture. | `node --test scripts/check-runtime-vendor-lanes.test.mjs`; `pnpm test:openclaw-vendor-lane` remains green while aliased; `pnpm test:release-docs`. |
| `scripts/check-openclaw-wecom-vendor-lane.test.mjs` | Verifies reserved WeCom adoption language in `openclaw-im-core` metadata. | Replace with native IM/WeCom gateway validation rather than another vendor metadata test. | Deprecate once WeCom is validated through runtime gateway/platform adapter checks and no OpenClaw IM vendor adoption path remains active. | `cargo test --test test_wecom_gateway`; `cargo test --test test_channel_connectors`; relevant `test_im_employee_agents` coverage; docs grep for WeCom OpenClaw-vendor guidance. |
| `docs/maintainers/openclaw-upgrade.md` | Legacy maintainer runbook for OpenClaw vendor sync and regression checks. | Retain as a historical runbook and link to this plan. | Deprecate after Stage 3 command removal; archive or rewrite as migration history during Stage 4 sidecar vendor deletion. | `pnpm test:release-docs`; grep proves active contributor/release guidance points to neutral commands and this runbook is clearly historical. |
| `apps/runtime/sidecar/vendor/openclaw-core/**` metadata | Vendored routing subset plus `README.md`, `UPSTREAM_COMMIT`, and `PATCHES.md`. | Retain temporarily as legacy sidecar routing vendor state. | Delete only in Stage 4 after sidecar route/browser/plugin/IM consumers are gone and native runtime checks replace sidecar tests. | Grep proves no active consumer of sidecar OpenClaw route engine; Rust route/gateway tests pass; sidecar removal checks pass in the deletion batch. |
| `apps/runtime/sidecar/vendor/openclaw-im-core/**` metadata | Reserved metadata-only IM vendor lane, currently `UPSTREAM_COMMIT` is `uninitialized`. | Retain temporarily as legacy metadata. | Delete in Stage 4 after native IM platform adapters are the only active path and no future connector plan depends on OpenClaw vendoring. | Native Feishu/WeCom/channel connector tests pass; grep proves no active docs recommend OpenClaw IM vendor adoption; release docs test passes. |

## Staged Migration Order

### Stage 0. Current batch: no-op plan only

Status: documented by this file.

Actions:
- Do not edit `package.json`, sync scripts, check scripts, sidecar vendor folders, runtime code, frontend code, DB schema, or package manager files.
- Document every current OpenClaw release/vendor artifact and map it to retain, replace, or deprecate.
- Keep release-sensitive checks in place until the replacement/deprecation path is implemented.

Validation:
- `git diff --check`
- `git status --short --branch`
- Scoped file check proving only `docs/plans/**/*.md` and `docs/maintainers/**/*.md` changed.
- Grep/count check proving `package.json`, `scripts/sync-openclaw-*.mjs`, and `scripts/check-openclaw-*.mjs` are unchanged.

### Stage 1. Add neutral/Hermes-named release checks while old names remain aliases

Actions:
- Add a neutral release check such as `test:runtime-vendor-lanes` or `test:legacy-vendor-lanes`.
- Keep `test:openclaw-vendor-lane` as a temporary alias so existing release scripts and maintainer habits continue to work.
- If sync commands still need discoverability, add neutral historical names first and make the OpenClaw names aliases, not the other way around.
- Do not remove vendor folders or legacy scripts in this stage.

Intended replacement scope:
- Verify legacy vendor lanes are isolated to migration/recovery surfaces.
- Verify docs say OpenClaw vendor sync is historical and not a product-default architecture.
- Verify no new OpenClaw vendor adoption path is presented for IM connectors.

Required validation:
- `pnpm test:release-docs`
- New neutral vendor-lane command
- Existing `pnpm test:openclaw-vendor-lane`
- `rg -n "test:openclaw-vendor-lane|sync:openclaw-core|sync:openclaw-im-core" AGENTS.md README.md README.en.md docs package.json scripts`
- `rg -n "OpenClaw.*product-default|OpenClaw.*forward architecture|OpenClaw.*new connector" docs README.md README.en.md`

### Stage 2. Migrate docs, AGENTS, and package guidance to neutral commands

Actions:
- Update `AGENTS.md`, release-sensitive command lists, README links, and maintainer docs to name the neutral check.
- Keep legacy OpenClaw command names only in historical runbooks and migration plans.
- Keep `package.json` aliases unchanged until all active guidance has moved.

Required validation:
- `pnpm test:release-docs`
- `rg -n "pnpm test:openclaw-vendor-lane|pnpm sync:openclaw|sync:openclaw-core|sync:openclaw-im-core" AGENTS.md README.md README.en.md docs .github package.json scripts`
- Manual classification of any remaining hits as historical, alias-only, or removal-blocking.

### Stage 3. Remove legacy OpenClaw command names only after downstream references are gone

Actions:
- Remove or stop exposing root OpenClaw command names only after Stage 2 grep checks show no active docs or scripts depend on them.
- Keep the underlying neutral checks if they still protect release behavior.
- If a legacy command remains for compatibility, mark it as an alias with an explicit removal milestone.

Required validation:
- `pnpm test:release-docs`
- `pnpm test:release`
- Neutral vendor-lane command
- `rg -n "\"test:openclaw-vendor-lane\"|\"sync:openclaw-core\"|\"sync:openclaw-im-core\"" package.json`
- `rg -n "pnpm test:openclaw-vendor-lane|pnpm sync:openclaw|node scripts/sync-openclaw|node --test scripts/check-openclaw" AGENTS.md README.md README.en.md docs .github scripts`

### Stage 4. Delete sidecar vendor folders only after route/browser/plugin/IM consumers are gone

Actions:
- Delete `apps/runtime/sidecar/vendor/openclaw-core/**` and `apps/runtime/sidecar/vendor/openclaw-im-core/**` only in the sidecar deletion/removal implementation batch.
- Remove sync/check scripts only when their target folders and aliases are gone.
- Update historical docs to say the vendor lanes were retired and identify the last release or commit that contained them.

Required validation:
- `pnpm test:rust-fast`
- `pnpm test:release-docs`
- `pnpm test:installer`
- `pnpm build:runtime`
- Native gateway/browser/plugin/IM checks selected by the implementation batch.
- `rg -n "apps/runtime/sidecar/vendor/openclaw-core|apps/runtime/sidecar/vendor/openclaw-im-core|sync-openclaw|check-openclaw-vendor-lane" .`

## Future Implementation Validation Matrix

| Future batch | Required checks |
| --- | --- |
| Add neutral vendor-lane checks | `pnpm test:release-docs`; new neutral vendor-lane command; existing `pnpm test:openclaw-vendor-lane`; grep for old command guidance. |
| Migrate AGENTS/docs/package guidance | `pnpm test:release-docs`; grep over `AGENTS.md`, README files, `docs`, `.github`, `package.json`, and `scripts` for OpenClaw command references. |
| Remove legacy package command names | `pnpm test:release-docs`; `pnpm test:release`; neutral vendor-lane command; grep proving removed command names are absent from active guidance. |
| Remove OpenClaw IM vendor lane | Native WeCom/channel connector tests; release docs test; grep proving no active OpenClaw IM vendor adoption guidance remains. |
| Delete sidecar vendor folders and scripts | `pnpm test:rust-fast`; `pnpm test:release-docs`; `pnpm test:installer`; `pnpm build:runtime`; implementation-specific browser/MCP/IM/plugin tests; grep proving no consumer references deleted paths. |

## Risks and Rollback

| Risk | Control | Rollback |
| --- | --- | --- |
| Removing legacy commands before docs and maintainer flows migrate breaks release habits. | Stage neutral commands first and keep aliases until grep proves active references are gone. | Restore the previous `package.json` aliases and rerun release-doc/vendor checks. |
| Replacing metadata checks with weak neutral checks silently loses release coverage. | Neutral check must validate legacy isolation, historical labeling, and absence from product-default guidance before old checks are removed. | Re-enable `test:openclaw-vendor-lane` and keep the old script until neutral coverage is corrected. |
| Deleting vendor folders while sidecar tests or plugin/IM flows still import them breaks runtime or release checks. | Vendor folders are Stage 4 only, after consumer grep plus native replacement checks. | Restore the deleted vendor folders and scripts from git, then rerun sidecar/runtime/release checks. |
| Historical docs continue to steer new work toward OpenClaw. | Stage 2 grep checks classify remaining references and require active guidance to use neutral/Hermes language. | Revert the misleading doc change or add a superseded/historical banner before merging removal work. |
| WeCom or future IM work reopens the OpenClaw IM vendor lane as a product default. | Batch 3C marks the lane unused legacy reserve; future IM work must use native runtime gateway/platform adapters. | Stop the new vendor adoption, document the attempted exception, and route implementation through the native IM roadmap. |

## Batch 3C Acceptance

- Current OpenClaw vendor sync and check scripts are mapped to replacement checks or explicit deprecation targets.
- Root `package.json` commands, AGENTS guidance, maintainer docs, and sidecar vendor metadata have a documented migration plan before command removal.
- Future release-sensitive validation is defined for neutral check introduction, docs migration, command removal, and vendor folder deletion.
- No vendor lane script, package script, runtime file, sidecar file, frontend file, schema file, package manager file, or vendored sidecar file is changed by this batch.
