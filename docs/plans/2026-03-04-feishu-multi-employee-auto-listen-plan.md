# Feishu Multi-Employee Auto Listen Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在保存智能体员工后自动维护“每员工一条飞书长连接”，在员工页展示红绿灰连接状态，并确保飞书消息自动创建/复用桌面会话并执行。

**Architecture:** 采用“Sidecar 多连接管理 + Rust 期望态对齐与健康监督 + 前端状态轮询”三层方案。Sidecar 负责连接生命周期与事件归属；Rust 负责依据员工配置对齐连接并维持 relay；前端只消费状态并可视化。

**Tech Stack:** Tauri (Rust + sqlx + tokio), Node sidecar (TypeScript + Hono + @larksuiteoapi/node-sdk), React + Vitest + Testing Library.

---

### Task 1: Sidecar 多连接管理器（先写失败测试）

**Files:**
- Create: `apps/runtime/sidecar/test/feishu.ws-multi-connection.test.ts`
- Modify: `apps/runtime/sidecar/src/feishu_ws.ts`

**Step 1: Write the failing tests**

```ts
import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { FeishuLongConnectionManager } from "../src/feishu_ws.js";

describe("FeishuLongConnectionManager multi connection", () => {
  test("reconcile starts two employees without overriding each other", () => {
    const m = new FeishuLongConnectionManager();
    const status = m.reconcile([
      { employee_id: "pm", app_id: "cli_pm", app_secret: "sec_pm" },
      { employee_id: "tech", app_id: "cli_tech", app_secret: "sec_tech" },
    ]);
    assert.equal(status.items.length, 2);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/sidecar test`  
Expected: fail with missing `reconcile`/multi-item status API.

**Step 3: Write minimal implementation**

- 在 `FeishuLongConnectionManager` 中引入 `Map<employee_id, connection>`。
- 增加 `reconcile(...)`、`statusAll()`、`drainAll(...)`。
- `FeishuWsEventRecord` 增加 `employee_id` 字段。

**Step 4: Run tests to verify pass**

Run: `pnpm --dir apps/runtime/sidecar test`  
Expected: 新增测试通过，现有 sidecar 测试保持通过。

**Step 5: Commit**

```bash
git add apps/runtime/sidecar/src/feishu_ws.ts apps/runtime/sidecar/test/feishu.ws-multi-connection.test.ts
git commit -m "test(sidecar): add multi-employee feishu ws manager coverage"
```

### Task 2: Sidecar Feishu WS API 扩展（reconcile/status/drain）

**Files:**
- Create: `apps/runtime/sidecar/test/feishu.ws-api.test.ts`
- Modify: `apps/runtime/sidecar/src/index.ts`

**Step 1: Write the failing API tests**

```ts
test("POST /api/feishu/ws/reconcile returns per-employee statuses", async () => {
  // call app.request(...) and assert two items
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime/sidecar test`  
Expected: endpoint not found or response schema mismatch.

**Step 3: Write minimal implementation**

- 新增 `POST /api/feishu/ws/reconcile`。
- `POST /api/feishu/ws/status` 返回全量 `items`。
- `POST /api/feishu/ws/drain-events` 返回含 `employee_id` 的事件列表。
- 旧 `/start|/stop|/status` 兼容保留（内部可转调新管理器）。

**Step 4: Run tests**

Run: `pnpm --dir apps/runtime/sidecar test`  
Expected: API 测试与既有测试均通过。

**Step 5: Commit**

```bash
git add apps/runtime/sidecar/src/index.ts apps/runtime/sidecar/test/feishu.ws-api.test.ts
git commit -m "feat(sidecar): expose feishu ws reconcile and per-employee status endpoints"
```

### Task 3: Rust 侧对齐逻辑（先写失败测试）

**Files:**
- Create: `apps/runtime/src-tauri/tests/test_feishu_multi_employee_connections.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn collect_enabled_employee_credentials_returns_all_bound_employees() {
    // seed 2 enabled employees with app_id/secret
    // assert helper returns 2 items
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_multi_employee_connections -- --nocapture`  
Expected: helper/struct/command does not exist yet.

**Step 3: Write minimal implementation**

- 在 `feishu_gateway.rs` 增加：
  - 员工凭据收集 helper（仅 enabled 且凭据非空）
  - sidecar reconcile payload/response struct
  - `reconcile_feishu_employee_connections_with_pool(...)`
- `FeishuWsEventRecord` 增加 `employee_id` 字段并在 `sync_feishu_ws_events_core` 赋值到 `ImEvent.role_id`。

**Step 4: Run tests**

Run:
- `cd apps/runtime/src-tauri && cargo test --test test_feishu_multi_employee_connections -- --nocapture`
- `cd apps/runtime/src-tauri && cargo test --test test_feishu_gateway -- --nocapture`

Expected: 新老测试通过。

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/tests/test_feishu_multi_employee_connections.rs
git commit -m "feat(runtime): reconcile per-employee feishu ws connections"
```

### Task 4: 员工保存/删除触发自动监听 + 启动恢复

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs` (if needed for exports)

**Step 1: Write failing tests**

优先在已有测试补断言：
- `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`

新增断言目标：
- upsert/delete 后会调用连接对齐入口（可通过提取 pure helper 或状态变更验证）。

**Step 2: Run tests to verify red**

Run: `cd apps/runtime/src-tauri && cargo test --test test_im_employee_agents -- --nocapture`  
Expected: 新断言失败。

**Step 3: Implement**

- `upsert_agent_employee`/`delete_agent_employee` 成功后触发 reconcile（失败不回滚 DB）。
- `lib.rs` 启动流程改为“全员工对齐 + relay 启动”，替代“单凭据恢复”。
- 增加健康监督循环（检查连接状态与 relay，指数退避重启）。

**Step 4: Run tests**

Run:
- `cd apps/runtime/src-tauri && cargo test --test test_im_employee_agents -- --nocapture`
- `cd apps/runtime/src-tauri && cargo test --test test_im_route_session_mapping -- --nocapture`

Expected: 全部通过。

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_im_employee_agents.rs
git commit -m "feat(runtime): auto-reconcile feishu ws on employee save/delete and boot"
```

### Task 5: 前端红绿灰状态点（先写失败测试）

**Files:**
- Create: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx`
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`

**Step 1: Write failing UI test**

```tsx
it("shows green dot when employee ws running and relay running", async () => {
  // mock invoke("get_feishu_employee_connection_statuses")
  // assert green marker appears for employee row
});
```

**Step 2: Run test to verify red**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView.feishu-connection-status`  
Expected: fail because status state/rendering not implemented.

**Step 3: Implement**

- 在 `types.ts` 增加连接状态 DTO。
- `EmployeeHubView` 中轮询状态。
- 员工列表渲染红绿灰点与错误提示文案。

**Step 4: Run tests**

Run:
- `pnpm --dir apps/runtime test -- EmployeeHubView.feishu-connection-status`
- `pnpm --dir apps/runtime test -- EmployeeHubView.employee-id-flow`

Expected: 新老测试通过。

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx
git commit -m "feat(ui): add per-employee feishu connection status indicators"
```

### Task 6: 端到端回归与验收

**Files:**
- Modify (if needed): `apps/runtime/src-tauri/tests/test_feishu_gateway.rs`
- Modify (if needed): `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`
- Update docs: `README.zh-CN.md` (连接状态与自动监听说明)

**Step 1: Add/adjust regression assertions**

- 多员工凭据场景下，事件能绑定到正确员工会话。
- relay 运行时触发 `im-role-dispatch-request` 路径不回归。

**Step 2: Run full verification**

Run:
- `pnpm --dir apps/runtime/sidecar test`
- `cd apps/runtime/src-tauri && cargo test --test test_feishu_gateway -- --nocapture`
- `cd apps/runtime/src-tauri && cargo test --test test_im_employee_agents -- --nocapture`
- `pnpm --dir apps/runtime test --passWithNoTests`

Expected: 全部通过。

**Step 3: Manual smoke**

- 启动应用并在员工页创建 2 个绑定飞书的员工。
- 观察员工列表状态点由灰/红转绿。
- 从飞书向其中一个员工发送消息，确认桌面侧自动出现/复用会话并执行。

**Step 4: Commit**

```bash
git add README.zh-CN.md apps/runtime/src-tauri/tests/test_feishu_gateway.rs apps/runtime/src-tauri/tests/test_im_employee_agents.rs
git commit -m "docs+test: cover feishu multi-employee auto-listen flow"
```

### Task 7: 最终收口

**Files:**
- `docs/plans/2026-03-04-feishu-multi-employee-auto-listen-design.md`
- `docs/plans/2026-03-04-feishu-multi-employee-auto-listen-plan.md`

**Step 1: Re-run required verification commands**

Run:
- `pnpm --dir apps/runtime/sidecar test`
- `cd apps/runtime/src-tauri && cargo test --test test_feishu_gateway -- --nocapture`
- `pnpm --dir apps/runtime test --passWithNoTests`

**Step 2: Check git diff scope**

Run: `git status --short`

Expected: 仅包含本需求相关改动。

**Step 3: Final commit**

```bash
git add .
git commit -m "feat(feishu): multi-employee auto-listen with health supervision and ui status"
```
