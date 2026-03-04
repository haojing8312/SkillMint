# 智能体员工飞书多长连接与自动监听设计

## 背景

当前 `智能体员工` 页面已经支持员工级飞书凭据录入，但运行时仍是“单长连接”模型：

- Sidecar 的飞书 WS 管理器只有一个 `wsClient`，新启动会覆盖旧连接。
- 保存员工只写数据库，不会触发连接重建/自动监听。
- 员工列表没有连接状态反馈（红绿点）。

结果是：即使配置了多个员工凭据，也无法稳定做到“每员工一条飞书长连接 + 自动收消息 + 自动建会话执行”。

## 需求确认

- 每个已绑定飞书凭据且启用的员工，都需要建立并维持长连接。
- 页面用红绿点展示每个员工飞书连接状态。
  - 绿：员工 WS `running=true` 且 relay `running=true`
  - 红：任一未运行/失败/异常
- 飞书来消息后，自动在桌面侧创建（或复用）会话并开始执行。
- 增加心跳与健康自愈策略，避免静默断线长期不可见。

## 非目标

- 不改动员工字段结构（沿用 `feishu_app_id/feishu_app_secret/feishu_open_id`）。
- 不引入 webhook 模式（本期只做 websocket 长连接）。
- 不重做 IM 路由模型（复用现有 `ensure_employee_sessions_for_event_with_pool`）。

## 方案概述（推荐）

采用“OpenClaw 风格的多账户连接管理”：

1. Sidecar 从“单连接”升级为“按员工 ID 管理多连接”。
2. Rust 侧新增“期望态对齐 + 健康监督器”，负责自动拉起、重连、限频恢复。
3. 前端员工列表轮询状态，展示红绿点与错误信息。
4. 飞书事件记录携带 `employee_id`，确保入站消息绑定到正确员工会话。

## 架构设计

### 1) Sidecar：多连接管理器

改造 [`feishu_ws.ts`](E:/code/yzpd/skillhub/apps/runtime/sidecar/src/feishu_ws.ts)：

- `Map<string, ConnectionState>` 管理所有员工连接（key=employee_id）。
- 每个连接持有：
  - `wsClient`
  - `running`
  - `started_at`
  - `last_event_at`
  - `last_error`
  - `reconnect_attempts`
- 事件队列从“全局无归属”改为“事件含 employee_id”。

新增/调整 sidecar API（[`index.ts`](E:/code/yzpd/skillhub/apps/runtime/sidecar/src/index.ts)）：

- `POST /api/feishu/ws/reconcile`
  - 输入：员工凭据清单（employee_id + app_id + app_secret）
  - 行为：差量启动、凭据变更重建、删除停连
- `POST /api/feishu/ws/status`
  - 输出：每员工连接状态集合
- `POST /api/feishu/ws/drain-events`
  - 输出：带 `employee_id` 的事件

兼容策略：

- 保留旧 `/start|/stop|/status` 入口一段时间，内部可委托到 reconcile 逻辑。

### 2) Rust：期望态对齐 + relay 自愈

改造 [`feishu_gateway.rs`](E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/commands/feishu_gateway.rs)：

- 增加“读取启用员工飞书凭据”的函数，构造 sidecar reconcile payload。
- 新增 `reconcile_feishu_employee_connections_with_pool(...)`：
  - 调 sidecar `/api/feishu/ws/reconcile`
  - 确保 relay 运行（未运行则自动启动）
- `sync_feishu_ws_events_core` 读取事件时，把 `employee_id` 映射到 `ImEvent.role_id`，确保路由命中对应员工。

改造 [`employee_agents.rs`](E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/commands/employee_agents.rs)：

- `upsert_agent_employee`/`delete_agent_employee` 后触发 reconcile（异步容错，不影响 DB 成功提交）。

改造 [`lib.rs`](E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/lib.rs)：

- 启动时从“单凭据恢复”改为“全员工凭据对齐 + relay 恢复”。
- 新增健康监督循环（类似 OpenClaw）：
  - 周期检查连接状态
  - 非手动停止状态下自动重启
  - 指数退避 + 每小时重启上限

### 3) 前端：红绿点连接状态

改造 [`EmployeeHubView.tsx`](E:/code/yzpd/skillhub/apps/runtime/src/components/employees/EmployeeHubView.tsx)：

- 进入页面后轮询连接状态（例如 5 秒）。
- 员工列表每项展示状态点：
  - 绿：该员工连接 running 且 relay running
  - 红：绑定凭据但未运行/报错
  - 灰：未绑定凭据或员工未启用
- 在步骤 2 区域显示状态文本与最近错误。

## 数据流（目标行为）

1. 用户保存员工（含飞书凭据）  
2. `upsert_agent_employee` 写库成功  
3. 后端自动 reconcile，多员工长连接进入运行  
4. relay 持续 drain sidecar 事件  
5. 入站事件按 `employee_id` 路由，自动创建/复用桌面会话  
6. 发出 `im-role-dispatch-request`，桌面自动执行用户指令

## 心跳与健康策略

参考 OpenClaw 的“重连+健康监控”思路：

- 重启退避：5s 起步，指数增长，最大 5min。
- 重试阈值：单连接连续失败超过阈值后进入限频恢复。
- 健康巡检：固定周期（如 30s）检查 `running/last_error` 与 relay 状态。
- 冷却与上限：避免抖动重启（例如 2 个周期冷却，每小时最多 3 次自动重启）。
- 可观测性：记录 `last_error/reconnect_attempts/last_event_at`，供 UI 与日志展示。

说明：WebSocket 协议层 ping/pong 由 SDK 处理；本设计增加的是“应用层健康监督与自动恢复”。

## 错误处理策略

- 数据持久化优先：员工保存成功不因连接启动失败而回滚。
- 连接失败透明化：状态点转红并展示 `last_error`。
- 凭据变更即重建：旧连接先停，再以新凭据启动。
- 删除/禁用员工后自动停连，避免幽灵连接。

## 测试策略

- Sidecar 单元测试：
  - 多员工并发启动/停止/重建
  - reconcile 差量正确性
  - drain 事件保留 `employee_id`
- Rust 测试：
  - 员工凭据收集与 reconcile payload
  - 事件到员工会话映射正确性
  - relay/重连状态逻辑
- 前端测试：
  - 红绿灰状态点展示
  - 状态文案与错误提示

## 验收标准

- 新建或更新员工后 10 秒内自动进入连接状态（有凭据且启用时）。
- 多员工可同时 `running=true`（互不覆盖）。
- 飞书消息到达后自动创建/复用会话并触发执行。
- 员工页可见每员工红绿灰状态与错误信息。
- 断连后可在健康监督下自动恢复（符合退避策略）。
