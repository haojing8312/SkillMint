## WorkClaw Windows Release

- Highlights in `v0.5.5`:
  - 中文:
    - 升级预装 Office Skills，Word、Excel、PDF 和 PPT 技能现在会以完整可运行的 skill 目录随应用分发，不再只是轻量提示模板。
    - 统一了预装技能与外置技能的安装和运行模型，技能来源、刷新与运行时投影链路更一致，后续升级第三方技能更轻。
    - 增强了技能运行前环境检查，能够更明确提示缺失的 `.NET`、Python、Node、Playwright 等依赖，减少运行时踩坑。
    - 改进了 OpenAI-compatible 模型兼容性与传输适配，补强了 `Qwen` 相关推理流和路由处理，减少这类模型在实际接入时的兼容问题。
    - 新增 OEM 品牌化构建链路，支持按品牌资源生成桌面图标、安装器视觉和应用标识，为后续定制发行做准备。
    - 改进了运行时轨迹、诊断与契约相关能力，并清理了一批过渡脚手架和测试夹具，让运行时行为更清晰、维护成本更低。
  - English:
    - Upgraded the preinstalled Office Skills. The Word, Excel, PDF, and PPT skill set now ships as full runnable skill directories instead of lightweight prompt-only built-ins.
    - Unified the installation and runtime model for preinstalled and external skills, making skill sources, refresh behavior, and workspace projection more consistent and easier to upgrade over time.
    - Improved runtime readiness checks for skills, with clearer diagnostics for missing dependencies such as `.NET`, Python, Node, and Playwright before execution.
    - Improved compatibility and transport handling for OpenAI-compatible model backends, including stronger `Qwen` reasoning-stream and routing support to reduce real-world integration issues.
    - Added an OEM branding build pipeline that can apply brand assets to desktop icons, installer visuals, and app identity, laying the groundwork for branded distributions.
    - Improved runtime traces, diagnostics, and contract-related behavior while removing transitional scaffolding and fixtures to keep the runtime easier to maintain.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
