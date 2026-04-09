## WorkClaw Windows Release

- Release scope: changes from `v0.5.8` to the current `v0.5.9` tag target.

## Highlights

- 中文:
  - 统一了桌面会话运行时内核。WorkClaw 现在把本地聊天、隐藏子会话和员工步骤会话收敛到同一条 session spine，减少不同执行通道之间的状态分叉与行为不一致。
  - 增强了长任务连续性。现在会记录 turn state、compaction boundary 和 continuation policy，压缩后的“继续”请求会更稳定地继承上一次的执行上下文、技能选择和恢复提示。
  - 优化了工具平台的使用体验。运行时现在支持 staged tool recommendation tiers、deferred loading 和更细的 tool recommendation planning，让工具池曝光更收敛，也更容易在需要时逐步扩展。
  - 改进了工具执行和审批反馈。聊天界面现在会更清楚地展示工具执行状态、审批原因和恢复信息，减少用户在长流程中的理解成本。
  - 补齐了更多跨 surface 的 journal、session recovery 和 resilience 链路，为后续更强的 harness-style agent 能力打下基础。

- English:
  - Unified the desktop session runtime kernel. WorkClaw now routes local chat, hidden child sessions, and employee step sessions through the same session spine, reducing state drift and behavior mismatches across execution paths.
  - Improved long-running task continuity. Turn state, compaction boundaries, and continuation policy now flow through the session lifecycle, so “continue” after compaction can more reliably resume the previous execution context, skill choice, and recovery hints.
  - Upgraded the runtime tool platform. WorkClaw now supports staged tool recommendation tiers, deferred loading, and more explicit tool recommendation planning, making tool exposure narrower by default and easier to expand when needed.
  - Improved tool execution and approval feedback. The chat UI now surfaces tool activity, approval rationale, and recovery context more clearly, reducing confusion during longer workflows.
  - Added broader cross-surface journal, session recovery, and resilience plumbing, laying the groundwork for stronger harness-style agent behavior in future releases.

## Notable Changes

- Session spine runtime:
  - Moved local chat, hidden child sessions, and employee step sessions onto shared runtime contracts.
  - Centralized execution context, turn state, route lanes, and outcome commit paths.

- Continuity and recovery:
  - Persisted turn state, compaction boundaries, and continuation preference through session journals and recovery flows.
  - Added clearer recovery messaging for continuation after compaction and max-turns style failures.

- Tool platform:
  - Added staged recommendation tiers, deferred tool loading, discovery hints, and richer tool recommendation observability.
  - Unified tool planning with runtime capability snapshots and prompt assembly.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Runtime kernel, session spine, and tool-platform changes verified with Rust fast-path checks and targeted runtime/frontend coverage.
- Release version files and release notes validated against the `v0.5.9` tag target.
- Local Windows packaging is re-run as part of this release flow.
- Release tag matches desktop app version.
