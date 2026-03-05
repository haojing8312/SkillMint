# 多智能体员工协作（主/子调度）Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让飞书群会话形成“主员工接管 -> 子员工委派执行 -> 主员工汇总输出”的稳定闭环，并在桌面端清晰区分主/子员工回复身份。

**Architecture:** 保持“飞书入站 -> Tauri 事件 -> App 会话桥接 -> ChatView 渲染”的现有链路，新增统一消息信封字段、协议化委派事件、澄清消息化与失败兜底；路由仍遵循“未@主员工优先、@命中子员工直达”。

**Tech Stack:** Rust (Tauri + sqlx), React + TypeScript + Vitest, SQLite.

---

### Task 1: 统一消息信封字段（前后端协议对齐）

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Test: `apps/runtime/src-tauri/tests/test_feishu_gateway.rs`

**Step 1: Write the failing test**

在 `test_feishu_gateway.rs` 增加用例，断言派发请求中包含 `task_id`、`sender_employee_id`、`target_employee_id`、`message_type`，且未 @ 时目标为主员工。

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_gateway -- --nocapture`  
Expected: FAIL（字段缺失或断言不匹配）。

**Step 3: Write minimal implementation**

在 `feishu_gateway.rs` 的 dispatch 结构体与 emit payload 中补齐新字段；在 `types.ts` 的 `ImRoleDispatchRequest` / `ImRoleTimelineEvent` 增加对应字段。

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_gateway -- --nocapture`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/tests/test_feishu_gateway.rs
git commit -m "feat(im): align multi-employee dispatch envelope fields"
```

### Task 2: 未@主员工接管 + @子员工直达路由稳定化

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

**Step 1: Write the failing test**

新增两个用例：

1. 群消息未 @ 时命中 `is_default` 员工；
2. 消息 @ 子员工时目标员工命中子员工，且同线程复用同一 session。

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_im_employee_agents --test test_im_route_session_mapping -- --nocapture`  
Expected: FAIL（路由或 session 映射不稳定）。

**Step 3: Write minimal implementation**

在 `ensure_employee_sessions_for_event_with_pool` 与 Feishu mention 解析链路中统一规则：

- 优先：明确 mention 命中员工
- 否则：主员工（`is_default`）
- 均未命中：首个启用员工兜底并写系统告警事件

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_im_employee_agents --test test_im_route_session_mapping -- --nocapture`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/tests/test_im_employee_agents.rs apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs
git commit -m "fix(im): stabilize default-main and mention-direct employee routing"
```

### Task 3: 委派闭环（delegate_request / delegate_result）

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Test: `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx`

**Step 1: Write the failing test**

在 `App.im-feishu-bridge.test.tsx` 增加用例：

- 主员工产生子员工流式输出后，飞书端收到带角色前缀的分段消息；
- 子员工完成后主员工能继续汇总输出。

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx`  
Expected: FAIL（缺少委派状态管理或角色回切）。

**Step 3: Write minimal implementation**

在 `App.tsx` 的 IM bridge context 中新增：

- `currentTaskId`、`parentTaskId`、`delegateTarget`
- 子员工流结束后自动恢复 `primaryRoleName`
- `sendTextToFeishu` 统一采用 `[角色名] 内容` 格式

并在 `ChatView.tsx` 中补充委派事件展示。

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx src/components/__tests__/ChatView.im-routing-panel.test.tsx`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/components/ChatView.tsx apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx
git commit -m "feat(im): close delegate loop from sub-agent execution to main summary"
```

### Task 4: 澄清请求消息化（替代桌面独占弹框）

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Test: `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`

**Step 1: Write the failing test**

新增用例：当运行态触发 ask-user 时，飞书收到澄清问题；用户后续飞书回复可进入 `answer_user_question` 而不是仅桌面弹框。

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx`  
Expected: FAIL（当前链路可能只部分覆盖）。

**Step 3: Write minimal implementation**

在 `App.tsx`：

- 将本地 ask-user 统一转成 `clarify_request` 消息；
- 维护 `waitingForAnswer` 与 `clarifyCorrelationId`；
- 飞书后续消息触发 `answer_user_question` 并关闭等待态。

在 `feishu_gateway.rs` 保证澄清相关事件完整透传。

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/components/ChatView.tsx apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx
git commit -m "fix(im): make clarification flow channel-agnostic for desktop and feishu"
```

### Task 5: 失败兜底（发送失败重试 + 降级说明）

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`

**Step 1: Write the failing test**

新增用例：

- `send_feishu_text_message` 首次失败时进入重试队列；
- 超过重试阈值后在桌面写入系统消息（说明已降级，仅本地继续）。

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx`  
Expected: FAIL（暂无重试或降级系统提示）。

**Step 3: Write minimal implementation**

在 `App.tsx` 增加轻量 outbox：

- `pendingMessages[]`（chatId、text、attempt、nextRetryAt）
- 指数退避重试（如 1s/3s/10s）
- 失败上限后写系统消息并标注 `delivery_status=degraded`

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/types.ts apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx
git commit -m "feat(im): add feishu delivery retry queue and degradation notice"
```

### Task 6: UI 区分主/子员工回复（消息头与委派卡片）

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/Sidebar.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx`
- Test: `apps/runtime/src/components/__tests__/Sidebar.session-source-badge.test.tsx`

**Step 1: Write the failing test**

新增断言：

- Chat 消息出现“主员工/子员工”标签；
- 委派卡片显示“项目经理 -> 开发团队”与状态；
- 会话若存在委派，侧栏显示团队协作标记。

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.im-routing-panel.test.tsx src/components/__tests__/Sidebar.session-source-badge.test.tsx`  
Expected: FAIL。

**Step 3: Write minimal implementation**

在 `ChatView.tsx` 为 IM 消息新增 role badge 渲染；在 `Sidebar.tsx` 增加“团队会话”轻标记（不新增复杂入口）。

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.im-routing-panel.test.tsx src/components/__tests__/Sidebar.session-source-badge.test.tsx`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/Sidebar.tsx apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx apps/runtime/src/components/__tests__/Sidebar.session-source-badge.test.tsx
git commit -m "feat(ui): distinguish main/sub employee replies and delegation cards"
```

### Task 7: 端到端回归（Rust + Frontend）

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_multi_role_e2e.rs`
- Modify: `apps/runtime/src-tauri/tests/test_feishu_gateway.rs`
- Modify: `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`

**Step 1: Write the failing test**

补充端到端回归用例覆盖：

1. 未@主员工接管并委派子员工
2. @子员工直达
3. 澄清请求与回复跨端闭环

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_im_multi_role_e2e --test test_feishu_gateway -- --nocapture`  
Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx`  
Expected: 至少一个 FAIL。

**Step 3: Write minimal implementation**

按失败点补齐字段透传、状态回切、容错路径，确保测试全部转绿。

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_im_multi_role_e2e --test test_feishu_gateway -- --nocapture`  
Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx src/components/__tests__/ChatView.im-routing-panel.test.tsx`  
Expected: PASS。

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_im_multi_role_e2e.rs apps/runtime/src-tauri/tests/test_feishu_gateway.rs apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx
git commit -m "test(im): add multi-employee orchestration end-to-end regressions"
```

### Task 8: 文档与验收清单

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Create: `docs/plans/2026-03-05-multi-employee-orchestration-acceptance.md`

**Step 1: Write the failing test**

本任务无自动化测试；采用人工验收 checklist。

**Step 2: Run verification**

执行以下命令确保主链路无回归：

Run: `cd apps/runtime/src-tauri && cargo test --test test_im_employee_agents --test test_im_route_session_mapping --test test_feishu_gateway --test test_im_multi_role_e2e -- --nocapture`  
Run: `pnpm --dir apps/runtime exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx src/components/__tests__/ChatView.im-routing-panel.test.tsx src/components/__tests__/Sidebar.session-source-badge.test.tsx`  
Expected: PASS。

**Step 3: Write docs**

- 更新 README 的“飞书群协作”说明（主员工接管、@子员工、澄清闭环、身份标签）
- 新增验收文档记录手工测试脚本与预期结果

**Step 4: Commit**

```bash
git add README.md README.zh-CN.md docs/plans/2026-03-05-multi-employee-orchestration-acceptance.md
git commit -m "docs(im): document multi-employee orchestration and acceptance checklist"
```
