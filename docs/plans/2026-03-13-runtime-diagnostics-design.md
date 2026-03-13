# Runtime Diagnostics Design

## Goal

为 WorkClaw 桌面端补齐本地诊断链路，让“闪退、异常退出、关键链路失败、用户难以描述的问题”能够在用户机器上沉淀为可回收证据，并通过 UI 一键导出诊断包。

## Problem

当前桌面端存在以下诊断缺口：

- `app_log_dir` 仅作为路径暴露，没有真正的文件日志管线。
- Rust release 使用 `panic = "abort"`，一旦命中 panic，进程直接退出，用户几乎无感知，维护者也拿不到上下文。
- 前端缺少统一的全局异常采集与落盘。
- sidecar 启停与健康检查仅通过 `eprintln!` 输出，安装版用户看不到。
- 排查过程依赖人工引导用户查事件查看器、找数据库、截图，成本高且不稳定。

## Non-Goals

第一版不做：

- 远程遥测、远程崩溃上报、第三方监控服务
- 全量数据库导出
- 高级日志搜索 / 在线筛选 UI
- 覆盖所有 OS 级 native crash
- 自动上传任何用户数据

## Approach Options

### Option A: Panic-Only Patch

仅补 Rust panic hook 和少量前端异常写文件。

优点：

- 改动最小
- 出版本最快

缺点：

- 只能抓崩溃，不足以排查大量“未崩溃但行为异常”的问题
- 仍然需要人工组合 route logs、环境信息和会话上下文

### Option B: Local Diagnostics Pipeline

补齐本地结构化日志、崩溃记录、前端异常捕获、sidecar 关键事件记录，以及一键导出诊断包。

优点：

- 能覆盖大多数桌面端疑难问题
- 保持本地优先，不引入远程依赖
- 用户操作成本低，可产品化

缺点：

- 改动面中等
- 需要同步后端、前端和测试

### Option C: Full Observability

在 Option B 基础上引入远程 crash/reporting 管线。

优点：

- 维护者获取数据最快

缺点：

- 隐私和合规复杂
- 与当前产品定位不一致

## Recommendation

采用 Option B。它在不引入远程服务的前提下，最大幅度降低排查成本，且可以自然扩展到后续版本。

## Design

### 1. Diagnostics Backend Module

新增统一诊断模块，负责：

- 初始化诊断目录结构
- 写运行日志
- 写前端异常日志
- 写崩溃/异常退出标记
- 生成诊断包

建议目录结构：

- `app_data_dir/diagnostics/`
- `app_data_dir/diagnostics/logs/`
- `app_data_dir/diagnostics/crashes/`
- `app_data_dir/diagnostics/exports/`
- `app_data_dir/diagnostics/state/`

其中：

- `logs/` 保存结构化运行日志
- `crashes/` 保存 panic/异常退出摘要
- `exports/` 保存导出的 zip
- `state/` 保存本次运行状态文件，例如 `active-run.json`

### 2. Structured Runtime Logging

后端统一提供轻量日志写入接口，不要求一次性把全项目替换为完整 tracing 体系。第一版以“关键路径明确打点”为主。

日志事件最少包含：

- timestamp
- level
- source
- event
- message
- optional context JSON

关键打点范围：

- app startup / shutdown
- db init success/failure
- sidecar start / health check / timeout / stop
- chat send_message lifecycle
- route execution success/failure
- diagnostics export success/failure
- explicit command failures that currently only surface as strings

日志格式采用 JSON Lines，便于后续机器处理，也便于人工查看。

### 3. Panic and Abnormal Exit Capture

启动时注册全局 panic hook：

- 在 hook 内写崩溃摘要文件
- 尽量记录 panic message、location、thread、backtrace
- 同步写入最近一次运行状态

同时使用运行状态文件判断“上次是否异常退出”：

- 启动时写 `active-run.json`
- 正常关闭时写 `last-clean-exit.json` 或移除 active 标记
- 下次启动如果发现存在未清理的 active 标记，则记为“上次运行疑似异常退出”

这能覆盖：

- panic abort 前留下的最后证据
- 无法直接拿到 Windows dump 的普通用户场景

### 4. Frontend Error Capture

前端全局注册：

- `window.onerror`
- `window.onunhandledrejection`

捕获到后通过 Tauri command 写入 diagnostics backend，至少记录：

- message
- stack
- source file / line / column
- current route/view
- timestamp

避免只在浏览器控制台丢失信息。

### 5. Diagnostics UI

在设置页“桌面 / 系统”区域增加诊断区块，展示：

- diagnostics 根目录
- logs 目录
- crashes 目录
- 最近一次崩溃摘要
- 上次运行是否疑似异常退出
- 导出诊断包按钮
- 打开诊断目录按钮

不做复杂日志浏览器，只给出状态和导出能力。

### 6. Export Bundle

新增命令生成 zip 诊断包，建议包含：

- 环境摘要
- 运行状态摘要
- 最近 200 条 route attempt logs
- 最近 100 条 session runs
- 最近 100 条 session run events（截断 payload）
- 最近若干日志文件
- 最近 1 条 crash 摘要
- 最近前端异常日志

不默认打包完整 `workclaw.db`，降低隐私和体积风险。

导出结果写到 `diagnostics/exports/`，并返回路径给前端。

### 7. Privacy and Scope

诊断包只保留排查所需最小数据：

- 对长文本内容截断
- 不导出完整会话正文历史
- 不导出完整数据库
- 不自动联网

后续如需更深入排查，再增加“手动附带数据库”的单独选项，而不是默认行为。

## Data Flow

### Runtime Event Flow

1. 应用启动
2. diagnostics 初始化目录与本次运行状态
3. 关键模块显式写事件日志
4. 若前端异常，则通过命令落盘
5. 若 panic，则 panic hook 写 crash 摘要
6. 正常退出时清理 active-run 状态

### Export Flow

1. 用户点击“导出诊断包”
2. 后端收集摘要数据与最近文件
3. 组装临时导出目录
4. 生成 zip
5. 返回 zip 路径供前端打开

## Testing Strategy

### Backend

- 诊断目录初始化测试
- 结构化日志写入测试
- 前端异常写入测试
- 上次异常退出判定测试
- crash summary 读取测试
- 诊断包生成测试

### Frontend

- 设置页展示诊断路径和状态
- 点击导出诊断包触发对应 invoke
- 导出成功后展示路径/提示

## Risks

- panic hook 在极端情况下可能来不及写完整文件，因此要尽量同步、最小化写入逻辑
- diagnostics 数据如果无限增长会占用磁盘，需要后续加入保留策略
- 若日志写入过多可能影响性能，因此第一版只做关键路径打点

## Rollout

第一版上线后，面向类似“个别用户闪退”问题，新的排查话术统一为：

1. 打开设置
2. 导出诊断包
3. 提交 zip

若连设置页都进不去，再退回到 Windows 事件查看器和系统 dump 方案。
