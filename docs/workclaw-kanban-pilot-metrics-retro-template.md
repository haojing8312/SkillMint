# WorkClaw Kanban 试点指标与复盘模板

对应 Kanban 任务：`t_28df385a`
日期：2026-05-12
适用范围：WorkClaw 3-7 天 Kanban 协作试点；tenant=`workclaw`；飞书项目作战群作为进度可见层，Kanban summary/metadata 作为状态与复盘事实源。

## 1. 目标与边界

目标：用最小数据口径支持 3-7 天试点复盘，让 PM 不依赖复杂新系统，也能从 Kanban task summary/metadata、task 事件、飞书开始/阻塞/完成通报和少量试点反馈中，形成每日/每批次可复制汇总。

边界：

- 本文只定义指标、模板、字段口径和复盘节奏，不新增埋点系统、不要求开发新看板。
- 所有指标优先手工或半自动采集，先保证可复盘，再考虑自动化。
- 涉及客户、合同、报价、交付周期、隐私、生产环境接入、公开案例授权的事项均标记为高风险，等待郝敬确认。
- 群聊和文档中不得出现密钥、token、cookies、proxy credential、客户敏感原始数据；必要时写 `[REDACTED]`。

## 2. 最小指标表

### 2.1 任务执行指标

| 指标 | 定义 | 计算口径 | 数据来源 | 3-7 天建议阈值 | 红/黄/绿判断 |
|---|---|---:|---|---|---|
| 任务完成率 | 已完成任务数 / 本批次应完成任务数 | done / (done + running + blocked + todo，取消项需单列说明) | Kanban status、complete summary | 首轮目标 >= 70% | 绿 >=70%；黄 40%-69%；红 <40% |
| 阻塞率 | 出现 blocked 或 metadata.blocking_findings 非空的任务数 / 本批次任务数 | blocked_count / total_tasks | Kanban status、block comments、metadata | 首轮目标 <= 25% | 绿 <=25%；黄 26%-40%；红 >40% |
| 阻塞关闭率 | 已关闭阻塞数 / 总阻塞数 | resolved_blockers / blockers_total | block/unblock 事件、PM 评论 | 首轮目标 >= 60% | 绿 >=60%；黄 30%-59%；红 <30% |
| 验证通过率 | 通过质量门或自检的任务数 / 需要验证的任务数 | validated_pass / validation_required | metadata.quality_gate、validation_commands、tests_run | 工程/QA 任务目标 >= 80% | 绿 >=80%；黄 50%-79%；红 <50% |
| 飞书通报率 | 完成开始/阻塞/完成必要通报的任务数 / 应通报任务数 | tasks_with_required_feishu_updates / tasks_requiring_updates | 飞书通报记录、metadata.feishu_notices_sent、summary `[FEISHU_UPDATE]` | 目标 100% | 绿 100%；黄 80%-99%；红 <80% |
| Summary 可复盘率 | summary 同时包含结论、交付物/证据、风险/下游影响的任务数 / done 任务数 | structured_summary_count / done_count | Kanban complete summary | 目标 >= 90% | 绿 >=90%；黄 70%-89%；红 <70% |
| Metadata 可聚合率 | metadata 包含最小字段的 done 任务数 / done 任务数 | structured_metadata_count / done_count | Kanban complete metadata | 目标 >= 80% | 绿 >=80%；黄 50%-79%；红 <50% |

### 2.2 产品与商业试点信号

| 指标 | 定义 | 采集方式 | 来源字段/模板 | 3-7 天判断口径 | 红/黄/绿判断 |
|---|---|---|---|---|---|
| 试点意向率 | 愿意回答资格确认问题或看演示的人数 / 有效触达人数 | 手工试点记录或飞书记录 | pilot_records.lead_status | 价值主张是否让目标用户愿意给时间 | 绿 >=50%；黄 25%-49%；红 <25% |
| 演示转试用率 | 完成演示后愿意用低敏真实任务继续试用的人数 / 完成演示人数 | 演示后反馈表 | pilot_records.demo_result、reuse_intent | “桌面 AI 团队指挥”是否比普通聊天更吸引 | 绿 >=50%；黄 25%-49%；红 <25% |
| 首任务完成率 | 完成一次发起 -> 进度 -> 结果沉淀闭环的试点数 / 开始试用数 | task_id、会话历史、文档路径 | pilot_records.first_task_done、linked_task_ids | 产品最小可用性是否成立 | 绿 >=70%；黄 40%-69%；红 <40% |
| 复用意愿率 | 明确愿意本周再用一次或推荐给相似用户的人数 / 有效试用人数 | 反馈问题“是否愿意再用一次” | pilot_records.reuse_intent | 是否有持续价值 | 绿 >=50%；黄 25%-49%；红 <25% |
| 可转任务密度 | 每次有效试用产出的可执行 Kanban 任务数 | PM 整理反馈，创建或建议创建 task_id | pilot_records.convertible_tasks_count | 反馈是否足够具体、能驱动迭代 | 绿 >=1.0；黄 0.5-0.9；红 <0.5 |
| 关键价值信号数 | 用户原话中明确表达“愿意复用/推荐/替代现有流程”的信号数 | 记录原话，不加工成营销话术 | pilot_records.value_quotes | 是否出现真实正反馈 | 绿 >=3；黄 1-2；红 0 |
| 关键阻塞信号数 | 用户原话中明确阻止继续使用的产品/信任/安全/成本/场景问题数 | 记录原话并归类 | pilot_records.blocker_quotes | 下一批产品/工程优先级输入 | 绿 <=2 且可转任务；黄 3-5；红 >5 或含 P0 |

### 2.3 质量门与放行指标

| 指标 | 定义 | 来源 | 判断 |
|---|---|---|---|
| 质量门状态 | not_started / demo_ready / internal_ready / deliverable / not_recommended | QA task metadata.quality_gate、PM 汇总 | 未经 QA 不建议对外演示；`not_recommended` 必须标红并列阻塞原因 |
| P0/P1 风险数 | P0/P1 风险条目数量 | metadata.risks、blocking_findings、PM 评论 | P0 必须等待郝敬确认；P1 必须有 owner 和 SLA |
| 需郝敬确认项数量 | 涉及高风险商业/客户/生产/隐私/优先级取舍的事项数 | metadata.needs_hao_jing_confirmation | 不追求为 0，但必须可见、可决策、不可混在普通 TODO 中 |

## 3. `[FEISHU_UPDATE]` 字段处理规则

### 3.1 必须出现的位置

1. 每个 worker 的 `kanban_complete summary` 必须包含 `[FEISHU_UPDATE]`。
2. 完成通报发送到飞书时，消息中应包含同一句或更短版 `[FEISHU_UPDATE]`。
3. 若开始或阻塞已通过飞书命令发送，则在 metadata.feishu_notices_sent 中记录状态；若未能发送，summary 仍必须保留可由 PM 复制转发的 `[FEISHU_UPDATE]`。

### 3.2 识别规则

PM 或后续自动汇总只抓取 summary 中第一段 `[FEISHU_UPDATE]` 后的一句话，作为群内可转发摘要：

```text
[FEISHU_UPDATE] <一句话说明任务完成/阻塞/关键结论；包含交付物路径或关键影响；不包含密钥、客户敏感信息、报价或承诺>
```

推荐长度：60-120 个中文字符。过长时 PM 可截断，但不得改变风险边界。

### 3.3 飞书通报率计数

| 场景 | 是否计入通报完成 | 处理方式 |
|---|---|---|
| 已调用 `workclaw-agent-progress-feishu ... start` 且成功 | 是 | metadata.feishu_notices_sent 记录 `{status:"start"}` |
| 已调用阻塞通报且 `kanban_block` | 是 | block reason 写清楚，metadata 或评论补充影响范围 |
| 已调用完成通报且 summary 有 `[FEISHU_UPDATE]` | 是 | 最佳状态 |
| 飞书命令失败，但 summary 有 `[FEISHU_UPDATE]` | 暂计为“待 PM 补发” | 飞书通报率单列为 yellow，PM 复制 summary 补发后转 green |
| summary 缺 `[FEISHU_UPDATE]` | 否 | PM 要求 owner 补正；视为 P2 过程问题 |

### 3.4 禁止写入 `[FEISHU_UPDATE]` 的内容

- 不写报价、折扣、合同、交付周期、客户定制承诺。
- 不写客户隐私、账号、密钥、token、cookies、proxy、生产系统细节。
- 不写“已对外承诺”“可直接上线”“保证交付”等最终商业承诺。
- 如必须提风险，写“需郝敬确认：<事项>”。

## 4. Kanban summary / metadata 建议字段

### 4.1 Summary 建议结构

完成时 summary 建议保持一段话，包含 5 个要素：

```text
[FEISHU_UPDATE] 已完成 <任务名/任务类型>：<关键结论>；交付物 <路径/结果>；验证/证据 <命令/自检/文档检查>；风险 <无/列表>；下游影响 <释放/输入给哪些 task_id 或 profile>。
```

阻塞时 reason 建议结构：

```text
<阻塞类型>：缺少 <具体输入/确认/权限/质量结论>，影响 <下游 task_id/演示/试点>，需要 <owner/郝敬/PM> 在 <checkpoint/SLA> 前确认。
```

### 4.2 所有任务通用 metadata 最小字段

```json
{
  "task_type": "product|commercial|architecture|delivery|engineering|quality|customer_success|analytics|pm",
  "deliverables": ["docs/...", "关键输出名"],
  "decisions": ["已形成的结论"],
  "risks": ["风险；无则 []"],
  "dependencies_released": ["t_xxx"],
  "dependencies_waiting": ["t_xxx"],
  "feishu_notices_sent": [
    {"status": "start", "ok": true},
    {"status": "done", "ok": true}
  ],
  "feishu_update_required": true,
  "quality_gate": "not_applicable|not_started|demo_ready|internal_ready|deliverable|not_recommended",
  "needs_hao_jing_confirmation": [],
  "validation": ["读回文档", "运行命令", "人工自检"],
  "changed_files": ["docs/..."],
  "next_actions": ["建议后续 task，不在当前任务内 scope creep"]
}
```

### 4.3 工程/质量任务额外字段

```json
{
  "validation_commands": ["pnpm --dir apps/runtime exec tsc --noEmit"],
  "tests_run": 0,
  "tests_passed": 0,
  "tests_failed": 0,
  "blocking_findings": [
    {"severity": "P0|P1|P2|P3", "area": "frontend|runtime|quality|security", "issue": "...", "owner": "..."}
  ],
  "files_touched_by_area": {"frontend": [], "rust": [], "docs": []},
  "rollback_notes": "如涉及代码变更，说明回滚方式；文档任务可写 not_applicable"
}
```

### 4.4 商业/客户成功试点记录字段

```json
{
  "pilot_records": [
    {
      "pilot_id": "pilot-YYYYMMDD-XX",
      "icp": "A_super_individual|B_delivery_team|other",
      "lead_source": "熟人|弱关系|社区|其他",
      "lead_score": 0,
      "lead_status": "contacted|qualified|demo_scheduled|demo_done|trial_started|trial_done|lost",
      "demo_task": "低敏任务一句话描述",
      "linked_task_ids": ["t_xxx"],
      "demo_result": "success|partial|failed|not_done",
      "first_task_done": true,
      "reuse_intent": "yes|conditional|no|unknown",
      "value_quotes": ["用户原话，必要时脱敏"],
      "blocker_quotes": ["用户原话，必要时脱敏"],
      "convertible_tasks_count": 0,
      "convertible_task_suggestions": [
        {"profile": "product-strategist|technical-lead-agent|customer-success-expert|business-analyst", "title": "...", "reason": "..."}
      ],
      "high_risk_items": ["报价|合同|交付周期|隐私|生产接入|公开案例授权；如无 []"]
    }
  ]
}
```

## 5. 每日 / 每批次复盘模板

### 5.1 PM 每日群内汇总模板

可直接复制到 WorkClaw 项目作战群：

```text
【WorkClaw Kanban试点每日复盘】
周期：D<数字> <AM/PM> / <日期>
总体状态：绿色/黄色/红色

1. 今日事实
- 已完成：<task_id + owner + [FEISHU_UPDATE]一句话>
- 进行中：<task_id + owner + 当前进展 + 下一步 + SLA>
- 阻塞：<task_id + 阻塞原因 + 影响范围 + owner + 预计关闭时间>
- 飞书通报：开始 <x>/<y>，阻塞 <x>/<y>，完成 <x>/<y>

2. 今日结论
- 产品：<MVP 边界是否更清楚；是否新增 must/should/won't-do>
- 商业：<ICP A/B 是否有新信号；不写报价/合同/交付承诺>
- 工程/质量：<质量门 not_started/demo_ready/internal_ready/deliverable/not_recommended>
- 协作：<Kanban summary/metadata 是否足够复盘>

3. 风险
- P0/P1：<必须列 owner、影响、需谁确认>
- P2/P3：<过程问题，如飞书缺通报、metadata 不完整>

4. 下一步
- 下一半日必须完成：<task_id + 动作>
- 建议新增任务：<标题 + 建议 assignee；不在群里口头派活，必须建 Kanban>

5. 需郝敬确认
- <只列商业承诺、对外口径、客户隐私/生产接入、优先级或资源取舍；无则写“暂无”>
```

### 5.2 每批次复盘模板

适用于一个工程批次、一个试点用户、或 D3/D7 阶段复盘：

```text
【WorkClaw Kanban试点批次复盘】
批次：<批次名 / D0-D3 / D4-D7 / pilot-YYYYMMDD-XX>
关联 task_id：<t_xxx, t_yyy>
复盘人：business-analyst / PM

一、事实
- 目标：<本批次原始目标>
- 实际完成：<完成 task_id、交付物路径、验证结果>
- 未完成/取消：<task_id + 原因>
- 阻塞：<阻塞次数、阻塞关闭情况、仍未关闭项>
- 试点反馈：<用户原话摘要，已脱敏>

二、指标
- 任务完成率：<x/y = z%>
- 阻塞率：<x/y = z%>
- 验证通过率：<x/y = z%>
- 飞书通报率：<x/y = z%>
- 试点意向率 / 演示转试用率 / 首任务完成率 / 复用意愿率 / 可转任务密度：<如适用>

三、结论
- 继续做：<哪些机制/功能/话术证明有效>
- 需要改：<哪些任务或流程导致卡点>
- 不继续做：<本轮被证伪或暂缓的范围>

四、风险
- P0：<需郝敬确认事项；无则“暂无”>
- P1：<影响下游的关键阻塞>
- P2/P3：<过程改进>

五、下一步
- 新增 Kanban 任务建议：<title / assignee / parent / 验收标准>
- 下一批次建议：<目标、边界、质量门>
- 需郝敬确认：<事项 + 推荐选项 + 不确认的影响>
```

### 5.3 单个试点用户记录模板

```text
试点记录
- pilot_id：pilot-YYYYMMDD-XX
- ICP：A 超级个体 / B 交付团队 / 其他
- 触达来源：熟人 / 弱关系 / 社区 / 其他
- 线索评分：<0-12>
- 演示任务：<低敏任务一句话>
- 关联 task_id：<如有>
- 演示结果：成功 / 部分成功 / 失败 / 未演示
- 首任务完成：是 / 否
- 用户最有感能力：<原话，脱敏>
- 最大困惑或阻塞：<原话，脱敏>
- 复用意愿：愿意 / 有条件 / 不愿意 / 未知
- 可转 Kanban 任务：<标题 + 建议 assignee + 原因>
- 高风险事项：报价 / 合同 / 交付周期 / 隐私 / 生产接入 / 公开案例授权；如有写“需郝敬确认”
```

## 6. 首轮 3-7 天复盘节奏建议

| 时间点 | BA/PM 动作 | 输入 | 输出 | 是否需要郝敬确认 |
|---|---|---|---|---|
| D0 启动后 | 建立任务清单和指标基线 | 当前 Kanban task 图、飞书开始通报 | 基线：本批次任务数、应通报任务数、依赖关系 | 否 |
| 每个半日 checkpoint | 更新任务执行指标 | Kanban status、summary、block reason、飞书通报 | PM 每日群内汇总；标红 P0/P1 | 仅 P0/P1 需要 |
| D2/D3 演示准入前 | 检查质量门与试点准备度 | 工程/QA summary、客户成功脚本、商业试点记录 | 是否 demo_ready/internal_ready/not_recommended | 若涉及对外演示口径、客户承诺，需要 |
| 每次试点演示后 24 小时内 | 录入单个试点记录 | 反馈原话、关联 task_id、演示结果 | pilot_records；可转任务建议 | 若涉及报价/合同/公开案例/隐私，需要 |
| D3 PM 首轮闭环 | 做第一版批次复盘 | D0-D3 完成/阻塞任务、质量门、试点信号 | 批次复盘：继续/修改/停止；下一批 task 建议 | 若决定对外口径或资源取舍，需要 |
| D6/D7 总复盘 | 做 3-7 天结论 | 全量任务、试点反馈、质量门、风险 | 是否进入下一批、放大试点、补工程/QA/客成任务 | 通常需要郝敬确认下一批方向 |

## 7. PM 可直接复制的红黄绿判断

```text
红色：任务完成率 <40%，或出现未关闭 P0，或 QA not_recommended，或飞书/Kanban 记录不足以复盘。
黄色：任务完成率 40%-69%，或阻塞率 >25%，或试点有兴趣但首任务/复用信号不足，或 metadata/飞书通报需要补正。
绿色：任务完成率 >=70%，无未关闭 P0，质量门至少 demo_ready，summary/metadata 可复盘，且出现明确试点价值信号或可转任务反馈。
```

注意：绿色不等于可以对外商业承诺；任何报价、合同、交付周期、客户名称公开、生产环境接入，仍需郝敬确认。

## 8. 本任务验收映射

| 验收项 | 本文对应位置 |
|---|---|
| 最小指标表：任务完成率、阻塞率、验证通过率、飞书通报率、试点反馈信号等 | 第 2 节 |
| 每日/每批次复盘模板：事实、结论、风险、下一步、需郝敬确认项 | 第 5 节 |
| Kanban task metadata/summary 建议字段 | 第 4 节 |
| 首轮试点复盘节奏建议 | 第 6 节 |
| 不依赖复杂新系统，优先使用 Kanban summary/metadata 和飞书通报 | 第 1、2、3、4 节 |
| 明确如何处理 `[FEISHU_UPDATE]` 字段 | 第 3 节 |
| 可被 PM 直接复制到项目群日/周汇总 | 第 5、7 节 |
