## WorkClaw Windows Release

- Release scope: changes from `v0.5.6` to the current `v0.5.7` tag target.

## Highlights

- 中文:
  - 新增自然语言隐式 skill 路由控制平面。WorkClaw 现在会先构建本地 skill 路由索引，再做候选召回、保守裁决、专用 runner 选择和结构化观测，减少“本来该走 skill 却掉进通用开放任务”的情况。
  - 新增显式 skill 快路径和一等 `exec` 工具。显式 skill 请求现在会更早命中 prompt-following skill，上下文和工具池收敛更快；Windows 本地命令执行也不再只是 `bash` 别名，而是有独立 `exec` 执行通道，更接近 Codex 的执行行为。
  - 新增真实 agent 回归测试能力。现在可以用真实模型、真实 skill、真实外部系统和匿名 YAML 场景，在本地手动回放关键任务，并自动产出 `pass / warn / fail` 报告、trace、journal 和 stdout/stderr 证据。
  - 补齐了真实评测的匿名 capability 映射和本地私有配置模型，支持在不提交敏感 skill 路径、凭证和本地环境细节的前提下，对核心 skill 场景做持续回归。
  - 改进聊天运行时的持久化和状态追踪，减少重复快照写入，并为 route、runner、session run 和 trace 链路补齐更多可观测数据。
  - 修复 Windows 安装包主程序选择错误。桌面安装包现在明确以 `runtime.exe` 作为主程序，不再把 `agent_eval.exe` 打成安装入口，并新增安装包回归测试防止再次回归。

- English:
  - Added an implicit skill-routing control plane for natural-language requests. WorkClaw now builds a local skill route index, performs candidate recall, applies conservative adjudication, selects a dedicated runner, and records structured routing observations before falling back to the general open-task lane.
  - Added an explicit skill fast path and a first-class `exec` tool. Explicit prompt-following skill requests now converge on the intended runtime context and tool pool earlier, and Windows local command execution now uses a dedicated `exec` lane instead of behaving like a thin `bash` alias.
  - Added a real-agent regression harness. WorkClaw can now replay critical tasks locally with real models, real skills, and real external systems, and emit `pass / warn / fail` reports together with trace, journal, and stdout/stderr artifacts.
  - Added anonymous capability mapping and local-only configuration for real-agent evals so sensitive skill paths, credentials, and machine-specific details stay out of git while core skill flows remain regression-testable.
  - Improved chat runtime persistence and runtime observability by reducing redundant snapshot writes and wiring more route, runner, session-run, and trace data into the diagnostic surface.
  - Fixed Windows packaging so the desktop installer now binds to `runtime.exe` instead of accidentally packaging `agent_eval.exe` as the main desktop binary, with a regression test guarding the installer entrypoint.

## Notable Changes

- Skill routing and execution:
  - Added route intents, route index projection, recall, adjudication, route runners, and route observations under `agent/runtime/skill_routing`.
  - Tightened prompt-skill execution so explicit skill requests and deterministic dispatch paths use narrower runtime context and cleaner tool setup.
  - Added a dedicated `exec` tool and process-manager support for shell-aware command spawning.

- Real-agent eval harness:
  - Added `agent-evals/` scenario support, local config example scaffolding, report generation, and a dedicated `agent_eval` CLI.
  - Added a golden real-agent scenario for PM weekly summary evaluation and supporting contract tests.
  - Hardened local model config handling so `api_key_env` must point to an environment variable name rather than storing a literal key.

- Runtime and UI resilience:
  - Reduced duplicate runtime persistence writes and improved session/runtime state coordination.
  - Added more structured route/session-run observability so real runs can be compared against Codex-style baselines.

- Packaging and installer:
  - Declared `runtime` as the default desktop binary in Cargo metadata and kept `agent_eval` as an explicit auxiliary binary.
  - Added an installer regression test to ensure future builds package `runtime.exe` as the desktop entrypoint.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Desktop bundle entrypoint verified as `runtime.exe`.
- Real-agent evaluation lane validated with a golden PM summary scenario.
- Release tag matches desktop app version.
