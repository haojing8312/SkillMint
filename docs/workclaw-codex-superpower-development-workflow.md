# WorkClaw Codex + Superpower 开发协作流程

**日期：** 2026-05-14
**适用范围：** WorkClaw 后续需要 Codex/技术负责人执行的功能、修复、重构、验证与工程收口任务。
**目标：** 把“PM 拆解 + Codex 执行 + PM/QA 验收”升级为 Superpower-style 的规格、计划、TDD、系统调试、独立审查和质量门流水线。

## 1. 结论

从下一批工程开发开始，WorkClaw 不再把 Codex 当作“直接改代码的黑盒执行器”。Codex 仍然负责一线代码实现，但每次调用 Codex 前后必须套入对应 Superpower-style skill：

- 先用 `writing-plans` / `plan` 明确规格、任务、文件范围和验证命令。
- 新功能、行为变化、Bug 修复默认走 `test-driven-development`。
- 不确定方案先走 `spike`，不要直接进入生产代码。
- Bug / 测试失败 / 构建失败必须走 `systematic-debugging`，先找根因再修。
- Codex 完成后必须走 `requesting-code-review` 维度的独立复核，再由 QA/PM 做 WorkClaw repo-local 验证。
- 多任务并行或需要新鲜上下文时，使用 `subagent-driven-development` 的“实现者 + 规格审查 + 质量审查”模式。

## 2. 可用 Superpower-style 能力映射

当前 Hermes 环境中可用、且适合 WorkClaw 的 Superpower/GSD 风格技能如下：

| Skill | 适用阶段 | 核心作用 | 默认责任人 |
|---|---|---|---|
| `plan` | 只要用户要求先出方案、不执行代码 | 写 `.hermes/plans/*` 计划，不改生产代码 | `workclaw-pm` / `technical-lead-agent` |
| `writing-plans` | 多步骤功能、重构、复杂修复前 | 产出可执行实现计划：目标、架构、文件、任务、测试、验收 | `technical-lead-agent`，PM 复核 |
| `spike` | 技术不确定、方案分歧、外部库/协议验证前 | 做一次性验证，输出 VALIDATED / PARTIAL / INVALIDATED | `agent-systems-architect` / `technical-lead-agent` |
| `test-driven-development` | 新功能、Bug 修复、行为变化、重构 | RED-GREEN-REFACTOR：先失败测试，再实现 | `technical-lead-agent` 指挥 Codex 执行，QA 复查 |
| `systematic-debugging` | Bug、测试失败、构建失败、线上异常 | 四阶段根因分析，禁止随机试修 | `technical-lead-agent`，必要时 `agent-systems-architect` |
| `requesting-code-review` | 提交前、推送前、质量门前 | 静态风险扫描、diff 审查、独立 reviewer、auto-fix loop | `quality-test-architect` / PM |
| `subagent-driven-development` | 任务多、风险高、需要新鲜上下文审查 | 每个任务独立实现者 + spec reviewer + quality reviewer | PM 编排，技术负责人执行 |
| `codex` | 具体编码实现、修复、重构、测试补齐 | 一线 coding agent，只按上游计划和 skill 约束执行 | `technical-lead-agent` 调用 |

## 3. 标准阶段门

### Gate 0：任务接收与分型

PM 先判断任务类型：

| 任务类型 | 必须使用的 skill |
|---|---|
| 新功能 / 行为变化 | `writing-plans` + `test-driven-development` + `codex` + `requesting-code-review` |
| Bug 修复 / 测试失败 | `systematic-debugging` + `test-driven-development` + `codex` + `requesting-code-review` |
| 大重构 / 架构迁移 | `writing-plans` + `spike`（如有不确定）+ `subagent-driven-development` + `codex` |
| 不确定技术方案 | `spike`，验证通过后再写实现计划 |
| 文档/流程方案 | `plan` 或 `writing-plans`，通常不需要 Codex |
| 提交/推送/放行 | `requesting-code-review` + WorkClaw repo-local verification |

### Gate 1：规格与计划

进入 Codex 前必须给出可执行计划，至少包含：

1. 目标 / 非目标。
2. roadmap 对齐项，尤其是 self-improving profile runtime 和 sidecar removal 边界。
3. 预期改动文件范围。
4. 测试策略：需要先写哪些失败测试、要跑哪些 focused 命令。
5. 验收标准：功能、架构边界、回归、风险。
6. 如果不可 TDD，必须说明原因：例如纯文档、纯配置、一次性调研。

### Gate 2：Codex 执行

技术负责人调用 Codex 时，prompt 必须显式包含：

```text
你正在执行 WorkClaw 工程任务。必须遵守：
1. WorkClaw AGENTS.md 和 repo-local workflow/roadmap 规则。
2. Superpower-style workflow：
   - 若是新功能/行为变化/Bug 修复，先写或确认失败测试，再实现。
   - 若遇到失败，不要随机试修；使用 systematic-debugging：复现、读错误、找根因、最小修复。
   - 不做计划外重构，不扩大文件范围。
3. 目标 / 非目标：...
4. 预期文件范围：...
5. 必跑验证命令：...
6. 完成后返回：改动摘要、TDD/调试证据、验证结果、遗留风险。
```

Codex 不能直接决定“完成”；它只能交付候选实现和证据。

### Gate 3：独立审查

Codex 完成后，由 PM/QA/独立 reviewer 做 `requesting-code-review` 维度检查：

- diff 是否符合计划。
- 是否新增 secret、凭据、危险命令、sidecar endpoint、OpenClaw 兼容目标。
- 是否有测试覆盖。
- 是否跑过必需验证。
- 是否有 scope creep 或临时文件。

### Gate 4：WorkClaw 质量门

根据改动面运行 WorkClaw 验证：

- 前端：focused vitest、`tsc --noEmit`、必要时 `vite build`。
- Rust/Tauri：`pnpm test:rust-fast` 或更小范围 Rust check/test。
- Builtin skills：`pnpm test:builtin-skills`。
- Release-sensitive：release/installer/vendor lane checks。
- 人工 smoke：只有在具备可用模型/Tauri 环境时执行；否则明确 blocked。

### Gate 5：提交与复盘

PM 只能在以下条件满足后提交/推送：

1. Codex 输出已被复核，不直接信任。
2. 必要测试通过。
3. 风险和未覆盖项写入文档或 Kanban。
4. Kanban task 有 owner、结果、阻塞状态。
5. commit message 包含主要改动和验证摘要。

## 4. 员工分工

### `workclaw-pm`

- 使用 `kanban-orchestrator` 拆任务。
- 使用 `plan` / `writing-plans` 固化目标、验收标准、任务图。
- 指定每个任务必须使用哪些 Superpower-style skill。
- 最后收敛验证结果、Kanban 状态、提交/推送。

### `technical-lead-agent`

- 是 Codex 的主要操作者。
- 开发前必须加载/遵守 `writing-plans`、`test-driven-development`、`systematic-debugging`、`codex`。
- 负责把计划转成 Codex prompt，并限制 Codex 的文件范围和验证命令。
- 不允许“直接让 Codex 改一把”跳过计划和测试。

### `quality-test-architect`

- 使用 `requesting-code-review` 和 WorkClaw change verification 做质量门。
- 验证 TDD 证据、focused tests、类型检查、构建、人工 smoke。
- 有权把任务 block 回技术负责人。

### `agent-systems-architect`

- 用于 profile runtime、toolsets、sidecar removal、gateway、memory/skill OS 等架构边界。
- 高不确定事项先使用 `spike`，再给技术负责人实现边界。

### `product-strategist`

- 只在 MVP 范围、验收口径、用户路径变化时参与。
- 输出验收标准后，PM/技术负责人再用 `writing-plans` 转工程任务。

### `delivery-project-manager`

- 只在本机桌面 smoke、客户现场试点、交付节奏需要时参与。
- 不参与常规代码实现。

## 5. 最小执行模板

后续每个开发 Kanban task body 默认包含：

```text
【必须使用的 workflow/skills】
- writing-plans：先明确任务计划、文件范围、验收标准。
- test-driven-development：新功能/行为变化/Bug 修复必须先有失败测试；如不适用，说明原因。
- systematic-debugging：遇到失败必须先找根因，不允许随机试修。
- codex：只作为编码执行器，按计划执行。
- requesting-code-review：提交前做 diff/安全/质量复核。

【Codex prompt 必须包含】
目标：...
非目标：...
WorkClaw 规则：遵守 AGENTS.md、roadmap、repo-local workflow。
文件范围：...
测试/验证命令：...
完成回报：改动摘要、TDD/调试证据、验证结果、遗留风险。
```

## 6. 例外规则

可以不走完整 Superpower-style 流程的情况：

- 纯错别字、纯 Markdown 排版、无工程影响的小文档更新。
- 只读调研，没有代码或配置变更。
- 用户明确要求跳过某个 gate。

但即便跳过，也必须在 Kanban summary 或 PM 汇报中说明：跳过了什么、为什么、风险是什么。

## 7. 从下一批开始的执行决策

1. 新增/修复类工程任务不再直接派 Codex。
2. PM 先创建/确认 Kanban task，并在 body 写入 Superpower-style skill 要求。
3. 技术负责人先输出计划，再调用 Codex。
4. QA 使用 review + verification gate 验收。
5. PM 只在所有 gate 关闭后提交/推送。

这套流程是 WorkClaw 当前阶段的默认研发协作协议；除非郝敬明确要求临时跳过，否则后续 Codex 开发按此执行。
