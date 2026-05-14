# WorkClaw 3-7天 MVP 架构边界与最小技术批次裁决

**Date:** 2026-05-12
**Kanban task:** `t_bf550684`
**Owner profile:** `agent-systems-architect`

## 1. 架构裁决摘要

3-7 天 MVP 的技术路线应聚焦在“可演示、可追踪、可回滚的 AI 员工/Profile Runtime 闭环”，而不是新增 OpenClaw 兼容层、扩展 Node sidecar，或重建一套审批/任务系统。

**MVP 最小闭环定义：**

```text
用户/PM 指令
  -> Hermes Kanban task_id 承载任务状态、依赖、运行记录
  -> WorkClaw profile_id 承载 AI 员工身份、记忆、技能、工具集、成长记录
  -> IM / Feishu / WeCom 只作为项目作战室通知和平台 adapter，不作为任务状态源
  -> 结果写回 Kanban summary/metadata，并可在 profile runtime 视角看到学习、技能、工具权限和 curator 状态
```

**核心边界：** `profile_id -> profiles/<profile_id>/...` 是 AI 员工运行时根；Kanban 是任务状态机；IM 是沟通/通知通道；Toolset Gateway 是工具授权和能力可观测边界；Memory OS / Skill OS / Curator / Growth Events 是 self-improving 闭环边界。

## 2. Roadmap phase / acceptance 对齐

本 MVP 不应新开一个偏离 roadmap 的产品分支，应直接推进现有 roadmap 的以下 phase / checkbox：

| 优先级 | Roadmap phase | MVP 推进的 acceptance checkbox | 本裁决建议 |
| --- | --- | --- | --- |
| P0 | Phase 1. Profile Runtime 重构 | `[ ]` 在员工中心展示 profile runtime 状态：记忆、技能、工具集、最近成长事件 | 作为最小可演示闭环的首选工程批次，优先展示已有能力，不新造后端模型。 |
| P0 | Phase 2. Memory OS | `[x]` Agent 可读写 profile memory；`[x]` session_search 能召回历史任务；`[x]` Memory 变更可回滚 | MVP 必须复用，不新增 `employee_id + skill_id` memory bucket。 |
| P0 | Phase 3. Skill OS | `[x]` progressive disclosure；`[x]` `skill_view`；`[~]` mutable skill diff/version/audit | MVP 必须复用 Skill OS；`.skillpack` 保持只读。 |
| P0 | Phase 6. Curator | `[~]` curator report 可理解可追溯；`[~]` low-risk stale 标记可回滚 | MVP 可展示 curator 最近报告，不做破坏性自动整理。 |
| P0 | Phase 7. Toolset Gateway | `[~]` Browser、IM、MCP 工具可通过 toolset manifest 可观测 | MVP 可展示 toolset projection；不要为了 MVP 新增绕过审批的权限路径。 |
| P0 | Phase 7B. Hermes-Aligned Sidecar Removal | `[ ]` Feishu/WeCom/channel connectors run through runtime gateway/platform adapters without sidecar base URL | 若 MVP 需要 IM 真实联动，优先做 platform adapter no-sidecar 收口，不新加 sidecar endpoint。 |
| P1 | Phase 8. Hermes Parity Evals | `[ ]` 员工团队评测：多个 profile 各自记忆和技能互不污染 | 作为质量验收输入，不作为 3-7 天第一批必做代码。 |

**不建议本轮推进：** Phase 7B Batch 6/7 的 native browser provider / 删除 sidecar process。它们是正确方向，但超出 3-7 天 MVP 的最小闭环，且容易拖入 Playwright/packaging/release 风险。

## 3. MVP 必须复用的 runtime 能力

| 能力 | MVP 用法 | 禁止另起炉灶 |
| --- | --- | --- |
| Profile Runtime | 所有 AI 员工状态以 `profile_id` 和 canonical profile home 为根。`employee_id` 只作为 UI/API alias 或迁移线索。 | 不新增 `employees/<employee>/openclaw/...` 作为运行时源。 |
| Memory OS | 复用 `memories/MEMORY.md`、Project Memory、session transcript、Profile Session Search、版本/rollback。 | 不新增 `employee_id + skill_id` bucket，不新增 memory proposal/approve-reject 队列。 |
| Skill OS | 复用 `skills_list`/`skill_view`/`skill_manage`、source boundary、版本、diff、lifecycle。 | 不解包、不 patch、不 reset `.skillpack`；不把所有 skill 全量注入 prompt。 |
| Curator | 复用 curator dry-run/report、低风险 stale 标记、restore、growth evidence。 | 不做默认人工审批 inbox；不做无审计删除。 |
| Toolsets | 复用 Toolset Gateway projection：`core`、`memory`、`skills`、`web`、`browser`、`im`、`desktop`、`media`、`mcp`。 | 不用 skill patch 绕过高风险工具确认；不新建隐形工具权限。 |
| IM / Gateway | 复用 runtime gateway/platform adapter 边界，Feishu/WeCom 是 adapter。 | 不新增 `/api/channels/*`、`/api/feishu/*`、`sidecar_base_url` 用户心智。 |
| Hermes Kanban | 复用 task_id、parents、summary、metadata、run history 作为项目协作状态系统。 | 不在 WorkClaw repo 里复制一套 Kanban 状态机；IM 群不作为唯一状态源。 |

## 4. 不可触碰边界

1. 不得新增 Node sidecar endpoint、sidecar health dependency、sidecar lifecycle dependency、`sidecar_base_url` 用户配置入口。
2. 不得新增 OpenClaw compatibility 作为产品目标；OpenClaw-shaped names/files 只可作为 legacy migration input 或临时 alias。
3. 不得以 `employee_id + skill_id` 作为新记忆、成长或 profile runtime 设计中心。
4. 不得新增 `memory_patch_proposals`、growth review inbox、默认 approve/reject 人工审批队列。
5. 不得让 MVP 依赖 `.skillpack` 可变更；`.skillpack` 是商业分发只读边界。
6. 不得把浏览器自动化作为 3-7 天 MVP 必经路径，除非先完成 native browser provider 边界；现有 sidecar browser 只能视为迁移债务。
7. 不得对外承诺价格、合同、交付周期、客户效果或生产环境变更；这些都需要郝敬确认。
8. 不得在 Kanban summary、docs、日志中写入密钥、token、App Secret、cookies、proxy credential。

## 5. 建议最小技术实现批次

### Batch A（首选）：Profile Runtime MVP 状态面板 / 员工首页闭环

**目标：** 在现有员工中心/员工首页中，把已有 Profile Runtime 能力聚合为一个“AI 员工运行时状态”闭环：profile home、memory、skills、toolsets、growth、curator、最近 session/task 证据。优先复用已有组件和命令，不新增复杂后端。

**推进 roadmap：**
- Phase 1 acceptance：`[ ]` 在员工中心展示 profile runtime 状态：记忆、技能、工具集、最近成长事件。
- UI 总体验收：`[~]` 员工详情页展示 Memory、Skills、Growth、Curator 四个区域；`[~]` 用户能查看 AI 员工学到了什么、来源是什么。
- Phase 7 acceptance：只读展示 toolset projection；不改变 allow/deny 策略。

**文件范围：**
- Frontend：
  - `apps/runtime/src/components/employees/overview/EmployeeOverviewSection.tsx`
  - `apps/runtime/src/components/employees/tools/EmployeeProfileMemoryStatusBar.tsx`
  - `apps/runtime/src/components/employees/tools/EmployeeProfileFilesSection.tsx`
  - `apps/runtime/src/components/employees/tools/EmployeeSkillOsSection.tsx`
  - `apps/runtime/src/components/employees/tools/EmployeeGrowthTimelineSection.tsx`
  - `apps/runtime/src/components/employees/tools/EmployeeCuratorReportsSection.tsx`
  - `apps/runtime/src/components/employees/hooks/useEmployeeHubTools.ts`
  - `apps/runtime/src/components/employees/hooks/useEmployeeHubRuntimeState.ts`
  - tests under `apps/runtime/src/components/employees/__tests__/`
- Backend only if existing API is insufficient：
  - `apps/runtime/src-tauri/src/commands/agent_profile.rs`
  - `apps/runtime/src-tauri/src/commands/employee_agents/memory_commands.rs`
  - `apps/runtime/src-tauri/src/agent/tools/toolsets_tool.rs`
- Docs：
  - `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md` only if a checkbox is actually completed.

**非目标：**
- 不新增 memory write API，不新增 schema，不新增审批队列。
- 不新增 sidecar/OpenClaw dependency。
- 不在本批次做 browser provider、Feishu/WeCom external e2e、release packaging。
- 不承诺完整 profile import/restore。

**风险：**
- UI 已有多个员工组件，容易把逻辑继续塞进大组件；应沿用 `components/employees/tools/*` 和 hook 边界。
- 如果 toolset projection 目前只在 agent tool 内可用，可能需要一个只读 Tauri 查询；新增时必须保持只读，不改变权限策略。
- “最近 session/task 证据”如果无法直接从 WorkClaw repo 读取 Hermes Kanban，不要复制 Kanban；只展示 profile/session/growth 可得证据，并把 Kanban task_id 留在 summary/metadata。

**验证命令：**

```bash
cd /mnt/d/code/workclaw
corepack pnpm --dir apps/runtime exec tsc --noEmit
corepack pnpm --dir apps/runtime test -- EmployeeHubView.memory-governance.test.tsx EmployeeHubView.overview-home.test.tsx
cd apps/runtime/src-tauri
cargo test --test test_toolsets_tool
cargo check
```

**完成定义：**
- 新手可在员工中心看到一个 AI 员工是否具备 profile home、memory、skills、toolsets、growth、curator evidence。
- 视图明确区分 canonical profile runtime 与 legacy/OpenClaw alias。
- 无新增 sidecar/OpenClaw/memory approval queue。

### Batch B（架构风险收口）：Feishu/WeCom runtime gateway no-sidecar MVP 切片

**目标：** 如果产品 MVP 需要 IM 项目作战室联动，优先把 Feishu/WeCom 的用户可见配置和公共发送/状态路径收口到 runtime gateway/platform adapter 边界，延续已有 WeCom T2A/T2B/T2C 方向，避免 `sidecar_base_url` 回流。

**推进 roadmap：**
- Phase 7B Batch 5：Feishu/WeCom/channel connector 迁入 gateway/platform adapter 边界。
- Sidecar removal checklist：`[ ]` Feishu/WeCom/channel connectors run through runtime gateway/platform adapters without sidecar base URL。

**文件范围：**
- Rust runtime：
  - `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
  - `apps/runtime/src-tauri/src/commands/openclaw_plugins/feishu_runtime_adapter.rs`
  - `apps/runtime/src-tauri/src/commands/channel_connectors.rs`
  - `apps/runtime/src-tauri/src/commands/im_host/**`
  - `apps/runtime/src-tauri/tests/test_feishu_gateway.rs`
  - `apps/runtime/src-tauri/tests/test_channel_connectors.rs`
  - `apps/runtime/src-tauri/tests/test_im_employee_agents/**`
- Frontend：
  - `apps/runtime/src/components/settings/feishu/**`
  - `apps/runtime/src/types/im.ts`
  - `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`
- Docs：
  - `docs/plans/2026-05-11-hermes-aligned-sidecar-removal-roadmap.md` only for actual partial result / checkbox movement.

**非目标：**
- 不删除 `apps/runtime/sidecar`。
- 不删除 OpenClaw plugin-host shim 或 public command，除非先有 neutral alias 和迁移测试。
- 不做 browser provider。
- 不做真实外部 Feishu/WeCom credential 测试，除非郝敬确认并提供安全环境。

**风险：**
- Feishu plugin-host/OpenClaw SDK compatibility 仍有 active shim；不能机械删除。
- 真实平台消息发送涉及凭据和外部副作用，必须脱敏并由郝敬确认。
- 若只改前端 copy 而后端仍需要 sidecar，会形成假迁移；必须用 grep 和 tests 约束公共路径。

**验证命令：**

```bash
cd /mnt/d/code/workclaw/apps/runtime/src-tauri
cargo test --test test_feishu_gateway
cargo test --test test_channel_connectors
cargo test --test test_im_employee_agents -- im_routing
cargo check

cd /mnt/d/code/workclaw
corepack pnpm --dir apps/runtime exec tsc --noEmit
corepack pnpm --dir apps/runtime test -- useFeishuSettingsController.test.tsx useFeishuRuntimeStatusController.test.tsx App.im-feishu-bridge.test.tsx
git grep -n "sidecar_base_url\|sidecarBaseUrl\|/api/feishu\|/api/channels" -- apps/runtime/src apps/runtime/src-tauri/src apps/runtime/src-tauri/tests
```

**完成定义：**
- 用户可见 Feishu/WeCom 配置不再鼓励填写 sidecar URL。
- 公共 start/status/send 路径默认不调用 `/api/channels/*` 或 `/api/feishu/*`。
- 剩余 OpenClaw/sidecar 命名只在明确 legacy shim/helper 中出现，并被测试或文档标注为临时兼容。

## 6. technical-lead-agent 开工输入

建议 `technical-lead-agent` 在父任务 `t_813defc2`（产品 MVP 边界）和本任务完成后，按以下输入开工：

1. **默认选择 Batch A**，除非产品父任务明确把真实 IM 联动列为 MVP 必需。
2. 开工前在任务评论中写明：目标、非目标、文件范围、验证命令、预计推进的 roadmap checkbox。
3. 任何代码批次都必须包含负向检查：
   - 不新增 sidecar endpoint。
   - 不新增 OpenClaw compatibility 目标。
   - 不新增默认人工审批队列。
   - 不新增 `employee_id + skill_id` memory bucket。
4. 如果选择 Batch B，真实外部发送、凭据读取、生产平台变更必须标记高风险并等待郝敬确认。
5. 如果发现需要 browser automation，先停止扩展现有 sidecar browser path，改为提出 Batch 6 native browser provider 子任务，不把它塞进 MVP 首批。
6. 验证完成后，只有实际完成 roadmap checkbox 时才更新 roadmap 状态标记；否则只在任务 summary/metadata 记录“推进但未完成”。

## 7. 需要架构确认 / 郝敬确认的事项

| 事项 | 风险等级 | 建议决策 |
| --- | --- | --- |
| 3-7 天 MVP 是否以 Batch A 的 profile runtime 可视化闭环为首批工程任务 | 中 | 建议确认；这是最小、最稳、最贴合 self-improving 方向的演示闭环。 |
| MVP 是否必须包含真实 Feishu/WeCom 外部消息发送 | 高 | 需要郝敬确认；涉及平台凭据、外部副作用和客户/团队沟通风险。 |
| 是否允许本轮碰 browser automation / Playwright sidecar | 高 | 建议不允许；若产品强依赖浏览器，应单独启动 native browser provider 批次。 |
| 是否在 MVP 中加入人工审批 inbox | 高 | 建议明确禁止；与 Hermes 对齐原则和撤回记录冲突。 |
| 是否对外承诺试点客户、价格、交付周期 | 高 | 不由 agent 承诺，必须郝敬最终确认。 |

## 8. 最终裁决

- 首选工程批次：**Batch A：Profile Runtime MVP 状态面板 / 员工首页闭环**。
- 备选/并行小批次：**Batch B：Feishu/WeCom runtime gateway no-sidecar MVP 切片**，仅当产品 MVP 明确需要 IM 平台联动。
- 任何 MVP 需求都必须落回现有 roadmap phase，不开 OpenClaw compatibility 新目标，不新增 sidecar endpoint，不新增默认人工审批队列。
