# Browser Compat Caller Audit

**Roadmap phase:** Hermes-aligned sidecar removal Batch 3D.

## Purpose and Verdict

This document records the caller audit for `/api/browser/compat` and the OpenClaw-style unified `browser` compatibility tool.

**Verdict:** caller audit is complete, but endpoint removal is not safe yet. The endpoint is still actively registered by runtime code through the unified `browser` tool. Removal is blocked until WorkClaw has either a native browser provider or a neutral runtime-owned `browser` wrapper that preserves the required Hermes-compatible browser behavior without depending on the sidecar compatibility endpoint.

This batch is documentation and planning only. It does not change runtime behavior, package scripts, sidecar implementation, Rust tests, TypeScript tests, vendored files, or prompt assembly code.

## Current Caller Map

### Non-Doc Grep Snapshot

At audit time, `git grep -n "/api/browser/compat"` shows these active non-documentation hits:

- `apps/runtime/src-tauri/src/agent/tools/browser_compat.rs`: the one runtime code consumer, registered through `SidecarBridgeTool`.
- `apps/runtime/sidecar/src/index.ts`: the one sidecar endpoint, `POST /api/browser/compat`.
- `apps/runtime/sidecar/test/browser.compat-api.test.ts`: the sidecar endpoint forwarding test.

Additional hits are historical or roadmap documentation.

### Runtime Registration

- `apps/runtime/src-tauri/src/agent/tools/browser_compat.rs` registers a `SidecarBridgeTool` endpoint `/api/browser/compat` under the unified tool name `browser`.
- `register_browser_compat_tool(...)` is called by `apps/runtime/src-tauri/src/agent/runtime/tool_registry_builder.rs` inside `register_browser_and_alias_tools(...)`.
- `register_browser_compat_tool(...)` is called by `apps/runtime/src-tauri/src/agent/runtime/kernel/tool_registry_setup.rs` during runtime tool registration.
- `apps/runtime/src-tauri/tests/test_browser_compat.rs` verifies the unified `browser` tool schema.

The current unified `browser` compatibility schema exposes this action enum:

```text
status, start, stop, profiles, tabs, open, focus, snapshot, act, upload
```

### Sidecar Endpoint

- `apps/runtime/sidecar/src/index.ts` exposes `POST /api/browser/compat`.
- The handler parses the request body and forwards it to `deps.browser.compat(body)`.
- The response is returned as JSON-stringified `output`, matching the existing `SidecarBridgeTool` response contract.

### Sidecar Controller

- `apps/runtime/sidecar/src/browser.ts` implements `BrowserController.compat(...)`.
- The controller supports the same compatibility actions: `status`, `start`, `stop`, `profiles`, `tabs`, `open`, `focus`, `snapshot`, `act`, and `upload`.
- The controller currently supports the `openclaw` profile for this compatibility path.

### Tests

- `apps/runtime/sidecar/test/browser.compat-api.test.ts` verifies that `POST /api/browser/compat` forwards action payloads to the browser controller.
- `apps/runtime/src-tauri/tests/test_browser_compat.rs` verifies that the runtime registers the unified `browser` schema and action set.

These tests should stay until replacement native-provider tests exist.

### Prompt and Docs Dependencies

- `packages/runtime-chat-app/src/prompt_assembly.rs` tells agents to use WorkClaw's built-in `browser` compatibility tool and aliases for OpenClaw/Xiaohongshu-like skills. This is a prompt-level dependency and must be neutralized in a later code batch, not this docs-only batch.
- `docs/browser-automation-integration.md` still describes the current local sidecar plus Playwright implementation and the P0 OpenClaw browser compatibility layer.
- Sidecar-removal roadmap and OpenClaw remnant classification docs reference this compatibility surface as a temporary migration/removal target.

## Retain vs Migrate Decision

| Surface | Decision | Reason |
| --- | --- | --- |
| Runtime unified `browser` tool | Retain temporarily as a compatibility wrapper. Later replace its implementation with a native provider or neutral wrapper without OpenClaw naming. | Runtime code still registers it and prompt guidance still directs agents to it. |
| `/api/browser/compat` endpoint | Retain temporarily until no runtime code calls it. | The registered unified `browser` tool currently bridges to this endpoint. |
| `BrowserController.compat(...)` | Retain temporarily only while the sidecar owns browser execution. | It is the current execution implementation behind the endpoint. |
| Rust and sidecar tests | Keep until replacement native-provider tests exist. | They protect current behavior while migration is blocked. |
| Docs and prompt guidance | Rewrite later to Hermes-neutral browser provider guidance. | Current wording still embeds sidecar and OpenClaw-style assumptions. |

## Deletion Prerequisites

Do not delete `/api/browser/compat`, `BrowserController.compat(...)`, or the sidecar compatibility tests until all of these are true:

- A native browser provider trait/mock or equivalent runtime-owned provider exists.
- Tests prove Hermes-compatible browser tool names remain registered and executable.
- Tests prove either the unified `browser` tool behavior still works through a neutral implementation or that the unified tool is explicitly deprecated with covered migration behavior.
- Prompt guidance no longer tells agents to depend on OpenClaw-style browser assumptions or a fixed local browser sidecar.
- `git grep /api/browser/compat` shows no runtime consumer outside deletion diffs and historical documentation.

## Safe Future Implementation Order

1. Introduce a runtime-owned browser provider boundary first.
2. Migrate the unified `browser` wrapper to the native provider or a neutral compatibility wrapper.
3. Migrate Rust and sidecar compatibility tests to provider-level and wrapper-level tests.
4. Neutralize prompt and docs guidance so agents no longer depend on OpenClaw-style browser assumptions.
5. Remove `/api/browser/compat` after grep proves no runtime consumer remains.
6. Remove `BrowserController.compat(...)` and `apps/runtime/sidecar/test/browser.compat-api.test.ts` after equivalent replacement checks pass.

## Verification Commands

Docs-only Batch 3D verification:

```bash
cd /mnt/d/code/workclaw
git diff --check
git diff --quiet -- package.json pnpm-lock.yaml pnpm-workspace.yaml scripts apps/runtime/src-tauri apps/runtime/sidecar packages/runtime-chat-app/src/prompt_assembly.rs
git grep -n "/api/browser/compat"
git grep -n "register_browser_compat_tool"
corepack pnpm test:release-docs
```

Later code-removal batch verification should add replacement-provider checks before endpoint deletion:

```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_browser_tools
cargo test --test test_browser_compat
cargo test --test test_toolsets_tool
cargo check

cd /mnt/d/code/workclaw
pnpm test:sidecar
git grep -n "/api/browser/compat" -- apps/runtime/src-tauri apps/runtime/sidecar packages/runtime-chat-app/src/prompt_assembly.rs
git grep -n "OpenClaw / Xiaohongshu\\|browser.*compat\\|localhost:8765" -- packages/runtime-chat-app/src/prompt_assembly.rs docs/browser-automation-integration.md
```

The later removal batch should only pass the endpoint grep when there are no runtime consumers left. Historical documentation may keep references if they are clearly marked as legacy.
