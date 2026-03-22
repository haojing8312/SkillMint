# Feishu Runtime Unification Design

**Date:** 2026-03-22

## Goal
把 WorkClaw 的飞书链路从“收消息走官方插件 runtime、发消息走 sidecar”重构成“收发、授权、审批、状态都走官方插件 runtime”，并把 sidecar 中所有 Feishu 相关职责彻底移除。

## Problem Summary
- 当前飞书收发不走同一条链路：
  - 收消息、授权、pairing 主要依赖官方插件 runtime
  - 发消息仍依赖 sidecar 的 `/api/feishu/send-message`
- 这会导致一类系统性问题：
  - 能收不能发
  - 重启后 runtime 和 sidecar 状态不一致
  - 诊断难度高，用户无法理解为什么“桌面有回复、飞书没回复”
- 产品层面的很多体验问题，本质上是底层架构分裂造成的，不适合继续靠补丁修文案解决。

## Product Decision
分两阶段推进：

### Phase 1
先统一飞书底层架构：
- 收消息：官方插件 runtime
- 发消息：官方插件 runtime
- 授权：官方插件 runtime
- pairing / 审批：官方插件 runtime + WorkClaw 状态机
- 状态展示：基于官方插件 runtime
- sidecar：不再承担任何 Feishu 职责

### Phase 2
在统一架构基础上，再优化产品功能和引导体验：
- 首次接入向导
- 创建 / 绑定机器人流程
- 批准接入提示
- 接待员工配置
- 错误与诊断体验

## Non-goals For Phase 1
- 不重做整个 IM 架构
- 不改 WeCom 或其他连接器
- 不改 sidecar 的浏览器、MCP 或其他非 Feishu 能力
- 不在这一阶段追求 UI 全面重构

## Current Architecture

### Inbound
- 官方插件 runtime 通过 `run-feishu-host.mjs` 持有 Feishu WebSocket 长连接
- runtime 事件进入 Tauri `openclaw_plugins.rs`
- Tauri 再桥接到 `feishu_gateway.rs` / `runtime_bridge.rs` / `App.tsx`

### Outbound
- Tauri `feishu_gateway.rs` 里的 `send_feishu_text_message*`
- 通过 sidecar HTTP：
  - `/api/feishu/send-message`
  - `localhost:8765`
- sidecar 再调用 Feishu 发送接口

### Resulting Mismatch
- 官方 runtime 正常，不代表 sidecar 正常
- sidecar 已启动，不代表官方 runtime 已授权
- 自动恢复时必须同时考虑两套运行时
- 一旦 sidecar 未就绪，飞书端就表现为“无回复”，但桌面端已经有完整输出

## Target Architecture

### Single Feishu Runtime Boundary
WorkClaw 中与 Feishu 相关的能力统一只通过官方插件 runtime 暴露：

- inbound event stream
- outbound message dispatch
- pairing request notification
- pairing approval / rejection
- auth status
- connection status
- diagnostics / recent logs

### Sidecar Boundary After Phase 1
sidecar 保留：
- browser automation
- MCP 相关桥接
- WeCom 或其他尚未迁出的连接器职责

sidecar 删除：
- `/api/feishu/send-message`
- `/api/feishu/list-chats`
- `/api/feishu/ws/start`
- `/api/feishu/ws/stop`
- `/api/feishu/ws/status`
- `/api/feishu/ws/drain-events`
- `/api/feishu/ws/reconcile`
- 及其对应的 Feishu adapter / client / tests

## Design Principles
- 单一真相来源：Feishu 运行时状态只来自官方插件 runtime
- 单一收发通道：Feishu outbound 不再经过 sidecar
- 兼容优先：Phase 1 不重做上层 IM 路由，先替换底层 transport
- 用户心智优先：产品层不再暴露“sidecar 是否在线”这种内部概念
- 迁移可验证：每一步都要能通过自动测试和真实飞书手测证明

## Required Backend Changes

### 1. Official Runtime Outbound Contract
需要在官方插件 host 层补齐 WorkClaw 可直接调用的 outbound 能力：
- 发送文本消息
- 必要时发送 markdown / 富文本
- 明确目标 thread / chat / reply target
- 返回统一的成功 / 失败结果和诊断信息

建议实现方式：
- 在 `run-feishu-host.mjs` 和 `plugin-host` runtime 中新增“send”命令或 host method
- 由 `openclaw_plugins.rs` 维护该 runtime 进程的 stdin / command 通道
- Tauri 侧提供 `send_feishu_text_message_via_plugin_runtime` 之类的能力给 gateway 调用

### 2. Gateway Rewire
`feishu_gateway.rs` 中所有 outbound 路径应改为：
- 不再依赖 `sidecar_base_url`
- 优先 / 仅通过官方插件 runtime 发送消息
- 如果 runtime 未启动，按统一策略：
  - 自动恢复
  - 恢复失败则返回明确错误

### 3. Unified Status And Recovery
自动恢复只围绕一套 runtime 运行：
- startup bootstrap
- settings page status
- auth refresh
- approval flow
- diagnostics

这意味着：
- 不再需要为 Feishu 额外等待 sidecar 健康检查
- “启动连接”按钮只作用于官方插件 runtime

### 4. Diagnostics
飞书诊断需要聚焦官方 runtime：
- 最近 inbound 事件
- 最近 outbound 尝试
- 最近 auth / pairing 状态
- 最近 runtime error

不要再把 sidecar 的 Feishu health 作为诊断前提。

## Required Frontend Changes

### 1. App IM Bridge
`App.tsx` 当前在 Feishu bridge 里会通过 `send_feishu_text_message` 触发回推。
Phase 1 中：
- 调用命令名可以不变，减少前端改动面
- 但其后端实现必须改成 official runtime path

### 2. Settings Surface
`SettingsView.tsx` 中 Feishu 状态展示要移除任何 sidecar 概念：
- 不再把 sidecar 健康当作飞书可发消息的前提
- 状态只展示：
  - 环境
  - 官方插件
  - 机器人信息
  - 授权 / pairing
  - runtime 运行状态

### 3. Diagnostics UI
高级控制台里不再展示 sidecar Feishu 调试项，只展示官方 runtime 输出和相关日志。

## Migration Strategy

### Step A
先新增 official runtime outbound 能力，不立刻删 sidecar Feishu 代码。

### Step B
让 `feishu_gateway.rs` 的 outbound 调用切到 official runtime。

### Step C
补齐回归测试和真实飞书收发手测。

### Step D
确认 inbound / outbound / pairing / restart restore 都稳定后，删除 sidecar Feishu 相关接口、实现和测试。

### Step E
再进入 Phase 2 产品体验优化。

## Compatibility Notes
- SQLite schema 预计不需要为了 transport 迁移增加新表，但若诊断或状态缓存新增字段，必须保持向后兼容。
- 对外 Tauri command 名称可暂时保持稳定，优先避免前端大面积改动。
- 当前 onboarding、pairing、employee routing 逻辑尽量不变，只替换底层 outbound transport。

## Success Criteria For Phase 1
- 飞书消息进入 WorkClaw 后，本地会话生成的回复可以稳定回发到飞书
- 重启 WorkClaw 后，Feishu runtime 可自动恢复，且收发都基于同一 runtime
- 不再出现“桌面端有回复、飞书端无回复，但 UI 误显示已正常接通”的分裂问题
- sidecar 中不再保留任何 Feishu 接口或 Feishu transport 代码

## Phase 2 Preview
当 Phase 1 完成后，再统一优化：
- 首次接入流程
- 创建 / 绑定机器人引导
- 审批状态显示
- 接待员工设置闭环
- 面向普通用户的错误提示和诊断收敛
