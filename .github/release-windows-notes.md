## WorkClaw Windows Release

- Highlights in `v0.5.2`:
  - 中文:
    - 新增本地技能批量导入能力，导入多个本地 Skill 包时更省步骤。
    - 持续收敛桌面端大模块拆分后的稳定性问题，提升设置页、员工中心和聊天相关流程的可靠性。
    - 重点修复飞书官方插件链路中的 Windows 启动、插件宿主兼容、诊断误报，以及批准接入后仍重复要求授权的问题，飞书接入流程更稳定。
  - English:
    - Adds batch local skill imports so multiple local Skill packs can be brought in with less manual setup.
    - Continues hardening the desktop app after the recent module splits, improving reliability across settings, employee hub, and chat flows.
    - Significantly improves the Feishu official-plugin path on Windows, including runtime startup, plugin-host compatibility, diagnostics accuracy, and pairing approval sync so approved users are no longer repeatedly asked to authorize.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
