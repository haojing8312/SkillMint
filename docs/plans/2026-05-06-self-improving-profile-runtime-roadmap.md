# WorkClaw 下一阶段迭代总纲：Hermes-Aligned Self-Improving Profile Runtime

## 1. 背景与目标

WorkClaw 下一阶段的核心研发方向是对齐 Hermes Agent 的核心能力：长期记忆、会话检索、渐进式技能加载、自我沉淀、curator、toolsets、多 agent/profile 成长循环。

当前 WorkClaw 已有员工、技能、团队模板、IM 指挥、审批总线、浏览器自动化、`.skillpack` 加密分发等基础，但长期记忆仍以 `employee_id + skill_id` bucket 为中心。这种设计适合早期隔离记忆，不适合作为成熟 self-improving runtime 的核心身份模型。

目标架构：

```text
profile_id -> AI employee runtime home
```

每个 AI 员工都是一个完整 profile，拥有自己的记忆、技能、经验库、工具权限、会话索引、curator 状态和成长记录。用户可以看到它学到了什么、改了什么、为什么改、如何回滚，并把成熟技能打包分发。

## 2. Hermes 对齐原则

1. 不再以 `employee_id + skill_id` 作为新记忆架构的设计中心。
2. 以 `profile_id` 作为 AI 员工运行时身份根。
3. 功能、能力、使用体验优先对齐 Hermes，不额外发明默认人工审批队列。
4. 不再把 OpenClaw 目录结构作为下一代 runtime 的兼容目标；`profiles/<profile_id>/...` 是唯一 canonical home。
5. 不以 MVP 思维做短期补丁；实现可以分阶段，架构必须按成熟产品形态设计。
6. `.skillpack` 是商业分发边界，默认不可被自动优化、删除或 patch。
7. 原 `builtin skill` 改为 `preset skill` 概念：预置技能可优化、删除、归档、重置。
8. Self-improving 写入必须有来源、版本、审计和回滚路径；普通低风险成长默认由 agent 直接沉淀，高风险/破坏性操作才触发明确确认。
9. 不再引入 `memory_patch_proposals`、growth review inbox、approve/reject no-op API 作为默认成长链路。这条路线已在 2026-05-07 被撤回。
10. 每个阶段完成后必须更新本文档状态标记，避免后续开发跑偏或遗漏。

## 3. 状态标记

- `[ ]` 未开始
- `[~]` 进行中
- `[x]` 已完成并通过验收
- `[!]` 阻塞，需要重新设计或等待外部条件
- `[d]` 延后，必须保留原因
- `[r]` 撤回，表示曾经实现或规划过但与 Hermes 对齐目标冲突

## 4. 目标架构

```text
profiles/<profile_id>/
  config.json
  instructions/
    RULES.md
    PERSONA.md
    USER_CONTEXT.md
  memories/
    MEMORY.md
    PROJECTS/<workspace_hash>.md
    history.jsonl
  skills/
    active/
    stale/
    archive/
    presets/
    versions/
  sessions/
  curator/
    usage.json
    reports/
  growth/
    learnings.jsonl
    changes.jsonl
```

核心表/索引方向：

- `agent_profiles`: profile 主表，承载 AI 员工身份、persona、默认工作区、默认 toolsets、UI 展示名。
- `profile_memories`: profile 记忆文件索引、预算、版本、更新时间、来源。
- `profile_session_index` / `profile_session_fts`: SQLite profile 会话检索索引，当前覆盖 bounded manifest 字段、run summary、tool summaries、compaction boundaries；后续扩展到用户消息、assistant 响应、最终结果。
- `profile_skills`: profile 下可见技能及生命周期状态。
- `skill_sources`: 技能来源，区分 `preset`、`local`、`agent_created`、`skillpack`。
- `skill_usage`: view/use/patch/last_used/pinned/lifecycle/curator score。
- `growth_events`: 成长记录，记录来源会话、工具证据、写入目标、版本和回滚路径；不是默认审批队列表。
- `curator_runs`: curator 整理报告、建议、执行结果。

旧表中的 `employee_id` 可作为 routing alias、UI label 和迁移线索继续存在，但新功能不得继续把 `employee_id + skill_id` 当作记忆隔离模型。旧 `employees/<employee>/openclaw/...` 目录只作为迁移输入读取，不再作为新 runtime 的镜像或兼容投影目标。

现有员工界面中的 `AGENTS.md / SOUL.md / USER.md` 不再被定义为 OpenClaw 兼容文件。它们应迁移为 profile instruction assets：

- `AGENTS.md` -> `instructions/RULES.md`
- `SOUL.md` -> `instructions/PERSONA.md`
- `USER.md` -> `instructions/USER_CONTEXT.md`

这些 instruction assets 描述行为规则、人格和用户/团队上下文，不承担长期学习记忆职责。长期学习统一进入 `memories/MEMORY.md`、project memory、session index 和 growth log。

## 5. 阶段路线图

### Phase 0. 基线治理与兼容边界

状态：`[~]`

已完成调研：

- `[x]` Identity / DB 依赖地图：`docs/plans/2026-05-06-profile-runtime-identity-db-map.md`
- `[x]` Memory path / runtime context 地图：`docs/plans/2026-05-06-profile-runtime-memory-map.md`
- `[x]` Skill OS source boundary 地图：`docs/plans/2026-05-06-skill-os-source-boundary-map.md`
- `[x]` Toolset gateway 地图：`docs/plans/2026-05-06-toolset-gateway-map.md`
- `[x]` Self-improving employee UX 设计：`docs/plans/2026-05-06-self-improving-employee-ux.md`
- `[x]` Phase 0 主会话综合裁决：`docs/plans/2026-05-06-profile-runtime-phase-0-synthesis.md`

验收标准：

- `[x]` 有文档列出所有受影响模块和迁移风险。
- `[~]` 新旧数据库都能启动，并有 legacy schema regression test。测试已编译进 Tauri lib test binary；当前 Windows 环境直接执行该 binary 失败于 `STATUS_ENTRYPOINT_NOT_FOUND`，需要后续环境修复后补跑。
- `[x]` 任何新增 self-improving 设计均不再依赖 `employee_id + skill_id` 作为核心 identity。
- `[x]` `.skillpack` 只读边界有测试或代码级防护。

### Phase 1. Profile Runtime 重构

状态：`[~]`

目标：建立成熟 AI 员工 profile runtime，替代旧员工记忆 bucket 设计。

任务：

- `[x]` Foundation Slice 1 plan：`docs/plans/2026-05-06-profile-runtime-foundation-slice-1-plan.md`
- `[x]` 新增 `agent_profiles` 及 profile runtime service。
- `[~]` 建立 profile home 目录创建、读取、迁移、恢复逻辑。当前画像应用和普通桌面员工 upsert 都会创建 `profiles/<profile_id>/instructions`、`memories`、`skills`、`sessions`、`growth`、`curator` 基础目录；员工 workbench 读取/导出会修复空 `profile_home` 到 canonical profile home；完整恢复/导入待后续 slice。
- `[x]` 将旧员工 `AGENTS.md / SOUL.md / USER.md` 迁移到 `instructions/RULES.md / PERSONA.md / USER_CONTEXT.md`，并停止生成 OpenClaw-style mirror 目录。
- `[~]` 将聊天会话、员工入口、IM 路由、团队运行入口接入 `profile_id`。
- `[~]` 保留旧 `employee_id` UI/API 兼容层，但内部新上下文使用 profile。
- `[ ]` 在员工中心展示 profile runtime 状态：记忆、技能、工具集、最近成长事件。

验收标准：

- `[x]` 新创建员工必须拥有 profile home。当前通过员工助手/团队模板应用画像以及普通桌面 upsert 都会创建 canonical profile home；员工详情读取/导出会修复空 `profile_home`。
- `[x]` 新创建员工不再依赖 `employees/<employee>/openclaw/...` 目录作为 profile source。
- `[ ]` 旧员工启动后能迁移或映射到 profile，不丢会话。
- `[ ]` 团队运行中的每个步骤绑定 profile，而不是只绑定文本 employee id。
- `[ ]` IM 路由能定位目标 profile。
- `[ ]` Profile home 删除、重置、导出有明确交互和风险确认。

### Phase 2. Memory OS

状态：`[~]`

目标：实现 Hermes-style 可感知、可编辑、可检索、可直接使用的记忆系统。

已完成：

- `[x]` Profile Memory Locator Slice 2 plan：`docs/plans/2026-05-06-profile-memory-locator-slice-2-plan.md`
- `[x]` 增加只读 `ProfileMemoryLocator`，描述 profile home 目标路径和 legacy memory 候选路径，不改变 prompt 输出。
- `[x]` 将 `profile_id` 贯穿 session execution context、turn execution context、runtime tool setup。
- `[x]` 增加 profile memory read bundle：优先读取 profile `MEMORY.md`，缺失时回退 legacy memory bucket。
- `[x]` 暴露只读 `get_employee_profile_memory_status` Tauri command。
- `[x]` 在员工详情页增加只读 Profile Home 状态条。
- `[x]` 普通 `memory` tool 注册已切到 profile `memories/` 目录；无 profile 时才回退 legacy bucket。IM `capture_im/recall_im` 仍保留在 legacy IM 子模型目录，避免线程/角色记忆被误并入主 `MEMORY.md`。
- `[x]` `memory` tool 已支持 profile 主文件动作：`view`、`add`、`replace`、`remove(confirm=true)`、`history`，并写入 `memories/MEMORY.md` 与 `memories/history.jsonl`。
- `[x]` 当前 workspace 的 Project Memory 已落到 `memories/PROJECTS/<workspace_hash>.md`，`memory` tool 可通过 `scope=project` 读写。
- `[x]` Profile Memory 注入已增加默认预算裁剪，避免无限注入 prompt；当前策略保留尾部新近内容并写明截断。
- `[x]` Profile session manifest 已落到 `profiles/<profile_id>/sessions/<session_id>/manifest.json`，记录 session、skill、workdir、journal/state/transcript 路径、最近 run summary、tool summaries 和 compaction boundaries，为后续 session transcript/index 打基础。
- `[x]` Profile session manifest 已写入 SQLite 检索索引：`profile_session_index` 元数据表与 `profile_session_fts` FTS5 表；runtime 写 manifest 后会刷新索引，并暴露 `search_profile_sessions` Tauri command。中文检索保留 profile 限定 `LIKE` fallback，避免 FTS5 分词漏召回。
- `[x]` `memory.search` 已接入 Profile Session Search，agent 可通过 `memory` tool 按 query 召回当前 profile 的历史 run summary、tool summaries 和 compaction summaries。
- `[x]` Profile Session Search 已扩展索引 DB `messages` 中的用户消息与 assistant 最终回答；主任务 terminal commit 成功后会刷新当前 session 的 profile index，让刚完成的任务可被后续 `memory.search` 召回。
- `[x]` Profile Session Search 已增加 run/turn 级 FTS 文档：索引 `session_runs` 绑定的用户消息、assistant 最终回答、buffered/error 文本，以及同 run 的 tool/compaction 摘要；搜索结果返回 `document_kind` 与 `matched_run_id`，agent 可定位命中的具体 run。
- `[x]` Profile Session Search 已支持 filters：`work_dir/workspace`、`updated_after`、`updated_before`、`skill_id`、`source`。`memory.search` 与 `search_profile_sessions` 共享同一过滤结构，且 session/run 结果都保留 `document_kind` 与 `matched_run_id`。
- `[x]` Profile/Project Memory 已增加文件系统版本库：`versions/profile/<version_id>.md|json` 与 `versions/projects/<workspace_key>/<version_id>.md|json`。`add/replace/remove` 会生成版本快照和增强 history，`versions`、`view_version`、`rollback(confirm=true)` 可查看和恢复任一版本。
- `[x]` Profile session transcript mirror 已落到 `profiles/<profile_id>/sessions/<session_id>/transcript.md`：索引 manifest 时写入 DB messages、run id、tool summaries 和 compaction boundaries，让 profile home 自身具备可读证据链。
- `[r]` Growth review inbox / memory proposal / approve-reject audit slices 已撤回并从代码中移除；它们与 Hermes 默认体验不一致。

下一步任务：

- `[x]` 为每个 profile 实现 `MEMORY.md` 的运行时写入入口。
- `[x]` 为每个 workspace 实现 project memory：`PROJECTS/<workspace_hash>.md`。
- `[x]` 实现 memory injection budget，避免无限注入 prompt。
- `[x]` 实现 `memory` tool：`view`、`add`、`replace`、`remove`、`history`、`search`。`remove` 需要 `confirm=true`；`search` 读取 Profile Session Search。
- `[x]` 实现记忆版本历史和回滚。
- `[x]` 实现 session transcript 保存到 profile sessions。当前已写入 profile session manifest 与 profile-local `transcript.md`，包含 DB messages、run id、最近 run summary、tool summaries 和 compaction boundaries，并建立 message-aware FTS5 索引。
- `[x]` 实现 SQLite FTS5 `session_search`，支持按 profile、workspace、time、skill/source 过滤。当前完成按 profile + query 搜索 bounded manifest 字段、用户消息、assistant 最终回答，并能返回 run/turn 级 `matched_run_id`。
- `[ ]` 在 UI 中展示记忆来源、最近变更、版本历史和回滚入口。

验收标准：

- `[x]` Agent 可通过 tool 读取当前 profile 记忆。
- `[x]` Agent 可直接写入低风险 profile memory，并记录来源、版本、变更摘要和回滚路径。
- `[~]` 高风险记忆操作，如删除、批量压缩、外部内容注入、冲突覆盖，触发风险确认或策略阻断。当前 `remove` 已要求 `confirm=true`，其他高风险策略待 Toolset/Growth slice 补齐。
- `[x]` session_search 能召回历史任务、工具调用摘要和最终结果。当前 agent 可通过 `memory.search` 召回 manifest run summary、tool summaries、compaction summaries、用户消息、assistant 最终回答，并定位命中的 run/turn；支持 workspace、time、skill/source 过滤。
- `[x]` 记忆注入不会超过默认预算。
- `[x]` Memory 变更可回滚到任一历史版本。

### Phase 3. Skill OS 与 Progressive Disclosure

状态：`[~]`

目标：让 skill 从“安装插件”升级为 profile 可主动检索、按需加载、可沉淀、可维护的方法库。

任务：

- `[x]` 建立 Skill OS read-only index，包含 manifest summary、tags、source policy、只读/可变更能力边界。
- `[x]` 实现 progressive disclosure：当前 agent 已有 `skills` tool，可通过 `skills_list` 看摘要、`skill_view` 按需读取单个 local/preset skill；默认 turn preparation 已改为 summary-first，不再同步/投影全部 installed skills。
- `[x]` 实现 `skills_list` tool。
- `[x]` 实现 `skill_view` tool。
- `[x]` 实现 `skill_manage` tool：create、patch、archive、restore、delete、reset。当前合并在 `skills` tool 下，已实现 `skill_create`、`skill_patch`、`skill_archive(confirm=true)`、`skill_restore`、`skill_delete(confirm=true)`、`skill_versions`、`skill_view_version`、`skill_rollback(confirm=true)`、`skill_reset(confirm=true)`。
- `[~]` 引入 skill source：当前 Skill OS index 归一 `preset`、`local`、`agent_created`、`skillpack`、`builtin`、`unknown`；ClawHub/industry provenance 细分待后续补齐。
- `[x]` 对 `skillpack` 强制只读。
- `[x]` 对 preset/local/agent_created 提供版本、diff、rollback/reset。当前 `skills` tool 已对目录型 local/preset/agent_created skill 提供 `skill_create`、`skill_patch`、`skill_archive(confirm=true)`、`skill_restore`、`skill_delete(confirm=true)`、`skill_versions`、`skill_view_version`、`skill_rollback(confirm=true)`、`skill_reset(confirm=true)`，版本快照落到 `skill_versions`，生命周期状态落到 `skill_lifecycle`。
- `[ ]` 在专家技能中心展示 profile skill library 与生命周期。

验收标准：

- `[x]` Agent 启动时不会全量注入所有 skill 内容。默认 turn preparation 只注入 summary-only `<available_skills>`，并提示使用 `skills.skill_view` 按需读取详情；默认路径不会把全部 installed skills 投影到 workspace。
- `[x]` Agent 可通过 `skills_list` 找到候选技能。
- `[x]` Agent 可通过 `skill_view` 读取单个技能完整内容。
- `[x]` Agent 可创建 `agent_created` skill，并记录来源会话。当前 `skills.skill_create` 会把新技能写入 `profiles/<profile_id>/skills/active/<skill_id>/SKILL.md`，登记 `installed_skills.source_type='agent_created'`，并写入 create 版本和 `growth_events`。
- `[~]` Agent 可修改 preset/local/agent_created skill，变更必须有 diff、版本和审计记录。当前目录型 local/preset/agent_created skill 已支持 patch + diff + version snapshot + rollback/reset + growth event；完整 UI 审计视图待补齐。
- `[~]` 高风险 skill 变更，如新增脚本、放宽 toolset、删除 active skill，触发风险确认。当前 `skill_archive`、`skill_delete`、`skill_rollback` 与 `skill_reset` 已要求 `confirm=true`；脚本/toolset 风险识别待补齐。
- `[x]` `.skillpack` 无法被 `skill_manage` 修改、删除或归档。Skill OS index/tool view 已把 `.skillpack` 标为 read-only/derived，且 patch/archive/delete/rollback/reset 不会解包或改写 `.skillpack`。
- `[~]` Skill 修改可回滚/重置。当前 `skill_rollback(confirm=true)` 可将目录型 local/preset/agent_created skill 回滚到 `skill_versions` 中的历史快照；`skill_reset(confirm=true)` 可重置到最早版本基线并生成 reset 版本记录。

### Phase 4. Preset Skill Migration

状态：`[~]`

目标：将现有 builtin skill 改造为 preset skill，允许优化、删除、归档、重置。

验收标准：

- `[ ]` 新装用户获得 preset skills。
- `[ ]` 已装用户迁移后不丢失现有技能。
- `[ ]` 修改过的 preset skill 升级时不被覆盖。
- `[~]` 用户或 agent 可重置 preset skill，reset 本身也生成版本记录。当前目录型 local/preset/agent_created skill 已可通过 agent `skill_reset(confirm=true)` 或员工详情页 `reset_skill_os` 重置到最早版本基线，并生成版本记录与 profile growth event；preset seed 完整迁移待补齐。
- `[ ]` `skillpack` 不参与 preset migration。

### Phase 5. Growth Loop

状态：`[ ]`

目标：实现任务后经验沉淀闭环，让 AI 员工能持续积累经验，且使用体验对齐 Hermes 的自然增长模型。

任务：

- `[ ]` 在每次任务完成后生成 after-action review。
- `[ ]` 识别 memory candidate、skill candidate、skill patch candidate、tool workflow candidate。
- `[ ]` 将低风险成长直接落到 Memory OS 或 Skill OS，记录来源、版本和回滚路径。
- `[ ]` 将高风险成长转为风险确认或策略阻断，不进入默认人工审批队列。
- `[~]` 记录成长项来源：session、tool trace、用户纠正、错误恢复。当前 `skill_create`/`skill_patch`/`skill_archive`/`skill_restore`/`skill_delete`/`skill_rollback`/`skill_reset` 与 `curator.scan`/`curator.run`/`curator.restore` 会写入 `growth_events`，记录 profile_id、session_id、skill_id、version_id、diff、curator report 或 lifecycle evidence 和 summary；memory/纠错来源待补齐。
- `[ ]` 在员工成长视图展示已学习内容、最近变化、可回滚项和被阻断的高风险项。

验收标准：

- `[~]` 每个 profile 有可见成长历史。当前 DB 已有 `growth_events`，Skill OS create/patch/archive/restore/delete/rollback/reset 与 curator scan 会写入 profile/session 关联事件；`list_employee_growth_events` 与员工详情页成长记录区已可展示最近事件，sessions/skills 深链和完整筛选待补齐。
- `[ ]` 任务结束后能生成结构化成长项。
- `[ ]` 后续任务能使用已沉淀的 memory 或 skill。
- `[ ]` 用户能撤销或回滚错误沉淀。
- `[~]` 所有沉淀均可追溯来源。当前 skill patch/rollback 已追溯到 session/profile/version/diff；curator scan 已追溯到 profile/report/findings；memory add/replace/remove/rollback 与用户纠错写入会记录 growth event 和版本证据。完整 session evidence drill-down 待补齐。

### Phase 6. Curator

状态：`[~]`

目标：防止 self-improving 变成 self-polluting，通过 curator 管理 skill 和 memory 的生命周期。

任务：

- `[x]` 实现 curator dry-run report。当前 agent `curator.scan` 会扫描重复记忆、可沉淀技能候选、低价值记忆碎片和低价值可变 skill 草稿，生成报告但不直接修改 memory/skill。
- `[x]` 记录 skill usage：view/use/patch/last_used/pinned/lifecycle。当前 `skill_lifecycle` 已记录 view/use/patch counts、last_* 时间、pinned 和 state；`skills.skill_view` 记录 view，patch/reset/rollback 等变更记录 patch/版本/growth，隐式路由的 inline/fork/direct-dispatch 技能执行和显式技能命令都会 best-effort 记录 `use_count` 与 `last_used_at`。
- `[~]` 对 agent_created、preset、local skill 提出 archive、merge、patch、delete 建议。当前 curator 通过 `content_chars`、低价值 manifest、`use_count`、`patch_count`、`pinned` 和 source type 生成可解释评分：未使用且未 patch 的 agent_created/local 草稿可被 `curator.run` 标记 stale；已被真实执行或正在 patch 演进的草稿只生成 `skill_improvement_candidate`，建议用 `skills.skill_patch` 补全说明；preset、builtin、skillpack 不参与 curator stale 标记；delete/merge 策略待补齐。
- `[x]` 支持 pinned skill 免整理。当前 `pin_skill_os` 可设置 pinned，员工 Skill OS 区域可固定/取消固定，`curator.run` 会跳过 pinned skill。
- `[~]` 支持 curator 执行后回滚。当前 `curator.restore` 可将 `curator.run` 标记为 `stale` 的 skill 恢复为 `active`，并写入 `curator_runs` 与 `growth_events.event_type='curator_restore'`；更完整的 report-level 批量回滚待补齐。
- `[~]` 对 memory 提出去重、压缩、过期、冲突提示。当前支持重复记忆、流程型技能候选和低价值碎片提示；过期/冲突提示待补齐。
- `[~]` 在 UI 中展示 curator 报告。当前员工详情页 Curator 区块可展示最近 report、finding 类型、严重度、目标、建议动作、状态变更和可恢复候选，并可展开完整结构化 report JSON；stale skill 可通过员工详情页触发 `curator.restore` 风格恢复。历史筛选和批量执行入口待补齐。

验收标准：

- `[~]` Curator 默认可自动执行低风险整理，破坏性操作需要确认。当前 `curator.run` 只会将未 pinned、未真实使用的 mutable 草稿 skill 标为 `stale`，不删除或改写内容；`curator.restore` 可恢复该生命周期状态。
- `[x]` Curator 不触碰 `.skillpack` 内容。
- `[x]` Pinned skill 不会被归档或删除。当前 `curator.run` 跳过 pinned skill，并在报告中记录 `pinned_skill_protected`。
- `[~]` Curator 生成的报告可被用户理解和追溯。当前报告写入 `curator_runs` 和 `profiles/<profile_id>/curator/reports/<run_id>.json`，并写入 `growth_events`；`curator.history` / `list_employee_curator_runs` 同时保留 raw report 并投影状态变更目标与可恢复候选，员工详情页已展示最近 Curator 报告摘要、findings、状态变更、恢复动作和完整结构化 report JSON。
- `[~]` Curator 操作可回滚。当前 stale 标记可通过 `curator.restore` 恢复，员工详情页已接入该恢复入口；批量 report rollback、memory 整理回滚和归档回滚待补齐。

### Phase 7. Toolset Gateway

状态：`[~]`

目标：以 toolset 为中心统一工具授权、skill 依赖、员工权限和风险策略。

任务：

- `[x]` 定义 toolset model：`core`、`memory`、`skills`、`web`、`browser`、`im`、`desktop`、`media`、`mcp`。当前 agent `toolsets` tool 已按这些名称输出 projection。
- `[~]` 将现有 tool metadata 映射到 toolset。当前 `toolsets` tool 使用 `ToolManifestEntry` 的 category/source/risk 字段，并通过工具名补齐 browser/mcp/im/media 等 projection；不改变原 metadata 和审批策略。
- `[ ]` Skill 支持 `requires_toolsets`、`optional_toolsets`、`denied_toolsets`。
- `[ ]` Profile 支持默认 allowed toolsets。
- `[~]` MCP 和 sidecar 工具纳入统一 toolset gateway。当前 `mcp_*` 自动投影到 `mcp`，`browser_*` 自动投影到 `browser`；sidecar metadata 原地不变。

验收标准：

- `[ ]` Skill 缺少必要 toolset 时不进入可执行状态。
- `[ ]` Profile 可限制特定 toolset。
- `[ ]` 高风险 toolset 操作触发现有工具风险确认。
- `[~]` Toolset 变更不会绕过现有危险命令拦截。当前第一刀只读 projection，不参与 allow/deny 或审批决策。
- `[~]` Browser、IM、MCP 工具都能通过 toolset manifest 可观测。当前 browser/mcp/name-based IM projection 已有测试覆盖 browser/mcp；真实 IM 工具映射待后续补齐。

### Phase 7B. Hermes-Aligned Sidecar Removal

状态：`[~]`

目标：一步一步去掉 `apps/runtime/sidecar`，把 browser、MCP、IM/channel、Feishu/WeCom、OpenClaw route compatibility 和 sidecar lifecycle 分别迁移到 Hermes-aligned runtime 边界：Rust ToolRegistry、Toolset Gateway、gateway/platform adapters、profile runtime 与 native providers。详细迁移计划见 `docs/plans/2026-05-11-hermes-aligned-sidecar-removal-roadmap.md`。

任务：

- `[x]` 完成 sidecar 职责盘点和 Hermes 参考架构映射，明确 sidecar 不再是未来产品边界。
- `[x]` 明确 OpenClaw 相关内容仅作为 legacy migration input；不再新增 OpenClaw compatibility、vendor sync 或 OpenClaw-shaped runtime 设计。
- `[x]` 第一批替换 `/api/openclaw/resolve-route`：IM route resolver 已由 Rust runtime 原生处理，保留当前调用契约和回归测试。
- `[ ]` 将 OpenClaw 命名的核心 routing/gateway 层迁移到中性 IM/profile runtime 命名，只保留必要的临时 adapter。
- `[ ]` 删除 OpenClaw browser compatibility、vendor sync lanes 和 sidecar route endpoint。
- `[ ]` 将 MCP server 管理、list/call tools 和动态工具注册迁入 native runtime，废弃 MCP sidecar bridge。
- `[ ]` 将 Feishu/WeCom/channel connector 迁入 gateway/platform adapter 边界，移除 `sidecar_base_url` 产品心智。
- `[ ]` 将 browser automation backend 迁入 native browser provider，保留 Hermes-compatible browser tool schema。
- `[ ]` 在所有消费者迁移后删除 `apps/runtime/sidecar`、sidecar lifecycle、bundle resources 和 sidecar build/test scripts。

验收标准：

- `[x]` Rust/Tauri 不再调用 `/api/openclaw/resolve-route`。
- `[ ]` OpenClaw compatibility 不再作为新 runtime 功能、文档或 release lane 的默认目标。
- `[ ]` Browser、MCP、IM 工具都通过 native provider + Toolset Gateway 可观测，而不是通过 sidecar bridge 推断。
- `[ ]` Runtime startup 不再启动或 health-check sidecar process。
- `[ ]` Desktop build/package 不再包含 `resources/sidecar-runtime`。

### Phase 8. Hermes Parity Evals

状态：`[ ]`

目标：建立真实评测，确保 WorkClaw 至少具备 Hermes 核心 self-improving 能力，并保留 WorkClaw 自身桌面/员工团队优势。

任务：

- `[ ]` 记忆召回评测：跨会话记住用户偏好并正确使用。
- `[ ]` 项目记忆评测：在指定 workspace 召回项目约定。
- `[ ]` session_search 评测：搜索过去会话并复用解决方案。
- `[ ]` skill 自生成评测：从重复任务沉淀 agent_created skill。
- `[ ]` skill patch 评测：根据用户纠正修补 skill。
- `[ ]` preset reset 评测：修改后可恢复预置版本。
- `[~]` curator 评测：归档 stale skill，不触碰 pinned 和 skillpack。当前 deterministic Rust tests 覆盖 unused draft -> stale、pinned skip、used draft -> improvement candidate；real-agent scenario `skill_curator_lifecycle_parity_2026_05_09` 覆盖 Skills + Curator 生命周期协同契约，真实模型运行仍需本地 provider 配置。
- `[ ]` toolset 评测：权限缺失时拒绝执行并解释。
- `[ ]` 员工团队评测：多个 profile 各自记忆和技能互不污染。

## 6. UI 总体验收

- `[ ]` 员工详情页展示 profile home 状态。
- `[~]` 员工详情页展示 Memory、Skills、Growth、Curator 四个区域。当前已有 Profile files、Profile Memory 状态、长期记忆工具、Growth Timeline、Curator 最近报告和 Skill OS 区域；Skill OS 区域可查看员工绑定技能的 source boundary、生命周期能力、toolset 声明、`SKILL.md` 内容和版本历史，并可对 mutable skill 执行确认式 patch/reset/rollback/archive/delete，以及恢复 archived skill。
- `[~]` 用户能查看 AI 员工学到了什么、何时学到、来源是什么。当前 Growth Timeline 展示事件类型、summary、时间、session_id、target_id 和版本证据；完整 evidence 展开和深链待补齐。
- `[~]` 用户能查看 memory/skill diff 和版本历史。当前 Growth Timeline 展示 memory/skill/curator 事件证据，Skill OS 区域展示最近 skill version history 并提供 rollback 入口；完整 diff 展开待补齐。
- `[ ]` 用户能撤销或回滚错误沉淀。
- `[~]` 用户能重置 preset skill。当前工具层和员工详情页均支持目录型 preset/local/agent_created skill reset；preset seed 完整迁移和更完整 diff 视图待补齐。
- `[x]` 用户能确认 `.skillpack` 处于只读保护状态。当前 Skill OS 员工区将 immutable skillpack 显示为 `.skillpack · 只读`，并说明不会被 patch/reset/curator 修改。
- `[~]` 用户能导出某个 profile 的成长记录。纠偏：不做独立 Growth Timeline 导出按钮；当前员工详情页可导出 Hermes-aligned Profile artifact zip，包含 resolved profile home 文件和 `PROFILE_EXPORT.json`，growth、curator reports、skill usage、memory versions 随 profile home 一起导出；后续补 profile import/restore。

## 7. 安全与合规验收

- `[ ]` 所有 memory/skill 写入都有来源记录。
- `[ ]` 所有自动修改都有版本或审计记录。
- `[ ]` `.skillpack` 不被自动修改。
- `[ ]` 高风险 toolset 不因 skill patch 绕过现有风险确认。
- `[ ]` Prompt injection 不能直接把外部指令写入长期记忆；外部/tool-derived 内容必须经过来源校验和风险策略。
- `[ ]` Curator 不能无审计删除用户内容。
- `[ ]` 所有 destructive 操作支持 rollback 或明确不可逆确认。

## 8. 当前纠偏记录

2026-05-07 用户明确要求：功能、能力、使用体验要对齐 Hermes，不要自己加东西。此前加入的 growth review inbox、`memory_patch_proposals`、approve/reject no-op API、review history API 和相关 UI/DB schema 属于偏航实现，已撤回并从代码中移除。后续开发不得沿这条默认审批队列继续扩展。

2026-05-07 用户进一步明确：不需要继续兼容 OpenClaw，下一代能力全面转向对标 Hermes。因此 Profile Home 是唯一 canonical runtime home；旧 `AGENTS.md / SOUL.md / USER.md` 只能作为迁移输入或 profile instruction 内容来源，不能再驱动新架构围绕 OpenClaw 目录兼容设计。

## 9. 完成定义

只有同时满足以下条件，才能认为 WorkClaw 进入 self-improving agent 产品阶段：

- `[ ]` AI 员工以 profile runtime 运行。
- `[ ]` Memory OS 可读、可写、可审、可检索、可回滚。
- `[ ]` Skill OS 支持 progressive disclosure、agent-created skill、diff、rollback。
- `[ ]` Preset skill 可进化，`.skillpack` 受保护。
- `[ ]` Growth loop 能从任务中沉淀经验。
- `[ ]` Curator 能整理成长资产且可审计。
- `[ ]` Toolset gateway 统一能力授权。
- `[~]` Hermes parity evals 通过核心场景。当前已登记 profile memory、multi-turn recall、Skill OS progressive loading、skill self-improvement、Skill OS + Curator lifecycle parity、curator scan、Toolset Gateway visibility 场景；真实运行通过本地 `config.local.yaml` 手动触发。
- `[ ]` 用户能在 UI 中理解并控制每个 AI 员工的成长。
