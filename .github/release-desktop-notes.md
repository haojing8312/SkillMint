## BifClaw Desktop Release

- Release scope: changes from `v0.6.1` to the current `v0.6.2` tag target.

## Highlights

- 中文:
  - 修复 IM 长对话下的会话绑定与复用问题，减少飞书和企业微信场景里的串会话与答非所问。
  - 补强 Feishu 和 WeCom 的 conversation-aware routing，让路由、回复和会话恢复链路更稳定。
  - 新增 WeCom topic 归一化与入站样本清洗工具，方便继续做渠道回归和问题定位。
  - 改进本地技能命令别名、聊天流式表现和部分模型错误处理，提升日常桌面使用稳定性。
  - 本次本地桌面打包产物使用 `BifClaw` 品牌名称。

- English:
  - Fixed IM session binding and reuse issues in long conversations to reduce cross-thread context mix-ups in Feishu and WeCom flows.
  - Strengthened conversation-aware routing across Feishu and WeCom so routing, replies, and session recovery stay more stable.
  - Added WeCom topic normalization and inbound sample sanitizing tooling to support ongoing channel regressions and troubleshooting.
  - Improved local skill command aliases, chat streaming behavior, and parts of model error handling for day-to-day desktop stability.
  - This local desktop package is branded as `BifClaw`.

## Notable Changes

- Messaging and channel alignment:
  - Completed the IM conversation identity cutover for Feishu and WeCom routing paths.
  - Reduced incorrect session reuse in long-running chat threads.

- Desktop and release hardening:
  - Added stronger IM host, employee-agent, and conversation-mapping regression coverage.
  - Produced the local desktop packages with the `BifClaw` brand assets.

- Windows recommended download: `*-setup.exe` for direct install.
- Linux x64 download: `*_amd64.deb`.
- Linux arm64 download: `*_arm64.deb`.

## Installation Guide

1. Most Windows users should install the `setup.exe` package.
2. Linux x64 users should install the `amd64.deb` package.
3. Linux arm64 users should install the `arm64.deb` package.

## Verification Checklist

- Frontend build and desktop packaging were verified for the `v0.6.2` release target.
- Windows `setup.exe`, Linux `amd64.deb`, and Linux `arm64.deb` packages are built by the release workflow.
- Release version files and release notes were validated against the `v0.6.2` tag target.
- Release tag matches desktop app version.
