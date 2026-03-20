# OpenClaw Plugin Host

This directory will contain WorkClaw's compatibility host for official OpenClaw native plugins.

## Why This Exists

Official OpenClaw plugins assume they are running inside an OpenClaw-shaped host. They do not just expose static files or isolated RPC handlers. They expect:

- manifest-first loading from `openclaw.plugin.json` and `package.json.openclaw`
- executable imports from `openclaw/plugin-sdk`
- a live `OpenClawPluginApi`
- a live `PluginRuntime`
- channel plugin registration and lifecycle handling
- setup/runtime registration modes such as `full`, `setup-only`, and `setup-runtime`

WorkClaw already has local skills, IM routing, channel diagnostics, and Feishu/OpenClaw ingress points. That is not enough to run official OpenClaw plugins directly. This host fills that gap.

## Phase 1 Scope

Phase 1 is Feishu-first and compatibility-first.

The host must support:

- installing official OpenClaw native plugins from npm
- parsing plugin manifests before loading code
- exposing a compatibility shim package named `openclaw`
- loading plugins in setup and full runtime modes
- registering channel plugins and required tools
- bridging plugin config, gateway, inbound, outbound, and status flows into WorkClaw

The first target plugin is `@larksuite/openclaw-lark`.

## Expected Runtime Surfaces

The compatibility layer must eventually provide these `openclaw/plugin-sdk` surfaces for supported plugins:

- `openclaw/plugin-sdk`
- `openclaw/plugin-sdk/core`
- `openclaw/plugin-sdk/compat`
- channel-specific subpaths needed by supported plugins, starting with `openclaw/plugin-sdk/feishu`

The plugin host also needs an `OpenClawPluginApi`-shaped registration object and a `PluginRuntime` bridge backed by WorkClaw runtime systems.

## Relationship To Tauri

The Node host is responsible for plugin loading and registration. The Tauri backend remains the source of truth for:

- persisted plugin install records
- settings storage
- IM inbox/session/routing state
- sidecar-backed connector operations
- desktop UI integration

The host should bridge to Tauri-owned state and services rather than duplicating them.
