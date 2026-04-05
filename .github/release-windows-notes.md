## WorkClaw Windows Release

- Highlights in `v0.5.7`:
  - 中文:
    - 新增了本地真实 agent 评测通道，可以用真实模型、真实 skill 和真实外部系统场景手动回放关键任务，并自动产出 `pass / warn / fail` 报告、trace、journal 和 stdout/stderr 证据。
    - 为真实评测补齐了匿名场景定义和本地私有配置映射，便于在不提交敏感 skill 路径与密钥的前提下，对 WorkClaw 的核心 skill 执行能力做回归验证。
    - 修复了真实评测中的结构化结果判定问题，现在可以正确识别像 `summaries[].daily_facts / plan_facts / report_facts` 这样的嵌套返回结构，减少误报失败。
    - 加强了本地模型配置的安全约束，`api_key_env` 现在要求填写环境变量名而不是明文 key，降低本地调试时把密钥写进配置和错误日志的风险。
    - 继续改进聊天运行时持久化，减少重复快照写入，同时保持会话恢复行为稳定。
  - English:
    - Added a local real-agent evaluation lane for replaying critical tasks with real models, real skills, and real external systems, with automatic `pass / warn / fail` reports plus trace, journal, and stdout/stderr artifacts.
    - Added anonymous scenario definitions and local-only capability mapping so WorkClaw can regression-test core skill execution without committing sensitive skill paths or secrets.
    - Fixed structured-result evaluation for real-agent runs so nested outputs such as `summaries[].daily_facts / plan_facts / report_facts` are interpreted correctly instead of being reported as false failures.
    - Hardened local model configuration safety by requiring `api_key_env` to be an environment-variable name rather than a literal API key, reducing the chance of leaking secrets into local config or error output.
    - Continued improving chat runtime persistence to reduce redundant snapshot writes while keeping session restore behavior stable.

- Recommended download: `*-setup.exe` for direct install.
- Enterprise deployment: `*.msi` for IT-managed installation and manual upgrades.

## Installation Guide

1. Most users should install the `setup.exe` package.
2. Enterprise or managed devices can use the `.msi` package.

## Verification Checklist

- Installer branding and Chinese setup wizard verified.
- Release tag matches desktop app version.
