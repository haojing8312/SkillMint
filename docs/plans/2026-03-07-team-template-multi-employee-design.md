# 团队模板化智能体员工系统 v1 设计稿

## 1. 背景

当前 WorkClaw 已具备以下基础能力：

- 智能体员工模型：`employee_id`、技能、默认工作目录、飞书绑定、长期记忆隔离
- 员工资料文件生成：`AGENTS.md` / `SOUL.md` / `USER.md`
- IM 路由：可按员工创建/复用会话，并将会话绑定到飞书线程
- 子任务工具：`task` 工具已支持子 Agent 委派和 `delegate_role_id`
- 团队协作骨架：已有 `employee_groups`、`group_runs`、`group_run_steps` 和前端 group run 面板

当前主要缺口：

1. 没有“团队模板”层，无法配置化定义一整套员工团队
2. 首次启动不会自动预置一套默认团队
3. 现有 `group orchestrator` 以模拟产出为主，不是真实多员工协作执行
4. 缺少正式的团队规则层，协作主要依赖 prompt 自觉
5. 审核/打回/汇总还没有形成运行时闭环

本设计目标：

1. 新用户首次启动时，自动预置一套默认团队实例（以“三省六部”为首个模板）
2. 该能力必须是通用模板机制，而不是在代码中写死一组特殊员工
3. 复杂任务必须能够由多个员工真实协作完成，而不是模拟群组摘要
4. 用户后续可以基于同一套能力创建其他智能体员工团队

## 2. 设计原则

### 2.1 模板与实例分离

- 系统内置的是“团队模板定义”
- 用户实际使用的是“团队实例”
- 模板升级不直接覆盖用户已经修改过的实例

### 2.2 协作规则显式化

- 谁是协调者
- 谁负责审核
- 谁能委派给谁
- 哪些阶段必须经过审核

这些规则必须成为运行时配置，而不是只写在 prompt 里。

### 2.3 运行时真实执行

- 复杂任务的每个关键步骤都绑定真实员工上下文
- 子步骤由目标员工的 skill / profile / memory / work_dir 驱动
- 协调、审核、执行、汇总在运行时闭环执行

### 2.4 通用能力优先

“三省六部”只是第一套默认模板，不应在 UI 或后端形成特殊分支。

## 3. 范围与非目标

### 3.1 范围

- 团队模板定义与实例化
- 首次启动自动 seed 默认团队
- 真实多员工任务协作运行时
- 团队规则配置
- 团队与运行看板 UI

### 3.2 非目标

- 本期不做可视化工作流编排器
- 不做复杂 BPM/DSL 级工作流引擎
- 不做全渠道统一协作，仅复用现有桌面与飞书链路
- 不做完整绩效评分系统

## 4. 方案比较

### 方案 A：仅预置员工与员工组

做法：

- 首启自动创建“三省六部”员工
- 自动创建一个员工组
- 协作继续依赖 `task` 工具和 prompt 自由发挥

优点：

- 改动最小
- 可以快速展示默认团队

缺点：

- 协作稳定性不足
- 缺少审核/打回和强约束阶段控制
- 无法支撑复杂任务的可观测执行

### 方案 B：团队模板 + 真实协作运行时

做法：

- 新增团队模板层
- 首启自动实例化默认模板
- 运行时按模板规则驱动协调、审核、执行、汇总

优点：

- 既能满足默认预置，又能支撑用户自建团队
- 能形成真正的多员工协作闭环
- 结构与产品长期方向一致

缺点：

- 需要补模板、规则、事件流和运行时能力

### 方案 C：完整工作流引擎

做法：

- 将团队协作抽象成可视化/DSL 工作流引擎

优点：

- 长期扩展能力最强

缺点：

- 范围过大
- 与当前项目成熟度不匹配

### 推荐

采用 **方案 B：团队模板 + 真实协作运行时**。

这是满足“首启自动预置 + 后续用户自建团队 + 复杂任务真实协作”三者平衡的最小可行方案。

## 5. 总体架构

本期新增一层“员工团队模板”能力：

1. 系统内置模板配置文件
2. 首启 bootstrap 自动实例化模板
3. 实例化后生成：
   - 员工
   - 员工 profile 文件
   - 团队实例
   - 团队规则
4. 用户发起复杂任务时，由团队运行时按规则创建真实 `group_run`
5. 协调员工、审核员工、执行员工在各自上下文中完成协作
6. 运行结果通过结构化 step/event 和聊天消息同步呈现

## 6. 模板设计

### 6.1 模板结构

建议模板由三层组成：

1. `team template`
2. `employee specs`
3. `rules`

### 6.2 team template 字段建议

- `template_id`
- `name`
- `description`
- `scenario_tags`
- `seed_on_first_run`
- `default_entry_employee_key`
- `default_review_mode`
- `default_execution_mode`
- `template_version`

### 6.3 employee specs 字段建议

- `employee_key`
- `employee_id`
- `name`
- `persona`
- `primary_skill_id`
- `skill_ids`
- `enabled_scopes`
- `openclaw_agent_id`
- `is_default_entry`
- `is_public_entry`
- `agents_md_template`
- `soul_md_template`
- `user_md_template`

### 6.4 rule 字段建议

- `from_employee_key`
- `to_employee_key`
- `relation_type`
  - `delegate`
  - `review`
  - `handoff`
  - `report`
- `phase_scope`
- `required`
- `priority`

## 7. 首次启动自动实例化

### 7.1 触发时机

在数据库初始化和 builtin skill 同步完成后执行 bootstrap 检查。

### 7.2 执行逻辑

1. 判断是否为新用户库
2. 查询是否已有 bootstrap 记录
3. 扫描内置模板目录
4. 找到 `seed_on_first_run = true` 的模板
5. 实例化：
   - 创建员工
   - 绑定员工技能
   - 落地员工 profile 文件
   - 创建团队实例
   - 创建团队规则
6. 写入 seed 记录，保证幂等

### 7.3 幂等与升级

- 首次 seed 成功后不重复导入
- 后续版本若模板升级，只提示“有新模板版本可同步”
- 不自动覆盖用户修改过的实例

## 8. 数据模型设计

### 8.1 保留并升级的现有表

- `agent_employees`
- `employee_groups`
- `group_runs`
- `group_run_steps`

### 8.2 现有表升级建议

#### `employee_groups`

从“员工组”升级为“团队实例”，建议补充字段：

- `template_id`
- `entry_employee_id`
- `review_mode`
- `execution_mode`
- `visibility_mode`
- `is_bootstrap_seeded`
- `config_json`

#### `group_runs`

建议补充字段：

- `entry_session_id`
- `main_employee_id`
- `current_phase`
- `review_round`
- `status_reason`
- `template_version`
- `waiting_for_employee_id`
- `waiting_for_user`

#### `group_run_steps`

建议补充字段：

- `parent_step_id`
- `phase`
- `step_kind`
- `requires_review`
- `review_status`
- `attempt_no`
- `session_id`
- `input_summary`
- `output_summary`
- `visibility`

### 8.3 新增表建议

#### `employee_group_rules`

用于描述团队内部规则：

- `id`
- `group_id`
- `from_employee_id`
- `to_employee_id`
- `relation_type`
- `phase_scope`
- `required`
- `priority`
- `created_at`

#### `group_run_events`

用于结构化记录运行时事件：

- `id`
- `run_id`
- `step_id`
- `event_type`
- `phase`
- `actor_employee_id`
- `target_employee_id`
- `payload_json`
- `created_at`

#### `seeded_team_templates`

用于记录模板实例化历史：

- `template_id`
- `template_version`
- `seeded_at`
- `instance_group_id`
- `instance_employee_ids_json`
- `seed_mode`

## 9. 运行时协作设计

### 9.1 从模拟运行升级为真实运行

当前流程：

- `start run -> simulate_group_run -> 写入 steps -> 展示`

目标流程：

- `start run -> 协调员工真实规划 -> 审核员工审批 -> 执行员工执行 -> 汇总员工总结`

### 9.2 通用阶段语义

建议统一成以下阶段：

- `intake`
- `plan`
- `review`
- `dispatch`
- `execute`
- `synthesize`
- `finalize`

“三省六部”模板只是把角色映射到这些通用阶段：

- 太子：`intake`
- 中书：`plan`
- 门下：`review`
- 尚书：`dispatch` + `synthesize`
- 六部：`execute`

### 9.3 真实步骤执行

每个 step 应绑定真实员工执行上下文：

- 员工 profile
- 员工主技能
- 员工记忆路径
- 员工工作目录
- 员工工具范围

### 9.4 审核与打回

支持两种审核模式：

- `soft_review`
  - 给建议，不阻断
- `hard_review`
  - 必须 `approve/reject`
  - `reject` 时退回前序阶段
  - 增加 `review_round`
  - 保留历史步骤和事件

## 10. 委派能力调整

### 10.1 保留现有 `task` 工具

继续支持自由子任务探索场景。

### 10.2 新增“按员工执行 step”的正式运行时入口

团队 orchestrator 不再仅依赖 prompt 指导 `task` 工具，而是通过运行时入口：

1. 解析 `target_employee_id`
2. 加载目标员工真实配置
3. 创建或复用该员工在当前 run 下的 session
4. 在其上下文中执行子任务
5. 回写 step/result/event

## 11. IM 与外部入口设计

支持两种入口：

1. `team entry`
   - 用户对团队下任务
   - 默认由入口员工接单
2. `employee direct`
   - 用户显式 @ 某员工
   - 直接进入该员工

对默认“三省六部”团队：

- 默认只暴露主入口员工
- 其他员工作为内部协作成员运行
- 高级模式下允许直接点名单个员工

## 12. 前端设计

### 12.1 员工中心三层结构

建议拆成：

- 员工
- 团队
- 运行

### 12.2 首启引导

新用户首次进入时提示：

- 已自动预置一套默认团队
- 可直接使用
- 也可复制为自己的团队

### 12.3 团队详情页

至少展示：

- 入口员工
- 协调者
- 审核者
- 执行成员
- 关系规则
- 默认阶段链路

### 12.4 运行看板

展示：

- 当前阶段
- 当前轮次
- step 负责人
- step 状态
- 最近事件流
- 是否等待用户
- 是否待审核

支持操作：

- 暂停
- 继续
- 重试
- 改派
- 通过
- 打回

### 12.5 用户自定义团队创建

支持两种模式：

1. 从模板创建
2. 从空白创建

并提供预置协作模式：

- 协调-执行-汇总
- 规划-审核-执行-汇总
- 路由-执行
- 研究-汇总

## 13. 测试设计

### 13.1 后端

- 首启 seed 幂等测试
- 模板实例化测试
- 团队规则验证测试
- 真实 run 生命周期测试
- 审核打回测试
- 失败重试/改派测试

### 13.2 前端

- 首启默认团队展示
- 团队详情展示
- 运行看板展示
- 审核/打回/改派交互
- 模板复制与新建团队

### 13.3 端到端

- 新库首次启动自动生成默认团队
- 用户下发复杂任务后触发真实多员工协作
- 审核员工可打回并重新流转
- 最终协调者汇总输出

## 14. 分期实施建议

### P1

- 模板定义与首启 seed
- 真实协作运行骨架
- run/event 结构化记录
- 团队与运行基础 UI

### P2

- 用户自定义团队
- 规则编辑
- 审核模式配置
- 模板复制

### P3

- 暂停/恢复/改派
- stuck 检测
- 指标统计
- 模板版本升级提示

## 15. 风险与边界

### 风险 1：范围膨胀

控制方式：

- 第一版不做可视化流程引擎
- 第一版只做模板化团队 + 真实协作闭环

### 风险 2：真实协作链路过度依赖 prompt

控制方式：

- 审核、打回、阶段流转由运行时控制
- prompt 只描述职责，不承担状态机职责

### 风险 3：预置模板绑定产品心智

控制方式：

- UI 中不做“三省六部模式”特殊分支
- 统一展示为默认团队模板实例

## 16. 验收标准

1. 新用户首次启动后自动拥有一套默认团队实例
2. 默认团队来自模板配置，而非硬编码逻辑
3. 复杂任务触发真实 `group_run`，由多员工真实执行
4. 审核员工可通过/打回，打回后能回退重跑
5. UI 能显示团队结构、运行阶段、步骤状态和事件流
6. 用户可复制默认团队并创建自己的团队
7. 用户可通过同一套机制创建新的员工团队

## 17. 最终结论

本期应实现的是：

**团队模板化智能体员工系统 v1**

而不是单独集成一套“三省六部”特殊逻辑。

“三省六部”只是第一套默认模板，真正应沉淀的是：

- 团队模板
- 首启实例化
- 真实多员工协作运行时
- 审核/打回机制
- 团队与运行看板

这套通用能力建好后，后续用户才能真正自定义和复制更多类似的智能体员工团队。
