# 多智能体员工团队协作（主/子员工调度）设计稿

## 1. 背景与目标

当前系统已具备：

- 飞书消息可入站并创建/映射桌面会话
- 主员工可在桌面端处理任务并向飞书同步分段输出
- 会话列表可标记来源（如飞书）

当前主要缺口：

- 主员工缺少稳定的“自动委派子员工并回收结果”的闭环
- 飞书 `@子员工` 时，命中与响应不稳定
- “需求澄清/用户确认”仍偏桌面弹框，不是跨端同构消息
- 桌面端消息可见性里，主/子员工身份区分不够明确

本设计目标：

1. 未 `@` 群消息默认由主员工接管，并可自动委派子员工
2. `@子员工` 时可直达子员工执行
3. 主/子员工消息在桌面端与飞书端统一可识别
4. 需求澄清改为消息协议，不再依赖桌面独占弹框
5. 不增加普通用户难理解的配置（如线程绑定）

## 2. 范围与非目标

范围：

- 飞书群会话团队协作（主员工 + 子员工）
- 桌面端消息结构、状态展示、身份标签
- 后端调度协议、状态机、失败兜底

非目标：

- 本期不做多渠道统一（先聚焦飞书）
- 不引入复杂可视化路由编排器
- 不要求用户手动配置每条线程绑定关系

## 3. 关键决策

1. 路由入口统一
- 群消息未 `@`：默认路由主员工
- 明确 `@某员工`：优先路由到该员工

2. 协议化委派（而非自然语言猜测）
- 主员工通过结构化委派事件发起子任务
- 子员工通过结构化结果事件回传

3. 澄清请求消息化
- `clarify_request` 与 `clarify_reply` 走同一消息总线
- 飞书与桌面都可发起/回复澄清

4. 1:1 机器人绑定
- 每个员工绑定一个飞书机器人（已在产品方向中确认）
- 群聊团队协作依赖“多机器人同群 + 协议调度”

## 4. 领域模型与数据协议

### 4.1 领域实体

- `Employee`: 员工定义（主员工、子员工、能力标签、机器人绑定）
- `TeamSession`: 团队会话（一个会话可有多员工参与）
- `Task`: 任务单元（主任务/子任务，通过 `parent_task_id` 串联）
- `DispatchRecord`: 委派记录（谁委派给谁、状态、超时、结果）

### 4.2 统一消息信封（MessageEnvelope）

```json
{
  "msg_type": "user_input|agent_chunk|agent_final|delegate_request|delegate_result|clarify_request|clarify_reply|system|error",
  "session_id": "session_xxx",
  "thread_id": "oc_xxx",
  "task_id": "task_xxx",
  "parent_task_id": "task_parent_xxx",
  "sender_role": "user|main_agent|sub_agent|system",
  "sender_employee_id": "project_manager",
  "target_employee_id": "tech_lead",
  "content": "text",
  "correlation_id": "corr_xxx",
  "source_channel": "desktop|feishu",
  "ts": 1710000000
}
```

约束：

- `task_id + correlation_id` 幂等
- `delegate_request` 必须带 `target_employee_id`
- `delegate_result` 必须带 `parent_task_id`

## 5. 消息流设计

### 5.1 普通群消息（未 `@`）

1. 飞书事件入站，标准化成 `user_input`
2. 路由器命中主员工（`is_main=true`）
3. 主员工开始流式输出（`agent_chunk`）
4. 主员工判断需专项能力时发送 `delegate_request`
5. 调度器创建子任务并触发子员工执行
6. 子员工输出 `agent_chunk/agent_final`，并产出 `delegate_result`
7. 主员工收到结果后汇总，输出 `agent_final`
8. 所有输出同步桌面与飞书

### 5.2 指定 `@子员工`

1. 入站事件提取 mention，映射到 `target_employee_id`
2. 路由直达子员工执行
3. 子员工直接回复（可选是否回传主员工汇总）
4. 桌面端显示为“子员工直接接管”

### 5.3 澄清请求（需求确认）

1. 任一员工触发 `clarify_request`
2. 会话状态切换 `WAITING_USER`
3. 飞书与桌面同时显示“待用户确认”
4. 用户任一端回复触发 `clarify_reply`
5. 任务从原状态恢复继续执行

## 6. 状态机设计

会话主状态：

- `IDLE`
- `ROUTING`
- `MAIN_RUNNING`
- `DELEGATING`
- `SUB_RUNNING`
- `MERGING`
- `WAITING_USER`
- `COMPLETED`
- `FAILED_RETRYABLE`
- `FAILED_TERMINAL`

核心迁移：

1. `IDLE -> ROUTING`: 收到新用户消息
2. `ROUTING -> MAIN_RUNNING|SUB_RUNNING`: 路由完成
3. `MAIN_RUNNING -> DELEGATING`: 主员工发出委派
4. `DELEGATING -> SUB_RUNNING`: 子任务启动
5. `SUB_RUNNING -> MERGING`: 子任务返回结果
6. `MERGING -> MAIN_RUNNING|COMPLETED`: 汇总后继续或结束
7. `*_RUNNING -> WAITING_USER`: 触发澄清
8. `WAITING_USER -> *_RUNNING`: 用户回复澄清
9. 任意运行态 -> `FAILED_RETRYABLE|FAILED_TERMINAL`: 异常分级

## 7. 失败兜底策略

1. Mention 解析失败
- 回退主员工接管
- 系统消息提示“未识别目标员工，已由主员工处理”

2. 子员工不可用/离线
- 主员工降级直接处理
- 输出降级说明，不阻断主链路

3. 飞书发送失败
- 本地执行不中断
- 进入重试队列，最终补发摘要

4. 流式回传异常
- 自动切换“分段摘要模式”（固定节拍输出）

5. 澄清消息投递失败
- 继续保持 `WAITING_USER`
- 双通道重试并在桌面端给出告警

6. 重复事件
- 通过 `event_id/correlation_id` 幂等去重

7. 会话映射失效（会话被删）
- 自动重建会话并写入“会话已重建”系统消息

8. 子任务超时
- 主员工收到超时事件后给出两种继续策略：
  - 重试子任务
  - 跳过子任务给出保守方案

## 8. UI 设计（主/子员工区分）

### 8.1 消息区

每条员工消息显示：

- 员工名称（如“项目经理”）
- 身份标签：`主员工` / `子员工`
- 任务标识：`主任务` / `子任务`
- 渠道标识：`飞书` / `桌面`

视觉建议：

- 主员工标签：蓝色
- 子员工标签：绿色/橙色（按角色配置）
- 流式时显示“{员工名} 正在回复…”

### 8.2 委派卡片

在消息流插入系统卡片：

- `项目经理 -> 开发团队`
- 状态：`进行中 / 已完成 / 已失败 / 已超时`
- 可展开查看子任务摘要与耗时

### 8.3 会话列表

- 会话来源徽标（飞书）继续保留
- 增加“团队会话”轻量标记（当会话存在至少一次委派）

## 9. 默认调度策略（零配置）

用户无需新增复杂配置；默认策略内置：

1. 群里未 `@` 一律主员工先接管
2. 主员工在以下场景自动委派子员工：
- 明确出现“技术方案/实现细节/架构评审”等技术关键词
- 主员工自评置信度不足（由策略阈值判定）
3. 子员工结果必须回传主员工汇总对外输出
4. 用户 `@某子员工` 时允许子员工直出

用户可配置的仅保留：

- 主员工是谁
- 员工与飞书机器人 1:1 绑定

## 10. 可观测性与验收

### 10.1 关键指标

- `route_main_hit_rate`: 未 `@` 命中主员工比例
- `delegate_success_rate`: 委派成功率
- `clarify_roundtrip_latency`: 澄清往返耗时
- `cross_channel_consistency_rate`: 双端消息一致率

### 10.2 验收标准

1. 未 `@` 群消息，主员工稳定接管且双端可见
2. 主员工可自动委派子员工，子结果回传并汇总
3. `@子员工` 可直接触发并回复
4. 需求澄清在飞书可见且可从飞书继续任务
5. 桌面端可明确区分主/子员工回复身份

## 11. 分期落地建议

P1（优先）：

- 协议化委派闭环 + UI 身份标签 + 澄清消息化

P2：

- 失败重试队列 + 超时策略 + 指标埋点看板

P3：

- 调度策略可视化（仅专家模式）
