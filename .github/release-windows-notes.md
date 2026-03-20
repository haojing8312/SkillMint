## WorkClaw Windows Release

- Highlights in `v0.5.0`:
  - 中文:
    - 新增 OpenClaw 飞书官方插件兼容宿主，WorkClaw 现在可以安装、识别并运行 `@larksuite/openclaw-lark`。
    - 重构飞书接入流程，支持官方插件安装、已有机器人绑定、配对审批、自动接待与高级配置面板。
    - 修复飞书消息入站、配对放行、自动启动和回复回传链路，并补强会话恢复、审计诊断与任务防护能力。
  - English:
    - Adds an OpenClaw-compatible host for the official Feishu plugin so WorkClaw can install, recognize, and run `@larksuite/openclaw-lark`.
    - Redesigns the Feishu onboarding flow with official-plugin install, existing-bot linking, pairing approval, auto-routing, and an advanced settings console.
    - Hardens the Feishu inbound, pairing, autostart, and reply return path while also improving session recovery, audit diagnostics, and task guardrails.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
