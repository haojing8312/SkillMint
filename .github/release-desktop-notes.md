## BifClaw Desktop Release

- Release scope: changes from `v0.6.3` to the current `v0.6.4` tag target.

## Highlights

- 中文:
  - 修复 `full_access` 下文件工具仍被会话目录限制的问题，现在可访问会话目录外的普通路径，同时继续保护敏感路径。
  - 增强附件粘贴、拖拽和本地任务产物处理体验。
  - 改进审批恢复、权限判断、运行时稳定性和脚本校验。
  - 拆分聊天界面和运行时状态模块，提升后续维护稳定性。
  - 本次本地桌面打包产物使用 `BifClaw` 品牌名称。

- English:
  - Fixed `full_access` file-tool behavior so ordinary paths outside the session directory can be accessed while sensitive paths remain protected.
  - Improved pasted/dropped attachments and local task artifact handling.
  - Hardened approval recovery, permission checks, runtime stability, and validation scripts.
  - Split chat UI and runtime state modules for better maintainability.
  - This local desktop package is branded as `BifClaw`.

## Notable Changes

- Runtime and permissions:
  - `full_access` now separates the session working directory from file-tool path authorization.
  - Standard and accept-edits modes remain workspace-only, while full access can use ordinary external paths with sensitive-path guards.

- Attachments and local task output:
  - Improved attachment intake for pasted and dropped files.
  - Hardened local task recovery and artifact handling around approval and file operations.

- Desktop and release hardening:
  - Split large chat UI modules for maintainability.
  - Added and repaired validation scripts for release and local build confidence.
  - Produced the local desktop packages with the `BifClaw` brand assets.

- Windows recommended download: `*-setup.exe` for direct install.
- Linux x64 download: `*_amd64.deb`.
- Linux arm64 download: `*_arm64.deb`.

## Installation Guide

1. Most Windows users should install the `setup.exe` package.
2. Linux x64 users should install the `amd64.deb` package.
3. Linux arm64 users should install the `arm64.deb` package.

## Verification Checklist

- Frontend build and desktop packaging were verified for the `v0.6.4` release target.
- Windows `setup.exe`, Linux `amd64.deb`, and Linux `arm64.deb` packages are built by the release workflow.
- Release version files and release notes were validated against the `v0.6.4` tag target.
- Release tag matches desktop app version.
