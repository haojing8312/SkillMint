## WorkClaw Windows Release

- Highlights in `v0.5.4`:
  - 中文:
    - 新增会话 PDF 附件支持，现在可以直接上传 PDF 到聊天中并提取正文供智能体分析。
    - 升级 OpenAI 工具调用链路到 Responses API，提升兼容性并改善工具调用稳定性。
    - 优化快速设置流程，支持跳过部分配置并在搜索能力上使用 MCP 回退路径，降低首次上手门槛。
    - 加强网络重试、会话恢复和运行时保护机制，复杂任务与长任务执行更稳定。
    - 修复聊天中 Markdown 链接的打开方式，外部链接现在会在系统浏览器中打开。
    - 改进窗口与界面细节，包括快速设置阶段标题栏保留等桌面体验优化。
  - English:
    - Added PDF attachments in chat, allowing PDFs to be uploaded directly into a conversation and their text extracted for agent analysis.
    - Upgraded OpenAI tool calling to the Responses API for better compatibility and more reliable tool execution.
    - Improved the quick setup flow with optional setup skipping and MCP fallback for search capabilities, reducing first-run friction.
    - Strengthened network retry, session recovery, and runtime guardrails to make complex and long-running tasks more stable.
    - Fixed Markdown link handling in chat so external links now open in the system browser.
    - Polished desktop window behavior and UI details, including preserving the title bar during quick setup.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
