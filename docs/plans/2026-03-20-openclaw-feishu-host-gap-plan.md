# OpenClaw Feishu Host Gap Closure Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade WorkClaw from a “minimum compatible host” for the official Feishu plugin to an OpenClaw-equivalent host for the parts that matter in real user workflows.

**Architecture:** Treat the official Feishu plugin as the source of truth and close the gap in four layers: host runtime contract, reply dispatch equivalence, `channels.feishu` config coverage, and CLI/onboarding parity. Keep WorkClaw-specific UI and employee routing only as bridges around an OpenClaw-compatible core, not as replacement behavior.

**Tech Stack:** Rust Tauri backend, TypeScript/Node `plugin-host`, official `@larksuite/openclaw-lark` plugin runtime, React settings UI, WorkClaw IM routing/session layer.

---

## Scope Definition

### Already Working

- Official plugin install, inspect, start/stop, status, and runtime log capture are present in [openclaw_plugins.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/openclaw_plugins.rs).
- Minimum `plugin-sdk` shim and long-lived host runner exist in [runtime.ts](/d:/code/WorkClaw/apps/runtime/plugin-host/src/runtime.ts) and [run-feishu-host.mjs](/d:/code/WorkClaw/apps/runtime/plugin-host/scripts/run-feishu-host.mjs).
- Pairing request ingestion, approval UI, and allow-from persistence already exist in [SettingsView.tsx](/d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx) and [feishu_gateway.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/feishu_gateway.rs).
- Messages already reach WorkClaw IM routing and employee dispatch bridges through [runtime_bridge.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/im/runtime_bridge.rs) and [App.tsx](/d:/code/WorkClaw/apps/runtime/src/App.tsx).

### Main Gap Categories

1. `P0` Reply/runtime contract is still only minimally compatible.
2. `P0` Feishu reply delivery semantics are not yet OpenClaw-equivalent.
3. `P1` `channels.feishu` config surface is only partially mapped.
4. `P1` Official in-chat commands are not fully guaranteed end-to-end.
5. `P2` `openclaw` CLI and “Create new bot” onboarding are not yet host-equivalent.

---

## P0: Host Runtime And Reply Contract

### Task P0.1: Make `plugin-host` reply runtime match OpenClaw semantics

**Files:**
- Modify: `apps/runtime/plugin-host/src/runtime.ts`
- Modify: `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`
- Test: `apps/runtime/plugin-host/src/runtime.test.ts`
- Reference: `references/openclaw/src/plugins/runtime/runtime-channel.ts`
- Reference: `references/openclaw/src/auto-reply/reply/reply-dispatcher.ts`

**Goal:** Ensure the host provides the same reply/runtime primitives that the official plugin expects, instead of ad hoc placeholders.

**Must cover:**
- `createReplyDispatcherWithTyping`
- `withReplyDispatcher`
- `dispatchReplyFromConfig`
- `dispatchReplyWithBufferedBlockDispatcher`
- `resolveHumanDelayConfig`
- reply lifecycle helpers such as `waitForIdle`, `markComplete`, `markDispatchIdle`, `markRunComplete`

**Success criteria:**
- Official plugin no longer fails on missing reply dispatcher shape.
- Host tests explicitly assert OpenClaw-compatible dispatcher shape and sequencing.

### Task P0.2: Make outbound Feishu replies follow OpenClaw target semantics

**Files:**
- Modify: `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src-tauri/src/im/feishu_adapter.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Test: `apps/runtime/plugin-host/src/runtime.test.ts`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Test: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Reference: `references/openclaw/extensions/feishu/src/reply-dispatcher.ts`
- Reference: `references/openclaw/extensions/feishu/src/channel.ts`

**Goal:** Ensure replies go back to the exact Feishu target that OpenClaw would use.

**Must cover:**
- `chat_id` vs `open_id` resolution
- DM vs group vs thread reply targets
- reply-in-thread behavior
- text vs markdown/card/media branching

**Success criteria:**
- Approved private chats produce visible replies in Feishu.
- Runtime diagnostics show outbound success instead of silent generation-only success.

### Task P0.3: Align host dispatch pipeline with OpenClaw’s inbound execution model

**Files:**
- Modify: `apps/runtime/plugin-host/src/runtime.ts`
- Modify: `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`
- Modify: `apps/runtime/src-tauri/src/im/runtime_bridge.rs`
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`
- Reference: `references/openclaw/src/auto-reply/dispatch.ts`
- Reference: `references/openclaw/src/auto-reply/reply/dispatch-from-config.ts`
- Reference: `references/openclaw/extensions/feishu/src/bot.ts`

**Goal:** Reduce the difference between “plugin dispatches into OpenClaw” and “plugin dispatches into WorkClaw bridge”.

**Must cover:**
- session key continuity
- message dedupe on real Feishu message IDs
- command vs normal reply flow
- tool/block/final reply lifecycle ordering

**Success criteria:**
- Same inbound message produces the same route/dispatch class in WorkClaw and OpenClaw for core DM cases.

---

## P1: Feishu Config Parity

### Task P1.1: Expand WorkClaw’s `channels.feishu` config projection

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/components/connectors/connectorSchemas.ts`
- Modify: `apps/runtime/src/types.ts`
- Reference: `references/openclaw/extensions/feishu/src/config-schema.ts`
- Reference: `references/openclaw/extensions/feishu/src/setup-surface.ts`

**Goal:** Stop exposing only a narrow subset of official config.

**Priority fields to add first:**
- `streaming`
- `footer.elapsed`
- `footer.status`
- `threadSession`
- `replyInThread`
- `groupPolicy`
- `groupAllowFrom`
- `groupSenderAllowFrom`
- `groups.*`
- `typingIndicator`
- `reactionNotifications`
- `tools.*`
- `connectionMode`
- `domain`
- multi-account `accounts.*` inheritance

**Success criteria:**
- WorkClaw settings can express the same core config examples documented for OpenClaw’s Feishu plugin.

### Task P1.2: Add config round-trip verification against official plugin snapshots

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Reference: `references/openclaw-lark/src/core/accounts.ts`
- Reference: `references/openclaw-lark/src/core/config-schema.ts`

**Goal:** Verify that projected WorkClaw config is interpreted by the official plugin the same way OpenClaw would.

**Success criteria:**
- Snapshot/inspection tests cover default account, inherited account fields, and policy interpretation.

---

## P1: Official Command And Auth Parity

### Task P1.3: Make `/feishu start`, `/feishu auth`, `/feishu doctor` fully host-backed

**Files:**
- Modify: `apps/runtime/plugin-host/src/runtime.ts`
- Modify: `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Reference: `references/openclaw-lark/src/commands/index.ts`
- Reference: `references/openclaw-lark/src/commands/auth.ts`
- Reference: `references/openclaw-lark/src/commands/doctor.ts`

**Goal:** Move from “UI guidance exists” to “official commands are known-good through our host”.

**Must cover:**
- command authorization context
- owner-only restrictions
- OAuth/account context
- diagnostics output and recent logs integration

**Success criteria:**
- In-chat `/feishu start`, `/feishu auth`, and `/feishu doctor` execute reliably in WorkClaw using the official plugin path.

---

## P2: CLI And Onboarding Parity

### Task P2.1: Add controlled `openclaw` CLI compatibility layer for installer-dependent paths

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Create or modify: `apps/runtime/plugin-host/scripts/openclaw-cli-shim.*`
- Reference: `references/openclaw/src/cli/plugins-cli.ts`
- Reference: `references/openclaw/src/cli/pairing-cli.ts`
- Reference: `references/openclaw/src/gateway/server-channels.ts`

**Goal:** Support the subset of `openclaw` CLI behavior that the official installer and docs depend on.

**Minimum command surface:**
- `openclaw -v`
- `openclaw config get`
- `openclaw config set`
- `openclaw gateway restart`
- `openclaw pairing approve`
- `openclaw pairing list`

**Success criteria:**
- “Create new bot” installer path no longer fails on missing `openclaw`.

### Task P2.2: Make “Create new bot” onboarding equivalent to official flow

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Reference: official guide link

**Official reference:** <https://bytedance.larkoffice.com/docx/MFK7dDFLFoVlOGxWCv5cTXKmnMh#M0usd9GLwoiBxtx1UyjcpeMhnRe>

**Goal:** Reproduce the official create-bot flow in a controlled WorkClaw UI.

**Must cover:**
- QR-code creation path
- installer stdout capture
- restart after successful creation
- next-step guidance for `/feishu start` and `/feishu auth`

---

## Recommended Execution Order

1. `P0.1` Host reply contract
2. `P0.2` Outbound target/delivery parity
3. `P0.3` Dispatch pipeline parity
4. `P1.1` Config projection coverage
5. `P1.2` Config round-trip verification
6. `P1.3` Official command/auth parity
7. `P2.1` CLI shim
8. `P2.2` Create-bot onboarding

---

## Verification Commands

- `pnpm --dir apps/runtime/plugin-host test`
- `cargo test -p runtime --manifest-path D:\code\WorkClaw\apps\runtime\src-tauri\Cargo.toml --lib openclaw_plugins -- --nocapture`
- `cargo test -p runtime --manifest-path D:\code\WorkClaw\apps\runtime\src-tauri\Cargo.toml --lib feishu_gateway -- --nocapture`
- `pnpm --dir apps/runtime exec vitest run ./src/components/__tests__/SettingsView.wecom-connector.test.tsx --passWithNoTests`
- `pnpm --dir apps/runtime exec vitest run ./src/__tests__/App.im-feishu-bridge.test.tsx --passWithNoTests`
- `pnpm test:rust-fast`

For user-visible reply fixes, also require a manual Feishu DM verification pass after each `P0` task.

