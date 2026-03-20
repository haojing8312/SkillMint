# OpenClaw Plugin Compatibility Host Design

**Date:** 2026-03-19

**Goal**

Enable WorkClaw to install and run official OpenClaw plugins, starting with the official Lark/Feishu plugin, without reimplementing plugin behavior inside WorkClaw.

## Problem

WorkClaw already has local skills, IM routing, Feishu/OpenClaw event ingestion, and sidecar-backed channel connector diagnostics. That is not sufficient to run official OpenClaw plugins.

Official OpenClaw plugins are not black-box feature bundles. They assume an OpenClaw-shaped host that provides:

- Manifest-first plugin discovery and install
- `openclaw.plugin.json` and `package.json.openclaw.*` metadata support
- `openclaw/plugin-sdk` runtime exports
- `OpenClawPluginApi`
- `PluginRuntime`
- `ChannelPlugin` registration and lifecycle
- `full`, `setup-only`, and `setup-runtime` registration modes
- Config layering across `plugins.entries.*` and `channels.*`
- Hook, tool, CLI, service, gateway, and HTTP route registration

Without that host contract, the official Feishu plugin can be installed but cannot actually run.

## Product Direction

WorkClaw should not reimplement official plugin functionality. It should provide a compatibility host so that:

- users can install official OpenClaw plugins in WorkClaw,
- official plugins can execute with minimal or no patching,
- future plugin upgrades can be adopted by reinstalling or upgrading the plugin,
- WorkClaw remains a functional replacement for OpenClaw rather than a forked reimplementation of selected channels.

## Scope

### In Scope

- A WorkClaw-managed Node-based plugin host for OpenClaw native plugins
- Manifest parsing for `openclaw.plugin.json` and `package.json.openclaw`
- npm-based plugin installation and provenance tracking
- A compatibility shim package named `openclaw` exposing `openclaw/plugin-sdk` subpaths
- A runtime bridge from plugin registration into WorkClaw registries and IM/channel infrastructure
- Channel plugin support sufficient to run the official Lark/Feishu plugin

### Out of Scope For Phase 1

- Full compatibility for every OpenClaw plugin type
- Provider plugin parity beyond what is required by the Feishu plugin host
- Full CLI parity with OpenClaw desktop/daemon commands
- Multi-process sandboxing for third-party plugins

## Current State

WorkClaw currently provides these adjacent capabilities:

- Local skill install/import: [`apps/runtime/src-tauri/src/commands/skills.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/skills.rs)
- GitHub/marketplace import flows: [`apps/runtime/src-tauri/src/commands/clawhub.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/clawhub.rs)
- Channel connector catalog and diagnostics: [`apps/runtime/src-tauri/src/commands/channel_connectors.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/channel_connectors.rs)
- IM event dedupe and inbox persistence: [`apps/runtime/src-tauri/src/commands/im_gateway.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/im_gateway.rs)
- Feishu/OpenClaw event ingestion and routing helpers: [`apps/runtime/src-tauri/src/commands/feishu_gateway.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/feishu_gateway.rs), [`apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/openclaw_gateway.rs)
- Employee/session/channel routing: [`apps/runtime/src-tauri/src/commands/employee_agents.rs`](D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents.rs)

What WorkClaw does not yet provide:

- A plugin registry compatible with `OpenClawPluginApi`
- A plugin runtime compatible with `PluginRuntime`
- An `openclaw/plugin-sdk` executable shim
- Channel plugin activation from plugin registration
- OpenClaw-style plugin config storage and schema validation
- Setup-time plugin loading modes

## Reference Model From OpenClaw

The compatibility target is visible in these OpenClaw files:

- Plugin loading: [`references/openclaw/src/plugins/loader.ts`](D:/code/WorkClaw/references/openclaw/src/plugins/loader.ts)
- Plugin registry: [`references/openclaw/src/plugins/registry.ts`](D:/code/WorkClaw/references/openclaw/src/plugins/registry.ts)
- Plugin API/types: [`references/openclaw/src/plugins/types.ts`](D:/code/WorkClaw/references/openclaw/src/plugins/types.ts)
- Plugin runtime: [`references/openclaw/src/plugins/runtime/types.ts`](D:/code/WorkClaw/references/openclaw/src/plugins/runtime/types.ts)
- Plugin SDK exports: [`references/openclaw/src/plugin-sdk/index.ts`](D:/code/WorkClaw/references/openclaw/src/plugin-sdk/index.ts)
- Channel plugin contract: [`references/openclaw/src/channels/plugins/types.plugin.ts`](D:/code/WorkClaw/references/openclaw/src/channels/plugins/types.plugin.ts)
- Channel plugin install/setup: [`references/openclaw/src/commands/channel-setup/plugin-install.ts`](D:/code/WorkClaw/references/openclaw/src/commands/channel-setup/plugin-install.ts)

The first concrete target plugin is:

- Official Lark/Feishu plugin: [`references/openclaw-lark/index.ts`](D:/code/WorkClaw/references/openclaw-lark/index.ts), [`references/openclaw-lark/src/channel/plugin.ts`](D:/code/WorkClaw/references/openclaw-lark/src/channel/plugin.ts)

## Proposed Architecture

### 1. Node Plugin Host

Add a dedicated Node host owned by WorkClaw. This host is responsible for:

- installing npm plugins into a managed plugin directory,
- parsing plugin manifests before execution,
- loading plugin entries through a TypeScript/ESM-capable loader,
- exposing the compatibility package `openclaw`,
- returning plugin registration data and runtime callbacks back into WorkClaw.

This should be a WorkClaw subsystem, not a user-managed external dependency.

### 2. Manifest And Install Layer

Introduce an OpenClaw-plugin-specific install path distinct from local skills.

Required behavior:

- accept npm specs like `@larksuite/openclaw-lark`,
- validate `package.json.openclaw.extensions`,
- parse `openclaw.plugin.json`,
- persist install provenance and version,
- support upgrade/reinstall,
- separate plugin metadata from runtime activation state.

### 3. Compatibility Shim Package

Inside the plugin host environment, provide a package named `openclaw` with executable exports for:

- `openclaw/plugin-sdk`
- `openclaw/plugin-sdk/core`
- `openclaw/plugin-sdk/compat`
- channel-specific subpaths needed by supported plugins, starting with `openclaw/plugin-sdk/feishu`

This shim should be implemented by WorkClaw and backed by WorkClaw runtime bridges, not copied blindly from OpenClaw.

### 4. Plugin Registry

Create a WorkClaw plugin registry mirroring the OpenClaw registration model:

- tools
- channels
- hooks
- gateway methods
- HTTP routes
- CLI registrars
- services
- commands

Phase 1 only needs to fully operationalize the surfaces required by the Feishu plugin, but the registry structure should be generic from day one.

### 5. Plugin Runtime Bridge

Implement `api.runtime` as a bridge into WorkClaw capabilities.

The minimum runtime namespaces for Feishu compatibility are:

- `config`
- `tools`
- `events`
- `logging`
- `state`
- `channel`
- `agent`
- `system`
- `media`
- `modelAuth`

`subagent` support should be designed in but can be staged if the Feishu plugin path proves not to require it for phase 1 acceptance.

### 6. Channel Host Bridge

A registered `ChannelPlugin` must become a live WorkClaw channel.

That bridge must translate between:

- OpenClaw channel concepts: setup, pairing, gateway, outbound, actions, directory, status, threading, group policy
- WorkClaw concepts: IM inbox events, employee routing, sidecar connector diagnostics, session linking, message links, runtime events

The key design rule is that WorkClaw owns the real transport/storage/session system, while the plugin owns channel-specific policy and behavior.

### 7. Config Model

WorkClaw needs an OpenClaw-compatible config view for plugins.

Two config layers must exist:

- plugin-scoped config: `plugins.entries.<pluginId>.config`
- channel-scoped config: `channels.<channelId>.*`

These should not force a total rewrite of WorkClaw settings storage. Instead, WorkClaw should expose a computed config view to the plugin host and persist plugin-owned config slices in dedicated storage.

## Phase Plan

### Phase 1: Compatibility Host For Official Feishu Plugin

Goal: install and run `@larksuite/openclaw-lark` in WorkClaw with enough fidelity for real user use.

Required outcomes:

- install plugin from npm
- parse manifest and metadata
- load in `setup-runtime` and `full` modes
- register channel and required tools
- read/write plugin config
- run OAuth/setup flow
- receive Feishu inbound events
- execute tools and send Feishu outbound messages
- support plugin upgrade/reinstall

### Phase 2: Harden Generic Plugin Host Surfaces

- broaden hook support
- broaden gateway/http route support
- expose richer runtime namespaces
- improve diagnostics and status reporting
- validate additional official plugins

## Risks

### Runtime Contract Risk

`openclaw/plugin-sdk` is a runtime dependency, not just type definitions. Missing helper behavior will cause subtle plugin failures.

### Config Drift Risk

If WorkClaw stores plugin state in a shape that cannot round-trip to OpenClaw-style config, setup and upgrades will be brittle.

### Lifecycle Risk

If `setup-only`, `setup-runtime`, and `full` are not respected, official plugins may appear installed but silently skip crucial behavior.

### Over-Scoping Risk

Trying to support all OpenClaw plugin surfaces before Feishu works will slow delivery. The right order is Feishu-first, genericize second.

## Recommended Smallest Safe Path

1. Build a Node-based OpenClaw plugin host owned by WorkClaw.
2. Support manifest parsing and npm install for OpenClaw native plugins.
3. Implement the smallest `openclaw/plugin-sdk` shim required by the official Feishu plugin.
4. Bridge `ChannelPlugin` into WorkClaw IM/gateway/session systems.
5. Verify the official Feishu plugin end-to-end before expanding the compatibility surface.

## Acceptance Criteria

- A user can install `@larksuite/openclaw-lark` from WorkClaw.
- WorkClaw loads the plugin without patching plugin source.
- The plugin setup flow can configure and persist Feishu settings.
- Inbound Feishu messages reach WorkClaw routing and session logic through the plugin.
- The plugin can send outbound Feishu replies and use its registered tools.
- Reinstalling or upgrading the plugin does not require WorkClaw code changes for ordinary plugin releases.

## Notes

This design intentionally treats official OpenClaw plugin compatibility as a host-platform feature, not a one-off Feishu integration. That is the only path that matches the product goal of replacing OpenClaw while preserving official plugin compatibility.
