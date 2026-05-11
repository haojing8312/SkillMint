# Plugin Host OpenClaw SDK Retirement Plan

**Date:** 2026-05-11

**Roadmap phase:** Hermes-aligned sidecar removal Batch 3E.

**Source of truth:** [Hermes-Aligned Sidecar Removal Roadmap](2026-05-11-hermes-aligned-sidecar-removal-roadmap.md) and [OpenClaw Remnant Classification](2026-05-11-openclaw-remnant-classification.md).

## Status and Decision

This is a documentation-only audit and retirement plan. It does not remove compatibility, rename public commands, change runtime behavior, edit tests, change package metadata, or touch plugin-host implementation files.

**Decision:** retain `apps/runtime/plugin-host/openclaw/**`, the `openclaw/plugin-sdk` shim, OpenClaw-named plugin-host package metadata, `openclaw-lark` service identifiers, and OpenClaw-named Tauri invoke commands as temporary compatibility surfaces.

Retirement is blocked until both of these exist:

- A Hermes-native platform adapter replacement for the current Feishu/plugin-host workflows.
- A public command and service alias migration path that lets frontend, Tauri, tests, package scripts, and persisted references move to neutral names before OpenClaw-named commands are removed.

OpenClaw remains a legacy migration input only. New WorkClaw platform work should use Hermes-aligned runtime and platform adapter boundaries rather than extending the OpenClaw SDK shape.

## Current Surface Inventory

### Plugin-Host Shim

- `apps/runtime/plugin-host/README.md` describes the OpenClaw Plugin Host and `openclaw/plugin-sdk` compatibility expectations.
- `apps/runtime/plugin-host/package.json` is named `@workclaw/openclaw-plugin-host`.
- `apps/runtime/plugin-host/openclaw/package.json` exports `./plugin-sdk`, `./plugin-sdk/compat`, and `./plugin-sdk/feishu`.
- `apps/runtime/plugin-host/openclaw/plugin-sdk/index.ts` exports compatibility types and utilities such as `OpenClawPluginApi` and `PluginRuntime`.

### Plugin-Host Loader, Runtime, Scripts, and Tests

- `apps/runtime/plugin-host/src/loader.ts` rewrites `openclaw/plugin-sdk` imports to local shim paths and installs `node_modules/openclaw`.
- `apps/runtime/plugin-host/src/api.ts` exposes the `OpenClawPluginApiLike` registration surface.
- Plugin-host tests and smoke/inspection scripts still cover the shim and loader behavior.
- The plugin-host test script includes `./plugin-host/src` and `./plugin-host/openclaw`, so shim coverage is still an active compatibility guard.

### Tauri Commands, State, and Schema

- `apps/runtime/src-tauri/src/lib.rs` still exposes `openclaw_plugins` commands.
- `apps/runtime/src-tauri/src/commands/openclaw_plugins*.rs` and `apps/runtime/src-tauri/src/commands/openclaw_plugins/**` own install, list, inspect, start, stop, status, advanced settings, installer session, and credential probe commands.
- Tauri command names and persisted state still include OpenClaw-shaped service identifiers and aliases that frontend settings and tests call directly.

### Frontend Settings, Types, and E2E

- Frontend settings still calls `get_openclaw_plugin_feishu_advanced_settings`, `set_openclaw_plugin_feishu_advanced_settings`, `probe_openclaw_plugin_feishu_credentials`, and `install_openclaw_plugin_from_npm`.
- Frontend settings and test fixtures still use `openclaw-lark` and `@larksuite/openclaw-lark` identifiers.
- E2E and runtime tests still contain OpenClaw-lark fixtures that protect current compatibility behavior.

### Docs and Release References

- Active planning docs classify OpenClaw compatibility as legacy migration input, temporary wrapper, or retirement target.
- Release and maintainer guidance still includes OpenClaw-named vendor or compatibility language in historical contexts.
- These references should be rewritten only when the corresponding neutral command or adapter path exists, except for docs that are explicitly historical.

## Classification

| Surface | Batch 3E classification | Future target | Removal condition |
| --- | --- | --- | --- |
| `apps/runtime/plugin-host/openclaw/**` shim package | Retain temporary shim. | Plugin compatibility bridge behind a neutral platform adapter host. | Remove only after no plugin-host loader, tests, package exports, or external plugin imports require `openclaw/plugin-sdk`. |
| `openclaw/plugin-sdk` import rewrite in plugin-host loader | Retain temporary shim behavior. | Neutral compatibility bridge resolver with explicit legacy import support. | Remove only after supported plugins import the neutral SDK or after an explicit legacy-retirement policy drops SDK compatibility. |
| `OpenClawPluginApiLike`, `OpenClawPluginApi`, and `PluginRuntime` type names | Rename behind neutral facade in future code batch. | `PlatformPluginApiLike`, `PlatformAdapterApi`, or equivalent runtime adapter types. | Keep aliases until TypeScript tests and plugin smoke tests prove old imports are no longer active. |
| `@workclaw/openclaw-plugin-host` package identity | Rename behind neutral facade or keep as historical package alias. | `@workclaw/platform-adapter-host` or `@workclaw/plugin-compatibility-bridge`. | Add neutral package/script aliases first; remove the old name only after release docs, scripts, and tests use neutral names. |
| `openclaw-lark` and `@larksuite/openclaw-lark` service IDs | Retain temporary service identifiers. | `feishu-platform-adapter` and `@workclaw/feishu-platform-adapter` or another reviewed neutral package name. | Remove only after frontend, Tauri, persisted state, tests, and install flows accept neutral aliases. |
| `openclaw_plugins` Tauri command module and invoke names | Rename behind neutral command aliases in future code batch. | `platform_adapters` or `platform_adapter_plugins` command family. | Remove public OpenClaw commands only after neutral aliases exist and tests cover both migration and final removal. |
| Frontend settings copy, types, and E2E fixtures | Rename after neutral backend aliases exist. | Feishu platform adapter settings and neutral service types. | Update once Tauri aliases are available; keep legacy fixture coverage until persisted aliases migrate. |
| Historical OpenClaw docs and old design plans | Historical docs only. | Keep as dated context with legacy wording. | Do not rewrite unless they are linked as current guidance or release instructions. |

## Neutral Target Names

These names are targets for future implementation batches, not names implemented by this documentation batch.

| Current name | Neutral target name | Notes |
| --- | --- | --- |
| OpenClaw Plugin Host | Platform Adapter Host | Host boundary for Hermes-native platform adapters. |
| `openclaw/plugin-sdk` | Plugin Compatibility Bridge SDK or Platform Adapter SDK | The bridge name should make legacy support explicit if OpenClaw imports remain accepted. |
| `OpenClawPluginApiLike` / `OpenClawPluginApi` | `PlatformPluginApiLike` / `PlatformAdapterApi` | Add aliases before removing old exported type names. |
| `openclaw_plugins` Tauri module | `platform_adapters` or `platform_adapter_plugins` | Public invoke aliases must precede command removal. |
| `openclaw-lark` | `feishu-platform-adapter` | Service ID target for Feishu/Lark integration. |
| `@larksuite/openclaw-lark` | `@workclaw/feishu-platform-adapter` | Package target should be reviewed before publishing or package-script changes. |
| `install_openclaw_plugin_from_npm` | `install_platform_adapter_from_npm` | Keep old invoke name as alias until frontend and docs migrate. |
| OpenClaw plugin advanced settings | Feishu platform adapter advanced settings | Frontend copy and type names should follow backend aliases. |

## Migration Order

1. Add neutral platform adapter host and compatibility bridge facades while retaining `openclaw/plugin-sdk` imports.
2. Add neutral Feishu platform adapter service identifiers and package/script aliases, with legacy `openclaw-lark` aliases still accepted.
3. Migrate frontend settings copy, settings service types, and E2E fixtures to neutral Feishu platform adapter names.
4. Add neutral Tauri invoke command aliases for install, list, inspect, start, stop, status, advanced settings, installer session, and credential probe flows.
5. Move plugin-host tests and smoke scripts to neutral names while keeping explicit legacy compatibility tests for `openclaw/plugin-sdk`.
6. Migrate persisted aliases and runtime state reads with backward-compatible fallbacks for existing databases and config.
7. Remove OpenClaw public commands, shim exports, and service identifiers only after grep and tests prove all active callers have alternative paths.

## Acceptance and Future Validation

Batch 3E documentation acceptance:

- `[x]` `apps/runtime/plugin-host/openclaw/**` and `openclaw/plugin-sdk` shim usage are classified as retained temporary shim surfaces.
- `[x]` Plugin-host loader/runtime/scripts/tests, Tauri commands/state/schema, frontend settings/types/E2E, and docs/release references are inventoried as active compatibility surfaces.
- `[x]` `openclaw-lark` public service names have neutral target names before public command removal.
- `[x]` Plugin-host compatibility is explicitly blocked from deletion until a Hermes-native platform adapter replacement and alias migration path exist.
- `[x]` No runtime code, frontend code, Tauri code, plugin-host code, sidecar code, scripts, package manager files, tests, or vendored files are changed by this batch.

Future code-batch validation commands should include the touched-area checks plus these targeted greps:

```bash
cd /mnt/d/code/workclaw
git diff --check
git status --short --branch
rg -n "openclaw/plugin-sdk|openclaw-lark|@larksuite/openclaw-lark|openclaw_plugin|OpenClawPlugin|install_openclaw_plugin|OpenClaw Plugin Host" apps/runtime/plugin-host apps/runtime/src apps/runtime/e2e apps/runtime/src-tauri package.json scripts docs
pnpm test:rust-fast
pnpm --dir apps/runtime exec tsc --noEmit
pnpm test:e2e:runtime
pnpm test:release-docs
```

When a batch changes SQLite-backed runtime state or startup-critical reads, add a regression test that uses a legacy schema or legacy persisted alias and proves old databases still load.

## Docs-Only Batch 3E Validation

Required validation for this documentation batch:

```bash
cd /mnt/d/code/workclaw
git diff --check
git status --short --branch
python3 - <<'PY'
import subprocess, sys
changed = subprocess.check_output(['git', 'diff', '--name-only'], text=True).splitlines()
outside = [path for path in changed if not path.startswith('docs/plans/')]
if outside:
    print('\n'.join(outside))
    sys.exit(1)
print('docs_plans_only=OK')
PY
corepack pnpm test:release-docs
```

The scoped file check should print `docs_plans_only=OK`. Any other changed path means the batch touched files outside `docs/plans/` and must be corrected before completion.

## Risks

| Risk | Control |
| --- | --- |
| Removing OpenClaw-named Tauri commands too early breaks frontend settings, E2E fixtures, or persisted aliases. | Add neutral aliases and legacy fallback tests before public command removal. |
| Renaming plugin-host package metadata before scripts and smoke tests move breaks plugin inspection or local compatibility testing. | Add neutral package/script aliases first and keep old names as explicit compatibility aliases. |
| Dropping `openclaw/plugin-sdk` imports breaks existing plugin packages. | Keep the shim until supported plugins have neutral SDK imports or an explicit legacy-retirement policy. |
| Frontend copy moves faster than backend aliases and creates broken invoke calls. | Migrate backend aliases first, then frontend copy/types/E2E fixtures. |
| Historical docs are mistaken for active guidance. | Keep historical plans dated and link current roadmap/classification docs from active guidance. |

## Non-Goals

- No runtime code changes.
- No frontend code changes.
- No Tauri command changes.
- No plugin-host, sidecar, script, package manager, test, or vendored file changes.
- No deletion of `apps/runtime/plugin-host/openclaw/**`, `openclaw/plugin-sdk` shim code, `openclaw-lark` command names, or Tauri invoke commands.
- No claim that plugin-host/OpenClaw SDK compatibility has been removed.
