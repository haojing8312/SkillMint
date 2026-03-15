# Approval Bus OpenClaw Alignment Design

**Problem**

WorkClaw 当前的高风险确认链路仍然是桌面端一次性弹窗加后端内存 `mpsc<bool>` 等待。它有四个根本限制：

1. 仅支持桌面前端确认，无法把飞书作为正式审批面。
2. 默认只等待 15 秒，超时后直接取消，不符合“必须明确审批后才能继续”的产品语义。
3. 审批状态不持久化，应用重启或前端丢事件后无法恢复。
4. `allow once / allow always / deny` 等审批语义没有结构化落库，只能做一次性的布尔确认。

这与 WorkClaw 作为 OpenClaw 桌面发行版、并计划与飞书协同工作的定位不一致。

**Goals**

- 建立一个通用审批总线，覆盖现有所有高风险工具调用。
- 让桌面和飞书成为平级审批面，任一端都可以批准或拒绝。
- 让 run 在 `approval-pending` 时暂停，并在获批后自动继续执行原工具调用。
- 让 pending approval 可持久化、可恢复、可审计、可防重。
- 为 `allow_always` 提供结构化规则存储，而不是全局开关。

**Non-Goals**

- 第一版不把审批总线抽成独立 sidecar 或远程 gateway 服务。
- 第一版不覆盖企业微信等所有 IM，只先打通桌面与飞书。
- 第一版不提供复杂的管理员策略 UI，只先提供后端能力与最小前端展示。

**Approved Direction**

1. 在 runtime 内新增 `ApprovalManager`，把审批变成正式的运行时协议与状态机。
2. 用持久化 `approvals` 记录替换当前的 `tool-confirm-event + mpsc<bool>` 临时确认链路。
3. 将桌面 UI 和飞书都接为 `ApprovalSurface`，共享同一套 `approval request / resolve / expire / resume` 协议。
4. 将 run 的等待状态显式投影为 `waiting_approval`，并补齐 session journal / session run event 观测。
5. 为 `allow_always` 新增结构化 `approval_rules` 存储，只在命中规则时自动放行。

**Why This Approach**

- 它最符合 OpenClaw 的审批思想：审批不是 UI 小细节，而是正式的异步协议。
- 它可以同时解决桌面、飞书、重启恢复和并发审批这些问题，而不是仅仅把 15 秒改长。
- 它能复用 WorkClaw 已有的 IM 桥接、session run 投影和 Feishu 网关能力，不需要推翻现有架构。

**Architecture**

- `ApprovalManager`
  - 负责创建 approval、状态迁移、持久化、广播事件、恢复执行和并发防重。
- `approvals` 表
  - 保存审批请求本体、当前状态、恢复上下文、审批人和审批面信息。
- `approval_rules` 表
  - 保存 `allow_always` 生成的结构化放行规则。
- `SessionRunStatus::WaitingApproval`
  - 表示某次 run 正在等待高风险审批，不再滥用 `waiting_user`。
- `SessionRunEvent`
  - 新增 `approval_requested`、`approval_resolved`、`approval_expired`、`approval_cancelled`、`approval_resumed`。
- `ApprovalSurface`
  - 桌面端：队列化 pending approval、调用 `resolve_approval`。
  - 飞书端：文本 `/approve` 与按钮卡片两条通路。

**Protocol And State Machine**

1. `executor` 检测到工具调用命中高风险规则。
2. 先走 `risk classification`，再走 `approval_rules` 匹配。
3. 若未命中长期放行规则，则创建 `approval` 记录并生成 `approvalId`。
4. approval 进入 `pending`，同时：
   - run 状态切到 `waiting_approval`
   - 追加 session run event / session journal event
   - 广播给桌面 UI
   - 向飞书线程发送审批通知
5. 当前 run 暂停等待，不再使用 15 秒短超时自动取消。
6. 任一审批面提交决策：
   - `allow_once`: 仅当前 `approvalId` 生效
   - `allow_always`: 当前 `approvalId` 生效，并写入 `approval_rules`
   - `deny`: 当前请求结束，不执行工具
7. 首个成功写入终态的审批结果获胜；其余并发审批请求收到“已处理”结果。
8. 若获批，则恢复原工具调用并让当前 run 自动继续；若被拒绝，则把结构化拒绝结果返回给模型主循环。

**Persistence And Recovery**

- `approvals` 至少包含以下字段：
  - `id`
  - `session_id`
  - `run_id`
  - `call_id`
  - `tool_name`
  - `input_json`
  - `summary`
  - `impact`
  - `irreversible`
  - `status`
  - `decision`
  - `notify_targets_json`
  - `resume_payload_json`
  - `resolved_by_surface`
  - `resolved_by_user`
  - `resolved_at`
  - `resumed_at`
  - `expires_at`
  - `created_at`
  - `updated_at`
- 创建顺序必须是“先落库，再广播，再等待”，避免 `/approve` 先到而 approval 尚未注册的竞态。
- 进程存活时保留 `approvalId -> waiter` 的内存映射，用于原地恢复。
- 应用重启后依赖 `resume_payload_json` 恢复：扫描
  - `status = pending` 的审批，重新投影到桌面 UI
  - `status = approved AND resumed_at IS NULL` 的审批，补启动恢复流程
- 默认不启用 15 秒短超时；`expires_at` 只作为治理手段，而非默认 UX 语义。

**Approval Rules**

- `allow_once`
  - 只影响当前 approval，不写长期规则。
- `deny`
  - 只拒绝当前 approval，不自动生成长期拒绝规则。
- `allow_always`
  - 生成结构化 `approval_rules` 记录，而非简单全局开关。
- `approval_rules` 至少包含：
  - `id`
  - `effect`
  - `scope_type`
  - `tool_name`
  - `matcher_json`
  - `created_by_surface`
  - `created_by_user`
  - `enabled`
  - `expires_at`
  - `created_at`
  - `updated_at`
- 匹配建议：
  - `file_delete`: `workspace + path_prefix + recursive`
  - `bash`: 规范化命令前缀或 action 指纹
  - `browser` 提交类动作: `hostname + action fingerprint`
- 默认作用域建议为 `workspace`，不默认开放 `global`。

**Desktop Surface**

- `ChatView` 不再只维护一个 `toolConfirm` 对象，而是维护 `pending approvals` 列表。
- `tool-confirm-event` 升级为审批状态事件流，例如：
  - `approval-created`
  - `approval-updated`
  - `approval-resolved`
- `RiskConfirmDialog` 继续复用为单条审批卡片的视觉容器，但底层交互改为 `resolve_approval({ approvalId, decision, source })`。
- UI 至少展示：
  - 当前 run 的等待审批状态
  - 审批摘要、影响、不可逆标记
  - `允许一次 / 始终允许 / 拒绝`
  - 已由桌面或飞书处理的状态回显

**Feishu Surface**

- 飞书通知复用现有 Feishu gateway 和 IM bridge 出站能力。
- 第一版同时支持：
  - 文本命令：`/approve <approvalId> allow_once|allow_always|deny`
  - 按钮卡片：三种决策按钮
- 飞书审批成功后，runtime 广播 `approval-resolved`，桌面同步更新。
- 飞书线程需回写审批结果消息，例如“某人已批准，任务继续执行中”。

**Security And Authorization**

- 审批权限不能与飞书接待员工或普通路由用户混用。
- 需要独立的审批人 allowlist：
  - 桌面端：当前登录桌面用户默认可审批
  - 飞书端：仅 allowlist 中的飞书用户可 `/approve`
- 审批审计字段至少包含：
  - `resolved_by_surface`
  - `resolved_by_user`
  - `resolved_at`
- 最终裁决必须以数据库原子状态迁移为准，避免双端同时审批导致重复恢复。

**Rollout**

建议使用 `approval_bus_v1` 特性开关分阶段切换：

1. 上后端审批总线和 `waiting_approval`，桌面仍先只支持单条审批展示。
2. 替换 `executor` 中旧的 `tool-confirm-event + bool` 等待逻辑。
3. 桌面接 `resolve_approval`，跑通自动续跑。
4. 接入飞书文本 `/approve`。
5. 接入飞书按钮卡片。
6. 最后接 `approval_rules` 和 `allow_always`。

**Operational Notes**

- 飞书通知失败不能阻塞桌面审批；桌面仍应可继续处理。
- 审批记录和 run 事件应便于导出、排障与回放。
- 所有新增事件命名和状态名应与 `session_runs` / `session_journal` 保持一致。

**Out Of Scope**

- 企业微信审批面
- 复杂的管理员审批规则可视化页面
- 将审批总线抽成独立服务并对外开放
