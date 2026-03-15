# Approval Bus OpenClaw Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 WorkClaw 引入参考 OpenClaw 的通用审批总线，让桌面与飞书共享同一套高风险审批、持久化恢复与自动续跑能力。

**Architecture:** 在 runtime 内新增 `ApprovalManager` 作为统一审批域对象，配套 `approvals` / `approval_rules` 持久化表、新的 session run 状态与事件、桌面审批队列 UI、飞书 `/approve` 与卡片入口，并在 `executor` 中用审批协议替换现有的 `tool-confirm-event + mpsc<bool>` 等待逻辑。实现顺序遵循 TDD，先打通后端持久化和桌面审批，再接飞书，再接长期规则。

**Tech Stack:** Rust, Tauri runtime, sqlx/sqlite, serde/serde_json, React, Vitest, cargo test, pnpm test

---

### Task 1: Approval Persistence And Run Projection Contract

**Files:**
- Create: `apps/runtime/src-tauri/tests/test_approval_bus.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Modify: `apps/runtime/src-tauri/src/session_journal.rs`
- Modify: `apps/runtime/src-tauri/src/commands/session_runs.rs`

**Step 1: Write the failing test**

在 `test_approval_bus.rs` 中新增数据库与 journal 投影测试，断言：
- 新建 pending approval 后会写入 `approvals`
- run 状态投影为 `waiting_approval`
- session run event / session journal 包含 `approval_requested`

**Step 2: Run test to verify it fails**

Run: `cargo test approval_records_persist_and_project_waiting_status --test test_approval_bus -- --exact`

Expected: FAIL，因为当前没有 `approvals` 表、没有 `waiting_approval` 状态，也没有审批事件类型。

**Step 3: Write minimal implementation**

最小实现以下内容：
- 在 `db.rs` 中新增 `approvals` 表
- 在 `session_journal.rs` 中新增 `SessionRunStatus::WaitingApproval`
- 在 `SessionRunEvent` 与 `append_session_run_event_with_pool` 中加入审批相关事件与状态投影

**Step 4: Run test to verify it passes**

Run: `cargo test approval_records_persist_and_project_waiting_status --test test_approval_bus -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/src/session_journal.rs apps/runtime/src-tauri/src/commands/session_runs.rs apps/runtime/src-tauri/tests/test_approval_bus.rs
git commit -m "feat: add approval persistence and waiting status"
```

### Task 2: Approval Manager Core And App State

**Files:**
- Create: `apps/runtime/src-tauri/src/approval_bus.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/tests/test_approval_bus.rs`

**Step 1: Write the failing test**

新增 manager 级测试，断言：
- 创建 approval 会返回稳定 `approvalId`
- 同一条 approval 只能被首个 resolver 成功终态化
- 第二个并发 resolver 会得到“已处理”结果

**Step 2: Run test to verify it fails**

Run: `cargo test approval_manager_allows_first_resolver_only --test test_approval_bus -- --exact`

Expected: FAIL，因为当前不存在 `ApprovalManager` 或审批状态 CAS 逻辑。

**Step 3: Write minimal implementation**

实现 `approval_bus.rs`，包含：
- `ApprovalRecord`
- `ApprovalDecision`
- `ApprovalManager`
- 内存 waiter map
- 基于数据库终态更新的 CAS 解析流程

并在 `chat.rs` / `lib.rs` 中注册新的 app state，保留旧 `ToolConfirmState` 仅作为迁移兼容壳。

**Step 4: Run test to verify it passes**

Run: `cargo test approval_manager_allows_first_resolver_only --test test_approval_bus -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/approval_bus.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_approval_bus.rs
git commit -m "feat: add approval manager core"
```

### Task 3: Executor Integration And New Approval Commands

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/approvals.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_control.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/tests/test_approval_bus.rs`
- Modify: `apps/runtime/src-tauri/tests/test_file_delete.rs`

**Step 1: Write the failing test**

新增集成测试，断言：
- 高风险 `file_delete` 命中审批总线时进入 pending
- 在 `resolve_approval(allow_once)` 之前不会执行工具
- 获批后会继续执行原工具并完成 run

**Step 2: Run test to verify it fails**

Run: `cargo test approval_pending_blocks_tool_until_resolved --test test_approval_bus -- --exact`

Expected: FAIL，因为当前 `executor` 仍依赖 `tool-confirm-event + mpsc<bool>` 和 15 秒超时。

**Step 3: Write minimal implementation**

完成以下替换：
- 在 `executor.rs` 中引入 `request_approval_and_wait`
- 让高风险工具调用先创建 approval，再等待 manager 恢复
- 新增 `resolve_approval` / `list_pending_approvals` 命令
- 将 `confirm_tool_execution` 收敛为兼容包装或直接迁移走

**Step 4: Run test to verify it passes**

Run: `cargo test approval_pending_blocks_tool_until_resolved --test test_approval_bus -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/approvals.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/commands/chat_control.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_approval_bus.rs apps/runtime/src-tauri/tests/test_file_delete.rs
git commit -m "feat: route critical tool calls through approval bus"
```

### Task 4: Desktop Approval Queue And Resolve Flow

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/RiskConfirmDialog.tsx`
- Modify: `apps/runtime/src/components/__tests__/ChatView.risk-flow.test.tsx`
- Create: `apps/runtime/src/components/__tests__/RiskConfirmDialog.test.tsx`

**Step 1: Write the failing test**

在前端测试中新增断言：
- 收到 `approval-created` 事件后展示审批卡
- 点击 `允许一次 / 始终允许 / 拒绝` 会调用 `resolve_approval`
- 飞书已处理时桌面卡片会切换为已处理状态

**Step 2: Run test to verify it fails**

Run: `pnpm --filter runtime test -- ChatView.risk-flow.test.tsx`

Expected: FAIL，因为当前前端只监听 `tool-confirm-event`，并且只会调用 `confirm_tool_execution({ confirmed })`。

**Step 3: Write minimal implementation**

最小改动包括：
- `ChatView` 改为维护 pending approval 列表
- 将旧 `toolConfirm` 状态迁移到审批事件流
- `RiskConfirmDialog` 支持三种决策按钮和处理中状态
- 组件卸载时不再自动发送“拒绝”，而是仅清理本地订阅

**Step 4: Run test to verify it passes**

Run: `pnpm --filter runtime test -- ChatView.risk-flow.test.tsx`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/RiskConfirmDialog.tsx apps/runtime/src/components/__tests__/ChatView.risk-flow.test.tsx apps/runtime/src/components/__tests__/RiskConfirmDialog.test.tsx
git commit -m "feat: add desktop approval queue flow"
```

### Task 5: Feishu Notification And Approve Command Flow

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_gateway.rs`
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src-tauri/tests/test_feishu_gateway.rs`
- Modify: `apps/runtime/src-tauri/tests/test_im_runtime_bridge.rs`
- Modify: `apps/runtime/src-tauri/tests/test_feishu_callback_idempotency.rs`

**Step 1: Write the failing test**

新增测试，断言：
- 创建 pending approval 时会向飞书线程发送审批通知
- 飞书文本 `/approve <id> allow_once` 可成功命中 pending approval
- 重复飞书事件不会导致重复审批

**Step 2: Run test to verify it fails**

Run: `cargo test feishu_approve_command_resolves_pending_approval --test test_feishu_gateway -- --exact`

Expected: FAIL，因为当前飞书侧没有审批通知或 `/approve` 解析逻辑。

**Step 3: Write minimal implementation**

完成以下实现：
- 构造审批通知文案与最小消息模板
- 在飞书入站命令中解析 `/approve`
- 将审批决策交给 `ApprovalManager`
- 审批成功后向飞书线程回写结果，并向桌面广播 `approval-resolved`

**Step 4: Run test to verify it passes**

Run: `cargo test feishu_approve_command_resolves_pending_approval --test test_feishu_gateway -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/src/commands/im_gateway.rs apps/runtime/src/App.tsx apps/runtime/src-tauri/tests/test_feishu_gateway.rs apps/runtime/src-tauri/tests/test_im_runtime_bridge.rs apps/runtime/src-tauri/tests/test_feishu_callback_idempotency.rs
git commit -m "feat: add feishu approval surface"
```

### Task 6: Resume, Restart Recovery, And Approval Replay

**Files:**
- Modify: `apps/runtime/src-tauri/src/approval_bus.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/commands/session_runs.rs`
- Modify: `apps/runtime/src-tauri/tests/test_approval_bus.rs`
- Modify: `apps/runtime/src-tauri/tests/test_session_export_recovery.rs`

**Step 1: Write the failing test**

新增恢复测试，断言：
- 应用重启后仍能列出 pending approval
- `status = approved AND resumed_at IS NULL` 的记录会在启动时被补恢复
- 恢复成功后 run 继续流转并更新 `resumed_at`

**Step 2: Run test to verify it fails**

Run: `cargo test approved_pending_work_resumes_after_restart --test test_approval_bus -- --exact`

Expected: FAIL，因为当前审批既不持久化恢复上下文，也没有启动补恢复逻辑。

**Step 3: Write minimal implementation**

最小实现以下能力：
- `resume_payload_json` 的序列化与反序列化
- 启动时扫描 pending / approved-but-not-resumed approvals
- 为失去内存 waiter 的 approval 重新拉起恢复执行

**Step 4: Run test to verify it passes**

Run: `cargo test approved_pending_work_resumes_after_restart --test test_approval_bus -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/approval_bus.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/src/commands/session_runs.rs apps/runtime/src-tauri/tests/test_approval_bus.rs apps/runtime/src-tauri/tests/test_session_export_recovery.rs
git commit -m "feat: recover pending approvals across restarts"
```

### Task 7: Structured Allow-Always Rules

**Files:**
- Create: `apps/runtime/src-tauri/src/approval_rules.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Modify: `apps/runtime/src-tauri/src/approval_bus.rs`
- Modify: `packages/runtime-policy/src/permissions.rs`
- Modify: `apps/runtime/src-tauri/tests/test_approval_bus.rs`
- Modify: `apps/runtime/src-tauri/tests/test_file_delete.rs`
- Modify: `apps/runtime/src-tauri/tests/test_bash.rs`

**Step 1: Write the failing test**

新增规则测试，断言：
- `allow_always` 会创建结构化 `approval_rules`
- 后续匹配相同 `file_delete` / 危险 `bash` 指纹时可直接放行
- 不匹配规则的高风险动作仍会进入审批

**Step 2: Run test to verify it fails**

Run: `cargo test allow_always_creates_reusable_rule_and_skips_reapproval --test test_approval_bus -- --exact`

Expected: FAIL，因为当前没有 `approval_rules` 表或匹配逻辑。

**Step 3: Write minimal implementation**

实现：
- `approval_rules` 表与运行时查询
- 工具指纹 / matcher 生成
- `executor` 中“风险识别后再规则匹配”的自动放行路径

**Step 4: Run test to verify it passes**

Run: `cargo test allow_always_creates_reusable_rule_and_skips_reapproval --test test_approval_bus -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/approval_rules.rs apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/src/approval_bus.rs packages/runtime-policy/src/permissions.rs apps/runtime/src-tauri/tests/test_approval_bus.rs apps/runtime/src-tauri/tests/test_file_delete.rs apps/runtime/src-tauri/tests/test_bash.rs
git commit -m "feat: add reusable approval rules"
```

### Task 8: Verification And Rollout Guardrails

**Files:**
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `docs/integrations/feishu-im-bridge.md`
- Modify: `README.md`

**Step 1: Add rollout flag and docs**

增加 `approval_bus_v1` 特性开关或等价配置入口，并在桥接文档与 README 中补充审批总线说明、桌面与飞书审批边界、降级路径。

**Step 2: Run focused Rust regression suite**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_approval_bus -- --nocapture`

Expected: PASS

**Step 3: Run critical tool regression tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_file_delete -- --nocapture`

Expected: PASS

**Step 4: Run Feishu bridge regression tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_feishu_gateway -- --nocapture`

Expected: PASS

**Step 5: Run desktop approval UI tests**

Run: `pnpm --filter runtime test -- ChatView.risk-flow.test.tsx`

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/lib.rs docs/integrations/feishu-im-bridge.md README.md
git commit -m "docs: document approval bus rollout"
```
