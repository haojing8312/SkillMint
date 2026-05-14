# WorkClaw MVP Batch A 质量验收与演示准入结论

**日期：** 2026-05-12
**Kanban task：** `t_2dfa2ad5`
**依赖工程任务：** `t_4bfa4c43`
**依赖产品边界：** `t_813defc2` / `docs/plans/2026-05-12-workclaw-3-7-day-mvp-product-closed-loop.md`

## 1. 质量门结论

**结论：有条件可演示 / 可内测，不建议直接进入外部试点或可交付。**

Batch A 是一个前端只读可见性切片：员工详情页新增「AI 员工运行时状态」面板，聚合 Profile Home、Memory OS、Skill OS、Toolsets、Growth、Curator 证据。代码级、类型检查、聚焦回归、生产构建均通过；未发现新增 sidecar endpoint、OpenClaw 兼容目标、`employee_id + skill_id` 新记忆身份中心或密钥泄露。

限制：当前环境没有完成模型/API Key 配置，真实桌面端完整 MVP smoke（首页 -> 单专家任务 -> 结果 -> 员工中心 -> 团队任务 -> 最近运行）无法闭环执行。本批次只能支持工程/产品演示中的「员工中心 profile runtime 状态可见」片段，不足以证明完整 3-7 天 MVP 对外试点闭环。

## 2. 测试范围

### 父任务实际变更文件

- `apps/runtime/src/components/employees/tools/EmployeeProfileRuntimeStatusPanel.tsx`
- `apps/runtime/src/components/employees/employee-details/EmployeeHubEmployeesSection.tsx`
- `apps/runtime/src/components/employees/__tests__/EmployeeHubView.memory-governance.test.tsx`
- `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`

### 覆盖的产品验收项

- 相关覆盖：验收项 3「员工选择 / Profile Home 状态展示」中的员工中心状态可见部分。
- 相关覆盖：验收项 6「结果沉淀」中 profile 变更后可在 Profile Home / Growth / Skill OS / Curator 查看证据的展示部分。
- 完整通过：验收项 7「架构边界验收」。
- 部分覆盖：验收项 5「进度可见性」仅通过既有 `TaskTabStrip` / `TaskJourneySummary` 聚焦测试回归，Batch A 没有改动完整运行状态链路。
- 未覆盖：验收项 1、2、4、8 的完整桌面人工路径，需要模型配置和可运行 runtime session 后补测。

## 3. 实际执行的验证命令与结果

```bash
corepack pnpm --dir apps/runtime exec tsc --noEmit
```

结果：通过。

```bash
corepack pnpm --dir apps/runtime exec vitest run \
  src/components/employees/__tests__/EmployeeHubView.memory-governance.test.tsx \
  src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx
```

结果：2 个测试文件通过，4 个测试通过。

```bash
corepack pnpm --dir apps/runtime exec vitest run \
  src/components/__tests__/NewSessionLanding.test.tsx \
  src/scenes/employees/__tests__/EmployeeHubScene.test.tsx \
  src/components/employees/__tests__/EmployeeHubView.employee-creator-entry.test.tsx \
  src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx \
  src/components/__tests__/TaskTabStrip.test.tsx \
  src/components/__tests__/TaskJourneySummary.test.tsx
```

结果：6 个测试文件通过，32 个测试通过。

```bash
git diff --check
git diff --no-index --check /dev/null apps/runtime/src/components/employees/tools/EmployeeProfileRuntimeStatusPanel.tsx
```

结果：通过，无 whitespace error。

```bash
python3 - <<'PY'
# added-line scan: sidecar/OpenClaw/legacy memory identity/secrets
PY
```

结果：扫描新增行 148 行；`sidecar_endpoint_or_url=0`、`opencLaw_product_goal=0`、`legacy_memory_identity=0`、`secrets=0`。

```bash
corepack pnpm --dir apps/runtime exec vite build
```

结果：通过。3551 modules transformed；生成 `dist/index.html`、CSS、JS、logo 资源。提示 JS chunk 大于 500KB，这是既有构建体积警告，不是本批次阻塞。

### PM 补充复验（2026-05-14）

```bash
cd apps/runtime && ./node_modules/.bin/vitest run \
  src/components/__tests__/Sidebar.settings-active-state.test.tsx \
  src/scenes/__tests__/browser-tauri-fallback.test.tsx \
  src/__tests__/App.model-setup-hint.test.tsx \
  src/components/employees/__tests__/EmployeeHubView.memory-governance.test.tsx \
  --pool=forks --poolOptions.forks.singleFork=true
```

结果：通过。4 个测试文件通过，34 个测试通过。

```bash
cd apps/runtime && ./node_modules/.bin/tsc --noEmit
```

结果：通过。

```bash
cd apps/runtime && node ./node_modules/vite/bin/vite.js build
```

结果：通过。3552 modules transformed；仍有既有 large chunk warning。

```bash
git diff --check
```

结果：通过。仅有 CRLF 将替换为 LF 的提示，无 whitespace error。

补充风险扫描：新增行未发现 secret-like 字段、新 sidecar endpoint、OpenClaw 兼容目标；文档中出现的 `employee_id + skill_id` / `employees/<employee>/openclaw` 均为禁止项说明，不是新增兼容实现。

## 4. 视觉 / Computer Use 验证

执行方式：启动 Vite dev server 后用浏览器打开 `http://127.0.0.1:5174/`。

观察结果：

- 首屏可见 WorkClaw 导航：开始任务、专家技能、智能体员工、设置。
- 当前环境进入「首次使用需要先连接一个大模型」配置拦截页。
- 可见「快速配置（1分钟）」与服务商模板列表。
- 由于未配置模型/API Key，点击「智能体员工」仍停留在配置拦截页，无法进入员工详情页做真实视觉确认。
- 浏览器可视化未发现首屏布局错位、遮挡、文字重叠；但浏览器 console 出现 6 条空 message exception，未能定位到具体堆栈，建议后续真实 Tauri 窗口 smoke 时复查。

截图证据：`/root/.hermes/profiles/quality-test-architect/cache/screenshots/browser_screenshot_187092178e0a4d068af9bc203720b950.png`

替代证据：`EmployeeHubView.memory-governance.test.tsx` 已断言员工详情页面板展示 `AI 员工运行时状态`、`profile-sales`、`Profile Memory 可用`、`2 个技能授权`、`memory · web`、`3 条成长证据`、`1 份 Curator 报告` 与 `Canonical Profile Runtime`。

## 5. 阻塞问题清单

### B1：完整 MVP 桌面 smoke 未执行

- **Severity：High / 演示准入风险**
- **模块：** 产品验收项 8；桌面人工路径。
- **证据：** 当前浏览器运行环境停在模型配置拦截页，无法进入员工中心真实页面，也无法创建 runtime session。
- **影响：** 不能证明完整 3-7 天 MVP 闭环可对外试点。
- **建议 owner：** delivery-project-manager + technical-lead-agent + quality-test-architect。
- **建议：** 在已配置可用模型的本机 Tauri 环境补跑：打开首页 -> 选场景 -> 单专家任务 -> 查看进度 -> 查看结果 -> 进入员工中心 -> 默认团队任务 -> 最近运行。

### B2：首屏浏览器 console 出现空 exception，需要真实窗口复查

- **Severity：Medium / 调试与稳定性风险**
- **模块：** Vite browser smoke / Tauri window control fallback。
- **证据：** `browser_console(clear=true)` 返回 6 条 `source=exception`、`message=""`。
- **影响：** 未见 UI 破损，且 tsc/vitest/build 均通过；但演示前应确认真实 Tauri 窗口无静默异常。
- **建议 owner：** technical-lead-agent。
- **建议：** 用 Tauri dev 或正式桌面包复现，并在 console 捕获具体 stack；如仅为浏览器环境触发的 window control/Tauri API fallback，可降级为非阻塞。

## 6. 非阻塞观察

- 新面板是只读汇总，不新增后端 API、sidecar endpoint 或数据写入；符合 Batch A 的最小工程边界。
- `skillCount` 直接使用授权技能 ID 数量；若授权技能不在 Skill OS index 中，Toolsets 卡片可能显示「未声明」。这是可接受的保守展示，但后续可补充 unknown skill 的提示文案。
- 生产构建存在大 chunk 警告，非本批次引入的功能性阻塞。

## 7. 放行判断

- **工程合入质量：通过。** 可进入后续质量门或内部演示准备。
- **内部演示：有条件通过。** 演示脚本应限定为员工中心 profile runtime 状态面板与架构边界说明；演示前需使用已配置模型的本机环境打开员工中心做一次人工确认。
- **外部试点 / 可交付：不建议放行。** 原因是完整 MVP smoke 和真实任务闭环未执行，本批次只证明「状态可见性切片」而不是完整 MVP 用户价值闭环。
