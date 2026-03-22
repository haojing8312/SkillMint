# Feishu Runtime Unification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove Feishu’s split receive/send architecture by migrating outbound delivery from sidecar to the official plugin runtime, then delete sidecar’s Feishu-specific responsibilities.

**Architecture:** Keep the current inbound, auth, pairing, and routing model centered on the official Feishu plugin runtime, and replace only the outbound transport first. Once outbound is stable through the same runtime, delete sidecar Feishu endpoints and diagnostics so Feishu has a single runtime boundary.

**Tech Stack:** React runtime UI, Tauri Rust backend, Node `plugin-host`, official `@larksuite/openclaw-lark` runtime, Rust SQLite-backed gateway/session routing, Vitest, Rust tests.

---

### Task 1: Document current Feishu touchpoints and choose the migration seam

**Files:**
- Modify: `docs/plans/2026-03-22-feishu-runtime-unification-design.md`
- Read: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Read: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Read: `apps/runtime/sidecar/src/index.ts`

**Step 1: Confirm the exact outbound seam**

Identify the current outbound entry points:
- `send_feishu_text_message`
- `send_feishu_text_message_with_pool`
- sidecar `/api/feishu/send-message`

Write down the chosen seam in the design doc: keep the Tauri command surface stable and swap the implementation under it.

**Step 2: Verify no additional hidden Feishu send paths exist**

Run:

```bash
rg -n "send_feishu_text_message|/api/feishu/send-message|/api/feishu/ws|list-chats" apps/runtime/src apps/runtime/src-tauri apps/runtime/sidecar -S
```

Expected: all Feishu transport touchpoints are now enumerated.

**Step 3: Commit notes into the design doc**

Add the confirmed seam and deleted-surface list if anything was missing.

### Task 2: Add a plugin-runtime outbound send command

**Files:**
- Modify: `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`
- Modify: `apps/runtime/plugin-host/src/runtime.ts`
- Test: `apps/runtime/plugin-host/src/runtime.test.ts`

**Step 1: Write the failing test**

Add a test that proves the plugin host can accept an outbound send command and route it through the official plugin runtime fixture without sidecar.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm --dir apps/runtime/plugin-host test -- --runInBand runtime
```

Expected: FAIL because no outbound command bridge exists yet.

**Step 3: Write minimal implementation**

Add a controlled outbound command path in the plugin host runtime:
- input contains account id, thread/chat target, payload text, and message mode
- runtime resolves the official plugin instance
- runtime returns a structured success/failure result

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm --dir apps/runtime/plugin-host test -- --runInBand runtime
```

Expected: PASS for the new outbound send test.

**Step 5: Commit**

```bash
git add apps/runtime/plugin-host/scripts/run-feishu-host.mjs apps/runtime/plugin-host/src/runtime.ts apps/runtime/plugin-host/src/runtime.test.ts
git commit -m "feat: add feishu outbound send path to plugin host"
```

### Task 3: Expose official-runtime outbound send from Tauri

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

**Step 1: Write the failing test**

Add a Rust test that simulates an active Feishu runtime session and expects a Tauri helper to send an outbound message through the plugin-host channel.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib openclaw_plugins -- --nocapture
```

Expected: FAIL because no Tauri outbound helper exists yet.

**Step 3: Write minimal implementation**

Implement a helper in `openclaw_plugins.rs` to:
- locate the active Feishu runtime session
- write a send command through the managed plugin-host channel
- await a structured result or timeout
- persist outbound diagnostics

Register any new command/export in `lib.rs` only if needed.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib openclaw_plugins -- --nocapture
```

Expected: PASS for the outbound-runtime helper coverage.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/openclaw_plugins.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat: expose feishu outbound send through plugin runtime"
```

### Task 4: Rewire gateway outbound send to the official runtime

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Test: `apps/runtime/src-tauri/tests/test_feishu_gateway.rs`

**Step 1: Write the failing test**

Add or update gateway tests so `send_feishu_text_message*` expects the plugin-runtime helper to be used rather than `call_sidecar_json("/api/feishu/send-message", ...)`.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_feishu_gateway -- --nocapture
```

Expected: FAIL because the gateway still targets sidecar.

**Step 3: Write minimal implementation**

Update `feishu_gateway.rs` so outbound send:
- resolves the current Feishu account/session target
- calls the new official-runtime send helper
- reports a clear error if runtime is missing or not ready
- no longer depends on `sidecar_base_url` for send-message

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_feishu_gateway -- --nocapture
```

Expected: PASS for outbound gateway tests.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/tests/test_feishu_gateway.rs
git commit -m "refactor: send feishu outbound replies via plugin runtime"
```

### Task 5: Remove Feishu send fallback assumptions from the frontend bridge

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`

**Step 1: Write the failing test**

Update the Feishu bridge test so the app no longer assumes sidecar HTTP semantics for outbound delivery timing or error shape.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx
```

Expected: FAIL because the bridge still encodes sidecar-era assumptions.

**Step 3: Write minimal implementation**

Adjust `App.tsx` so Feishu bridge behavior:
- keeps the same user-facing flow
- treats `send_feishu_text_message` as official-runtime backed
- uses clearer diagnostics for runtime-not-ready errors

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx
```

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx
git commit -m "test: align feishu app bridge with runtime outbound path"
```

### Task 6: Remove sidecar Feishu outbound and ws endpoints

**Files:**
- Modify: `apps/runtime/sidecar/src/index.ts`
- Modify: `apps/runtime/sidecar/test/channel-endpoints.test.ts`
- Modify: `apps/runtime/sidecar/test/feishu.ws-api.test.ts`

**Step 1: Write the failing test**

Replace old sidecar Feishu endpoint tests with assertions that those routes are absent or intentionally unsupported after migration.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm test:sidecar
```

Expected: FAIL because sidecar still exposes Feishu routes.

**Step 3: Write minimal implementation**

Delete Feishu sidecar route handlers from `index.ts`:
- `/api/feishu/send-message`
- `/api/feishu/list-chats`
- `/api/feishu/ws/start`
- `/api/feishu/ws/stop`
- `/api/feishu/ws/status`
- `/api/feishu/ws/drain-events`
- `/api/feishu/ws/reconcile`

Then update or remove sidecar tests accordingly.

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm test:sidecar
```

Expected: PASS with Feishu route coverage removed or inverted.

**Step 5: Commit**

```bash
git add apps/runtime/sidecar/src/index.ts apps/runtime/sidecar/test/channel-endpoints.test.ts apps/runtime/sidecar/test/feishu.ws-api.test.ts
git commit -m "refactor: remove feishu responsibilities from sidecar"
```

### Task 7: Clean up settings and diagnostics to remove sidecar-era Feishu assumptions

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add or update tests so Feishu settings no longer rely on sidecar health or sidecar-specific transport wording.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx
```

Expected: FAIL if existing UI still references sidecar-era behavior.

**Step 3: Write minimal implementation**

Update `SettingsView.tsx` to:
- present Feishu status purely in terms of official plugin runtime
- remove sidecar transport expectations
- keep diagnostics focused on official runtime, auth, pairing, and employee routing

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm exec vitest run src/components/__tests__/SettingsView.wecom-connector.test.tsx
```

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "refactor: remove feishu sidecar assumptions from settings"
```

### Task 8: Run integrated verification for Phase 1

**Files:**
- No code changes required unless fixes are found

**Step 1: Run Rust fast verification**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS.

**Step 2: Run sidecar verification**

Run:

```bash
pnpm test:sidecar
```

Expected: PASS with Feishu removed from sidecar.

**Step 3: Run targeted frontend verification**

Run:

```bash
pnpm exec vitest run src/__tests__/App.im-feishu-bridge.test.tsx src/components/__tests__/SettingsView.wecom-connector.test.tsx
```

Expected: PASS.

**Step 4: Run desktop build sanity check**

Run:

```bash
pnpm build:runtime
```

Expected: PASS.

**Step 5: Manual end-to-end verification**

Use a clean local runtime state and verify:
1. 新建机器人或绑定已有机器人
2. 完成授权
3. 批准接入
4. 飞书发消息
5. WorkClaw 收到消息
6. WorkClaw 生成回复
7. 飞书收到同一条回复
8. 关闭并重启 WorkClaw
9. 飞书再次发消息，确认自动恢复后仍能收发

Expected: 收发、审批、重启恢复都稳定。

**Step 6: Commit**

```bash
git add -A
git commit -m "test: verify feishu runtime unification phase 1"
```

### Task 9: Prepare Phase 2 handoff

**Files:**
- Modify: `docs/plans/2026-03-22-feishu-runtime-unification-design.md`
- Modify: `docs/plans/2026-03-22-feishu-runtime-unification-plan.md`

**Step 1: Capture follow-up product work**

Document remaining Phase 2 work items:
- onboarding simplification
- approval UX polish
- reception setup guidance
- diagnostics wording
- ordinary-user error handling

**Step 2: Save explicit handoff notes**

Record any remaining gaps discovered during Phase 1 implementation and testing.

**Step 3: Commit**

```bash
git add docs/plans/2026-03-22-feishu-runtime-unification-design.md docs/plans/2026-03-22-feishu-runtime-unification-plan.md
git commit -m "docs: capture phase 2 feishu product follow-ups"
```
