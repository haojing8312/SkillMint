# WorkClaw 3-7 天 MVP 产品闭环定义：新手友好的桌面 AI 智能体 / AI 团队指挥

**日期：** 2026-05-12
**Kanban task：** `t_813defc2`
**角色：** product-strategist
**范围：** 产品边界、MVP 用户路径、验收标准、后续工程输入。本文不做商业报价、交付周期承诺或外部客户承诺。

---

## 1. 当前阶段输入

已参考当前仓库与路线图：

- `README.md`：WorkClaw 当前定位是 Hermes-aligned、本地优先的桌面 AI 员工运行时和工作台；OpenClaw 仅作为历史来源和遗留迁移输入。
- `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`：下一阶段架构根是 `profile_id -> AI employee runtime home`，Memory OS、Skill OS、Growth、Curator、Toolset Gateway 按 profile 边界演进。
- `docs/plans/2026-05-09-hermes-parity-stabilization-checklist.md`：当前优先稳定 profile runtime、memory、skills、curator、toolsets、employee workbench，而不是扩展新大功能。
- `docs/plans/2026-05-11-hermes-aligned-sidecar-removal-roadmap.md`：sidecar removal 进行中；MCP 已迁入 native runtime，browser/IM/Feishu/WeCom 仍需分批，不允许新增 sidecar/OpenClaw 兼容目标。
- 现有前端入口：`NewSessionLanding.tsx` 已有一句话开始任务、附件、工作目录、精选场景、团队协作入口、最近会话。
- 现有员工中心：`EmployeeHubScene.tsx` / `EmployeeHubView.tsx` 已有员工选择、员工创建助手入口、Profile Home、Profile Memory、Skill OS、Growth、Curator、团队与最近运行等基础展示。
- 现有进度展示：`TaskTabStrip.tsx`、`ToolIsland.tsx`、`EmployeeRunsSection.tsx`、`TaskJourneySummary.tsx` 已具备会话状态、工具步骤、团队运行、结果文件入口等组件基础。

---

## 2. MVP 产品定位

### 一句话定位

WorkClaw 3-7 天 MVP 不是“大而全 Agent 平台”，而是：

> 让新手用户首次打开桌面应用后，能在 10 分钟内选择一个 AI 员工或默认团队，发起一个真实电脑任务，看到执行进度，并在任务结束后回到会话 / Profile Home 查看结果与沉淀证据。

### MVP 成功定义

一个非技术用户完成以下闭环即算 MVP 成功：

1. 我知道从哪里开始。
2. 我知道当前由哪个 AI 员工 / 团队在处理。
3. 我知道它正在做什么、卡在哪里、是否需要我确认。
4. 我拿到一个可复用结果：文件、总结、会话记录、profile 成长记录或技能/记忆沉淀证据。
5. 我能在下一次打开时找到这次任务和对应 AI 员工。

### MVP 非目标

本轮不追求：

- 不做新商业报价、试点付费承诺、交付周期承诺。
- 不把 OpenClaw 当作新的兼容目标；如出现 OpenClaw，只能是历史、迁移或临时 shim 说明。
- 不新增 sidecar endpoint，不为 MVP 反向扩大 sidecar 边界。
- 不做完整插件市场、技能交易市场、企业后台、多人权限系统。
- 不做完整 Feishu 移动端指挥闭环，只保留“桌面为主，IM/gateway 为后续增强”的边界。
- 不做默认人工审批队列；继续遵循 Hermes 风格：普通低风险成长 agent-managed，高风险动作触发现有风险确认。

---

## 3. 3-7 天 MVP 用户路径

| 阶段 | 用户动作 | 系统反馈 | 3-7 天必须达到 | 可转 Kanban / 验收项 |
| --- | --- | --- | --- | --- |
| 0. 首次打开 | 用户打开 WorkClaw 桌面应用 | 首页直接看到“开始任务”、精选场景、工作目录、附件、团队协作入口 | 不要求用户理解 profile/runtime/sidecar；页面文案要能解释“先描述任务” | `frontend`: 首页新手文案与首启空状态验收 |
| 1. 选择任务入口 | 用户输入一句话，或点精选场景，或选择团队入口 | 输入框自动填充 / 可编辑；按钮状态清楚 | 单专家任务与团队任务入口必须区分：简单任务“开始任务”，复杂任务“交给团队处理” | `frontend`: Landing 入口分流与按钮状态验收 |
| 2. 创建 / 选择 AI 员工 | 用户进入员工中心，选择默认员工，或通过员工创建助手生成员工 | 员工被高亮；可以“设为主力并进入任务”；能看到 Profile Home 状态 | 先复用现有默认员工和员工创建助手，不要求完整自由编排员工组织架构 | `product+frontend`: 员工选择 / 创建最短路径验收 |
| 3. 发起专家任务 | 用户用选定员工 / 技能启动任务，可带附件和工作目录 | 创建 runtime session，进入任务 tab，显示运行状态 | 任务能完成一个低风险本地任务：文件整理方案、资料总结、代码排查或浏览器公开信息整理 | `technical-lead`: 单专家任务 session 创建与运行验收 |
| 4. 发起团队任务 | 用户点击默认团队卡片“交给团队处理” | 创建 `sessionMode=team_entry` 会话，使用团队入口员工，保留 `teamId` | 只要求跑通一个默认团队入口，不要求动态创建复杂团队 | `technical-lead`: 默认团队入口 session 与 teamId 绑定验收 |
| 5. 看到进度 | 用户停留在任务 tab 或员工中心“最近运行” | 看到运行中 / 工具执行 / 等待确认 / 完成 / 失败；工具步骤可展开 | 至少要有 tab 状态、ToolIsland 步骤、最近运行列表三处之一能解释当前状态 | `frontend+quality`: 进度可见性 smoke test |
| 6. 获得结果沉淀 | 任务完成后，用户看到最终回答、产出文件入口、最近会话；如发生记忆/技能变化，可在 Profile Home / Growth / Skill OS 查看证据 | 结果不只是一段临时聊天；能回看、能定位文件或沉淀记录 | 任务结果必须进入会话历史；有文件时显示文件入口；有 profile 变更时显示 growth / version evidence | `quality`: 结果沉淀与回看验收 |
| 7. 下一次复用 | 用户重新打开或进入最近会话 / 员工中心 | 能继续或查看之前任务 | 只要求能找到最近会话、员工和运行记录；不做复杂搜索体验 | `quality`: 二次打开回看 smoke test |

---

## 4. 必须做 / 暂不做 / 风险项

### 4.1 必须做（3-7 天内）

| 优先级 | 必须做 | 产品边界 | 验收方式 |
| --- | --- | --- | --- |
| P0 | 首页新手路径收敛 | 首页只解释“描述任务 / 选场景 / 交给团队 / 选择工作目录”，不解释底层架构名词 | 首次打开空状态下，新手能在 60 秒内找到开始任务入口 |
| P0 | 单专家任务闭环 | 复用现有 `start-task`、skill、employee、runtime session；不新增复杂调度器 | 输入一句任务后创建 session，tab 显示运行/完成状态，结果进入最近会话 |
| P0 | 默认团队任务闭环 | 复用已有员工团队实例与 `team_entry` session；只跑通一个默认团队入口 | 点击团队卡片后创建带 `teamId` 的会话，最近运行可看到该团队任务 |
| P0 | 进度可见性 | 优先用已有 `TaskTabStrip`、`ToolIsland`、`EmployeeRunsSection`；不做全新流程图大组件 | 运行中能看到状态，工具调用能看到步骤，失败/等待确认有可理解提示 |
| P0 | 结果沉淀 | 结果沉淀到 session；文件类结果走任务文件入口；profile 变更走 Profile Home / Growth / Skill OS 证据 | 完成后能在最近会话或员工中心找到结果；有文件时能点“查看此任务中的所有文件” |
| P1 | 员工选择 / 创建最短路径 | 先复用员工中心、默认员工、员工创建助手；不做复杂组织架构编辑 | 用户能选择默认员工并进入任务；创建助手生成员工后能高亮并进入任务 |
| P1 | 手工 smoke test | 先做人工验收清单，不扩展大规模自动评测 | QA 能按 8 条产品验收标准逐项 pass/fail |
| P1 | 后续工程边界注入 | 给技术任务明确“能改哪些文件、不能碰哪些边界、验证哪些命令” | `t_4bfa4c43` 与 `t_2dfa2ad5` 可直接引用本文第 7 节 |

### 4.2 暂不做（后续阶段）

| 暂不做 | 原因 | 后续触发条件 |
| --- | --- | --- |
| 完整 OpenClaw 兼容或新 OpenClaw 入口 | 与当前 Hermes-aligned 路线冲突 | 仅允许作为历史迁移输入或临时 shim 删除计划的一部分出现 |
| 新 sidecar endpoint 或 sidecar 产品配置 | sidecar removal 进行中；新增会增加迁移债务 | 必须先完成 native provider / platform adapter 设计与验收 |
| 完整移动端 IM 指挥闭环 | Feishu/WeCom/channel connector 仍在迁移，MVP 主路径应在桌面 | 平台 adapter 边界完成后再定义移动端 MVP |
| 技能市场 / 插件市场 / 付费分发后台 | 3-7 天内无法稳定闭环，且涉及商业承诺风险 | 商业验证任务给出明确试点约束后另开产品任务 |
| 完整 profile import/restore | 当前 Profile export 已有基础，restore 风险高 | Memory/Skill/Growth rollback UI 和安全策略补齐后再做 |
| 大规模 UI 重设计 | 会吞噬 MVP 时间 | 先用现有组件完成路径收敛和文案修正 |
| 默认人工审批队列 | 已被路线图撤回，偏离 Hermes parity | 只保留危险操作现有确认机制 |
| 全量 sidecar 删除 | 依赖 browser/IM/platform adapter 后续批次 | 等 sidecar removal roadmap Batch 5/6/7 验收通过 |

### 4.3 风险项

| 风险 | 等级 | 影响 | 控制措施 | 可转验收项 |
| --- | --- | --- | --- | --- |
| 新手没有模型 / API key 配置，无法完成首个任务 | 高 | 首次体验中断 | 首页或任务创建失败时给出“先配置模型”的明确提示；不在此任务内承诺自动配置所有 provider | 创建 session 失败时错误文案可理解 |
| 团队任务链路过复杂，3-7 天跑不稳 | 高 | MVP 失败 | 团队 MVP 只跑默认团队入口，不做动态团队创建和复杂审议 UI | 默认团队入口 smoke pass 即可 |
| 进度展示只显示底层工具名，新手看不懂 | 中 | 用户信任低 | ToolIsland / tab / 最近运行至少一处展示自然语言状态 | 运行中、等待确认、失败、完成四种状态均有文案 |
| 结果只留在聊天气泡，无法回看 | 中 | 闭环不成立 | 最近会话、任务文件入口、Profile Home / Growth 作为沉淀入口 | 任务完成后可从最近会话回看 |
| 工程为了 MVP 新增 sidecar 或 OpenClaw 兼容 | 高 | 架构倒退 | 将“不得新增 sidecar/OpenClaw 目标”写入技术验收 | 代码 review grep 无新增 sidecar/OpenClaw 产品入口 |
| 低风险 memory/skill 成长被误解为不可控自动修改 | 中 | 用户担忧 | 只展示来源、版本、可回滚证据；危险操作继续确认 | Growth/Skill OS 展示来源与版本证据 |
| Windows 环境部分 Rust binary 直接运行异常 | 中 | 自动验收不稳定 | 采用已知可运行的 integration tests、`cargo check`、前端 tsc 与人工 smoke | 验收报告标注环境 caveat |
| 外部试点客户需求牵引导致范围膨胀 | 高 | 3-7 天目标失焦 | 商业、价格、客户交付承诺全部交由郝敬确认，本任务只给产品 MVP 边界 | 非目标清单在 PM 汇总中保留 |

---

## 5. 产品验收标准（5-8 条）

1. **首启入口验收：** 新用户打开桌面应用后，在首页不阅读文档也能看到“开始任务”、精选场景、工作目录、附件和“交给团队处理”入口；空状态不出现必须理解 profile、sidecar、OpenClaw 才能继续的文案。
2. **单专家任务验收：** 在已配置可用模型的前提下，用户输入一句低风险任务并点击“开始任务”，系统能创建 runtime session，进入任务 tab，并在 tab / ToolIsland / 聊天区展示运行、完成或失败状态。
3. **员工选择验收：** 用户能在员工中心选择一个已有 AI 员工，设为主力并进入任务；如果通过员工创建助手生成员工，返回员工中心时该员工被高亮，且具备 Profile Home 状态展示。
4. **团队任务验收：** 用户在首页点击一个默认团队的“交给团队处理”，系统创建 `sessionMode=team_entry` 会话，并保留 `teamId`、入口员工、默认工作目录；员工中心“最近运行”能看到该团队任务或引导进入对应会话。
5. **进度可见性验收：** 运行过程中至少展示以下状态之一：`running` / `tool_calling` / `waiting_approval` / `completed` / `failed`；工具步骤可展开查看，等待确认和失败状态必须有新手可理解的说明。
6. **结果沉淀验收：** 任务完成后，最终结果进入会话历史；若有产出文件，出现“查看此任务中的所有文件”或等价入口；若发生 memory/skill/curator 变更，员工中心能看到 Profile Home / Growth / Skill OS / Curator 的来源、版本或报告证据。
7. **架构边界验收：** MVP 工程实现不得新增 OpenClaw 兼容目标、不得新增 sidecar endpoint、不得把 `employee_id + skill_id` 作为新记忆身份中心；新 profile/growth/skill/memory 相关行为必须挂到 profile runtime 边界。
8. **Smoke test 验收：** QA 至少完成一次桌面人工路径：打开首页 -> 选场景 -> 单专家任务 -> 查看进度 -> 查看结果 -> 进入员工中心 -> 发起默认团队任务 -> 查看最近运行。失败项必须形成后续 Kanban 任务，而不是在当前 MVP 内无边界扩张。

---

## 6. 3-7 天建议工程批次

这些批次不是要求 product-strategist 直接实现，而是给后续 `technical-lead-agent`、`quality-test-architect`、`delivery-project-manager` 使用的产品边界输入。

| 批次 | 目标 | 建议 owner | 可能触达文件 | 完成定义 |
| --- | --- | --- | --- | --- |
| E1. 首页新手路径收敛 | 首页文案、空状态、按钮分流更清楚 | `technical-lead-agent` | `apps/runtime/src/components/NewSessionLanding.tsx` 及测试 | 新手能区分“开始任务”和“交给团队处理”；错误状态能提示模型/配置问题 |
| E2. 员工选择 / 创建最短路径 | 员工中心到任务入口更连贯 | `technical-lead-agent` | `EmployeeHubScene.tsx`、`EmployeeHubView.tsx`、员工相关 tests | 选择员工 -> 设为主力 -> 进入任务；创建助手返回后高亮员工 |
| E3. 团队任务最小闭环 | 默认团队入口稳定创建 team_entry session | `technical-lead-agent` | `NewSessionLanding.tsx`、`useEmployeeSessionLaunchCoordinator.ts`、team/session tests | `teamId`、entry employee、session title、pending initial message 均正确 |
| E4. 进度与结果沉淀展示 | 让新手看懂任务正在做什么和做完后在哪里看 | `technical-lead-agent` + `quality-test-architect` | `TaskTabStrip.tsx`、`ToolIsland.tsx`、`EmployeeRunsSection.tsx`、`TaskJourneySummary.tsx` | 运行、等待确认、失败、完成、文件入口和最近运行均可验收 |
| E5. 产品 smoke test | 把本文件第 5 节转成手工验收报告 | `quality-test-architect` | 测试报告文档或 Kanban summary | 8 条产品验收标准逐项 pass/fail，并列出阻塞 |

---

## 7. 对后续技术实现任务的产品边界输入

### 7.1 可复用能力

后续工程应优先复用：

- 首页任务入口：`NewSessionLanding.tsx` 的一句话输入、附件、工作目录、精选场景、团队卡片。
- 会话启动链路：`useEmployeeSessionLaunchCoordinator.ts` / runtime session 创建逻辑。
- 任务状态展示：`TaskTabStrip.tsx`、`ToolIsland.tsx`、Sidebar runtime status。
- 员工中心：`EmployeeHubScene.tsx`、`EmployeeHubView.tsx`、Profile Memory / Skill OS / Growth / Curator 现有区域。
- 团队运行展示：`EmployeeRunsSection.tsx` 与 group run summary。
- 结果文件入口：`TaskJourneySummary.tsx`。
- Profile runtime：`profile_id -> profiles/<profile_id>/...` 作为 AI 员工 runtime home。

### 7.2 不可触碰边界

- 不新增 OpenClaw 作为产品目标、配置目标、用户入口或兼容承诺。
- 不新增 sidecar HTTP endpoint；browser/IM/Feishu/WeCom 的迁移必须按 sidecar removal roadmap 分批。
- 不新增默认人工审批队列、growth review inbox 或 `memory_patch_proposals` 式流程。
- 不修改 `.skillpack` 只读边界。
- 不把 `employee_id + skill_id` 作为新 memory/growth identity；旧字段只能作 alias、UI label 或迁移线索。
- 不在 MVP 内引入价格、合同、客户交付周期或外部 SLA 承诺。

### 7.3 建议验证命令

若只改前端产品路径，优先：

```bash
cd /mnt/d/code/workclaw
pnpm --dir apps/runtime exec tsc --noEmit
pnpm --dir apps/runtime test -- src/components src/scenes
```

若触及 Tauri runtime session / employee / profile 边界，补充：

```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo check
cargo test --test test_agent_profile_docs --test test_profile_memory_runtime --test test_employee_growth
```

若触及 sidecar removal 相关边界，必须按 roadmap 选择更窄的命令，不允许用 MVP 名义跳过 caller audit / provider replacement。

---

## 8. PM / 飞书汇总口径

[FEISHU_UPDATE] 产品策略结论：WorkClaw 3-7 天 MVP 应收敛为“桌面首页一句话任务 + 选择/创建 AI 员工 + 默认团队入口 + 进度可见 + 结果沉淀到会话/Profile Home”的最小闭环；本轮不新增 OpenClaw 兼容目标、不新增 sidecar endpoint、不做商业报价或外部交付承诺。后续工程应优先复用现有 Landing、EmployeeHub、TaskTabStrip、ToolIsland、EmployeeRuns、TaskJourneySummary 与 profile runtime 能力，按本文 8 条产品验收标准做 smoke test。
