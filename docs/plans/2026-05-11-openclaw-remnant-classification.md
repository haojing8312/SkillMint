# OpenClaw Remnant Classification

**Roadmap phase:** Hermes-aligned sidecar removal Batch 3A.

## Purpose and Scope

This document classifies remaining tracked `OpenClaw` references before removal planning. It is classification only: no runtime behavior, tests, package scripts, release scripts, sidecar implementation, frontend implementation, database schema, or package manager files are changed by this batch.

OpenClaw is legacy migration input only. The target architecture remains Hermes-aligned runtime boundaries: Rust ToolRegistry, native providers, platform gateways, profile-owned runtime state, Skill OS, memory, curator, and growth events.

## Snapshot Method

The classification is based on tracked repository data from these commands, run before this document was added:

```bash
git grep -n -i openclaw
python3 - <<'PY'
import subprocess, collections
lines = subprocess.run(
    ['git', 'grep', '-n', '-i', 'openclaw', 'HEAD', '--'],
    text=True,
    stdout=subprocess.PIPE,
).stdout.splitlines()
line_counts = collections.Counter()
file_sets = collections.defaultdict(set)
def bucket(path):
    if path.startswith('apps/runtime/src-tauri/'):
        return 'src-tauri'
    if path.startswith('docs/'):
        return 'docs'
    if path.startswith('apps/runtime/src/') or path.startswith('apps/runtime/e2e/'):
        return 'frontend'
    if path.startswith('apps/runtime/sidecar/'):
        return 'sidecar'
    if path.startswith('scripts/'):
        return 'scripts'
    if path.startswith('apps/runtime/plugin-host/') or path.startswith('packages/') or path.startswith('agent-evals/'):
        return 'plugin-host/other'
    return 'root/release-ci'
for line in lines:
    path = line.removeprefix('HEAD:').split(':', 1)[0]
    area = bucket(path)
    line_counts[area] += 1
    file_sets[area].add(path)
for area in ['src-tauri', 'docs', 'frontend', 'plugin-host/other', 'sidecar', 'scripts', 'root/release-ci']:
    print(area, line_counts[area], len(file_sets[area]))
PY
```

The counts below are the pre-Batch 3A baseline from `HEAD` before this classification commit is created. They are matching-line counts and distinct tracked-file counts, not symbol counts. `git grep` ignores untracked local files. The `frontend` bucket includes `apps/runtime/src/` and `apps/runtime/e2e/`. The `plugin-host/other` bucket includes `apps/runtime/plugin-host/`, `packages/`, and `agent-evals/`. The `root/release-ci` bucket is the catch-all for tracked root guidance, README, release CI, and local workflow files outside the narrower buckets.

| Area | Matching lines | Files |
| --- | ---: | ---: |
| `src-tauri` | 1253 | 109 |
| `docs` | 1206 | 108 |
| `frontend` | 339 | 63 |
| `plugin-host/other` | 176 | 28 |
| `sidecar` | 124 | 20 |
| `scripts` | 16 | 4 |
| `root/release-ci` | 39 | 7 |

## Classification Taxonomy

### A. Temporary legacy adapters that must remain until public callers migrate

These are compatibility boundaries that still anchor public commands, persisted aliases, or UI service contracts. They should be kept thin and marked temporary until callers move to neutral names.

Representative files:
- `apps/runtime/plugin-host/README.md`
- `apps/runtime/plugin-host/package.json`
- `apps/runtime/plugin-host/openclaw/package.json`
- `apps/runtime/plugin-host/openclaw/plugin-sdk/index.ts`
- `apps/runtime/plugin-host/src/loader.ts`
- `apps/runtime/plugin-host/src/api.ts`
- `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
- `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- `apps/runtime/src-tauri/src/commands/openclaw_plugins/tauri_commands.rs`
- `apps/runtime/src-tauri/src/im/openclaw_adapter.rs`
- `apps/runtime/src-tauri/src/commands/employee_agents/types.rs`
- `apps/runtime/src/types/im.ts`
- `apps/runtime/src/components/settings/feishu/feishuSettingsService.ts`

Removal rule: do not delete these until the Tauri command surface, frontend callers, persisted aliases such as `openclaw_agent_id`, plugin-host SDK imports, and IM host dispatch paths have neutral replacements with regression coverage.

### B. Internal neutralization candidates that can be renamed or moved safely in small batches

These are internal Rust/TypeScript names where the OpenClaw label no longer needs to define ownership, provided public wrappers stay intact during migration.

Representative files:
- `apps/runtime/src-tauri/src/commands/openclaw_plugins/runtime_service.rs`
- `apps/runtime/src-tauri/src/commands/openclaw_plugins/feishu_runtime_adapter.rs`
- `apps/runtime/src-tauri/src/commands/openclaw_plugins/wecom_runtime_adapter.rs`
- `apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs`
- `apps/runtime/src-tauri/src/commands/feishu_gateway/relay_service.rs`
- `apps/runtime/src-tauri/tests/test_openclaw_gateway.rs`
- `packages/runtime-skill-core/src/skill_config.rs`

Removal rule: rename in narrow batches with alias tests. Preserve parsing of legacy `openclaw` frontmatter and response shapes until migration rules explicitly deprecate them.

### C. Removable OpenClaw vendor/browser compatibility surfaces after replacement checks exist

These surfaces are closest to deletion, but only after current consumers and replacement checks prove no active path depends on them.

Representative files:
- `apps/runtime/src-tauri/src/agent/tools/browser_compat.rs`
- `apps/runtime/sidecar/src/openclaw-bridge/route-engine.ts`
- `apps/runtime/sidecar/vendor/openclaw-core/**`
- `apps/runtime/sidecar/vendor/openclaw-im-core/**`
- `apps/runtime/sidecar/test/openclaw.route-api.test.ts`
- `apps/runtime/sidecar/test/openclaw.route-engine.test.ts`
- `apps/runtime/sidecar/test/browser.compat-api.test.ts`

Removal rule: first prove no caller uses `/api/browser/compat` or the sidecar OpenClaw route engine, then remove sidecar tests and vendored code with replacement native provider or gateway coverage.

Batch 3D caller audit result: `docs/plans/2026-05-11-browser-compat-caller-audit.md` shows browser compatibility is a known active temporary wrapper, not safe to delete yet. Runtime registration still exposes the unified `browser` compatibility tool through `/api/browser/compat`, so deletion is blocked until a native browser provider or neutral wrapper replaces that path.

Batch 3E retirement plan result: `docs/plans/2026-05-11-plugin-host-openclaw-sdk-retirement-plan.md` classifies plugin-host/OpenClaw SDK compatibility as a retained temporary shim, not removed. Deletion is blocked until a Hermes-native platform adapter replacement exists and public command, package, service, frontend, test, and persisted alias callers have migrated to neutral names.

### D. Release-sensitive scripts, checks, and docs that require replacement or explicit deprecation

These affect release expectations or maintainer workflows and should not be deleted as ordinary cleanup.

Representative files:
- `package.json`
- `scripts/sync-openclaw-core.mjs`
- `scripts/sync-openclaw-im-core.mjs`
- `scripts/check-openclaw-vendor-lane.test.mjs`
- `scripts/check-openclaw-wecom-vendor-lane.test.mjs`
- `AGENTS.md`
- `docs/maintainers/openclaw-upgrade.md`

Removal rule: replace the release/vendor lane with a Hermes-native check or explicitly deprecate it in release docs before deleting scripts or package commands.

### E. Product, docs, and frontend copy that should be rewritten to Hermes-native language

These references shape user and maintainer understanding. They should move away from presenting OpenClaw compatibility as the product direction.

Representative files:
- `README.md`
- `README.en.md`
- `docs/architecture/openclaw-im-host/**`
- `docs/plans/2026-05-11-hermes-aligned-sidecar-removal-roadmap.md`
- `docs/superpowers/plans/2026-04-22-openclaw-im-reuse-rearchitecture-plan.md`
- `apps/runtime/src/components/settings/feishu/FeishuAuthorizationPanel.tsx`
- `apps/runtime/src/components/settings/feishu/feishuSelectors.ts`
- `packages/runtime-chat-app/src/prompt_assembly.rs`

Rewrite rule: keep historical docs discoverable, but mark them superseded where needed. Product copy should say WorkClaw owns the runtime and platform adapters; OpenClaw-shaped files are legacy migration inputs only.

## Recommended Batch 3 Sub-Batches

### Batch 3A. Classification

Status: `[x]`

Acceptance:
- `[x]` Current tracked `git grep -i openclaw` results are grouped by ownership area.
- `[x]` Remnants are classified into categories A-E.
- `[x]` The sidecar removal roadmap points implementers to this classification before deletion work.
- `[x]` No runtime, test, package, release, sidecar, frontend, schema, or package manager files are changed.

### Batch 3B. Docs/product copy and roadmap wording update

Status: `[x]`

Acceptance:
- `[x]` README and active planning docs no longer describe OpenClaw compatibility as the forward product architecture.
- `[x]` Historical OpenClaw IM docs are marked as superseded or historical where they conflict with Hermes direction.
- `[x]` Frontend visible copy stops telling users to think in OpenClaw-compatible mode, unless the copy is explicitly about a temporary legacy shim.
- `[x]` Docs-only validation runs with `git diff --check` and a scoped grep summary.

#### Batch 3B-1. README/docs/historical banners

Status: `[x]`

Handled:
- Reworded `README.md` and `README.en.md` so WorkClaw is presented as a Hermes-aligned, local-first desktop AI employee runtime/workbench.
- Marked OpenClaw as historical inspiration, legacy migration input, or temporary compatibility surface rather than a forward product architecture.
- Added superseded/historical guidance to `docs/architecture/openclaw-im-reuse.md`, a folder-level `docs/architecture/openclaw-im-host/README.md`, and the entry document `docs/architecture/openclaw-im-host/00-context-and-goals.md`.
- Marked `docs/maintainers/openclaw-upgrade.md` as a legacy vendor lane and deferred replacement/deprecation planning to Batch 3C.

Follow-on boundaries:
- Batch 3C documented the release/vendor lane replacement path.
- Batch 3D and 3E still own browser compatibility and plugin-host/OpenClaw SDK compatibility removal planning.

#### Batch 3B-2. Frontend visible copy

Status: `[x]`

Acceptance:
- `[x]` Active UI strings and settings copy are rewritten to Hermes-native language where they still describe OpenClaw compatibility as the user-facing mode.
- `[x]` Temporary legacy shim copy remains explicit and scoped.
- `[x]` Frontend/runtime validation is selected in the implementation batch.

Handled:
- Reworded active Feishu and WeCom settings copy to platform-adapter, compatibility-bridge, and connector-host language.
- Kept OpenClaw wording only in compatibility identifiers, fixture payloads, and explicit legacy shim explanation.
- Updated frontend test assertions for the new visible copy while preserving legacy-shaped command and service identifiers.

### Batch 3C. Release/vendor lane replacement plan

Status: `[x]`

Acceptance:
- `[x]` Current OpenClaw vendor sync and check scripts are mapped to either replacement Hermes-native checks or explicit deprecation.
- `[x]` Release-sensitive commands in `package.json`, AGENTS guidance, and release docs have a reviewed migration plan.
- `[x]` No vendor lane script is removed until the replacement/deprecation path is documented.

Handled:
- Added `docs/plans/2026-05-11-release-vendor-lane-replacement-plan.md`.
- Mapped `test:openclaw-vendor-lane`, `sync:openclaw-core`, `sync:openclaw-im-core`, sync scripts, check scripts, `docs/maintainers/openclaw-upgrade.md`, and both sidecar vendor metadata folders to retain, replace, or deprecate actions.
- Defined Stage 0 through Stage 4 migration order so neutral release checks are introduced before legacy OpenClaw command names are removed.
- Documented future validation, including release-doc/vendor tests and grep checks over `package.json`, `AGENTS.md`, docs, scripts, and vendor paths.

Left for later:
- Stage 1 must implement neutral package/script checks while preserving old OpenClaw command names as aliases.
- Stage 2 must migrate active docs and AGENTS guidance to neutral commands.
- Stage 3 may remove legacy OpenClaw command names only after downstream references are gone.
- Stage 4 may delete sidecar vendor folders only after route, browser, plugin, and IM consumers are gone.

### Batch 3D. Browser compatibility endpoint removal after caller audit

Status: `[~]`

Audit: `docs/plans/2026-05-11-browser-compat-caller-audit.md`

Acceptance:
- `[x]` `git grep` proves all `/api/browser/compat` callers are known and either migrated or intentionally retained as temporary wrappers.
- `[ ]` Native browser provider checks exist before sidecar browser compatibility tests are deleted.
- `[x]` `apps/runtime/src-tauri/src/agent/tools/browser_compat.rs` has a clear remove-or-wrap decision: retain temporarily as the unified `browser` compatibility wrapper, then migrate to a native provider or neutral wrapper before endpoint deletion.

Result:
- `/api/browser/compat` remains active and must not be deleted in Batch 3D.
- Prompt guidance in `packages/runtime-chat-app/src/prompt_assembly.rs` remains a later code-batch dependency because it still tells agents to use WorkClaw's built-in `browser` compatibility tool for OpenClaw/Xiaohongshu-like skills.
- No runtime, sidecar, package, script, vendored, test, or prompt assembly files are changed by the Batch 3D documentation step.

### Batch 3E. Plugin-host/OpenClaw SDK compatibility retirement plan

Acceptance:
- `[x]` `apps/runtime/plugin-host/openclaw/**` and `openclaw/plugin-sdk` shim usage are classified as retained temporary shim surfaces.
- `[x]` Official plugin host behavior has a Hermes-native platform adapter replacement plan or an explicit legacy-retirement plan.
- `[x]` Frontend and Tauri service contracts for `openclaw-lark` have neutral target names before public command removal.
- `[ ]` Hermes-native platform adapter replacement exists and active callers have migrated to neutral aliases.

Result:
- Batch 3E is documented in `docs/plans/2026-05-11-plugin-host-openclaw-sdk-retirement-plan.md`.
- Plugin-host/OpenClaw SDK compatibility remains active as a temporary shim. It is not removed in Batch 3E.
- Future code batches should introduce a neutral platform adapter host, plugin compatibility bridge, Feishu platform adapter service names, and public command aliases before removing OpenClaw-named surfaces.

## Risks

- Removing compatibility names too early can break public Tauri commands, frontend settings flows, persisted `openclaw_agent_id` aliases, or imported skill metadata.
- Deleting vendor lane scripts without a replacement can weaken release checks and make old release docs inaccurate.
- Treating historical docs as active guidance can steer new work back toward OpenClaw-shaped architecture.
- Browser and plugin-host compatibility surfaces may have low static visibility but real manual workflows; removal needs caller audit plus replacement checks.

## Non-Goals

- No runtime implementation changes.
- No sidecar deletion.
- No package script or release script changes.
- No package manager changes or installs.
- No schema changes or migrations.
- No claim that browser compatibility, vendor lanes, or plugin-host compatibility are removed in Batch 3A.
