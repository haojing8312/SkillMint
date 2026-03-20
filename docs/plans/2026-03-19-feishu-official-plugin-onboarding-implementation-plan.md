# Feishu Official Plugin Onboarding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 WorkClaw 的飞书官方插件接入体验重构为“安装向导主入口 + 设置页状态面板/高级配置面板”，对齐 OpenClaw 官方流程。

**Architecture:** 以前端安装向导承接“新建机器人 / 关联已有机器人 / 校验凭证 / 自动启动 runtime”的主流程；Tauri 后端补充官方插件安装与向导期操作命令；现有飞书设置页保留为状态与诊断面板。运行态仍然走 `start_openclaw_plugin_feishu_runtime -> run-feishu-host -> gateway.startAccount -> websocket monitor`。

**Tech Stack:** React + Tauri + Rust commands + Node plugin-host + Vitest + Rust unit tests

---

### Task 1: Document OpenClaw-Aligned UX States

**Files:**
- Modify: `docs/plans/2026-03-19-feishu-official-plugin-onboarding-design.md`
- Reference: `references/openclaw/src/cli/plugins-cli.ts`
- Reference: `references/openclaw-lark/src/channel/onboarding.ts`

**Step 1: Write a checklist of UI states**

Enumerate:
- 未安装
- 安装中
- 新建机器人
- 关联已有机器人
- 凭证验证成功/失败
- runtime 启动中
- runtime 运行中
- 飞书内下一步提示

**Step 2: Verify the checklist maps to real OpenClaw behavior**

Check each state against local references and the official document link.

**Step 3: Commit**

```bash
git add docs/plans/2026-03-19-feishu-official-plugin-onboarding-design.md
git commit -m "docs: clarify feishu official plugin onboarding states"
```

### Task 2: Add Backend Command for Official Plugin Guided Install Metadata

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

**Step 1: Write the failing Rust test**

Add a unit test for a helper that returns the official plugin install target metadata:
- plugin id
- npm spec
- doc link
- supported onboarding modes (`create_bot`, `link_existing_bot`)

**Step 2: Run the Rust test to verify it fails**

Run:

```bash
cargo test -p runtime --manifest-path apps/runtime/src-tauri/Cargo.toml --lib openclaw_plugins -- --nocapture
```

Expected: fail because metadata helper/command does not exist.

**Step 3: Implement the minimal backend types/command**

Add a serializable type like `OpenClawOfficialFeishuPluginGuideInfo` and a Tauri command that returns static metadata and the official doc link.

**Step 4: Re-run the Rust test**

Expected: pass.

### Task 3: Add Frontend Install Wizard Entry Point

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing frontend test**

Add a test that expects the flybook/feishu page to show a primary CTA such as:
- `开始安装向导`
- or `重新运行安装向导`

when official plugin mode is not fully onboarded.

**Step 2: Run the single test to verify RED**

```bash
pnpm --dir apps/runtime exec vitest run ./src/components/__tests__/SettingsView.wecom-connector.test.tsx -t "shows official plugin install wizard entry point"
```

Expected: fail because no such CTA exists.

**Step 3: Implement minimal UI**

Add:
- a primary CTA for install/onboarding
- a short explanatory block referencing the official doc link
- explicit text that飞书对话内 `/feishu start` `/feishu auth` `/feishu doctor` are the next verification steps

**Step 4: Re-run the single test**

Expected: pass.

### Task 4: Add Wizard Mode Selection UI

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Expect that clicking the wizard CTA reveals:
- `新建机器人`
- `关联已有机器人`

**Step 2: Run the targeted test**

Expected: fail.

**Step 3: Implement minimal mode selector**

Use a lightweight inline panel or modal state in `SettingsView.tsx`.

**Step 4: Re-run test**

Expected: pass.

### Task 5: Add Existing Bot Credential Verification Flow

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write failing backend and frontend tests**

Backend:
- add a command test for credential verification request normalization

Frontend:
- expect entering `App ID / App Secret` in “关联已有机器人” mode to call a verify command and show success/failure feedback

**Step 2: Run targeted tests**

Expected: fail.

**Step 3: Implement minimal backend verify command**

This can initially call the existing official plugin runtime/config machinery or a lightweight probe path.

**Step 4: Implement frontend form + feedback**

Show:
- `正在校验`
- `凭证校验成功`
- `凭证无效`

**Step 5: Re-run targeted tests**

Expected: pass.

### Task 6: Add Create-Bot Guided Placeholder Flow

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Optional Modify later: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

**Step 1: Write failing frontend test**

Expect `新建机器人` mode to show:
- QR / 创建入口占位
- official doc link
- next-step guidance

**Step 2: Run targeted test**

Expected: fail.

**Step 3: Implement minimal placeholder**

Even before fully automating QR creation, add the correct structure:
- creation instructions
- official link
- success next steps

**Step 4: Re-run test**

Expected: pass.

### Task 7: Reframe Settings Page as Status + Diagnostics

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write failing test**

Expect the page to clearly show:
- `最近事件`
- `最近日志`
- `/feishu start`
- `/feishu auth`
- `/feishu doctor`

as first-class diagnostics/help content.

**Step 2: Run targeted test**

Expected: fail.

**Step 3: Implement minimal UI**

Add a diagnostic help block and make the official-plugin section clearly secondary to the wizard and status information.

**Step 4: Re-run test**

Expected: pass.

### Task 8: Auto-Start Runtime After Successful Onboarding Path

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Test: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

**Step 1: Write failing tests**

Expect that after successful install/verify/onboarding completion:
- frontend triggers runtime start
- status becomes running

**Step 2: Run targeted tests**

Expected: fail.

**Step 3: Implement minimal auto-start glue**

Reuse existing `start_openclaw_plugin_feishu_runtime`.

**Step 4: Re-run tests**

Expected: pass.

### Task 9: Add Real-World Debug Visibility for Message Failures

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Test: existing frontend/Rust tests

**Step 1: Write failing test**

Expect runtime status with:
- `last_event_at`
- `recent_logs`

to render clearly in the Feishu page.

**Step 2: Run targeted test**

Expected: fail.

**Step 3: Implement minimal rendering and label cleanup**

Show the last 1-3 log lines and recent event time in a readable format.

**Step 4: Re-run tests**

Expected: pass.

### Task 10: End-to-End Manual Validation

**Files:**
- No required code files
- Update docs if needed: `docs/plans/2026-03-19-feishu-official-plugin-onboarding-design.md`

**Step 1: Launch dev app**

```bash
pnpm app
```

**Step 2: Validate these user flows manually**

1. Open Feishu connector page
2. Start install/onboarding flow
3. Choose an onboarding mode
4. Save or verify credentials
5. Confirm runtime reaches `运行中`
6. Confirm page shows `最近事件` and `最近日志`
7. Send a Feishu message
8. Confirm either:
   - the bot replies, or
   - the page logs the exact plugin/runtime error

**Step 3: Record remaining gaps**

If the bot still does not reply, write down whether:
- websocket never connected
- event never arrived
- event arrived but plugin processing failed

### Task 11: Final Verification

**Files:**
- All touched files above

**Step 1: Run frontend verification**

```bash
pnpm --dir apps/runtime exec vitest run ./src/components/__tests__/SettingsView.wecom-connector.test.tsx --passWithNoTests
```

**Step 2: Run Rust verification**

```bash
cargo test -p runtime --manifest-path apps/runtime/src-tauri/Cargo.toml --lib openclaw_plugins -- --nocapture
```

**Step 3: Run repo-required fast Rust path**

```bash
pnpm test:rust-fast
```

**Step 4: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx apps/runtime/src/types.ts apps/runtime/src-tauri/src/commands/openclaw_plugins.rs apps/runtime/src-tauri/src/lib.rs docs/plans/2026-03-19-feishu-official-plugin-onboarding-design.md docs/plans/2026-03-19-feishu-official-plugin-onboarding-implementation-plan.md
git commit -m "feat: align feishu official plugin onboarding with openclaw"
```
