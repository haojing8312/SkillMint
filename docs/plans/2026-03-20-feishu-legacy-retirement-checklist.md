# Feishu Legacy Retirement Checklist

## Strategy Summary
- Change surface: WorkClaw 旧飞书 connector 主入口、旧 gateway/relay/sidecar 链路、员工飞书绑定字段、以及设置页中仍残留的旧连接器心智。
- Affected modules: `apps/runtime/src/components/SettingsView.tsx`、`apps/runtime/src-tauri/src/lib.rs`、`apps/runtime/src-tauri/src/commands/feishu_gateway.rs`、`apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`、`apps/runtime/sidecar/src/index.ts`、`apps/runtime/sidecar/src/feishu_ws.ts`、员工飞书关联 UI 与相关数据库字段。
- Main risk: 旧飞书实现目前仍承担部分官方插件宿主尚未完全接住的桥接职责。过早删除会导致 pairing、普通消息回复、员工接待路由、状态诊断一起失效。
- Recommended smallest safe path: 先让官方插件成为唯一主入口和唯一主运行路径；旧飞书实现只保留为内部桥接和兼容层。等官方插件宿主的消息闭环稳定后，再按批次删除旧代码。
- Required verification: 官方插件安装、关联已有机器人、pairing approve、批准后普通私聊回复、默认接待员工、群聊/mention 策略、设置页状态、runtime 最近日志/最近错误。
- Release impact: 高。飞书属于外部集成面，退役旧实现会改变用户接入和运行方式，必须按阶段验证。

## Goal

将 WorkClaw 的飞书接入收口为：

- 唯一主路径：`@larksuite/openclaw-lark` 官方插件宿主
- 设置页定位：状态、诊断、高级配置
- 员工页定位：接待范围和默认接待员工
- 旧 WorkClaw 飞书实现：先退出主流程，再逐步删除

这份清单只定义“删旧”的顺序和前置条件，不定义新的实现细节。

## Current Boundary

截至当前状态：

- 官方插件安装、识别、runtime 启动、pairing 入库、pairing approve 已经具备。
- 官方插件普通消息已经能进入 WorkClaw 本地路由，但回复链路仍在继续收口。
- 旧 `feishu_gateway` / sidecar Feishu WS 仍承担部分桥接和诊断职责。

因此，当前不适合“一刀切删除旧飞书实现”，而适合：

1. 先断主入口
2. 再断旧主运行路径
3. 最后删除桥接层和旧字段

## Classification

### A. 立刻可下线

这些内容不应该再作为用户可见主流程存在。

1. 设置页中的旧飞书连接器主路径
- 现状：飞书页已经以官方插件为主，但代码中仍保留旧 connector 状态和兼容分支。
- 目标：用户不再通过旧 connector 概念配置飞书。
- 主要位置：
  - `apps/runtime/src/components/SettingsView.tsx`

2. 旧的“保存配置后启动 sidecar ws”主心智
- 现状：前端测试已经限制官方模式下不再触发 `start_feishu_long_connection`。
- 目标：旧 long connection 只能作为内部兼容路径，不再暴露为主流程。
- 主要位置：
  - `apps/runtime/src/components/SettingsView.tsx`
  - `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

3. 旧 connector 诊断文案和 UI 心智
- 现状：部分字段和命令仍以旧 connector 术语存在。
- 目标：统一改为“官方插件 / runtime / pairing / 最近日志 / 最近错误”。
- 主要位置：
  - `apps/runtime/src/components/SettingsView.tsx`

### B. 必须暂留

这些模块当前还承担必要桥接职责，不能直接删。

1. `feishu_gateway.rs`
- 仍承担：
  - pairing request 持久化
  - allow-from 存储
  - 飞书文本消息发送
  - 将官方插件 dispatch 事件桥接进 WorkClaw IM 路由
  - 员工飞书连接状态查询
- 删除前置条件：
  - 官方插件宿主独立完成普通消息回复闭环
  - pairing / allowFrom / 员工路由不再依赖本文件
- 主要位置：
  - `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`

2. `lib.rs` 中的 Feishu relay/bootstrap 和相关命令注册
- 仍承担：
  - 旧 relay 生命周期
  - 旧 gateway/sidecar 命令暴露
  - 部分启动期协调
- 删除前置条件：
  - 官方插件 runtime 稳定接管全部飞书流量
  - 设置页和员工页不再读取旧 relay 状态
- 主要位置：
  - `apps/runtime/src-tauri/src/lib.rs`

3. sidecar 中的 Feishu WS/HTTP 接口
- 仍承担：
  - 旧 Feishu WS 长连接能力
  - 旧 send/list/status/drain/reconcile 端点
  - 某些兼容诊断能力
- 删除前置条件：
  - 官方插件 runtime 状态、最近事件、发送结果都稳定可观测
  - 不再有任何前端/后端命令调用 `/api/feishu/ws/*`
- 主要位置：
  - `apps/runtime/sidecar/src/index.ts`
  - `apps/runtime/sidecar/src/feishu_ws.ts`

4. 员工飞书关联字段与 UI
- 仍承担：
  - 默认接待员工
  - 指定群/会话范围
  - 员工接待状态展示
- 删除前置条件：
  - 员工飞书接待配置完全迁移到官方插件原生配置视图或等价模型
  - 现有员工页不再依赖 `feishu_open_id / feishu_app_id / feishu_app_secret`
- 主要位置：
  - `apps/runtime/src/components/employees/EmployeeFeishuAssociationSection.tsx`
  - `apps/runtime/src/components/employees/EmployeeHubView.tsx`
  - `apps/runtime/src-tauri/src/db.rs`

### C. 删除前必须先替代

这些能力如果没有新的等价实现，删了就会直接丢功能。

1. 普通消息回复闭环
- 现状：pairing 已通过后，普通消息回复仍在继续收口。
- 需要替代完成：
  - 官方插件 dispatch -> WorkClaw 本地执行 -> 正确目标 chat 回复
- 依赖模块：
  - `apps/runtime/plugin-host/src/runtime.ts`
  - `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`
  - `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
  - `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
  - `apps/runtime/src-tauri/src/im/runtime_bridge.rs`

2. pairing 批准后 runtime 生效
- 现状：已有重载逻辑，但必须稳定验证批准后不再重复 pairing。
- 需要替代完成：
  - 批准 -> allowFrom 生效 -> runtime reload 生效

3. 员工接待路由
- 现状：官方插件消息已桥接到 WorkClaw 的 role dispatch，但仍是 WorkClaw 自定义桥。
- 需要替代完成：
  - 默认接待员工
  - 指定群/会话范围
  - 私聊/群聊/话题行为一致

4. 运行时诊断
- 现状：已有最近事件/最近日志/最近错误，但仍混有旧 relay/connector 心智。
- 需要替代完成：
  - 全部诊断以官方插件 runtime 为准
  - 不再需要旧 sidecar WS 状态页

## Retirement Phases

### Phase 1: 断掉旧主入口

目标：
- 让“官方插件安装/绑定/状态”成为唯一飞书主入口

动作：
- 删除或隐藏设置页中仍残留的旧飞书 connector 主文案和主按钮
- 明确前端不再触发旧 `start_feishu_long_connection`
- 统一文案为“官方插件 / runtime / pairing / 诊断”

验收：
- 新用户看不到旧飞书接入主路径
- 设置页中不存在“必须先启动旧 long connection”的心智

### Phase 2: 断掉旧主运行路径

目标：
- 所有飞书主流程只走官方插件 runtime

动作：
- 禁止主流程再依赖旧 relay/bootstrap
- 禁止主流程再依赖 sidecar `/api/feishu/ws/*`
- 主状态来源全部切为官方插件 runtime 状态

验收：
- 安装、绑定、pairing、普通消息回复都不需要旧 long connection
- 设置页状态与 runtime 最近日志一致

### Phase 3: 缩减为内部桥接层

目标：
- 仅保留旧模块中仍未被替代的桥接职责

动作：
- `feishu_gateway.rs` 只保留必要桥接函数
- sidecar Feishu WS 功能不再被前端可见入口调用
- 员工页仅保留当前必须的接待配置

验收：
- 旧链路代码仍存在，但已不参与主流程
- 所有主路径联调都走官方插件 runtime

### Phase 4: 删除旧桥接层

目标：
- 删除旧 gateway / relay / sidecar Feishu 实现

动作：
- 删除不再调用的 Tauri 命令
- 删除 sidecar Feishu WS manager 与相关路由
- 删除不再需要的 DB 字段和 UI 组件

验收：
- 代码库中不再存在旧飞书 connector 主实现
- WorkClaw 只保留官方插件宿主飞书方案

## Preconditions Before Final Deletion

以下条件必须全部满足，才能进入最终删除阶段：

1. `关联已有机器人` 路径稳定
- 安装/绑定/启动无人工补丁步骤

2. `pairing approve` 稳定
- 批准后不再重复 pairing

3. 普通私聊回复稳定
- pairing 后消息能稳定回复

4. 员工接待稳定
- 默认接待员工和指定范围规则生效

5. 诊断稳定
- 最近事件 / 最近日志 / 最近错误 能直接解释问题

6. 前端不再调用旧 sidecar Feishu API

7. Tauri 不再依赖旧 relay/bootstrap 作为主路径

## Practical Delete Map

### 第一批候选删除/下线对象
- `SettingsView.tsx` 中旧 connector 主入口和旧文案
- 任何仍会触发旧 `start_feishu_long_connection` 的前端分支

### 第二批候选删除/下线对象
- `lib.rs` 中旧 Feishu relay bootstrap 主流程
- 旧 Feishu WS 状态/启动命令的用户可见入口

### 第三批候选删除对象
- `sidecar/src/feishu_ws.ts`
- `sidecar/src/index.ts` 中 `/api/feishu/ws/*`
- `feishu_gateway.rs` 中仅服务旧链路的逻辑
- 员工旧飞书字段和相关 DB migration 残留

## Working Rule

退役旧飞书实现时，遵循一个规则：

> 先断入口，再断主流程，再删桥接，最后删数据结构。

不要反过来。

如果需要保守一点，可以在最终删除前保留一个内部开关，但不再让普通用户看到旧飞书路径。
