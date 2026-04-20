## WorkClaw Windows Release

- Release scope: changes from `v0.5.10` to the current `v0.6.0` tag target.

## Highlights

- 中文:
  - 新增统一附件平台，聊天场景下的附件处理与扩展能力更完整。
  - 抽取共享 `im_host` 运行时，统一了飞书与企业微信消息收发相关的主机能力与回复链路。
  - 对齐渠道注册表与会话启动流程，设置页中的 IM 渠道配置与运行时行为保持一致。
  - 修复 Windows 桌面发布链路中的关键构建问题，恢复主分支本地打包与安装包生成能力。
  - 补强 IM host 回归验证与发布校验，提升桌面版本发布稳定性。

- English:
  - Added a unified attachment platform to make chat attachment handling and extension flows more complete.
  - Extracted a shared `im_host` runtime to align the host capabilities and reply flow used by Feishu and WeCom messaging.
  - Aligned the channel registry with session launch flows so IM channel settings now match runtime behavior more consistently.
  - Fixed the key Windows desktop release build issues and restored local packaging and installer generation on `main`.
  - Strengthened IM host regressions and release verification to improve desktop release stability.

## Notable Changes

- Messaging and channel alignment:
  - Unified the shared IM host runtime used by Feishu and WeCom flows.
  - Brought channel registry and session launch behavior onto the same runtime contract.

- Attachment platform:
  - Shipped the unified attachment platform for chat workflows.
  - Reduced drift between attachment handling branches and follow-up release verification.

- Desktop and release hardening:
  - Restored frontend build compatibility and desktop packaging on `main`.
  - Added stronger IM host regression coverage for the Windows release path.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Frontend build and Windows desktop packaging were verified for the `v0.6.0` release target.
- Release version files and release notes were validated against the `v0.6.0` tag target.
- Local Windows packaging is re-run as part of this release flow.
- Release tag matches desktop app version.
