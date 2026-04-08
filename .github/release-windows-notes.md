## WorkClaw Windows Release

- Release scope: changes from `v0.5.7` to the current `v0.5.8` tag target.

## Highlights

- 中文:
  - 新增统一的数据根目录与迁移机制。数据库、缓存、日志、诊断、插件状态和会话记录现在会围绕同一个运行时根目录组织，桌面设置里也支持选择目录、迁移并在失败时自动回退。
  - 优化飞书插件接入稳定性。现在兼容新版 `@larksuite/openclaw-lark` 的混合模块输出，安装向导对宿主机 Node.js 版本会提前做 `22+` 检查，旧环境会在设置页直接提示而不是走到中途失败。
  - 明显减少飞书插件卸载拖慢桌面卸载的问题。插件删除时会同步清理插件目录与状态目录，并去掉额外的 `installer-tools` 二次安装，降低 Windows 上海量小文件删除带来的长时间卡顿。
  - 新增 OEM 桌面打包能力。默认品牌仍然是 WorkClaw，但现在可以通过一个品牌配置文件或单个 `--brand` 参数快速生成自己的品牌安装包，数据根目录、启动标识和安装器品牌资源会随品牌切换。
  - 调整了桌面设置与员工页里的部分文案，把“WorkClaw 数据根目录”等硬编码品牌描述改成更通用的“数据根目录”，方便默认版和 OEM 版共用同一套界面文案。

- English:
  - Added a unified runtime root and migration flow. Databases, caches, logs, diagnostics, plugin state, and session records now live under one runtime root, and desktop settings can move that root with automatic rollback if migration fails.
  - Improved Feishu plugin compatibility and setup stability. WorkClaw now supports the newer mixed-module output from `@larksuite/openclaw-lark`, and the setup flow checks for host Node.js `22+` up front so unsupported environments fail early with clearer guidance.
  - Reduced the long uninstall delays caused by Feishu plugin cleanup on Windows. Plugin removal now deletes plugin directories and state directories together, and no longer bootstraps an extra `installer-tools` install that inflated the file count.
  - Added OEM desktop packaging support. WorkClaw remains the default brand, but OEM installers can now be produced from a single brand config or a single `--brand` build argument, with brand-driven runtime roots, startup identity, and installer assets.
  - Updated parts of the desktop settings and employee UI to use neutral storage wording such as “data root directory”, so the same UI copy works for both the default WorkClaw build and OEM-branded builds.

## Notable Changes

- Unified runtime root:
  - Added stable runtime bootstrap discovery, migration scheduling validation, rollback protection, and settings-driven root switching.
  - Added regression coverage for legacy-root persistence and migration recovery.

- Feishu plugin flow:
  - Added compatibility for newer OpenClaw Lark package layouts.
  - Added host Node.js version gating and clearer setup guidance.
  - Reduced plugin uninstall cleanup overhead and removed the extra installer-tools bootstrap install.

- OEM packaging:
  - Added a single brand selection config, `--brand` build override, generated Rust branding constants, and a sample `bifclaw` brand.
  - Kept `workclaw` as the default local build target while allowing OEM-branded MSI and NSIS packages from the same build pipeline.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Runtime root migration flow and desktop settings updates verified with targeted runtime tests.
- Feishu plugin host compatibility, host Node.js gating, and installer checks verified.
- Default WorkClaw packaging and OEM `--brand` packaging both verified locally.
- Release tag matches desktop app version.
