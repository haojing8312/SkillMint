# Feishu Connector Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the WorkClaw Feishu settings surface into a task-first setup flow for non-technical users while preserving advanced capability behind secondary UI.

**Architecture:** Replace the current tabbed Feishu control console with a single setup workflow driven by aggregated progress state. Add a Tauri-side environment/progress aggregation layer so the React UI renders one clear next step at a time, keep the official plugin as the only supported path, freeze the main flow to existing-bot binding over WebSocket, and move diagnostics plus advanced settings below the primary setup path.

**Tech Stack:** React, TypeScript, Tauri, Rust, sqlx, Playwright, Vitest/Jest-style runtime UI tests

---

### Task 1: Lock Product Scope In Tests

**Files:**
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/e2e/im-connectors.feishu.spec.ts`

**Step 1: Write failing UI assertions for the new visible scope**

Add expectations that the Feishu page:
- shows a task-first setup summary instead of the `连接配置 / 官方插件 / 配对与授权` tab row
- does not show `Verification Token`
- does not show `Encrypt Key`
- does not show `Webhook Host`
- does not show `Webhook Port`
- does not show `新建机器人`
- shows environment-check content before install/start guidance

**Step 2: Run the targeted frontend test to verify it fails**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: FAIL because the old tabbed console and old field labels still render.

**Step 3: Run the Feishu E2E spec to capture current behavior**

Run: `pnpm test:e2e:runtime -- --grep "feishu"`

Expected: Existing Feishu settings assertions still reflect the old tabbed UI and old connector text.

**Step 4: Commit the red test change**

```bash
git add apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx apps/runtime/e2e/im-connectors.feishu.spec.ts
git commit -m "test(feishu): lock redesigned connector scope"
```

### Task 2: Add A Tauri Environment Status Command

**Files:**
- Modify: `d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `d:/code/WorkClaw/apps/runtime/src-tauri/src/lib.rs`

**Step 1: Write the failing Rust test for environment detection**

Add a new serializable struct:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FeishuPluginEnvironmentStatus {
    pub node_available: bool,
    pub npm_available: bool,
    pub node_version: Option<String>,
    pub npm_version: Option<String>,
    pub can_install_plugin: bool,
    pub can_start_runtime: bool,
    pub error: Option<String>,
}
```

Add unit tests that stub command probing behind helper functions and assert:
- missing Node returns `node_available = false`
- missing npm returns `npm_available = false`
- both present yields `can_install_plugin = true`

**Step 2: Run the targeted Rust test to verify it fails**

Run: `pnpm test:rust-fast -- openclaw_plugins`

Expected: FAIL because `FeishuPluginEnvironmentStatus` and the command do not exist yet.

**Step 3: Implement minimal environment probing**

In `openclaw_plugins.rs`:
- add helpers that run `node --version` and `npm --version`
- map command failures into the new status struct
- add a public Tauri command such as `get_feishu_plugin_environment_status`

In `lib.rs`:
- register the new Tauri command

**Step 4: Run the targeted Rust test to verify it passes**

Run: `pnpm test:rust-fast -- openclaw_plugins`

Expected: PASS for the new environment detection tests.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/openclaw_plugins.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(feishu): add plugin environment detection"
```

### Task 3: Add A Tauri Setup Progress Aggregation Command

**Files:**
- Modify: `d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `d:/code/WorkClaw/apps/runtime/src-tauri/src/lib.rs`

**Step 1: Write the failing Rust test for setup progress**

Add a serializable aggregate type such as:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FeishuSetupProgress {
    pub environment: FeishuPluginEnvironmentStatus,
    pub credentials_configured: bool,
    pub credentials_validated: bool,
    pub bot_name: Option<String>,
    pub bot_open_id: Option<String>,
    pub plugin_installed: bool,
    pub plugin_version: Option<String>,
    pub runtime_running: bool,
    pub runtime_last_error: Option<String>,
    pub auth_status: String,
    pub pending_pairings: usize,
    pub default_routing_employee_name: Option<String>,
    pub scoped_routing_count: usize,
    pub summary_state: String,
}
```

Add tests that assert summary-state selection for:
- missing environment
- credentials empty
- plugin not installed
- authorization pending
- routing not configured
- fully ready

**Step 2: Run the targeted Rust test to verify it fails**

Run: `pnpm test:rust-fast -- openclaw_plugins`

Expected: FAIL because the aggregate command and summary logic do not exist.

**Step 3: Implement the aggregate command**

In `openclaw_plugins.rs`:
- compose current app settings, runtime status, pairing requests, installed plugin records, and employee routing counts
- derive a single `summary_state`
- expose a command such as `get_feishu_setup_progress`

In `lib.rs`:
- register the new command

**Step 4: Run the targeted Rust test to verify it passes**

Run: `pnpm test:rust-fast -- openclaw_plugins`

Expected: PASS for summary-state aggregation.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/openclaw_plugins.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(feishu): add setup progress aggregation"
```

### Task 4: Remove The Old Installer-Led Main Flow From SettingsView

**Files:**
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test for removed primary controls**

Add assertions that the main Feishu surface no longer renders:
- the `feishuConsoleSection` tab group
- `运行新建机器人向导`
- the installer session console output box
- manual installer input field

**Step 2: Run the targeted UI test to verify it fails**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: FAIL because the old controls still exist.

**Step 3: Implement the minimal removal/refactor**

In `SettingsView.tsx`:
- remove `feishuConsoleSection` from the primary rendering path
- stop rendering the create-bot installer controls in the user-facing setup path
- stop rendering the installer black console box in the main view
- keep any low-level installer code only if still needed internally for link mode

**Step 4: Run the targeted UI test to verify it passes**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: PASS for the removed main-flow controls.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "refactor(feishu): remove old installer-led settings flow"
```

### Task 5: Build The New Summary And Environment Cards

**Files:**
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuSetupSummaryCard.tsx`
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuEnvironmentCheckCard.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing component-level expectations**

Add assertions that the Feishu page shows:
- one summary card with a single primary next action
- an environment card listing Node.js and npm readiness
- install-step guidance when environment is missing

**Step 2: Run the targeted UI test to verify it fails**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: FAIL because the new cards do not exist.

**Step 3: Implement the cards and wire them up**

In `FeishuSetupSummaryCard.tsx`:
- accept `summaryState`, title, description, and primary action props

In `FeishuEnvironmentCheckCard.tsx`:
- accept `FeishuPluginEnvironmentStatus`
- render rows for Node.js and npm
- render `查看安装步骤` and `重新检测环境`

In `SettingsView.tsx`:
- call `get_feishu_plugin_environment_status`
- call `get_feishu_setup_progress`
- render the new cards above all other Feishu content

**Step 4: Run the targeted UI test to verify it passes**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: PASS for the new summary/environment presentation.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/settings/FeishuSetupSummaryCard.tsx apps/runtime/src/components/settings/FeishuEnvironmentCheckCard.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat(feishu): add summary and environment setup cards"
```

### Task 6: Replace The Connector Form With Existing-Bot Binding Only

**Files:**
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuRobotBindingCard.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/e2e/im-connectors.feishu.spec.ts`

**Step 1: Write the failing test for the new binding card**

Add assertions that the page renders:
- `App ID`
- `App Secret`
- `验证机器人信息`
- verification result fields for bot name and open_id after success

And does not render:
- token/key fields
- webhook settings
- new-bot wording

**Step 2: Run the targeted tests to verify they fail**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Run: `pnpm test:e2e:runtime -- --grep "feishu"`

Expected: FAIL because the existing connector form still renders old fields.

**Step 3: Implement the new binding card**

In `FeishuRobotBindingCard.tsx`:
- render only App ID and App Secret inputs
- wire `验证机器人信息` to `probe_openclaw_plugin_feishu_credentials`
- surface bot name and open_id on success

In `SettingsView.tsx`:
- remove token/key/webhook fields from the main Feishu form
- keep only existing-bot binding in the main flow

**Step 4: Run the targeted tests to verify they pass**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Run: `pnpm test:e2e:runtime -- --grep "feishu"`

Expected: PASS for the simplified binding flow.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/settings/FeishuRobotBindingCard.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx apps/runtime/e2e/im-connectors.feishu.spec.ts
git commit -m "feat(feishu): simplify main flow to existing bot binding"
```

### Task 7: Build Authorization And Routing Guidance Cards

**Files:**
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuAuthorizationCard.tsx`
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuRoutingEntryCard.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/employees/EmployeeFeishuAssociationSection.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx`

**Step 1: Write the failing tests for authorization/routing guidance**

Add assertions that:
- authorization appears as a guided step, not a generic `配对与授权` tab
- routing state shows default employee and scoped-rule count
- employee-facing copy is result-oriented instead of plugin-jargon-heavy

**Step 2: Run the targeted tests to verify they fail**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView.feishu-connection-status.test.tsx`

Expected: FAIL because the employee copy and routing entry are still old.

**Step 3: Implement the cards and copy changes**

In `FeishuAuthorizationCard.tsx`:
- render authorization status
- render 3-step instructions
- render `刷新状态`

In `FeishuRoutingEntryCard.tsx`:
- render default-routing employee and scoped rule count
- render buttons to open employee routing configuration

In `EmployeeFeishuAssociationSection.tsx`:
- replace old passive warning text with setup-aware guidance

**Step 4: Run the targeted tests to verify they pass**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView.feishu-connection-status.test.tsx`

Expected: PASS for the updated guidance and routing entry.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/settings/FeishuAuthorizationCard.tsx apps/runtime/src/components/settings/FeishuRoutingEntryCard.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/employees/EmployeeFeishuAssociationSection.tsx apps/runtime/src/components/employees/__tests__/EmployeeHubView.feishu-connection-status.test.tsx
git commit -m "feat(feishu): add auth and routing guidance cards"
```

### Task 8: Move Diagnostics And Advanced Settings Below The Main Flow

**Files:**
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuConnectionDetailsCard.tsx`
- Create: `d:/code/WorkClaw/apps/runtime/src/components/settings/FeishuAdvancedSettingsPanel.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing tests for the new secondary sections**

Add assertions that:
- connection details render as a compact secondary card
- advanced settings are grouped and visually separated
- advanced settings remain available
- the main setup area no longer opens with raw log blocks or huge flat JSON forms

**Step 2: Run the targeted UI test to verify it fails**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: FAIL because diagnostics and advanced settings are still flat and primary.

**Step 3: Implement the secondary sections**

In `FeishuConnectionDetailsCard.tsx`:
- render runtime status, plugin version, current account, last event, last error

In `FeishuAdvancedSettingsPanel.tsx`:
- group settings into:
  - message and presentation
  - chat rules
  - runtime behavior
- keep save handling from existing advanced settings logic

In `SettingsView.tsx`:
- mount these sections below the setup path

**Step 4: Run the targeted UI test to verify it passes**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: PASS for the new secondary-section layout.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/settings/FeishuConnectionDetailsCard.tsx apps/runtime/src/components/settings/FeishuAdvancedSettingsPanel.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "refactor(feishu): separate details and advanced settings"
```

### Task 9: Clean Up Obsolete Feishu Settings Surface And Names

**Files:**
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`
- Modify: `d:/code/WorkClaw/apps/runtime/e2e/im-connectors.feishu.spec.ts`

**Step 1: Write the failing assertions for obsolete labels**

Add assertions that the main Feishu settings no longer expose:
- `官方插件宿主`
- `配对与授权`
- `连接配置`
- `待处理配对`
- `未识别` for user-facing primary status

And now expose friendlier labels such as:
- `飞书连接`
- `完成飞书授权`
- `绑定已有机器人`
- `接待设置`

**Step 2: Run the targeted tests to verify they fail**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Run: `pnpm test:e2e:runtime -- --grep "feishu"`

Expected: FAIL because old labels still exist.

**Step 3: Implement the copy cleanup**

Replace engineering-facing labels with task-facing wording in the Feishu settings path only. Keep internal identifiers unchanged where possible.

**Step 4: Run the targeted tests to verify they pass**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Run: `pnpm test:e2e:runtime -- --grep "feishu"`

Expected: PASS for updated user-facing naming.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx apps/runtime/e2e/im-connectors.feishu.spec.ts
git commit -m "chore(feishu): rewrite connector copy for task-first setup"
```

### Task 10: Run Full Verification For The Changed Surface

**Files:**
- Modify if needed: any files touched during fixes from verification failures

**Step 1: Run runtime UI tests**

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx EmployeeHubView.feishu-connection-status.test.tsx`

Expected: PASS

**Step 2: Run Feishu E2E**

Run: `pnpm test:e2e:runtime -- --grep "feishu"`

Expected: PASS

**Step 3: Run Rust fast-path verification**

Run: `pnpm test:rust-fast`

Expected: PASS for `openclaw_plugins` and any touched command registration paths.

**Step 4: Run release-sensitive desktop build sanity because startup/install flow changed**

Run: `pnpm build:runtime`

Expected: PASS because this change affects desktop-visible plugin install/start behavior.

**Step 5: Commit final verification fixes**

```bash
git add -A
git commit -m "test(feishu): verify redesigned connector setup flow"
```

### Task 11: Optional Follow-Up Cleanup After Merge Confidence

**Files:**
- Modify: `d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`
- Modify: `d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx`

**Step 1: Audit dead code paths**

Look for unused create-mode installer state, webhook-only UI paths, and no-longer-rendered helper branches.

**Step 2: Remove only code proven unused by tests**

Do not remove latent backend compatibility fields unless the tests prove there is no remaining caller.

**Step 3: Run focused verification**

Run: `pnpm test:rust-fast -- openclaw_plugins`

Run: `pnpm --dir apps/runtime test -- SettingsView.wecom-connector.test.tsx`

Expected: PASS

**Step 4: Commit cleanup**

```bash
git add apps/runtime/src-tauri/src/commands/openclaw_plugins.rs apps/runtime/src/components/SettingsView.tsx
git commit -m "refactor(feishu): remove obsolete setup-path branches"
```
