# WorkClaw Hermes-Aligned Self-Improving Employee UX

Date: 2026-05-06

Status: Corrected on 2026-05-07 after Hermes parity review.

## 1. Product Direction

WorkClaw 的 AI 员工详情页应该像 Hermes-style agent workbench：让用户看到员工的 profile home、记忆、技能、成长记录和 curator 状态，同时保持 agent 自然沉淀能力。默认体验不是“所有成长都等用户审批”，而是：

- 低风险 memory/skill growth 由 agent 直接沉淀。
- 每次沉淀都有来源、版本、变更摘要和回滚路径。
- 用户可以查看、编辑、撤销、回滚、导出。
- 高风险或破坏性操作才触发明确确认。
- `.skillpack` 内容始终只读保护。
- 不再为下一代 Profile Home 保留 OpenClaw 目录兼容；`profiles/<profile_id>/` 是唯一 canonical source。

## 2. 不做什么

以下设计已被撤回，不再作为默认方向：

- Growth review inbox 作为成长主入口。
- `memory_patch_proposals` 作为默认写入中间表。
- approve/reject no-op API。
- 把所有 memory/skill 写入都改成人工审批。
- 在员工详情里堆积企业审批工作流，导致体验偏离 Hermes。
- 继续把 `employees/<employee>/openclaw/...` 当作新 runtime 的镜像或兼容目标。

现有工具高风险审批总线可以继续服务危险工具调用，但它不是 self-improving memory/skill 的默认产品模型。

## 3. Employee Detail IA

保留 `智能体员工 -> 员工详情` 作为主工作台，不新增营销式页面。

推荐结构：

```text
员工详情
  Profile Home status
  Identity and routing aliases
  Instructions
  Memory
  Skills
  Growth
  Curator
  Profile files and exports
  Danger zone
```

四个核心区域：

- `Memory`: 当前 profile 记住了什么、最近变更、来源、版本历史、回滚。
- `Instructions`: 员工行为规则、persona、用户/团队上下文。它接收旧 `AGENTS.md / SOUL.md / USER.md` 的迁移内容，但不再以 OpenClaw 文件名作为产品边界。
- `Skills`: profile 可用技能、来源、生命周期、版本、diff、reset/rollback。
- `Growth`: 最近从任务中学到了什么、哪些变更被应用、哪些高风险项被阻断。
- `Curator`: usage、整理建议、自动整理结果、需确认的破坏性操作。

## 4. Profile Home Status

员工详情顶部展示：

```text
Profile Home
  Status: Ready / Needs migration / Read-only issue / Error
  Profile ID: <profile_id>
  Aliases: employee code, IM route aliases, legacy ids
  Home: profiles/<profile_id>/
  Memory: active source, file count, last changed
  Skills: active, preset, agent_created, protected skillpack
  Growth: learned today, recent changes, blocked high-risk changes
  Curator: last run, suggestions, skipped protected assets
```

第一阶段可以只读展示 profile home 和 legacy fallback 状态；不要再加入 pending approval count。

## 5. Instruction Files UX

截图中的 `AGENTS.md / SOUL.md / USER.md` 不是一套要长期并存的新系统，也不再是 OpenClaw 兼容层。它们应迁移为 profile instruction assets：

```text
profiles/<profile_id>/instructions/
  RULES.md
  PERSONA.md
  USER_CONTEXT.md
```

迁移关系：

- `AGENTS.md` -> `RULES.md`: 工作规则、协作原则、任务边界。
- `SOUL.md` -> `PERSONA.md`: 员工人格、角色气质、表达风格。
- `USER.md` -> `USER_CONTEXT.md`: 用户、团队、业务角色和关系上下文。

Instruction assets 和 Memory OS 的边界：

- Instructions 描述“这个员工应该如何工作”。
- Memory 描述“这个员工后来学到了什么”。
- 普通 self-improving 不应频繁改 instructions。
- 修改 persona/rules/user context 属于行为层变更，应展示 diff，并对高风险变更要求确认。
- 长期学习默认写入 `memories/MEMORY.md` 或 skill，不写入 `instructions/`。

## 6. Memory UX

Memory 区域应展示真实的 profile memory，而不是审批收件箱。

核心能力：

- View: 查看 `MEMORY.md` 和 project memory。
- Add/Replace/Remove: agent 或用户可写入 profile memory。
- History: 查看每次变更的来源、时间、变更摘要、版本。
- Rollback: 恢复到旧版本。
- Search: 从 session/memory 索引召回过去任务。

Memory 变更展示：

```text
Memory Change
  Operation: add / replace / remove / compress
  Target: MEMORY.md / PROJECTS/<hash>.md
  Source: session / tool call / user correction / IM thread / curator
  Summary: what changed
  Version: before -> after
  Actions: View source / Edit / Rollback
```

风险规则：

- 单条用户偏好、明确用户纠正、低风险事实可直接写入并记录版本。
- 外部网页、IM 他人消息、工具输出中的指令不得直接变成长期记忆。
- 删除、批量压缩、冲突覆盖、跨 profile 导入、低置信外部事实需要风险确认或策略阻断。

## 7. Skill UX

技能应作为 profile 的方法库，而不是一次性安装物。

来源类型：

- `preset`: WorkClaw 预置技能，可进化、归档、删除、重置。
- `local`: 用户本地技能，可由用户或 agent 修改，需保留版本。
- `agent_created`: agent 从重复任务中沉淀的新技能。
- `skillpack`: 加密商业分发，内容只读。

Skill 区域展示：

```text
Profile Skills
  name, source, lifecycle, last used, last patched
  actions: view, edit, rollback, reset, archive, delete
  protected state: Skillpack · Read-only
```

Skill 变更展示：

```text
Skill Change
  Skill: name
  Source: preset / local / agent_created
  Files changed: SKILL.md, scripts/...
  Diff: before/after
  Evidence: session, failed attempt, user correction, repeated pattern
  Version: previous -> current
  Actions: View source / Rollback / Reset if preset
```

风险规则：

- 修改描述、trigger、示例、普通 prompt 可直接落版本并显示 diff。
- 新增脚本、放宽 toolset、扩大文件/浏览器/桌面权限、删除 active skill、reset preset 等需要风险确认。
- `.skillpack` 禁止 patch、reset、curator mutation、agent delete；允许查看安全 metadata 和显式用户卸载安装记录。

## 8. Growth UX

Growth 不应是审批队列，而应是员工的成长时间线。

展示内容：

- 任务结束后学到的 memory。
- 新创建或修改的 skill。
- 被 curator 整理的项目。
- 被策略阻断的高风险尝试。
- 用户手动回滚或纠正。

Growth item：

```text
Growth Event
  Type: memory_write / skill_create / skill_patch / curator_action / blocked_risk / rollback
  Source: session / tool / user correction / IM thread
  Result: applied / blocked / rolled_back
  Target: memory file or skill path
  Version: before -> after when available
  Actions: Open source / View diff / Rollback
```

验收标准：

- 用户能看懂员工“学到了什么”。
- 用户能追溯“为什么学到”。
- 用户能撤销错误沉淀。
- 不要求用户为每条普通成长逐个审批。

## 9. Curator UX

Curator 用于减少长期污染，而不是制造审批工作流。

默认能力：

- 统计 memory/skill usage。
- 发现重复、过期、冲突、低价值内容。
- 自动执行低风险整理并记录。
- 对删除、批量压缩、合并 skill、归档 active skill 等破坏性操作发起风险确认。
- 跳过 `.skillpack` 内容，并显示 skipped reason。

Curator report：

```text
Curator Report
  Scope: profile / memory / skills
  Summary: applied, suggested, blocked, skipped
  Evidence window: sessions and usage range
  Items: change summary, source, rollback path
```

## 10. Acceptance Criteria

Profile Home:

- `[ ]` 员工详情显示 `profile_id`、aliases、profile home path、migration state。
- `[ ]` 员工详情显示 Instructions、Memory、Skills、Growth、Curator 区域。
- `[x]` 旧 `AGENTS.md / SOUL.md / USER.md` 内容迁移到 profile instructions，不再依赖 OpenClaw 目录。
- `[ ]` Profile export 包含 growth events、versions、source evidence references。

Memory:

- `[ ]` Agent 可直接写入低风险 profile memory。
- `[ ]` Memory 变更展示 source、summary、version、rollback。
- `[ ]` 删除、批量压缩、外部内容长期化等高风险操作被确认或阻断。

Skills:

- `[ ]` Skill list 显示 source：preset、local、agent_created、skillpack。
- `[ ]` Mutable skill 变更展示 diff、source evidence、version。
- `[ ]` 用户能 rollback mutable skill。
- `[ ]` 用户能 reset preset skill。
- `[ ]` `.skillpack` 不能被 self-improvement tools 修改、reset、归档或删除。

Growth:

- `[ ]` Growth timeline 展示 applied、blocked、rolled_back 事件。
- `[ ]` 每条事件都有来源和目标。
- `[ ]` 用户能从 growth item 跳到 source session 或 diff。

Curator:

- `[ ]` Curator report 显示 applied、suggested、blocked、skipped。
- `[ ]` Curator 不触碰 `.skillpack` 内容。
- `[ ]` Curator 的破坏性操作有风险确认。

## 11. Recommended First Slice

下一刀应回到 Hermes 对齐路径：

1. 保留已完成的只读 Profile Home 状态条。
2. `[x]` 建立 canonical `profiles/<profile_id>/instructions/` 和 `profiles/<profile_id>/memories/` 目录。
3. `[x]` 将旧 `AGENTS.md / SOUL.md / USER.md` 作为迁移输入，导入 `RULES.md / PERSONA.md / USER_CONTEXT.md`。
4. `[ ]` 实现 profile `MEMORY.md` 文件创建与读取。
5. 将 memory tool 接到 profile memory：`view`、`add`、`replace`、`remove`、`history`。
6. 给 memory 写入增加版本记录、来源记录和 rollback。
7. 只对删除、外部内容长期化、批量改写等高风险操作触发确认。

不要恢复 growth review inbox、memory proposal queue 或 approve/reject no-op API。
