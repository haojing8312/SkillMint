# 员工身份模型（employee_id）

本文档说明 WorkClaw 当前的员工身份字段、会话绑定、长期记忆隔离，以及团队模板与团队运行态模型。

## 统一字段

- 对外统一使用单字段：`employee_id`（员工编号）
- 前端界面仅暴露 `employee_id`，减少概念复杂度

## 兼容映射策略

- 保存时自动镜像：`role_id = employee_id`
- 保存时自动镜像：`openclaw_agent_id = employee_id`
- 数据迁移回填：当 `employee_id` 为空时，自动使用历史 `role_id`

## 配置向导产物

- 员工页支持问答式生成并预览：`AGENTS.md` / `SOUL.md` / `USER.md`
- 一键应用后写入目录：`<employee_work_dir>/openclaw/<employee_id>/`

## 团队模板与团队实例

WorkClaw 在“单个员工”之上增加了“团队模板 -> 团队实例 -> 团队运行”的三层模型。

- **团队模板**：系统内置 JSON 模板定义团队元信息、员工规格、角色分工、协作规则和首启预置策略。当前内置模板包含默认“三省六部”复杂任务团队。
- **团队实例**：模板实例化后写入 `employee_groups`，并保存 `template_id`、`entry_employee_id`、`review_mode`、`execution_mode`、`visibility_mode`、`is_bootstrap_seeded`、`config_json` 等字段。
- **团队规则**：`employee_group_rules` 保存实例内部的委派、审议、汇报等关系，是运行时协作权限矩阵的基础。

模板和实例严格分离：

- 系统维护模板定义，用户编辑的是自己库里的团队实例。
- 复制预置团队时，会保留模板元数据和规则，但新团队实例会标记为 `is_bootstrap_seeded = false`，表示它已成为用户自主管理的团队。

## 会话绑定模型

- `sessions` 表新增 `employee_id` 字段，用于标记会话归属员工
- 普通会话（非员工入口）允许 `employee_id = ''`，保持历史兼容
- 员工入口创建会话时，前端会显式传入 `employeeId`
- IM 路由创建会话时，也会回填 `sessions.employee_id`

团队运行时，组内步骤也会绑定到对应员工的会话或运行上下文，使“谁执行了这一步”可以落到真实员工身份，而不只是文本标签。

## 长期记忆隔离策略

- 记忆根目录：`<app_data_dir>/memory/`
- 普通会话（兼容旧路径）：`memory/<skill_id>/`
- 员工会话（隔离路径）：`memory/employees/<employee_bucket>/skills/<skill_id>/`
- `employee_bucket` 由 `employee_id` 归一化得到（小写、非字母数字转 `_`）

该策略保证：
- 不同员工使用同一技能时，长期记忆互不污染
- 同一员工在同一技能下可持续积累偏好与上下文
- 历史非员工会话无需迁移即可继续读取原记忆

## 首启预置策略

首次启动时，系统会扫描内置团队模板，并将启用了首启预置的模板实例化到当前用户库中。

- `seeded_team_templates` 用于记录某个模板是否已实例化、实例化版本、生成时间和对应的团队实例 ID。
- 预置逻辑要求幂等：已完成 seed 的模板不会重复导入。
- 用户后续修改预置团队时，模板定义不会被反向污染。

这一策略保证新用户首次进入员工中心时，能直接看到可用的默认团队，而不是从空白状态开始。

## 团队运行态模型

复杂任务由 `group_runs -> group_run_steps -> group_run_events` 三层结构表示：

- `group_runs`：团队任务运行容器，保存当前阶段 `current_phase`、审议轮次 `review_round`、等待对象 `waiting_for_employee_id`、是否等待用户 `waiting_for_user` 等运行态字段。
- `group_run_steps`：具体步骤记录，承载 `plan`、`review`、`execute`、`synthesize`、`final` 等步骤类型及负责人、状态、输入输出摘要。
- `group_run_events`：结构化事件流，记录步骤派发、执行完成、审议通过/打回、暂停/恢复、改派等关键事件。

这一层设计的目标是让团队协作可观测、可审计、可恢复，而不是只返回最终总结文本。

## 相关文档

- 飞书路由集成：`docs/integrations/feishu-routing.md`
- 飞书 IM 闭环桥接：`docs/integrations/feishu-im-bridge.md`
- 技能安装排错：`docs/troubleshooting/skill-installation.md`
