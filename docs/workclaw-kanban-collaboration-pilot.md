# WorkClaw Kanban 驱动协作试点

## 目标

把 WorkClaw 项目群从“飞书群 @ 智能体”升级为：

- 飞书群：项目作战室 / 可视化沟通现场
- Hermes Kanban：任务状态机 / 回调与结果沉淀层
- WorkClaw PM：任务拆解、分派、跟踪、汇总
- 专家智能体：按 profile 领取任务、执行、写回结果

## 为什么要这么做

飞书群消息适合让人看到过程，但不适合承担任务状态系统。它缺少稳定的 task_id、状态、依赖、完成回调、运行日志和重试机制。

Hermes Kanban 是官方提供的多 profile 协作任务板，任务会写入共享 SQLite 看板，并由 dispatcher 拉起对应 profile 执行。worker 完成后会写回 summary、状态、运行记录和日志。

## 试点项目

- 项目：WorkClaw
- tenant：`workclaw`
- 项目经理 profile：`workclaw-pm`
- 项目目录：`/mnt/d/code/workclaw`
- 飞书群：WorkClaw 项目作战群

## 当前默认参与角色

当前 WorkClaw 处于 MVP 工程收口与质量门阶段，默认采用精简小组，避免无关专家常驻。

### 默认常驻

- `workclaw-pm`：项目经理 / 调度中心，负责拆任务、收敛结论、控制范围。
- `technical-lead-agent`：技术研发，负责工程实现、修复、验证证据和提交准备。
- `quality-test-architect`：质量测试与验收，负责质量门、focused 回归、桌面 smoke 结论。
- `agent-systems-architect`：AI Agent 系统架构，低频参与 runtime/profile/toolset/sidecar 边界裁决。

### 按需调用

- `product-strategist`：仅当 MVP 范围、验收口径或演示路径变化时调用。
- `delivery-project-manager`：仅当需要真实桌面 smoke、环境协调或客户现场节奏时调用。

### 当前暂停常驻

- `chief-business-architect`、`solution-architect`、`growth-sales-strategist`、`customer-success-expert`、`business-analyst` 以及内容、视觉、传播类角色不参与默认工程收口；只有进入明确商业化、行业方案、客户成功或复盘任务时再单独建 Kanban 调用。

## 用户发指令的推荐格式

在 WorkClaw 项目作战群里 @ WorkClaw 项目经理，使用如下格式：

```text
@workclaw项目经理
【Kanban任务】
项目：WorkClaw
目标：<一句话说明要完成什么>
背景：<为什么现在要做，已有信息在哪里>
交付物：<希望产出什么：方案/代码/测试报告/文档/飞书总结等>
参与角色：<可选；不知道就写“由PM自行拆解”>
验收标准：<什么结果算完成>
优先级：P0/P1/P2
截止时间：<可选>
要求：请用 Hermes Kanban 创建任务/子任务，分配给对应 profile，完成后汇总 task_id、状态、结论和下一步。
```

## PM 处理规则

PM 收到 `【Kanban任务】` 后：

1. 判断是否需要拆成多个专家任务。
2. 在 Hermes Kanban 创建任务，tenant 使用 `workclaw`。
3. 给每个任务设置：assignee、body、workspace、验收标准。
4. 在飞书群回复任务编号与分工。
5. 跟踪任务状态，查看 `show/runs/log`。
6. 所有子任务完成后，输出总汇总与下一步建议。

## 飞书进度通报规则

每个专家 profile 在执行 WorkClaw Kanban 任务时，必须在关键节点向 WorkClaw 项目作战群通报简短进度：

- 认领/开始任务时：说明 task_id、计划产出、预计下一步。
- 遇到阻塞时：说明 task_id、阻塞点、需要谁决策。
- 完成任务时：说明 task_id、结论、交付物、下一步。

可使用项目专用 helper：

```bash
HERMES_HOME=/root/.hermes/profiles/<profile> \
  /root/.local/bin/workclaw-agent-progress-feishu <task_id> <status> "<简短进度>"
```

如果某个 worker 无法直接发送飞书消息，必须把 `[FEISHU_UPDATE]` 写入 `kanban_complete` summary，由 WorkClaw PM 代为转发。

## Kanban CLI 参考

```bash
hermes kanban create "任务标题" \
  --tenant workclaw \
  --assignee product-strategist \
  --workspace dir:/mnt/d/code/workclaw \
  --body "任务背景、目标、交付物、验收标准" \
  --json

hermes kanban list --tenant workclaw
hermes kanban show <task_id>
hermes kanban runs <task_id>
hermes kanban log <task_id>
hermes kanban dispatch --max 3
```

## 试点原则

- 飞书群只做可见协作，不再作为唯一任务状态来源。
- 每个可追踪任务必须有 Kanban task_id。
- 专家输出必须写回 Kanban summary，再由 PM 汇总到飞书群。
- 先跑小闭环，不一次性改全公司流程。
