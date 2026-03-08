# 首页默认单助手与团队显式入口设计

## 背景

当前首页“开始任务”会走 [App.tsx](/E:/code/yzpd/skillhub/apps/runtime/src/App.tsx) 里的 `handleCreateSession(...)`，它会把当前 `selectedEmployeeId` 一起传给后端 `create_session(...)`。后端在 [chat.rs](/E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/commands/chat.rs) 的 `send_message(...)` 中，如果发现该 session 的 `employee_id` 命中了某个团队的 `entry_employee_id`，就会直接触发 [employee_agents.rs](/E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/commands/employee_agents.rs) 的 `maybe_handle_team_entry_session_message_with_pool(...)`。

结果是，用户在首页输入一句“你好”，也可能直接进入多智能体团队协作看板。这条链路技术上可用，但产品心智错误：首页默认任务入口不应该隐式升级成团队编排入口。

## 目标

- 首页默认永远进入“通用单助手”会话。
- 多智能体团队协作只能从显式入口进入。
- 员工直聊、团队协作、首页通用聊天三种模式在前后端都能明确区分。
- 既有未上线数据不做兼容迁移，可以直接按新模型处理。

## 非目标

- 不在本次改动中做“复杂任务自动建议转团队”的智能分流。
- 不改 IM / 飞书的显式团队绑定逻辑。
- 不重做团队看板本身的视觉样式。

## 推荐方案

### 方案 A：首页默认单助手，团队协作显式进入

这是推荐方案。

- 首页“开始任务”只创建普通聊天会话。
- 首页增加“团队协作入口”卡片区，用户主动点击团队后才进入团队会话。
- 员工页保留团队入口，但入口文案与普通员工直聊明确区分。
- 后端只在显式 `team_entry` 会话上触发团队 run。

优点：

- 用户心智最清楚。
- 首页一句话不会突然看到审议、步骤链路、改派等内部协作信息。
- 团队系统仍然完整保留，但只在用户明确选择时启用。

缺点：

- 需要补一层显式会话模式。
- 首页要增加一个团队快捷入口区块。

### 方案 B：首页默认单助手，复杂任务时弹确认转团队

优点是“更智能”，缺点是分类误判概率高，而且会让首页交互变复杂。当前不建议做。

### 方案 C：维持自动进入团队，只补文案说明

改动最小，但不能解决根本体验问题，不采用。

## 最终设计

### 1. 会话模式显式化

给 session 引入两个新字段：

- `session_mode`
  - `general`
  - `employee_direct`
  - `team_entry`
- `team_id`
  - 仅 `team_entry` 时有值

判定规则改成显式模式，而不是继续依赖 `employee_id == entry_employee_id` 的隐式推断。

### 2. 首页入口策略

首页“开始任务”永远创建 `session_mode = general` 的会话，并默认绑定内置通用助手技能，不携带 `team_id`。

首页新增“团队协作入口”区块，展示若干已存在团队：

- 默认复杂任务团队
- 用户后续自定义团队

用户点击某张团队卡片后，再创建 `session_mode = team_entry` 且带 `team_id` 的会话。

### 3. 员工页入口策略

员工页应明确区分两类入口：

- `与该员工开始对话`
  - 创建 `employee_direct`
- `以团队模式发起任务`
  - 创建 `team_entry`

不能再让“进入员工”与“进入团队”共享同一个会话创建语义。

### 4. 后端团队触发条件

后端 `send_message(...)` 中的团队入口判定改成：

- 只有当 `session_mode == team_entry`
- 且 `team_id` 非空

才允许调用 `maybe_handle_team_entry_session_message_with_pool(...)`。

`team_entry` 会话应直接按 `team_id` 找团队实例，而不是再根据 `employee_id` 推断入口团队。

### 5. 首页默认助手

首页默认使用一个内置通用助手，不再把首页默认入口指向“三省六部”团队的入口员工。

这意味着：

- 首页用于通用任务
- 团队入口用于复杂任务协作
- 员工页用于组织视角操作

### 6. 会话展示语义

会话列表与会话页增加模式标签：

- `通用助手`
- `员工`
- `团队`

最近会话中点击历史团队会话仍然正常打开团队看板。禁止的是“首页新建任务自动进入团队”，不是“查看已有团队会话”。

## 数据处理策略

产品尚未上线，本次不做复杂兼容迁移。

处理原则：

- 允许直接扩展 `sessions` 表结构。
- 旧数据可按默认值视为 `general`。
- 如有必要，开发环境数据可以直接清空重建。

## 影响范围

### 前端

- [App.tsx](/E:/code/yzpd/skillhub/apps/runtime/src/App.tsx)
- [NewSessionLanding.tsx](/E:/code/yzpd/skillhub/apps/runtime/src/components/NewSessionLanding.tsx)
- [types.ts](/E:/code/yzpd/skillhub/apps/runtime/src/types.ts)
- 首页与员工页相关测试

### 后端

- [db.rs](/E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/db.rs)
- [chat.rs](/E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/commands/chat.rs)
- [employee_agents.rs](/E:/code/yzpd/skillhub/apps/runtime/src-tauri/src/commands/employee_agents.rs)
- 相关 Rust 集成测试

## 测试标准

至少覆盖以下场景：

1. 首页输入“你好”，进入普通聊天，不出现团队协作看板。
2. 首页点击团队快捷入口，再输入任务，进入团队协作会话。
3. 员工页点击普通员工，进入员工直聊。
4. 员工页点击团队入口，进入团队协作会话。
5. 历史团队会话仍能正常打开团队看板。

## 决策总结

本次统一采用一句产品原则：

- 首页默认单助手
- 团队协作必须显式进入

这能把通用任务入口和多智能体组织入口清晰拆开，避免首页被团队运行时语义污染。
