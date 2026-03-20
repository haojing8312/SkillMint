# OpenClaw Plugin Compatibility Host Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first-phase WorkClaw compatibility host that can install and run the official OpenClaw Lark/Feishu plugin.

**Architecture:** Add a WorkClaw-owned Node plugin host that loads OpenClaw native plugins from manifest metadata, exposes an `openclaw/plugin-sdk` compatibility shim, and bridges registered `ChannelPlugin` behavior into WorkClaw’s existing IM, gateway, session, and settings infrastructure. Deliver Feishu-first compatibility, then harden the generic plugin host surface around it.

**Tech Stack:** Rust Tauri backend, TypeScript/Node plugin host, npm package installation, JSON schema validation, WorkClaw IM routing/session systems.

---

### Task 1: Create A Plugin Host Architecture Note In Code

**Files:**
- Modify: `docs/plans/2026-03-19-openclaw-plugin-compat-design.md`
- Create: `apps/runtime/plugin-host/README.md`

**Step 1: Write the failing test**

No code test for this task. Capture the host scope and directory ownership in documentation first.

**Step 2: Run test to verify it fails**

No runtime test. Verify the target directory does not exist yet.

Run: `Test-Path apps/runtime/plugin-host`
Expected: `False`

**Step 3: Write minimal implementation**

Create `apps/runtime/plugin-host/README.md` describing:

- why the host exists,
- why it must expose `openclaw/plugin-sdk`,
- which runtime surfaces are phase-1 mandatory,
- how it talks to the Tauri backend.

**Step 4: Run test to verify it passes**

Run: `Get-Content apps/runtime/plugin-host/README.md`
Expected: file exists with the host overview.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-19-openclaw-plugin-compat-design.md apps/runtime/plugin-host/README.md
git commit -m "docs: define openclaw plugin host architecture"
```

### Task 2: Add OpenClaw Plugin Install Records In WorkClaw

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`
- Create: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

**Step 1: Write the failing test**

Add Rust tests for:

- recording an installed OpenClaw plugin entry,
- persisting npm spec, plugin id, version, install path,
- listing installed plugins separately from local skills.

**Step 2: Run test to verify it fails**

Run: `pnpm test:rust-fast -- openclaw_plugins`
Expected: FAIL because command/storage module does not exist yet.

**Step 3: Write minimal implementation**

Implement a new command module for OpenClaw-native plugin records and database helpers. Keep this separate from `installed_skills`.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast -- openclaw_plugins`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/openclaw_plugins.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/commands/skills.rs
git commit -m "feat: add openclaw plugin install records"
```

### Task 3: Scaffold The Node Plugin Host Package

**Files:**
- Create: `apps/runtime/plugin-host/package.json`
- Create: `apps/runtime/plugin-host/tsconfig.json`
- Create: `apps/runtime/plugin-host/src/index.ts`
- Create: `apps/runtime/plugin-host/src/types.ts`
- Test: `apps/runtime/plugin-host/src/index.test.ts`

**Step 1: Write the failing test**

Add a host bootstrap test that expects:

- host startup config parsing,
- plugin root resolution,
- manifest-first load entrypoint presence.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/plugin-host test`
Expected: FAIL because package and bootstrap files do not exist yet.

**Step 3: Write minimal implementation**

Create the package, TS config, and a bootstrap module that accepts:

- plugin root dir,
- runtime bridge config,
- registration mode,
- install root.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime/plugin-host test`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host
git commit -m "feat: scaffold openclaw plugin host package"
```

### Task 4: Implement Manifest-First Plugin Discovery

**Files:**
- Create: `apps/runtime/plugin-host/src/manifest.ts`
- Create: `apps/runtime/plugin-host/src/discovery.ts`
- Test: `apps/runtime/plugin-host/src/manifest.test.ts`
- Test: `apps/runtime/plugin-host/src/discovery.test.ts`

**Step 1: Write the failing test**

Add tests for:

- reading `openclaw.plugin.json`,
- reading `package.json.openclaw`,
- validating required fields,
- resolving `extensions`, `setupEntry`, and install hints.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/plugin-host test manifest discovery`
Expected: FAIL because parsing modules are missing.

**Step 3: Write minimal implementation**

Implement manifest parsing for local installed plugins. Do not load plugin code yet.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime/plugin-host test manifest discovery`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/src/manifest.ts apps/runtime/plugin-host/src/discovery.ts apps/runtime/plugin-host/src/*.test.ts
git commit -m "feat: add openclaw plugin manifest discovery"
```

### Task 5: Implement Registration Modes And Loader

**Files:**
- Create: `apps/runtime/plugin-host/src/loader.ts`
- Create: `apps/runtime/plugin-host/src/registration-mode.ts`
- Test: `apps/runtime/plugin-host/src/loader.test.ts`

**Step 1: Write the failing test**

Add tests for:

- `full` mode load,
- `setup-only` mode load,
- `setup-runtime` mode load,
- `setupEntry` fallback selection.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/plugin-host test loader`
Expected: FAIL because loader does not exist.

**Step 3: Write minimal implementation**

Implement a TypeScript/ESM-capable loader using a runtime import strategy compatible with official plugins.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime/plugin-host test loader`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/src/loader.ts apps/runtime/plugin-host/src/registration-mode.ts apps/runtime/plugin-host/src/loader.test.ts
git commit -m "feat: add openclaw plugin loader with registration modes"
```

### Task 6: Build The WorkClaw Plugin Registry

**Files:**
- Create: `apps/runtime/plugin-host/src/registry.ts`
- Create: `apps/runtime/plugin-host/src/api.ts`
- Test: `apps/runtime/plugin-host/src/registry.test.ts`

**Step 1: Write the failing test**

Add tests covering:

- `registerChannel`,
- `registerTool`,
- `registerCli`,
- `registerGatewayMethod`,
- `registerCommand`,
- `on(...)` hook registration.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/plugin-host test registry`
Expected: FAIL because registry is not implemented.

**Step 3: Write minimal implementation**

Implement the registration store and expose an `OpenClawPluginApi`-shaped object.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime/plugin-host test registry`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/src/registry.ts apps/runtime/plugin-host/src/api.ts apps/runtime/plugin-host/src/registry.test.ts
git commit -m "feat: add workclaw openclaw-plugin registry"
```

### Task 7: Implement The `openclaw/plugin-sdk` Compatibility Shim

**Files:**
- Create: `apps/runtime/plugin-host/openclaw/package.json`
- Create: `apps/runtime/plugin-host/openclaw/plugin-sdk/index.ts`
- Create: `apps/runtime/plugin-host/openclaw/plugin-sdk/feishu.ts`
- Create: `apps/runtime/plugin-host/openclaw/plugin-sdk/compat.ts`
- Test: `apps/runtime/plugin-host/openclaw/plugin-sdk/index.test.ts`

**Step 1: Write the failing test**

Add tests asserting that plugin code can import:

- `openclaw/plugin-sdk`
- `openclaw/plugin-sdk/feishu`
- `openclaw/plugin-sdk/compat`

and receive executable helpers, not just types.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/plugin-host test plugin-sdk`
Expected: FAIL because shim exports do not exist.

**Step 3: Write minimal implementation**

Implement the smallest executable shim needed by the official Feishu plugin. Prefer adapters that delegate into WorkClaw-owned helpers.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime/plugin-host test plugin-sdk`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/openclaw
git commit -m "feat: add openclaw plugin-sdk compatibility shim"
```

### Task 8: Bridge Plugin Config To WorkClaw Storage

**Files:**
- Create: `apps/runtime/plugin-host/src/config-view.ts`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Test: `apps/runtime/plugin-host/src/config-view.test.ts`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

**Step 1: Write the failing test**

Add tests for:

- projecting WorkClaw plugin state into `plugins.entries.<id>.config`,
- projecting channel state into `channels.feishu.*`,
- round-tripping updates from plugin setup flows.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/plugin-host test config-view`
Expected: FAIL because config bridge is missing.

**Step 3: Write minimal implementation**

Implement a computed OpenClaw-compatible config view backed by WorkClaw persistence.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime/plugin-host test config-view`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/src/config-view.ts apps/runtime/src-tauri/src/commands/openclaw_plugins.rs
git commit -m "feat: bridge plugin config into openclaw-compatible view"
```

### Task 9: Bridge Channel Plugins Into WorkClaw IM/Gateway Flow

**Files:**
- Create: `apps/runtime/plugin-host/src/channel-host.ts`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_gateway.rs`
- Test: `apps/runtime/plugin-host/src/channel-host.test.ts`

**Step 1: Write the failing test**

Add integration-style tests for:

- registered channel plugin activation,
- inbound event handoff into WorkClaw,
- outbound reply delegation back through the plugin.

**Step 2: Run test to verify it fails**

Run: `pnpm test:rust-fast -- im_gateway feishu_gateway openclaw_gateway`
Expected: FAIL because no plugin-host bridge exists.

**Step 3: Write minimal implementation**

Implement the bridge that maps registered `ChannelPlugin` behavior onto WorkClaw routing/session/message systems.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast -- im_gateway feishu_gateway openclaw_gateway`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/src/channel-host.ts apps/runtime/src-tauri/src/commands/openclaw_gateway.rs apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/src/commands/im_gateway.rs
git commit -m "feat: bridge openclaw channel plugins into workclaw runtime"
```

### Task 10: Support Official Feishu Plugin Installation From WorkClaw

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Create: `apps/runtime/src-tauri/src/commands/openclaw_plugin_install.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugin_install.rs`

**Step 1: Write the failing test**

Add tests for:

- installing `@larksuite/openclaw-lark`,
- validating metadata before activation,
- storing install provenance,
- reinstall/upgrade behavior.

**Step 2: Run test to verify it fails**

Run: `pnpm test:rust-fast -- openclaw_plugin_install`
Expected: FAIL because install path does not exist.

**Step 3: Write minimal implementation**

Add a dedicated install command for OpenClaw-native plugins. Do not overload local skill install behavior.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast -- openclaw_plugin_install`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/openclaw_plugin_install.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/commands/clawhub.rs
git commit -m "feat: add official openclaw plugin install flow"
```

### Task 11: Add WorkClaw UI For OpenClaw Plugin Lifecycle

**Files:**
- Create: `apps/runtime/src/components/connectors/OpenClawPluginPanel.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/components/connectors/__tests__/OpenClawPluginPanel.test.tsx`

**Step 1: Write the failing test**

Add UI tests for:

- showing installed plugin status,
- install/upgrade actions,
- setup/runtime state,
- error visibility for manifest/load/runtime failures.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- OpenClawPluginPanel`
Expected: FAIL because panel does not exist.

**Step 3: Write minimal implementation**

Add a focused management surface for OpenClaw-native plugins.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- OpenClawPluginPanel`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/connectors/OpenClawPluginPanel.tsx apps/runtime/src/components/connectors/__tests__/OpenClawPluginPanel.test.tsx apps/runtime/src/App.tsx
git commit -m "feat: add openclaw plugin management panel"
```

### Task 12: Verify Official Feishu Plugin End-To-End

**Files:**
- Modify: `apps/runtime/e2e/im-connectors.feishu.spec.ts`
- Create: `apps/runtime/plugin-host/test-fixtures/openclaw-lark-fixture.ts`
- Test: `apps/runtime/e2e/im-connectors.feishu.spec.ts`

**Step 1: Write the failing test**

Add an end-to-end path that asserts:

- official plugin install,
- setup/runtime load success,
- inbound event acceptance,
- outbound send success,
- connector diagnostics remain coherent.

**Step 2: Run test to verify it fails**

Run: `pnpm test:e2e:runtime -- im-connectors.feishu.spec.ts`
Expected: FAIL until the host is fully wired.

**Step 3: Write minimal implementation**

Add only the fixture and assertions needed to validate the new host path.

**Step 4: Run test to verify it passes**

Run: `pnpm test:e2e:runtime -- im-connectors.feishu.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/e2e/im-connectors.feishu.spec.ts apps/runtime/plugin-host/test-fixtures/openclaw-lark-fixture.ts
git commit -m "test: verify official feishu plugin compatibility end-to-end"
```

### Task 13: Final Verification

**Files:**
- Verify touched files above

**Step 1: Run host package tests**

Run: `pnpm --dir apps/runtime/plugin-host test`
Expected: PASS

**Step 2: Run Rust fast-path verification**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 3: Run runtime targeted tests**

Run: `pnpm --dir apps/runtime test -- OpenClawPluginPanel`
Expected: PASS

**Step 4: Run Feishu runtime E2E**

Run: `pnpm test:e2e:runtime -- im-connectors.feishu.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add .
git commit -m "feat: add openclaw plugin compatibility host for official feishu plugin"
```
