## WorkClaw Windows Release

- Highlights in `v0.5.1`:
  - 中文:
    - 新增本地 SkillHub 技能库索引和技能发现页，专家能力浏览与检索体验更完整。
    - 新增豆包模型预设，并优化模型鉴权与错误反馈，模型连接失败时更容易定位问题。
    - 继续重构飞书接入链路，统一官方插件 runtime 的收发与审批流程，补强自动恢复、会话显示和单聊外发稳定性。
  - English:
    - Adds a local SkillHub index and improved skill discovery flows for a more complete expert browsing experience.
    - Adds a Doubao provider preset and improves model authentication plus error feedback so connection issues are easier to diagnose.
    - Continues the Feishu connector rebuild by unifying more of the official-plugin runtime flow, including approvals, auto-recovery, session visibility, and more reliable direct-message outbound delivery.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
