# Feishu Browser Bridge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Chrome extension plus Native Messaging bridge that guides users through Feishu enterprise self-built app setup in their default browser and returns the collected credentials to local WorkClaw for binding.

**Architecture:** Add a Chrome-only browser bridge lane with three pieces: a Chrome extension on `open.feishu.cn`, a thin local native host bridge, and a Tauri-side Feishu setup orchestrator that owns workflow state, recovery, and local binding. Reuse the existing Feishu binding/routing stack instead of inventing a parallel integration path.

**Tech Stack:** Chrome Extension Manifest V3, TypeScript, Node.js native messaging host, Tauri Rust commands/state, existing WorkClaw React UI, Vitest/Node test runner, Rust cargo tests.

---

### Task 1: Define the browser bridge message contract and session state model

**Files:**
- Create: `apps/runtime/browser-bridge/shared/protocol.ts`
- Create: `apps/runtime/browser-bridge/shared/feishu-setup.ts`
- Create: `apps/runtime/browser-bridge/shared/__tests__/protocol.test.ts`
- Create: `apps/runtime/browser-bridge/shared/__tests__/feishu-setup.test.ts`

**Step 1: Write the failing protocol tests**

```ts
import { describe, expect, it } from "vitest";
import {
  isBridgeEnvelope,
  type BridgeEnvelope,
  type BridgeRequest,
  type BridgeResponse,
} from "../protocol";

describe("browser bridge protocol", () => {
  it("accepts valid request envelopes", () => {
    const msg: BridgeEnvelope<BridgeRequest> = {
      version: 1,
      sessionId: "sess-1",
      kind: "request",
      payload: { type: "session.start", provider: "feishu" },
    };

    expect(isBridgeEnvelope(msg)).toBe(true);
  });

  it("rejects envelopes without version/session metadata", () => {
    expect(isBridgeEnvelope({ kind: "request" })).toBe(false);
  });
});

describe("feishu setup states", () => {
  it("starts at INIT and can move to LOGIN_REQUIRED", () => {
    const state = createFeishuSetupState();
    expect(state.step).toBe("INIT");

    const next = transitionFeishuSetup(state, { type: "login.required" });
    expect(next.step).toBe("LOGIN_REQUIRED");
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `pnpm exec vitest run apps/runtime/browser-bridge/shared/__tests__/protocol.test.ts apps/runtime/browser-bridge/shared/__tests__/feishu-setup.test.ts`

Expected: FAIL with missing module or symbol errors.

**Step 3: Write minimal shared protocol and state model**

```ts
export type BridgeRequest =
  | { type: "session.start"; provider: "feishu" }
  | { type: "session.resume"; sessionId: string }
  | { type: "page.report"; page: FeishuDetectedPage }
  | { type: "credentials.report"; appId: string; appSecret: string };

export type BridgeResponse =
  | { type: "action.open"; url: string }
  | { type: "action.detect_step" }
  | { type: "action.collect_credentials" }
  | { type: "action.pause"; reason: string };

export interface BridgeEnvelope<T> {
  version: 1;
  sessionId: string;
  kind: "request" | "response" | "event";
  payload: T;
}

export function isBridgeEnvelope(value: unknown): value is BridgeEnvelope<unknown> {
  return typeof value === "object" && value !== null
    && (value as { version?: unknown }).version === 1
    && typeof (value as { sessionId?: unknown }).sessionId === "string"
    && typeof (value as { kind?: unknown }).kind === "string";
}
```

**Step 4: Run tests to verify they pass**

Run: `pnpm exec vitest run apps/runtime/browser-bridge/shared/__tests__/protocol.test.ts apps/runtime/browser-bridge/shared/__tests__/feishu-setup.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/browser-bridge/shared
git commit -m "feat(browser-bridge): define feishu setup protocol"
```

### Task 2: Add the local Feishu setup orchestrator in Tauri

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/feishu_browser_setup.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Create: `apps/runtime/src-tauri/tests/test_feishu_browser_setup.rs`

**Step 1: Write the failing Rust tests**

```rust
#[tokio::test]
async fn start_session_returns_login_required_when_browser_reports_logged_out() {
    let state = FeishuBrowserSetupStore::default();
    let session = state.start_session("feishu".to_string()).await.unwrap();

    let updated = state
        .apply_event(session.session_id.clone(), SetupEvent::LoginRequired)
        .await
        .unwrap();

    assert_eq!(updated.step, "LOGIN_REQUIRED");
}

#[tokio::test]
async fn credentials_report_transitions_to_bind_local() {
    let state = FeishuBrowserSetupStore::default();
    let session = state.start_session("feishu".to_string()).await.unwrap();

    let updated = state
        .apply_event(
            session.session_id.clone(),
            SetupEvent::CredentialsReported {
                app_id: "cli_x".into(),
                app_secret: "sec_x".into(),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.step, "BIND_LOCAL");
}
```

**Step 2: Run tests to verify they fail**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_browser_setup -- --nocapture`

Expected: FAIL because the command module and types do not exist yet.

**Step 3: Implement the minimal orchestrator**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuBrowserSetupSession {
    pub session_id: String,
    pub provider: String,
    pub step: String,
    pub app_id: Option<String>,
    pub app_secret_present: bool,
}

pub enum SetupEvent {
    LoginRequired,
    CredentialsReported { app_id: String, app_secret: String },
    BindSucceeded,
    BindFailed { reason: String },
}

impl FeishuBrowserSetupStore {
    pub async fn start_session(&self, provider: String) -> Result<FeishuBrowserSetupSession, String> {
        // create session at INIT
    }

    pub async fn apply_event(
        &self,
        session_id: String,
        event: SetupEvent,
    ) -> Result<FeishuBrowserSetupSession, String> {
        // transition to next explicit step
    }
}
```

Expose Tauri commands for:

- `start_feishu_browser_setup`
- `get_feishu_browser_setup_session`
- `apply_feishu_browser_setup_event`

**Step 4: Run tests to verify they pass**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_browser_setup -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/feishu_browser_setup.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_feishu_browser_setup.rs
git commit -m "feat(runtime): add feishu browser setup orchestrator"
```

### Task 3: Implement the Native Messaging host

**Files:**
- Create: `apps/runtime/browser-bridge/native-host/package.json`
- Create: `apps/runtime/browser-bridge/native-host/src/index.ts`
- Create: `apps/runtime/browser-bridge/native-host/src/client.ts`
- Create: `apps/runtime/browser-bridge/native-host/src/__tests__/native-host.test.ts`
- Create: `scripts/install-chrome-native-host.mjs`

**Step 1: Write the failing native-host tests**

```ts
import { describe, expect, it } from "vitest";
import { decodeNativeMessage, encodeNativeMessage } from "../index";

describe("native messaging framing", () => {
  it("round-trips a bridge envelope", () => {
    const encoded = encodeNativeMessage({
      version: 1,
      sessionId: "sess-1",
      kind: "request",
      payload: { type: "session.start", provider: "feishu" },
    });

    expect(decodeNativeMessage(encoded)).toMatchObject({
      sessionId: "sess-1",
      kind: "request",
    });
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `pnpm exec vitest run apps/runtime/browser-bridge/native-host/src/__tests__/native-host.test.ts`

Expected: FAIL with missing module errors.

**Step 3: Implement the native host**

```ts
export function encodeNativeMessage(message: unknown): Buffer {
  const json = Buffer.from(JSON.stringify(message), "utf8");
  const header = Buffer.alloc(4);
  header.writeUInt32LE(json.length, 0);
  return Buffer.concat([header, json]);
}

export function decodeNativeMessage(buffer: Buffer) {
  const size = buffer.readUInt32LE(0);
  return JSON.parse(buffer.subarray(4, 4 + size).toString("utf8"));
}
```

Forward each validated bridge envelope to local WorkClaw over a loopback HTTP or command channel and return the response to Chrome.

**Step 4: Run tests to verify they pass**

Run: `pnpm exec vitest run apps/runtime/browser-bridge/native-host/src/__tests__/native-host.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/browser-bridge/native-host scripts/install-chrome-native-host.mjs
git commit -m "feat(browser-bridge): add native messaging host"
```

### Task 4: Build the Chrome extension shell and page detector

**Files:**
- Create: `apps/runtime/browser-bridge/chrome-extension/manifest.json`
- Create: `apps/runtime/browser-bridge/chrome-extension/src/background.ts`
- Create: `apps/runtime/browser-bridge/chrome-extension/src/content.ts`
- Create: `apps/runtime/browser-bridge/chrome-extension/src/feishu-detector.ts`
- Create: `apps/runtime/browser-bridge/chrome-extension/src/overlay.ts`
- Create: `apps/runtime/browser-bridge/chrome-extension/src/__tests__/feishu-detector.test.ts`

**Step 1: Write the failing detector tests**

```ts
import { describe, expect, it } from "vitest";
import { detectFeishuPage } from "../feishu-detector";

describe("detectFeishuPage", () => {
  it("detects logged-out state", () => {
    document.body.innerHTML = `<button>登录</button>`;
    expect(detectFeishuPage(document).kind).toBe("login");
  });

  it("detects credential page", () => {
    document.body.innerHTML = `<div>凭证与基础信息</div><div>App ID</div><div>App Secret</div>`;
    expect(detectFeishuPage(document).kind).toBe("credentials");
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `pnpm exec vitest run apps/runtime/browser-bridge/chrome-extension/src/__tests__/feishu-detector.test.ts`

Expected: FAIL with missing module errors.

**Step 3: Implement the minimal extension shell**

```ts
export function detectFeishuPage(doc: Document): { kind: string; confidence: number } {
  const text = doc.body?.innerText ?? "";
  if (text.includes("登录")) return { kind: "login", confidence: 0.9 };
  if (text.includes("凭证与基础信息") && text.includes("App ID")) {
    return { kind: "credentials", confidence: 0.9 };
  }
  return { kind: "unknown", confidence: 0.1 };
}
```

Manifest requirements:

- MV3
- host permissions only for `https://open.feishu.cn/*`
- background service worker
- content scripts on `open.feishu.cn`

**Step 4: Run tests to verify they pass**

Run: `pnpm exec vitest run apps/runtime/browser-bridge/chrome-extension/src/__tests__/feishu-detector.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/browser-bridge/chrome-extension
git commit -m "feat(browser-bridge): add chrome extension shell"
```

### Task 5: Implement credential extraction and local bind integration

**Files:**
- Modify: `apps/runtime/browser-bridge/chrome-extension/src/content.ts`
- Modify: `apps/runtime/browser-bridge/chrome-extension/src/background.ts`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_browser_setup.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_routing.rs`
- Create: `apps/runtime/src-tauri/tests/test_feishu_browser_setup_binding.rs`

**Step 1: Write the failing integration test**

```rust
#[tokio::test]
async fn credentials_report_runs_local_binding_and_marks_secret_present() {
    let store = FeishuBrowserSetupStore::default();
    let session = store.start_session("feishu".to_string()).await.unwrap();

    let updated = store
        .report_credentials_and_bind(
            session.session_id,
            "cli_test".to_string(),
            "sec_test".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(updated.step, "ENABLE_LONG_CONNECTION");
    assert!(updated.app_secret_present);
}
```

**Step 2: Run tests to verify they fail**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_browser_setup_binding -- --nocapture`

Expected: FAIL because bind integration does not exist yet.

**Step 3: Implement the minimal bind bridge**

```rust
pub async fn report_credentials_and_bind(
    &self,
    session_id: String,
    app_id: String,
    app_secret: String,
) -> Result<FeishuBrowserSetupSession, String> {
    // persist masked state
    // call existing local Feishu binding logic
    // move to ENABLE_LONG_CONNECTION on success
}
```

Extension behavior:

- ask for explicit one-time extraction approval
- read `App ID` and `App Secret` from the current page
- send them to the native host in a bridge envelope
- never store the secret in extension storage

**Step 4: Run tests to verify they pass**

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_browser_setup_binding -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/browser-bridge/chrome-extension/src/content.ts apps/runtime/browser-bridge/chrome-extension/src/background.ts apps/runtime/src-tauri/src/commands/feishu_browser_setup.rs apps/runtime/src-tauri/src/commands/im_routing.rs apps/runtime/src-tauri/tests/test_feishu_browser_setup_binding.rs
git commit -m "feat(runtime): bind feishu credentials from browser bridge"
```

### Task 6: Add desktop status UX and resume/retry actions

**Files:**
- Create: `apps/runtime/src/components/employees/FeishuBrowserSetupView.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/types.ts`
- Create: `apps/runtime/src/components/employees/__tests__/FeishuBrowserSetupView.test.tsx`

**Step 1: Write the failing React test**

```tsx
import { render, screen } from "@testing-library/react";
import { FeishuBrowserSetupView } from "../FeishuBrowserSetupView";

test("shows login-required guidance", () => {
  render(
    <FeishuBrowserSetupView
      session={{ session_id: "sess-1", step: "LOGIN_REQUIRED", app_secret_present: false }}
      onRetry={() => Promise.resolve()}
      onOpenBrowser={() => Promise.resolve()}
    />
  );

  expect(screen.getByText("请先登录飞书")).toBeInTheDocument();
});
```

**Step 2: Run tests to verify they fail**

Run: `pnpm --dir apps/runtime test FeishuBrowserSetupView`

Expected: FAIL because the component does not exist yet.

**Step 3: Implement the minimal status view**

```tsx
export function FeishuBrowserSetupView({ session, onRetry, onOpenBrowser }: Props) {
  if (session.step === "LOGIN_REQUIRED") {
    return <div>请先登录飞书</div>;
  }
  return <div>当前步骤：{session.step}</div>;
}
```

Expose actions for:

- `Open browser`
- `Retry current step`
- `Resume session`
- `Cancel session`

Do not embed Feishu pages in this UI.

**Step 4: Run tests to verify they pass**

Run: `pnpm --dir apps/runtime test FeishuBrowserSetupView`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/employees/FeishuBrowserSetupView.tsx apps/runtime/src/components/employees/__tests__/FeishuBrowserSetupView.test.tsx apps/runtime/src/App.tsx apps/runtime/src/types.ts
git commit -m "feat(ui): add feishu browser setup status view"
```

### Task 7: Add E2E coverage and installation docs

**Files:**
- Create: `apps/runtime/e2e/feishu-browser-setup.spec.ts`
- Create: `docs/integrations/feishu-browser-setup.md`
- Modify: `README.md`
- Modify: `README.en.md`

**Step 1: Write the failing E2E spec**

```ts
import { test, expect } from "@playwright/test";

test("pauses for login and resumes through credential collection", async ({ page }) => {
  await page.goto("/mock-feishu/login");
  await expect(page.getByText("请先登录飞书")).toBeVisible();
});
```

**Step 2: Run the E2E spec to verify it fails**

Run: `pnpm --dir apps/runtime test:e2e --grep "feishu browser setup"`

Expected: FAIL because the scenario and mock flow are not wired yet.

**Step 3: Implement the mocked E2E flow and docs**

```md
## Install Chrome Bridge

1. Install the WorkClaw Chrome extension.
2. Run the native-host installer script.
3. Restart Chrome.
4. Start Feishu setup from WorkClaw.
```

Document:

- extension install
- native host install/uninstall
- security scope
- troubleshooting for login pause, bind failure, and page drift

**Step 4: Run the relevant verification**

Run: `pnpm --dir apps/runtime test:e2e --grep "feishu browser setup"`

Expected: PASS

Run: `pnpm exec vitest run apps/runtime/browser-bridge/shared/__tests__/protocol.test.ts apps/runtime/browser-bridge/shared/__tests__/feishu-setup.test.ts apps/runtime/browser-bridge/native-host/src/__tests__/native-host.test.ts apps/runtime/browser-bridge/chrome-extension/src/__tests__/feishu-detector.test.ts`

Expected: PASS

Run: `cd apps/runtime/src-tauri && cargo test --test test_feishu_browser_setup --test test_feishu_browser_setup_binding -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/e2e/feishu-browser-setup.spec.ts docs/integrations/feishu-browser-setup.md README.md README.en.md
git commit -m "docs: add feishu browser bridge setup guide"
```
