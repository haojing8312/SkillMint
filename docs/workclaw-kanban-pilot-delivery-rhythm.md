# WorkClaw Kanban 试点 3-7 天交付节奏与飞书通报机制

对应 Kanban 任务：`t_5e87f620`

## 1. 目标与边界

目标：把当前 WorkClaw 3-7 天试点任务图转成可执行的半日节奏，确保专家任务、飞书进度通报、PM 汇总、阻塞升级和最终复盘形成闭环。

适用范围：当前 tenant=`workclaw`、workspace=`dir:/mnt/d/code/workclaw` 的 Kanban 任务图。

历史任务图：

| 层级 | task_id | assignee | 角色定位 | 当前推进规则 |
|---|---|---|---|---|
| 并行输入 | `t_813defc2` | `product-strategist` | 产品 MVP 闭环 | 已完成；后续仅在 MVP 范围变化时调用 |
| 并行输入 | `t_783e862f` | `growth-sales-strategist` | 最小商业化验证 | 已完成；当前工程收口阶段暂停常驻 |
| 并行输入 | `t_bf550684` | `agent-systems-architect` | MVP 架构边界 | 已完成；后续仅在 runtime/profile/toolset/sidecar 边界变化时调用 |
| 并行输入 | `t_5e87f620` | `delivery-project-manager` | 交付节奏与通报机制 | 已完成；后续仅在桌面 smoke/现场节奏需要时调用 |
| 依赖执行 | `t_4bfa4c43` | `technical-lead-agent` | 首个最小工程批次 | 已完成；当前默认工程 owner |
| 依赖执行 | `t_2dfa2ad5` | `quality-test-architect` | 工程批次质量验收 | 已完成；当前默认质量门 owner |
| 依赖执行 | `t_bfdb0eff` | `customer-success-expert` | 客户成功脚本与反馈闭环 | 已完成；当前工程收口阶段暂停常驻 |
| 依赖执行 | `t_28df385a` | `business-analyst` | 指标与复盘模板 | 已完成；当前工程收口阶段暂停常驻 |

当前默认精简推进组：`workclaw-pm` + `technical-lead-agent` + `quality-test-architect`；涉及 profile runtime / toolsets / sidecar 边界时低频加入 `agent-systems-architect`；真实桌面 smoke 或客户现场节奏才加入 `delivery-project-manager`。商业、内容、增长、客户成功、数据复盘角色不再常驻。

边界：

- 不要求所有专家同时在线；按依赖推进。
- 飞书群用于可见协作和短通报，Kanban 是任务状态源。
- PM 不直接替技术、产品、商业角色做专业结论；PM 只做节奏、依赖、风险、汇总和升级。
- 涉及客户承诺、价格、合同、交付周期、生产环境变更、隐私或凭证的事项一律标记高风险，等待郝敬确认。
- 群内不得发送密钥、凭证、原始 token、cookies、proxy credential 或敏感原始 ID。

## 2. 3-7 天节奏表

> D0 表示当前任务图启动日。若实际启动在下午，则把 D0 PM 和 D1 AM 合并为首个 checkpoint；不强制等自然日。

| 时间窗 | PM checkpoint | 应完成/推进的任务 | 输入/输出 | 阻塞处理 |
|---|---|---|---|---|
| D0 AM/启动后 30 分钟内 | PM 在飞书群确认任务图、task_id、依赖、通报规则 | `t_813defc2`、`t_783e862f`、`t_bf550684`、`t_5e87f620` 认领/开始 | 每个 worker 发开始通报；Kanban 状态进入 running | 未认领：PM 记录 owner，进入 2 小时催办 SLA |
| D0 PM | 第一次半日汇总：谁已开始、谁阻塞、哪些结论可复用 | 四个并行输入任务继续产出 | 初版产品边界、商业假设、架构边界、交付节奏 | 任一输入缺失超过 SLA：PM 在任务评论催办；关键依赖缺失则飞书升级 |
| D1 AM | 依赖收口 checkpoint：优先收 `t_813defc2` + `t_bf550684` | 产品/架构结论完成后自动释放 `t_4bfa4c43` | 工程批次目标、非目标、文件范围、验证命令的上游输入 | 产品或架构结论不完整：工程任务先 block，不自行猜测实现范围 |
| D1 PM | 工程开工 checkpoint：确认工程任务是否已开工、是否写了任务评论 | `t_4bfa4c43` 制定并执行首个最小工程批次 | 工程任务评论必须含目标/非目标/范围/验证命令 | 如工程发现父任务冲突，PM 拉产品/架构 owner 返修，不扩大范围 |
| D2 AM | 工程进度 checkpoint：看 diff、验证命令、遗留风险 | `t_4bfa4c43` 尽量完成；`t_bfdb0eff` 在产品+商业完成后可启动 | 工程输出应足够 QA 判断；客户成功开始脚本草案 | 工程超时：拆小；保留一个可演示最小闭环，剩余进入下一批 |
| D2 PM | 质量入口 checkpoint：工程完成则释放 `t_2dfa2ad5` | `t_2dfa2ad5` 执行质量验收；`t_bfdb0eff` 完成客户脚本 | QA 输出 pass/fail、阻塞问题、演示准入结论 | QA 发现 P0/P1：创建/升级修复任务给技术 owner；不得带病演示 |
| D3 AM | 演示准入 checkpoint：PM 汇总产品/商业/工程/质量/客成状态 | 若 QA 可演示，PM 准备给郝敬的试点汇总 | 可演示/可内测/不可放行结论；需确认项清单 | 不可放行：明确最小修复清单和 owner，不做对外承诺 |
| D3 PM | 首轮闭环汇总：发飞书项目群总结并给郝敬列决策项 | `t_28df385a` 在依赖满足后产出指标/复盘模板 | PM 汇总 task_id、状态、结论、风险、下一步 | 若指标任务未完成，PM 先做事实汇总，BA 后补复盘模板 |
| D4-D5 | 修复/补证窗口：每天 AM/PM 两次 checkpoint | 仅处理 QA 阻塞、演示缺口、反馈脚本缺口 | 每个新增任务必须有 task_id、assignee、SLA | 禁止无 task_id 的飞书群口头派活 |
| D6-D7 | 复盘与下一批规划窗口 | 汇总所有完成/阻塞/取消任务，决定是否进入下一批 | 复盘报告、下一批候选任务、需郝敬确认项 | 若未形成可演示闭环，输出“不建议放行”与原因 |

## 3. 半日 checkpoint 固定格式

### 3.1 PM 半日巡检清单

每个 AM/PM checkpoint，PM 只检查 6 件事：

1. 状态：每个 task_id 是 todo / running / blocked / done / cancelled 中哪一种。
2. 依赖：下游任务是否可释放；是否存在父任务结论不足。
3. 交付物：当前输出是否满足任务 body 的交付物和验收标准。
4. 飞书通报：开始/阻塞/完成三类通报是否已发；未发则要求 worker 在 summary 写 `[FEISHU_UPDATE]`。
5. 风险：是否出现任务堆积、无人响应、实现越界、验证缺失、商业/客户承诺风险。
6. 下一步：每个未完成任务必须有 owner、下一动作、SLA。

### 3.2 PM 向郝敬汇总节奏

默认每天一次，遇到 P0/P1 阻塞立即追加。汇总格式：

```text
【WorkClaw Kanban试点PM汇总】
周期：D<数字> AM/PM
总体状态：绿色/黄色/红色
已完成：<task_id + 一句话结论>
进行中：<task_id + owner + 下一步 + SLA>
阻塞：<task_id + 阻塞原因 + 需要谁决策>
质量门：可演示/可内测/可交付/不建议放行/尚未进入质量门
需郝敬确认：<只列商业承诺、对外口径、优先级冲突、资源取舍>
下一步：<下一半日动作>
```

## 4. 任务类型 Owner / SLA / 完成定义

| 任务类型 | owner profile | 默认 SLA | 完成定义 DoD | PM 关注点 |
|---|---|---:|---|---|
| 产品 MVP 边界 | `product-strategist` | 0.5-1 天 | 产出 3-7 天用户路径、必须做/暂不做/风险项、5-8 条验收标准、给工程的边界输入；summary 含 `[FEISHU_UPDATE]` | 是否能转成工程任务和 QA 验收项 |
| 商业化验证 | `growth-sales-strategist` | 0.5-1 天 | 产出 1-2 个客户画像/场景假设、话术、演示场景、成功标准、3-5 个本周指标；不含价格/合同/交付承诺；summary 含 `[FEISHU_UPDATE]` | 是否存在需郝敬确认的对外承诺 |
| 架构边界 | `agent-systems-architect` | 0.5-1 天 | 产出可复用能力、不可触碰边界、1-2 个最小工程批次、验证命令、给技术 owner 的输入；summary 含 `[FEISHU_UPDATE]` | 是否约束不新增 sidecar endpoint / OpenClaw 兼容目标 |
| 交付节奏 | `delivery-project-manager` | 0.5 天 | 产出本文档：节奏表、owner/SLA/DoD、通报模板、风险清单；summary 含 `[FEISHU_UPDATE]` | 是否能直接用于当前 task_id 图 |
| 工程批次 | `technical-lead-agent` | 1-2 天 | 开工前评论目标/非目标/文件范围/验证命令；完成代码或工程计划；实际运行验证；summary 含 `[FEISHU_UPDATE]` | 不得自行扩大范围；必须基于产品+架构父任务 |
| 质量验收 | `quality-test-architect` | 0.5-1 天 | 输出测试范围、验证命令与结果、阻塞问题清单、质量门结论；summary 含 `[FEISHU_UPDATE]` | 未经 QA 不进入演示/试点放行 |
| 客户成功脚本 | `customer-success-expert` | 0.5-1 天 | 输出 30 分钟内可执行 onboarding 脚本、反馈清单、成功/失败/跟进判定、PM 汇总模板；summary 含 `[FEISHU_UPDATE]` | 不做对外商业承诺；反馈可映射回任务 |
| 指标与复盘 | `business-analyst` | 0.5 天 | 输出最小指标表、每日/每批次复盘模板、metadata/summary 字段建议、首轮复盘节奏；summary 含 `[FEISHU_UPDATE]` | 指标必须能从 Kanban summary/metadata + 飞书通报收集 |
| PM 汇总与升级 | `workclaw-pm` / `delivery-project-manager` | 半日 | 维护状态、依赖、风险、通报闭环；必要时创建后续任务给正确 profile | 不直接替专业角色做结论或承诺 |

SLA 口径：

- 开始响应 SLA：ready/running 后 2 小时内应有开始通报或 Kanban 状态变化。
- 阻塞响应 SLA：worker block 后 2 小时内 PM 必须确认 owner；P0/P1 当半日 checkpoint 内升级。
- 完成通报 SLA：kanban_complete 后 30 分钟内应有飞书完成通报或 summary 中可转发的 `[FEISHU_UPDATE]`。
- 超时处理：超过默认 SLA 仍无有效输出时，PM 不催“快点做”，而是要求拆小、降范围、block 或改派。

## 5. 阻塞处理规则

阻塞分级：

| 级别 | 触发条件 | 处理方式 | 升级对象 |
|---|---|---|---|
| P0 | 涉及商业承诺、合同、报价、生产变更、密钥/隐私、对外放行；或 QA 明确“不建议放行” | 立即停止相关下游；飞书标红；等待郝敬确认 | 郝敬 + PM |
| P1 | 父任务结论不足导致工程/QA无法继续；架构与产品范围冲突；关键 task 超 SLA 1 个半日 | 任务 block；PM 指定返修 owner 和截止 checkpoint | 对应专业 owner + PM |
| P2 | 文档不完整、飞书未通报、验证命令未贴、metadata 不规范 | PM 评论补正；下个 checkpoint 前关闭 | 当前 task owner |
| P3 | 表述可优化、模板字段缺失但不影响推进 | PM 记录复盘项；不阻塞当前闭环 | PM / BA |

阻塞处理动作：

1. worker 必须 `kanban_block(reason="...")` 或在任务评论中说明具体缺口，不接受“卡住了”这种泛化描述。
2. PM 在飞书群发阻塞通报，注明 task_id、影响范围、需要谁决策、最晚响应时间。
3. 若阻塞来自父任务输出不完整，返修父任务 owner；下游不得猜测范围继续实现。
4. 若阻塞来自质量门，技术 owner 只修复对应最小缺口；不得借机扩展大范围重构。
5. 阻塞关闭后，原任务继续推进；如果新增工作超过半日，创建新 task_id。

## 6. 飞书通报模板

### 6.1 开始通报

```text
[Kanban开始]
task_id：<task_id>
owner：<profile>
目标：<一句话目标>
计划产出：<交付物1/2/3>
依赖：<无 / 依赖 task_id>
预计下一次更新：<D0 PM / D1 AM / 具体半日checkpoint>
风险提示：<如涉及商业/客户/生产/隐私则标高风险；无则写“暂无”>
```

推荐命令：

```bash
HERMES_HOME=/root/.hermes/profiles/<profile> \
  /root/.local/bin/workclaw-agent-progress-feishu <task_id> start "<简短进度>"
```

### 6.2 阻塞通报

```text
[Kanban阻塞]
task_id：<task_id>
owner：<profile>
阻塞点：<具体缺什么，不写泛化描述>
影响范围：<影响哪些下游 task_id / 是否影响演示准入>
需要决策/输入：<谁在什么时候前给什么>
当前降级方案：<可选；如先输出计划、不改代码、拆小任务>
风险级别：P0/P1/P2/P3
```

推荐命令：

```bash
HERMES_HOME=/root/.hermes/profiles/<profile> \
  /root/.local/bin/workclaw-agent-progress-feishu <task_id> blocked "<简短阻塞说明>"
```

### 6.3 完成通报

```text
[Kanban完成]
task_id：<task_id>
owner：<profile>
结论：<一句话结论>
交付物：<文档路径 / Kanban summary / 关键列表>
验证/证据：<验证命令、质量结论或“文档任务无需测试”>
下游影响：<释放哪些 task_id / 给哪些 owner 作为输入>
遗留风险：<无 / 风险列表>
[FEISHU_UPDATE] <可由PM直接转发的一句话摘要>
```

推荐命令：

```bash
HERMES_HOME=/root/.hermes/profiles/<profile> \
  /root/.local/bin/workclaw-agent-progress-feishu <task_id> done "<简短完成结论>"
```

## 7. 风险清单

| 风险 | 触发信号 | 影响 | owner | 预防/处理 |
|---|---|---|---|---|
| 任务堆积 | 多个下游任务 todo，父任务迟迟未 done | 工程/QA/客成无法启动 | PM | 半日 checkpoint 查依赖；超过 1 个半日拆小或 block |
| 专家未通报 | running 但飞书无开始/完成/阻塞消息 | 郝敬和 PM 看不到过程 | 当前 owner + PM | 强制开始/阻塞/完成三类通报；summary 保留 `[FEISHU_UPDATE]` |
| 实现先于产品/架构决策 | 工程任务在父任务完成前自行确定范围 | 做错方向、返工 | technical-lead-agent + PM | `t_4bfa4c43` 必须等待 `t_813defc2` + `t_bf550684` |
| 验证缺失 | 工程完成但无命令、无结果、无 QA 结论 | 无法演示/交付 | technical-lead-agent + quality-test-architect | 工程提交验证证据；QA 给质量门状态 |
| 商业承诺越界 | 话术中出现价格、合同、具体交付日期、外部承诺 | 高风险外部承诺 | growth-sales-strategist + PM | 标记“需郝敬确认”；未经确认不得对外发送 |
| 架构越界 | 新增 sidecar endpoint、OpenClaw 兼容目标、默认人工审批队列 | 偏离当前 WorkClaw 路线 | agent-systems-architect + technical-lead-agent | 架构任务先裁决；工程任务按边界执行 |
| 质量门被绕过 | 未经 `quality-test-architect` 仍准备演示/试点 | 质量风险外显 | PM + quality-test-architect | PM 汇总必须包含质量门：可演示/可内测/可交付/不建议放行 |
| 飞书替代 Kanban | 群里口头派活但没有 task_id | 丢状态、丢依赖、无法复盘 | PM | 所有新增工作必须建 Kanban task；群聊只做可见通报 |
| metadata 不可复盘 | summary 只有大段自然语言，没有 changed_files/tests/decisions 等结构字段 | 后续 BA 无法自动汇总 | 当前 owner + business-analyst | 完成时写结构化 metadata；BA 任务制定字段规范 |
| 凭证/隐私泄露 | 群内粘贴 token、cookies、proxy、原始客户隐私 | 安全事故 | 全员 + PM | 一律 `[REDACTED]`；需要凭证则走安全渠道，不进群 |

## 8. 完成后如何汇总给郝敬

当核心任务达到以下任一状态时，PM 应向郝敬汇总：

- 首轮四个并行输入任务全部完成：`t_813defc2`、`t_783e862f`、`t_bf550684`、`t_5e87f620`。
- 工程 + 质量形成演示准入结论：`t_4bfa4c43`、`t_2dfa2ad5`。
- 出现 P0/P1 阻塞，需要郝敬决策。
- D3 PM / D7 PM 到达固定复盘点。

汇总给郝敬的最小结构：

```text
【WorkClaw 3-7天Kanban试点汇总】
1. 当前结论：<能否演示/能否内测/是否不建议放行>
2. 已完成任务：<task_id + owner + 一句话结果>
3. 进行中/阻塞任务：<task_id + owner + 原因 + 下一动作>
4. 质量门：<quality-test-architect 结论；若无则写“尚未进入质量门”>
5. 商业/客户信号：<只写事实，不承诺价格/合同/交付周期>
6. 需要郝敬确认：<优先级、外部口径、商业承诺、资源取舍>
7. 下一批建议：<新增 task_id / 建议创建的 task>
```

## 9. Kanban 完成时建议 metadata 字段

各任务完成时建议在 metadata 中至少包含：

```json
{
  "task_type": "product|commercial|architecture|delivery|engineering|quality|customer_success|analytics",
  "deliverables": ["..."],
  "decisions": ["..."],
  "risks": ["..."],
  "dependencies_released": ["t_xxx"],
  "feishu_update_required": true,
  "quality_gate": "not_applicable|not_started|demo_ready|internal_ready|deliverable|not_recommended",
  "needs_hao_jing_confirmation": ["..."]
}
```

工程/质量任务额外字段：

```json
{
  "changed_files": ["..."],
  "validation_commands": ["..."],
  "tests_run": 0,
  "tests_passed": 0,
  "blocking_findings": []
}
```

## 10. 本任务验收映射

| 验收项 | 本文对应位置 |
|---|---|
| 3-7 天节奏表：每日/半日 checkpoint、PM 汇总时点、阻塞规则 | 第 2、3、5 节 |
| 每类任务 owner/SLA/完成定义 | 第 4 节 |
| 飞书通报模板：开始、阻塞、完成 | 第 6 节 |
| 风险清单：任务堆积、专家未通报、实现先于产品/架构决策、验证缺失等 | 第 7 节 |
| 可直接用于当前 Kanban task_id 图 | 第 1、2 节使用当前 task_id |
| 不要求所有专家同时在线；按依赖推进 | 第 1、2、5 节 |
| 明确完成后如何汇总给郝敬 | 第 8 节 |
