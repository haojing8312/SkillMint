# Runtime Diagnostics

适用于 WorkClaw 桌面端出现以下问题时的本地排查：

- 安装版闪退
- 配置模型后发送第一条消息即退出
- 页面无报错但功能异常
- 仅个别用户可复现的问题

## 用户侧操作

1. 打开 `设置 -> 桌面 / 系统`
2. 查看“诊断状态”
3. 点击 `导出诊断包`
4. 将导出的 zip 提供给维护者

如果应用仍能打开，优先使用诊断包，不要让用户手工查数据库或事件查看器。

## 诊断包内容

诊断包默认包含：

- 环境摘要
- 最近 route attempt logs
- 最近 session runs
- 最近 session run events 摘要
- 最近运行日志文件
- 最近崩溃摘要

默认不包含完整 `workclaw.db`，以降低隐私风险。

## 本地诊断目录

应用会在 `app_data_dir/diagnostics/` 下生成本地诊断数据，主要包括：

- `logs/`：结构化运行日志（JSONL）
- `crashes/`：panic / 崩溃摘要
- `exports/`：用户导出的诊断包
- `state/`：当前运行状态与上次正常退出标记

## 支持同学优先查看

收到诊断包后，优先查看：

1. `environment-summary.md`
2. `latest-crash.json`
3. `route-attempt-logs.json`
4. `session-runs.json`
5. `logs/runtime-*.jsonl`

## 仍需系统级排查的场景

如果用户连设置页都无法打开，或者诊断包为空，再退回到：

- Windows 事件查看器
- Windows Error Reporting dump
- 手工提取 `workclaw.db`
