## WorkClaw Windows Release

- Highlights in `v0.4.1`:
  - 中文:
    - 修复 Windows 发布流程在 GitHub Actions 上的打包兼容性问题。
    - 远端发布脚本现在会根据 `pnpm` 版本自动选择合适的 sidecar 部署参数，避免因 `pnpm 9` 不支持 `--legacy` 而导致发布失败。
    - 本次为发布稳定性热修复，不包含新的用户功能变更。
  - English:
    - Fixes the Windows release pipeline compatibility issue in GitHub Actions.
    - The remote packaging script now adapts its sidecar deploy arguments based on the detected `pnpm` version, preventing release failures caused by `pnpm 9` not supporting `--legacy`.
    - This is a release stability hotfix with no new user-facing features.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
