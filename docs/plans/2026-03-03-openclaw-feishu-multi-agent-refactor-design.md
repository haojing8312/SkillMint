# OpenClaw Feishu Multi-Agent Refactor Design

## 背景与目标

当前项目已经具备“智能体员工 + 飞书事件接入 + 多角色会话派发”的基础能力，但路由语义、隔离边界和工具策略尚未与 OpenClaw 多智能体核心保持一致。  
本次重构目标是：

- 直接内置 OpenClaw 多智能体核心能力（源码移植，不依赖外部 OpenClaw 进程）
- 先聚焦飞书通道打通端到端可用链路
- 在前端提供用户友好的快速配置向导
- 保证未来可低成本跟进 OpenClaw 上游升级

非目标（本期不做）：

- 全量支持 OpenClaw 所有渠道（Slack/Discord/WhatsApp 等）
- 先期就做跨渠道统一规则引擎 UI
- 重写 OpenClaw 核心路由与策略逻辑

## 关键决策

- 采用方案 A：`Vendor OpenClaw Core + SkillHub Adapter`
- OpenClaw 上游代码按 MIT 协议引入，保留版权与许可声明
- 飞书作为唯一上线通道，其他渠道字段仅做结构预留
- 规则优先级与会话 key 语义优先对齐 OpenClaw，而非延续旧行为

## 架构设计

### 1) 分层边界

- `apps/runtime/sidecar/vendor/openclaw-core/`  
  放置“原样迁入 + 最小构建补丁”的 OpenClaw 核心子集（routing/agent-scope/tool-policy/sandbox-policy）。
- `apps/runtime/sidecar/src/openclaw-bridge/`  
  SkillHub 自定义适配层，负责：
  - Feishu 事件 -> OpenClaw 路由输入
  - 路由结果 -> SkillHub session/employee 模型映射
  - 配置读写映射（UI 配置 <-> 内部规则）

约束：

- vendor 内禁止业务逻辑改写
- 业务差异只在 bridge 层实现

### 2) 升级机制（Upstream-Friendly）

- 新增 `UPSTREAM_COMMIT` 锚点（记录上游基线 commit）
- 新增 `scripts/sync-openclaw-core.mjs`：按清单同步指定文件到 vendor 目录
- 新增 `vendor/openclaw-core/PATCHES.md`：记录本地补丁（文件、原因、风险、回收策略）
- 新增回归测试向量：同一输入必须得到与 OpenClaw 对齐的路由结果

目标：后续升级步骤固定为“同步 -> 重放补丁 -> 回归测试”。

## 飞书路由与会话模型

### 1) 入站标准化

基于现有 `ImEvent` 扩展路由输入字段：

- `channel = "feishu"`
- `account_id`（飞书 app 账号维度）
- `peer.kind/peer.id`（direct/group/channel/thread 归一）
- `parent_peer`（线程父会话继承）
- `tenant_id`
- `member_role_ids`（预留）

`feishu_gateway` 将 webhook/ws 事件统一映射为上述结构。

### 2) 路由决策优先级

复用 OpenClaw `resolveAgentRoute` 语义：

- `peer` 精确命中
- `guild + roles`（飞书先预留）
- `guild`（预留）
- `team`（预留）
- `account`
- `channel`
- `default`

输出标准结果：

- `agent_id`
- `session_key`
- `main_session_key`
- `matched_by`

### 3) 会话映射与兼容

- 保留 `agent_employees`，新增 OpenClaw 对齐字段：
  - `openclaw_agent_id`
  - `routing_priority`
  - `enabled_scopes`
- `im_thread_sessions` 会话生成改为“按 `session_key` 稳定映射”避免重复建会话
- 旧 `thread_employee_bindings` 作为覆盖层：
  - 显式绑定存在时优先
  - 否则走 OpenClaw 自动路由

## 配置与用户体验

### 1) 飞书多智能体配置向导

在 Settings/Feishu 提供 4 步向导：

1. 飞书连接配置与连通性验证
2. 员工身份绑定（role/open_id）
3. 路由规则可视化配置（优先级拖拽）
4. 模拟事件测试（显示命中 `agent_id/matched_by/session_key`）

### 2) 配置表现层

- 默认用户不接触 OpenClaw 原始配置文件
- 提供“专家模式”查看只读 JSON
- 每条规则显示启用状态、命中次数、最近命中时间

### 3) 平滑迁移

- 首次进入向导时自动导入旧绑定配置
- 导入后保留旧表结构，避免一次性破坏性迁移
- UI 显示“新路由/旧覆盖”的生效路径，降低认知负担

## 错误处理与可观测性

- 路由异常：自动回退 default agent，trace 记录 fallback 原因
- 配置校验失败：局部规则失效但主链路不中断
- 重复事件：继续使用 `im_event_dedup`
- vendor 升级冲突：保持旧 vendor 可运行，并输出升级报告

前端 Trace 新增路由决策卡片：

- 输入事件摘要
- 命中规则
- 生成 `session_key`
- 派发结果与耗时

## 测试与验收

### 1) 测试策略

- 单元测试：
  - 路由优先级一致性
  - `session_key` 稳定性
  - 旧覆盖规则与新路由共存行为
- 集成测试：
  - 飞书事件 -> 路由 -> 会话 -> 回包全链路
  - 向导配置保存后即时生效
- 升级回归：
  - 固定测试向量对比 vendor 行为

### 2) 验收标准

- 不依赖外部 OpenClaw 进程
- 飞书群消息可稳定命中目标员工
- 用户可在 5 分钟内完成配置并验证
- 升级流程具备脚本化与可追踪补丁记录

## 风险与缓解

- 风险：vendor 代码与现有 TypeScript 工程边界不清导致耦合增长  
  缓解：强制只从 bridge 暴露稳定接口
- 风险：旧数据迁移行为与新路由冲突  
  缓解：引入覆盖优先级标识与回滚开关
- 风险：升级时局部 patch 漂移  
  缓解：PATCHES.md + 自动化 smoke 测试
