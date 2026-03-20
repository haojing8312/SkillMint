# Feishu Settings Console Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor the existing Feishu settings page into a structured Feishu console with `连接配置` / `官方插件` / `配对与授权` sections, and wire the pairing approval workflow into the UI.

**Architecture:** Keep the existing `SettingsView` as the host surface, but carve the Feishu panel into second-level sections with independent data loaders. Reuse the Tauri commands already built for official plugin inspection and pairing approval so the UI stays thin and declarative.

**Tech Stack:** React, TypeScript, Tauri invoke commands, existing WorkClaw settings UI patterns, Rust Tauri backend commands.

---

### Task 1: Add Feishu Console View State

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add a test that expects the Feishu settings area to render second-level section tabs:

- `连接配置`
- `官方插件`
- `配对与授权`

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime exec vitest run ./src/components/__tests__/SettingsView.wecom-connector.test.tsx --passWithNoTests`

Expected: FAIL because the new section tabs do not exist yet.

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:

- add Feishu subsection state
- render the section tabs only when the active top-level tab is `feishu`
- keep the current Feishu content under `连接配置` first

**Step 4: Run test to verify it passes**

Run the same Vitest command.

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: add feishu console section tabs"
```

### Task 2: Move Official Plugin Content Into Dedicated Section

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Extend the Feishu settings test to assert:

- official plugin host content appears only in `官方插件`
- `连接配置` keeps connection/diagnostic content

**Step 2: Run test to verify it fails**

Run the existing Feishu settings Vitest command.

Expected: FAIL because the plugin content still sits inline in the base page.

**Step 3: Write minimal implementation**

Refactor the JSX:

- extract a `连接配置` content block
- extract an `官方插件` content block
- keep current data loading unchanged
- do not add new business logic yet

**Step 4: Run test to verify it passes**

Run the same Vitest command.

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: split feishu plugin status into dedicated section"
```

### Task 3: Add Pairing Request Types And Invoke Calls

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add a test that mocks Tauri responses for:

- `list_feishu_pairing_requests`

Then expect a pending request row to render in `配对与授权`.

**Step 2: Run test to verify it fails**

Run the Feishu settings Vitest command.

Expected: FAIL because the request types and invoke path do not exist in the component.

**Step 3: Write minimal implementation**

Add frontend types for pairing records in `types.ts`, then in `SettingsView.tsx`:

- load pairing requests when Feishu tab is active
- store them in local state
- render a basic list in `配对与授权`

**Step 4: Run test to verify it passes**

Run the same Vitest command.

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: show feishu pairing requests in settings"
```

### Task 4: Add Approve And Deny Actions

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add tests that:

- click `批准`
- click `拒绝`
- assert the component invokes:
  - `approve_feishu_pairing_request`
  - `deny_feishu_pairing_request`
- assert the list refreshes afterward

**Step 2: Run test to verify it fails**

Run the Feishu settings Vitest command.

Expected: FAIL because no action buttons or refresh logic exist yet.

**Step 3: Write minimal implementation**

In `SettingsView.tsx`:

- add action buttons for pending requests
- call the matching Tauri command
- optimistically disable buttons while the request is resolving
- refresh pairing requests after success

**Step 4: Run test to verify it passes**

Run the same Vitest command.

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: add feishu pairing approval actions"
```

### Task 5: Add Feishu Status Strip

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Test: `apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx`

**Step 1: Write the failing test**

Add expectations for a compact Feishu status strip showing:

- connection status
- official plugin status
- default account
- pending pairing count

**Step 2: Run test to verify it fails**

Run the Feishu settings Vitest command.

Expected: FAIL because the status strip does not exist.

**Step 3: Write minimal implementation**

Render a compact summary row above the subsection tabs using already loaded Feishu state and pairing list state.

**Step 4: Run test to verify it passes**

Run the same Vitest command.

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.wecom-connector.test.tsx
git commit -m "feat: add feishu console status strip"
```

### Task 6: Add Backend Coverage For Pairing Commands

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Test: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`

**Step 1: Write the failing test**

Add focused Rust tests for:

- listing pairing requests by status
- approving a request updates persisted state
- denying a request does not add to allow-from

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime --manifest-path apps/runtime/src-tauri/Cargo.toml --lib feishu_gateway -- --nocapture`

Expected: FAIL because at least one of the new query/update behaviors is not covered yet.

**Step 3: Write minimal implementation**

If needed, tighten the backend helpers so the tests pass with no UI-specific behavior leaking into them.

**Step 4: Run test to verify it passes**

Run the same Cargo command.

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/feishu_gateway.rs
git commit -m "test: cover feishu pairing request commands"
```

### Task 7: Run Full Relevant Verification

**Files:**
- No code changes required unless verification exposes defects

**Step 1: Run frontend Feishu settings test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run ./src/components/__tests__/SettingsView.wecom-connector.test.tsx --passWithNoTests
```

Expected: PASS.

**Step 2: Run sidecar test suite**

Run:

```bash
pnpm --dir apps/runtime/sidecar test
```

Expected: PASS.

**Step 3: Run focused Rust pairing test suite**

Run:

```bash
cargo test -p runtime --manifest-path apps/runtime/src-tauri/Cargo.toml --lib feishu_gateway -- --nocapture
```

Expected: PASS.

**Step 4: Run repo Rust fast-path verification**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS.

**Step 5: Commit**

```bash
git add -A
git commit -m "test: verify feishu settings console integration"
```
