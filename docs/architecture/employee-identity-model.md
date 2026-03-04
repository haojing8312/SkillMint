# 员工身份模型（employee_id）

本文档说明 WorkClaw 当前的员工身份字段、会话绑定与长期记忆隔离策略。

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

## 会话绑定模型

- `sessions` 表新增 `employee_id` 字段，用于标记会话归属员工
- 普通会话（非员工入口）允许 `employee_id = ''`，保持历史兼容
- 员工入口创建会话时，前端会显式传入 `employeeId`
- IM 路由创建会话时，也会回填 `sessions.employee_id`

## 长期记忆隔离策略

- 记忆根目录：`<app_data_dir>/memory/`
- 普通会话（兼容旧路径）：`memory/<skill_id>/`
- 员工会话（隔离路径）：`memory/employees/<employee_bucket>/skills/<skill_id>/`
- `employee_bucket` 由 `employee_id` 归一化得到（小写、非字母数字转 `_`）

该策略保证：
- 不同员工使用同一技能时，长期记忆互不污染
- 同一员工在同一技能下可持续积累偏好与上下文
- 历史非员工会话无需迁移即可继续读取原记忆

## 相关文档

- 飞书路由集成：`docs/integrations/feishu-routing.md`
- 飞书 IM 闭环桥接：`docs/integrations/feishu-im-bridge.md`
- 技能安装排错：`docs/troubleshooting/skill-installation.md`
