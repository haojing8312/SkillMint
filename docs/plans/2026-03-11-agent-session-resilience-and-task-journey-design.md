# Agent Session Resilience and Task Journey Design

**背景**

当前 WorkClaw 的会话主数据以 SQLite 为中心，用户消息会即时入库，但 assistant 的流式 token、工具轨迹、错误终态和交付摘要大多依赖“本轮流式执行完整结束后一次性保存”。这套模型在正常路径下足够简单，但在以下场景会暴露系统性缺陷：

- 上游模型欠费、额度耗尽、鉴权失败
- SSE 流结束标记已到达，但连接未及时关闭
- 前端看到了流式内容，后端却没有完成最终落库
- 任务进度与交付结果以“整场会话聚合卡片”的形式显示在主消息流顶部，破坏时间顺序

你给出的案例同时命中了这四个问题：模型欠费导致本轮失败；前端仍显示“正在分析任务”；导出文件缺失流式过程中已经展示过的内容；任务完成卡跑到了消息流最上方。

## 目标

本次设计解决四个核心问题：

1. 会话记录必须具备实时或准实时持久化能力，避免异常中断后丢失过程记录。
2. 上游模型失败必须进入明确终态，尤其要单独识别欠费/额度不足类问题。
3. 任务进度与交付结果必须遵守消息时间顺序，不能作为全局卡片悬浮在会话顶部。
4. 保留当前 SQLite 的快速查询能力，同时引入类似 Claude Code / Codex / OpenClaw 的文件级会话可审计性。

## 非目标

- 本轮不重做全部聊天 UI 风格。
- 本轮不引入复杂的多端同步协议。
- 本轮不把每个 token 都单独写数据库。
- 本轮不改变现有工具语义，仅增强运行态和持久化。

## 方案选型

### 方案 A：继续 DB-only

做法：

- 将更多流式状态增量写入 SQLite
- 为运行中消息增加草稿字段

优点：

- 改动面最小
- 查询和列表能力天然存在

缺点：

- 仍缺乏可审计的文件级原始记录
- 异常排障体验差
- 会话导出依旧依赖数据库投影是否完整

### 方案 B：改为 File-only

做法：

- 所有会话事件写入 `jsonl` 或 `md`
- UI 直接从文件回放

优点：

- 最接近 Codex / OpenClaw 的会话痕迹
- 崩溃恢复友好

缺点：

- 搜索、过滤、统计会退化
- 会话列表和侧栏聚合都需要额外扫描

### 方案 C：Hybrid Session Journal + SQLite Projection

做法：

- 文件日志作为事实源
- SQLite 作为查询投影和索引
- 前端优先读取投影，必要时可从 journal 恢复

优点：

- 兼顾可靠性、可审计性与 UI 查询性能
- 对异常中断最稳健
- 适合后续扩展 run 级别状态、恢复、导出、回放

缺点：

- 需要引入双写和重放逻辑
- 设计上必须明确“事实源”和“投影”的边界

**推荐：方案 C。**

这是最适合 WorkClaw 当前阶段的路线。它不会抛弃现有 SQLite 资产，但能补上会话过程不可恢复的问题。

## 目标架构

### 1. Session Journal 作为事实源

每个 session 在 `app_data_dir/sessions/<session_id>/` 下维护独立目录：

- `events.jsonl`
- `state.json`
- `transcript.md`
- `artifacts.json`

其中：

- `events.jsonl` 记录追加型事件
- `state.json` 记录最新快照，便于快速恢复
- `transcript.md` 记录面向用户可读的准实时抄本
- `artifacts.json` 记录本轮生成文件、失败项、交付摘要等聚合结果

### 2. SQLite 作为投影层

数据库继续承载：

- sessions 列表
- messages 列表
- 路由尝试日志
- 右侧面板聚合数据

但数据库中的 assistant 消息不再是唯一事实源，而是 journal 重放后的投影。

新增建议表：

- `session_runs`
- `session_run_events`
- `message_drafts`

最小化原则：

- journal 必须先写成功，再更新 SQLite 投影
- SQLite 丢失时可从 journal 重建
- 导出会话时优先读取投影，若发现当前 run 未完成则回退 journal 补齐

## 运行态模型

当前系统把一次 `send_message` 视为一轮黑盒执行。后续应显式引入 `run_id`。

### Run 状态机

每轮 assistant 执行拥有独立 `run_id`，状态限定为：

- `queued`
- `thinking`
- `tool_calling`
- `waiting_user`
- `completed`
- `failed`
- `cancelled`

### Run 事件类型

建议最小事件集：

- `user_message_created`
- `assistant_run_started`
- `assistant_chunk_appended`
- `tool_started`
- `tool_completed`
- `tool_failed`
- `provider_attempt_started`
- `provider_attempt_failed`
- `assistant_run_completed`
- `assistant_run_failed`
- `assistant_run_cancelled`

assistant 失败不应等价于“没有消息”。失败本身就是一条需要持久化、可回放、可展示的运行结果。

## 持久化策略

### 实时/准实时写入原则

不建议逐 token 写盘，但必须准实时 flush。

推荐节奏：

- token 缓冲每 `500ms` 或达到 `512-1024` 字符写入一次 `assistant_chunk_appended`
- 工具开始/结束即时写入 event
- provider 错误即时写入 event
- run 终态即时写入 event 和快照

这样即使窗口被关掉、应用崩溃、模型欠费或网络断开，也最多损失最后半秒左右的缓冲内容。

### Transcript 策略

`transcript.md` 不应只在上下文压缩时生成，而应按 run 持续更新：

- 用户消息立即追加
- assistant 流式文本按批次追加
- 工具调用以折叠块或简洁列表形式追加
- run 失败时追加错误总结

这会让“会话导出”不再是额外动作，而是天然产物。

## 错误分类与终态收口

本次案例说明，系统需要把“上游失败”作为一等公民处理，而不是通用异常。

### Provider 错误分类

建议扩展为：

- `auth_invalid`
- `insufficient_balance`
- `quota_exceeded`
- `rate_limited`
- `timeout`
- `network`
- `provider_protocol`
- `unknown`

其中 `insufficient_balance` 和 `quota_exceeded` 要单独展示，不应混在“网络错误”或“未知错误”里。

### 终态收口原则

无论失败发生在：

- 首次请求模型前
- 流式响应中
- 工具执行后再次请求模型时
- 所有候选模型均失败后

系统都必须执行统一收尾：

1. 写入 `assistant_run_failed`
2. flush 当前流式缓冲
3. 更新 SQLite 投影
4. 向前端发出 `done=true`
5. 向前端发出终态 `failed`
6. 在消息流中插入失败结果卡片

不能再出现“前端看见过程但最终没有保存”的情况。

## 主会话区信息架构

### 当前问题

现在 `TaskJourneySummary` 是整场会话聚合结果，而且被固定渲染在消息列表之前。这会造成：

- 交付结果跑到会话最上面
- 用户最新提问反而出现在它下面
- 聚合卡看起来像“当前正在发生”，实际却可能是上一次 run 的结果

这违反了消息产品最重要的原则：**时间顺序必须优先于运营摘要。**

### 新原则

主会话区只允许两类运行态展示：

1. 顶部 `live banner`
   - 仅表示“当前正在运行的 run”
   - 文案如：`正在分析任务`、`正在调用工具`、`等待用户确认`
   - 一旦 run 结束，banner 立即消失
2. 内联 `run summary card`
   - 绑定到对应 `run_id`
   - 出现在该轮 assistant 消息之后
   - 包含该轮任务进度、交付结果、失败项

### 不再允许

- 会话级交付卡固定显示在消息流顶部
- 把整场会话聚合结果伪装成当前 run 结果
- 让右侧面板和主消息区显示同一份聚合卡，但位置和语义不同

## 任务进度与交付结果重构

### 会话主区

主区展示“run 级别”的进度和交付：

- 当前步骤
- 本轮生成的文件
- 本轮失败项
- 本轮继续补做入口

### 右侧面板

右侧面板继续保留“会话级聚合”能力，但语义要明确：

- `当前运行`
- `最近一次交付`
- `本会话产物`
- `本会话错误`

换言之：

- 主区是时间顺序和因果关系
- 侧栏是聚合检索和横向回看

## 数据建模建议

前端消息模型建议增加：

- `messageId`
- `runId`
- `terminalState`
- `draft`
- `deliverySummary`
- `failureSummary`

这样才能把 `TaskJourneySummary` 从“全局聚合组件”改造成“每条 assistant 回复的附属卡片”。

后端读取接口建议增加：

- `list_session_runs(session_id)`
- `get_session_run_events(session_id, run_id)`
- `get_session_live_state(session_id)`
- `rebuild_session_projection(session_id)`

## 兼容与迁移

历史会话没有 journal，需要兼容：

- 保持现有 `messages` 读取逻辑
- 新会话启用 journal
- 对旧 assistant JSON 格式继续兼容 `streamItems`
- 仅在需要恢复或导出增强时回退旧逻辑

迁移原则：

- 先双写，不立刻移除旧路径
- 验证 1-2 个版本后再把 journal 提升为默认导出源

## 观测与告警

需要新增可观测指标：

- run 未正常终止率
- `done=true` 缺失率
- 欠费/额度不足错误次数
- 流式缓冲未落盘恢复次数
- journal 重放恢复次数

当发现以下异常时应主动提示用户：

- 模型余额不足
- 当前模型不可用，已尝试切换候选失败
- 会话异常中断，已从本地运行日志恢复

## 分阶段实施

### 第一阶段：可靠性兜底

- 为 provider 失败补终态
- 为流式缓冲补准实时 journal
- 为欠费错误补专用分类
- 为导出逻辑补 journal 回退

### 第二阶段：前端运行态重构

- live banner 仅代表当前 run
- 去掉主区顶部的全局任务聚合卡
- 改为在对应 assistant 消息后渲染 run summary

### 第三阶段：恢复与运维

- 支持从 journal 重建 SQLite 投影
- 支持打开 session 文件夹
- 支持导出完整会话目录

## 成功标准

以下条件全部满足才算改造成功：

1. 用户关闭应用或模型欠费后，已展示过的过程信息不会整体丢失。
2. 欠费、额度不足、鉴权失败等错误会明确展示并进入失败终态。
3. 主消息流中不再出现“交付结果跑到最上面”的展示错误。
4. 会话导出能够包含当前 run 的过程与失败原因，而不是只保留用户消息。
5. 旧会话可继续读取，新会话具备 journal 能力。
