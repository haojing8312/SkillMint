# Hermes-Aligned Sidecar Removal Roadmap

> **For Hermes:** Use subagent-driven-development skill to execute this plan task-by-task. Each implementation task must preserve current user-visible behavior unless the task explicitly says the compatibility surface is being removed.

**Date:** 2026-05-11

**Goal:** Remove `apps/runtime/sidecar` step by step and replace its responsibilities with Hermes-aligned runtime boundaries: native tool registry/toolsets, gateway/platform adapters, profile runtime, native providers, and auditable self-improving flows.

**Architecture:** WorkClaw should not keep a Node HTTP sidecar as a product architecture boundary. The Rust/Tauri runtime already owns profile identity, tool registry, Toolset Gateway, memory, Skill OS, growth events, curator, and IM host surfaces; sidecar capabilities should be migrated into those native boundaries and then deleted. OpenClaw compatibility is not a future target: OpenClaw-shaped code, route names, vendor sync lanes, browser compatibility wrappers, and `openclaw` directories are legacy migration inputs only.

**Tech Stack:** Rust/Tauri runtime, React desktop UI, SQLite-backed runtime data, Hermes Agent reference code under `references/hermes-agent`, existing WorkClaw tests and root `package.json` scripts.

---

## 1. Product Direction

WorkClaw's next runtime should be Hermes-aligned, not OpenClaw-compatible:

- **Canonical identity:** `profile_id -> profiles/<profile_id>/...`.
- **Tool exposure:** Rust `ToolRegistry` + Toolset Gateway, not sidecar-discovered HTTP endpoints.
- **Gateway:** platform-neutral runtime ingress/egress with Feishu/WeCom/etc. adapters as thin platform adapters.
- **Providers:** Browser, MCP, IM and web tools should be native providers registered through the runtime registry.
- **Self-improvement:** memory, skills, curator, growth events and profile exports remain auditable and reversible.
- **Legacy rule:** OpenClaw names and shapes may exist only as temporary migration/alias layers while callers are converted. Do not add new features to OpenClaw-shaped modules.

## 2. Current Sidecar Responsibility Inventory

`apps/runtime/sidecar` currently mixes several unrelated responsibilities behind local HTTP endpoints:

| Sidecar area | Current files | Current endpoints | Target Hermes-aligned owner |
| --- | --- | --- | --- |
| Browser automation | `apps/runtime/sidecar/src/browser.ts`, `browser_uploads.ts` | `/api/browser/*`, `/api/browser/compat` | Native browser provider registered through Rust `ToolRegistry` and `browser` toolset |
| MCP bridge | `apps/runtime/sidecar/src/mcp.ts` | `/api/mcp/add-server`, `/api/mcp/list-servers`, `/api/mcp/list-tools`, `/api/mcp/call-tool` | Native MCP runtime/manager, dynamic tool registration, `mcp` toolset |
| Channel kernel / IM | `apps/runtime/sidecar/src/adapters/**` | `/api/channels/*` | Runtime gateway + `commands/im_host/*` + platform adapters |
| Feishu legacy | `apps/runtime/sidecar/src/feishu.ts`, `feishu_ws.ts`, `adapters/feishu/**` | `/api/feishu/*`, `/api/feishu/ws/*` | Feishu platform adapter and runtime gateway, no sidecar URL setting |
| WeCom legacy | `apps/runtime/sidecar/src/adapters/wecom/**` | `/api/channels/*` with `wecom` adapter | WeCom platform adapter and runtime gateway |
| OpenClaw routing | `apps/runtime/sidecar/src/openclaw-bridge/**`, `vendor/openclaw-*` | `/api/openclaw/resolve-route` | Native IM/profile route resolver; OpenClaw route endpoint deleted |
| Lifecycle / packaging | `apps/runtime/src-tauri/src/sidecar.rs`, bundle scripts | `/health`, sidecar process startup | Deleted after all consumers are migrated |

## 3. Target Module Map

| Target layer | WorkClaw target modules | Notes |
| --- | --- | --- |
| Core agent runtime | `apps/runtime/src-tauri/src/agent/runtime/**` | Keep agent loop and registry in Rust. Do not add new sidecar execution paths. |
| Tool registry | `agent/runtime/kernel/tool_registry_setup.rs`, `agent/tools/**` | Replace `SidecarBridgeTool` with native adapters per provider. |
| Toolsets | `agent/tools/toolsets_tool.rs`, manifest metadata, skill frontmatter | Browser/MCP/IM tools must become observable through `browser`, `mcp`, `im` toolsets by metadata, not bridge naming. |
| Gateway | `commands/im_host/**`, `commands/feishu_gateway/**`, `commands/wecom_gateway/**` | Platform adapters are thin I/O surfaces; routing and dispatch stay runtime-owned. |
| Profile home | `runtime_paths.rs`, `profile_runtime/**`, `commands/agent_profile.rs`, `commands/employee_agents/profile_service.rs` | All new runtime state must hang from profile boundaries. |
| MCP provider | New native MCP service modules under `src-tauri` plus existing `commands/mcp.rs` facade | Model after `references/hermes-agent/tools/mcp_tool.py`: manage servers, list tools, register dynamic tools, call tools in runtime. |
| Browser provider | Native browser runtime/adapter modules plus existing `browser_tools.rs` schema layer | Preserve Hermes-compatible tool names while replacing HTTP execution backend. |
| Scheduler/curator | Existing curator scheduler first; future generic scheduler only if roadmap approves | Sidecar removal does not require a generic cron rewrite. |
| Legacy migration | Small, explicitly named migration modules only | Do not let OpenClaw compatibility define new product contracts. |

## 4. Migration Batches

### Batch 0. Planning and guardrails

**Status:** `[x]`

**Objective:** Record the sidecar removal direction and prevent future work from extending the sidecar or OpenClaw compatibility surfaces.

**Files:**
- Create: `docs/plans/2026-05-11-hermes-aligned-sidecar-removal-roadmap.md`
- Modify: `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`
- Modify: `AGENTS.md`

**Acceptance:**
- `[x]` A plan exists that maps sidecar responsibilities to Hermes-aligned target layers.
- `[x]` AGENTS guidance says OpenClaw is legacy migration input only and new sidecar dependencies are not allowed.
- `[x]` Roadmap includes a sidecar removal phase or slice.

**Verification:**
- `git diff --check`
- `git diff -- AGENTS.md docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md docs/plans/2026-05-11-hermes-aligned-sidecar-removal-roadmap.md`

### Batch 1. Native IM route resolver replaces `/api/openclaw/resolve-route`

**Status:** `[x]`

**Objective:** Remove the first concrete sidecar dependency by replacing the OpenClaw route endpoint with a native Rust resolver while keeping existing caller contracts stable.

**Roadmap phase:** Phase 7B, acceptance: native route resolution no longer calls sidecar.

**Non-goals:**
- Do not delete `apps/runtime/sidecar` yet.
- Do not remove MCP/browser/Feishu/WeCom sidecar paths yet.
- Do not rename all `openclaw_*` symbols in the same batch.
- Do not change DB schemas or Tauri command names.

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
- Create: `apps/runtime/src-tauri/src/commands/openclaw_gateway/route_resolver.rs`
- Modify: `apps/runtime/src-tauri/tests/test_openclaw_gateway.rs`
- Modify: `apps/runtime/src-tauri/tests/test_openclaw_route_regression.rs`

**Implementation notes:**
1. Add a pure Rust resolver that consumes `ImEvent` plus `list_im_routing_bindings_with_pool()` output.
2. Preserve route decision JSON shape used by dispatch code:
   - `agentId`
   - `matchedBy`
   - optional future-neutral metadata only if tests require it.
3. Preserve current matching semantics:
   - peer binding wins over account binding;
   - account binding wins over channel binding;
   - channel binding wins over default;
   - `event.account_id` falls back to `event.tenant_id`;
   - empty channel falls back to `app`;
   - `*` account remains wildcard where legacy tests depend on it;
   - same-tier ordering follows `priority ASC, updated_at DESC` from `list_im_routing_bindings_with_pool()`.
4. Replace `call_sidecar_json("/api/openclaw/resolve-route", ...)` in `resolve_openclaw_route_with_pool()` with the native resolver.
5. Replace `simulate_im_route()` sidecar forwarding with the same native resolver.
6. Remove now-unused imports of `call_sidecar_json` and sidecar base URL resolution from `openclaw_gateway.rs` if no longer needed there.

**Tests:**
- Convert mock-sidecar route tests into pure native resolver tests.
- Keep existing route regression vectors for peer/account/channel/default precedence.
- Add a search check that no Rust route resolver still calls `/api/openclaw/resolve-route`.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_openclaw_gateway --test test_openclaw_route_regression
cargo test --test test_im_route_session_mapping
cargo test --test test_im_employee_agents -- im_routing
cargo check
cd /mnt/d/code/workclaw
git grep -n 'api/openclaw/resolve-route\|call_sidecar_json("/api/openclaw/resolve-route"' -- apps/runtime/src-tauri/src apps/runtime/src-tauri/tests
```

**Exit criteria:**
- Rust/Tauri code no longer calls `/api/openclaw/resolve-route`.
- IM dispatch behavior remains covered by native regression tests.
- Sidecar route endpoint can be treated as unused by Rust callers.

### Batch 2. Rename routing compatibility layer to Hermes-native IM/profile language

**Status:** `[x]`

**Objective:** Stop presenting IM routing as an OpenClaw gateway internally.

**Non-goals:**
- Do not remove all OpenClaw plugin integration in one pass.
- Do not break public Tauri commands until UI callers are migrated.

**Candidate files:**
- `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
- `apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs`
- `apps/runtime/src-tauri/src/commands/feishu_gateway/relay_service.rs`
- `apps/runtime/src-tauri/src/im/agent_session_runtime.rs`
- `apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs`
- Relevant tests under `apps/runtime/src-tauri/tests/`

**Implementation notes:**
1. Introduce neutral functions such as `resolve_im_route_with_pool`, `plan_im_role_events`, and `plan_im_role_dispatch_requests`.
2. Keep temporary wrappers for old function names only if existing callers need a short bridge.
3. Update `matchedBy` strings only when downstream tests and UI expectations are migrated.
4. Add comments marking any remaining `openclaw_*` names as temporary legacy adapters.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_openclaw_gateway --test test_openclaw_route_regression
cargo test --test test_im_route_session_mapping
cargo test --test test_feishu_gateway
cargo test --test test_wecom_gateway
cargo check
```

**Exit criteria:**
- New code imports neutral IM/profile routing functions.
- Remaining OpenClaw names are thin adapters, not core routing ownership.

### Batch 3. Remove OpenClaw browser compatibility and vendor lanes

**Objective:** Delete OpenClaw-specific browser compatibility, route engine tests, and vendor sync lanes after native routing has replaced consumers.

**Candidate files:**
- `apps/runtime/src-tauri/src/agent/tools/browser_compat.rs`
- `apps/runtime/sidecar/src/openclaw-bridge/**`
- `apps/runtime/sidecar/vendor/openclaw-core/**`
- `apps/runtime/sidecar/vendor/openclaw-im-core/**`
- `apps/runtime/sidecar/test/openclaw.*.test.ts`
- `apps/runtime/sidecar/test/browser.compat-api.test.ts`
- `scripts/sync-openclaw-core.mjs`
- `scripts/sync-openclaw-im-core.mjs`
- `scripts/check-openclaw-vendor-lane.test.mjs`
- `scripts/check-openclaw-wecom-vendor-lane.test.mjs`
- Root `package.json` OpenClaw sync/check scripts

**Implementation notes:**
1. Confirm `git grep -n 'openclaw'` results and classify each remaining hit as legacy migration, UI copy, test fixture, or removable compatibility.
2. Remove `/api/browser/compat` callers before deleting the endpoint.
3. Remove vendor sync scripts only after release-sensitive checks are updated.
4. Update docs that still describe OpenClaw compatibility as an active architecture.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw
pnpm test:sidecar
pnpm test:rust-fast
pnpm test:release-docs
pnpm test:openclaw-vendor-lane # expected to be removed or intentionally replaced in this batch
```

**Exit criteria:**
- OpenClaw vendor lanes are removed or marked obsolete in release docs.
- No new runtime path depends on OpenClaw compatibility.

### Batch 4. Native MCP runtime

**Objective:** Replace sidecar MCP HTTP bridge with runtime-owned MCP server management and dynamic tool registration.

**Hermes reference:** `references/hermes-agent/tools/mcp_tool.py`, `references/hermes-agent/tools/registry.py`, `references/hermes-agent/toolsets.py`.

**Candidate files:**
- `apps/runtime/src-tauri/src/commands/mcp.rs`
- `apps/runtime/src-tauri/src/lib.rs` MCP restore path
- `apps/runtime/src-tauri/src/agent/tools/sidecar_bridge.rs`
- `apps/runtime/src-tauri/src/agent/runtime/kernel/tool_registry_setup.rs`
- New native MCP service modules under `apps/runtime/src-tauri/src/`

**Implementation notes:**
1. Split command facade, repository, runtime manager, and dynamic tool adapter.
2. Restore saved MCP servers without HTTP sidecar calls.
3. Register dynamic MCP tools directly in Rust `ToolRegistry` with `mcp` toolset metadata.
4. Fix the current camelCase/snake_case contract ambiguity by removing the HTTP boundary rather than preserving both shapes.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_mcp_commands
cargo test --test test_toolsets_tool
cargo check
```

**Exit criteria:**
- MCP add/list/call/list-tools paths work without `localhost:8765`.
- No MCP path uses `SidecarBridgeTool`.

### Batch 5. Native IM platform adapters replace sidecar channel kernel

**Objective:** Move Feishu/WeCom/channel connector I/O behind runtime gateway/platform adapters.

**Hermes reference:** `references/hermes-agent/gateway/platforms/feishu.py`, `references/hermes-agent/gateway/platforms/wecom.py`, `references/hermes-agent/gateway/session.py`, `references/hermes-agent/tools/send_message_tool.py`.

**Candidate files:**
- `apps/runtime/src-tauri/src/commands/im_host/**`
- `apps/runtime/src-tauri/src/commands/feishu_gateway/**`
- `apps/runtime/src-tauri/src/commands/wecom_gateway.rs`
- `apps/runtime/src-tauri/src/commands/wecom_gateway/**`
- `apps/runtime/src-tauri/src/commands/channel_connectors.rs`
- `apps/runtime/src-tauri/src/commands/openclaw_plugins/*runtime_adapter.rs`
- `apps/runtime/src/components/settings/**`

**Implementation notes:**
1. Replace `sidecar_base_url` configuration with platform adapter configuration.
2. Keep inbound event normalization and outbound delivery under `im_host` and platform modules.
3. Remove sidecar channel diagnostics/catalog/status from UI once equivalent runtime diagnostics exist.
4. Keep credentials and secrets out of tracked config and test fixtures.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_feishu_gateway
cargo test --test test_wecom_gateway
cargo test --test test_channel_connectors
cargo test --test test_im_employee_agents
cargo check
cd /mnt/d/code/workclaw
pnpm --dir apps/runtime exec tsc --noEmit
```

**Exit criteria:**
- Feishu/WeCom runtime paths no longer call `/api/channels/*` or `/api/feishu/*` sidecar endpoints.
- Settings UI no longer presents sidecar as the connector architecture.

### Batch 6. Native browser provider replaces sidecar Playwright HTTP bridge

**Objective:** Preserve Hermes-compatible browser tool names while replacing sidecar HTTP execution with a native provider boundary.

**Hermes reference:** `references/hermes-agent/tools/browser_tool.py`, `references/hermes-agent/model_tools.py`.

**Candidate files:**
- `apps/runtime/src-tauri/src/agent/tools/browser_tools.rs`
- `apps/runtime/src-tauri/src/agent/tools/sidecar_bridge.rs`
- New browser provider modules under `apps/runtime/src-tauri/src/agent/` or `apps/runtime/src-tauri/src/commands/`
- `apps/runtime/src-tauri/tests/test_browser_tools.rs`
- `apps/runtime/src-tauri/tests/test_sidecar_bridge.rs`

**Implementation notes:**
1. Keep tool names and schemas stable: `browser_navigate`, `browser_snapshot`, `browser_click`, etc.
2. Move execution into a provider trait so the provider can be mocked in tests.
3. Only delete Playwright sidecar tests after equivalent provider tests exist.
4. Do not reintroduce OpenClaw-style browser compatibility.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_browser_tools
cargo test --test test_toolsets_tool
cargo check
```

**Exit criteria:**
- Browser tools are registered through Rust registry and execute without sidecar HTTP.
- Browser tools remain visible in the `browser` toolset projection.

### Batch 7. Remove sidecar process lifecycle and packaging

**Objective:** Delete the sidecar process, build bundle, package scripts, and runtime resources after all functional consumers are migrated.

**Candidate files:**
- `apps/runtime/src-tauri/src/sidecar.rs`
- `apps/runtime/src-tauri/src/lib.rs` sidecar bootstrap
- `apps/runtime/src-tauri/tauri.conf.json` sidecar resources
- `apps/runtime/sidecar/**`
- `scripts/run-sidecar-tests.mjs`
- `scripts/prepare-sidecar-runtime-bundle.mjs`
- `scripts/build-runtime.mjs` sidecar build step
- Root `package.json` sidecar/browser/OpenClaw scripts
- Installer/build tests expecting `sidecar-runtime`

**Implementation notes:**
1. Run `git grep -n 'sidecar\|localhost:8765\|sidecar-runtime'` and classify every remaining hit before deletion.
2. Remove lifecycle tests only after there is no sidecar lifecycle.
3. Update Windows contributor docs and release docs to remove sidecar runtime packaging instructions.
4. Run release-sensitive checks because packaging changes are high impact.

**Verification commands:**
```bash
cd /mnt/d/code/workclaw
pnpm test:rust-fast
pnpm --dir apps/runtime exec tsc --noEmit
pnpm test:release-docs
pnpm test:installer
pnpm build:runtime
```

**Exit criteria:**
- No `apps/runtime/sidecar` package remains.
- Runtime startup does not start or health-check a sidecar process.
- Packaged desktop app no longer includes `resources/sidecar-runtime`.

## 5. Implementation Order Recommendation

**Batch 1 is complete**: Rust/Tauri now resolves IM routes natively and no longer calls `/api/openclaw/resolve-route`.

Next start with **Batch 2: Rename routing compatibility layer to Hermes-native IM/profile language**.

Batch 1 was chosen first because:

1. It is the smallest sidecar removal with a clear success check.
2. It removes an OpenClaw compatibility endpoint without touching browser/MCP/IM platform adapters.
3. The behavior is mostly pure routing logic and already has regression tests.
4. It creates the pattern for later: replace sidecar consumer first, delete endpoint later.
5. It advances the Hermes direction by making routing runtime-owned and profile/IM aligned.

Do **not** start with browser or MCP unless there is a blocking bug there. Those areas have external process/provider behavior and are higher risk.

## 6. Acceptance Checklist

- `[x]` No new code path calls `/api/openclaw/resolve-route`.
- `[ ]` No new code path calls `/api/browser/compat`.
- `[ ]` OpenClaw vendor sync lanes are removed or explicitly marked historical.
- `[ ]` MCP server restore/list/call works without sidecar HTTP.
- `[ ]` Browser tools execute without sidecar HTTP and remain in `browser` toolset.
- `[ ]` Feishu/WeCom/channel connectors run through runtime gateway/platform adapters without sidecar base URL.
- `[ ]` Root build/runtime scripts no longer build or package `apps/runtime/sidecar`.
- `[ ]` `apps/runtime/sidecar` is deleted after all consumers are migrated.
- `[ ]` Release docs and contributor docs no longer mention sidecar as an active runtime requirement.

## 7. Risks and Controls

| Risk | Control |
| --- | --- |
| Big-bang sidecar deletion breaks app startup | Migrate one consumer at a time; delete lifecycle last. |
| OpenClaw removal breaks existing imported workflows unexpectedly | Treat OpenClaw as legacy migration input; document intentional removal; preserve only temporary wrappers needed to migrate callers. |
| MCP migration loses dynamic tool registration | Model native manager on Hermes MCP tool flow and verify toolset projection. |
| Browser migration breaks real automation | Keep browser tool schemas stable and create provider-level mocks before removing sidecar tests. |
| IM migration breaks Feishu/WeCom user workflows | Migrate platform adapters behind runtime gateway and run gateway/connector tests before UI cleanup. |
| Packaging cleanup removes needed resources too early | Run release-sensitive checks only in final lifecycle removal batch. |

## 8. Non-Goals

- No new OpenClaw compatibility features.
- No new Node sidecar endpoints.
- No default manual approval queue for self-improvement.
- No `.skillpack` mutation or unpacking as part of sidecar removal.
- No generic scheduler rewrite unless a separate roadmap slice approves it.
- No destructive deletion of generated/runtime-owned data without explicit reviewed cleanup scope.
