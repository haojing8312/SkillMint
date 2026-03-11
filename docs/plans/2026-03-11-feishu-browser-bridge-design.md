# Feishu Browser Bridge Design

## Goal

Build a Chrome extension plus local bridge flow that configures a Feishu enterprise self-built app for ordinary users inside their default browser, then returns the collected credentials to local WorkClaw and completes the binding flow.

The target user should not need to understand:

- how to create a Feishu enterprise self-built app
- where to find App ID / App Secret
- how to add bot capability
- how to import the required permission JSON
- how to enable long-connection event receiving
- how to publish the app version

WorkClaw should orchestrate the setup. The browser should remain the user's own Chrome rather than an embedded desktop webview.

## Current State

WorkClaw already has two strong prerequisites:

- local browser automation primitives exposed through sidecar browser tools
- existing Feishu routing and local binding capabilities

Relevant existing pieces:

- `apps/runtime/src-tauri/src/agent/tools/browser_tools.rs`
- `apps/runtime/src-tauri/tests/test_browser_tools.rs`
- `apps/runtime/src/components/employees/FeishuRoutingWizard.tsx`
- `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
- `docs/integrations/feishu-routing.md`

However, the current browser automation model is desktop-owned Playwright orchestration. It is not yet a browser-native product flow running in the user's default Chrome session.

## Product Scope

### In Scope for MVP

- Chrome extension for `open.feishu.cn`
- Native Messaging bridge between Chrome and local WorkClaw
- guided setup for Feishu `企业自建应用`
- automatic page navigation and field filling after the user is logged in
- automatic import of the required permission JSON
- automatic reading and local return of `App ID` and `App Secret`
- local WorkClaw binding using those credentials
- browser-side continuation for long connection, event subscription, and version publishing
- resumable setup sessions with explicit step state

### Out of Scope for MVP

- non-Chrome browsers
- general-purpose browser agent for arbitrary sites
- Feishu app marketplace flow
- full unattended login
- bypassing enterprise admin approval
- remote cloud bridge for sensitive credentials

## User Flow

The intended guided path is:

1. User clicks `Configure Feishu Bot` in WorkClaw.
2. WorkClaw checks whether the Chrome extension and local native host are available.
3. WorkClaw opens `https://open.feishu.cn/` in the user's default Chrome.
4. The extension detects whether the user is logged in.
5. If login is required, automation pauses and the user logs in manually.
6. After login, the extension guides and executes:
   - open developer console
   - create `企业自建应用`
   - fill app name and app description
   - add `机器人` capability
   - import the required permission JSON
7. The extension navigates to `凭证与基础信息`, reads `App ID` and `App Secret`, and sends them to local WorkClaw.
8. WorkClaw performs the local bind operation.
9. If binding succeeds, WorkClaw tells the extension to continue:
   - open `事件与回调`
   - enable `长连接接受事件`
   - add the `接受消息` event
   - create and publish a version
10. WorkClaw shows the final prompt telling the user to add the bot to favorites in Feishu mobile.

## Chosen Approach

Use `Chrome extension + Native Messaging host + WorkClaw local orchestrator`.

This separates concerns cleanly:

- the extension owns browser-native page awareness
- the native host owns Chrome-to-local transport
- WorkClaw owns workflow state, sensitive local storage, and product logic

This is preferred over embedding a browser in the desktop app or moving the full workflow into the extension itself.

## Alternatives Considered

### 1. Reuse desktop-owned Playwright only

Pros:

- lowest engineering lift
- reuses current browser automation tools directly

Cons:

- does not operate in the user's default browser session
- awkward login and session continuity
- worse user trust and product feel

Rejected because the product requirement is browser-native execution in the user's own Chrome.

### 2. Put workflow logic mostly inside the extension

Pros:

- browser-first UX
- less local orchestration code initially

Cons:

- poor maintainability under MV3 lifecycle constraints
- weaker logging and recovery
- harder to extend for additional channels later

Rejected because the orchestration logic belongs in WorkClaw, not in a fragile extension runtime.

### 3. Chrome extension plus Native Messaging plus local orchestrator

Pros:

- matches the desired UX
- keeps sensitive handling local
- enables resumable workflow state
- fits current WorkClaw architecture best

Cons:

- requires extension packaging and native host registration
- Chrome-specific first release

Chosen.

## Architecture

The system has four layers.

### 1. Chrome Extension

Responsibilities:

- run only on `open.feishu.cn`
- detect current page and workflow step
- execute page actions
- collect visible app credentials
- present lightweight in-browser prompts when manual user action is needed
- communicate with the local native host

Suggested submodules:

- `background` / service worker
- `content script`
- step detector
- DOM action adapter
- in-page guidance overlay

### 2. Native Messaging Host

Responsibilities:

- receive requests from the extension
- forward them to local WorkClaw
- return WorkClaw responses back to the extension
- provide a narrow authenticated local message boundary

This layer should stay intentionally thin and avoid business logic.

### 3. WorkClaw Feishu Setup Orchestrator

Responsibilities:

- create setup sessions
- hold workflow state
- decide next step
- validate returned credentials
- run the local bind operation
- log step results and failures
- support recovery after browser close or user interruption

### 4. Existing Feishu Binding and Routing Layer

Responsibilities:

- persist credentials and settings
- connect the Feishu bot configuration into WorkClaw's existing channel and routing stack
- run connectivity validation where possible

## Workflow State Machine

WorkClaw should model setup as an explicit state machine.

Suggested states:

- `INIT`
- `EXTENSION_REQUIRED`
- `LOGIN_REQUIRED`
- `CREATE_APP`
- `ADD_BOT_CAPABILITY`
- `IMPORT_PERMISSIONS`
- `COLLECT_CREDENTIALS`
- `BIND_LOCAL`
- `ENABLE_LONG_CONNECTION`
- `ADD_MESSAGE_EVENT`
- `CREATE_AND_PUBLISH_VERSION`
- `DONE`
- `FAILED`

Rules:

- each state has a clear expected page shape
- each state emits structured progress events
- the workflow can pause and resume safely
- failures should not force a full restart unless the app itself was not created

## Security Model

This feature intentionally handles sensitive data. The system must behave like a constrained trusted local assistant rather than a broad browser monitor.

Hard requirements:

- extension host permissions restricted to `https://open.feishu.cn/*`
- explicit one-time user approval before credential extraction
- `App Secret` must be sent only to local WorkClaw
- extension must not persist secrets in extension storage
- local logs must redact secrets by default
- local credential storage must be encrypted at rest if the project already has a secure storage path
- the website itself must not be able to command the extension to exfiltrate data
- every setup session must use a fresh session token

Recommended audit fields:

- `session_id`
- `current_step`
- `page_url`
- `action_name`
- `result`
- `error_code`
- `captured_at`

Secrets must never appear in audit logs.

## Page Automation Strategy

The browser product flow should not depend on brittle fixed selectors alone.

Preferred strategy:

1. Use step-specific semantic detectors first.
2. Fall back to text-based and structural matching.
3. Use resilient selector groups for known Feishu page variants.
4. If a page cannot be identified safely, pause and ask the user to navigate manually to the expected section.

Automation should be deterministic for the documented MVP path only.

## Permission JSON

The MVP should ship with the exact JSON required for the Feishu bot path:

```json
{
  "scopes": {
    "tenant": [
      "contact:user.base:readonly",
      "im:message",
      "im:message.group_at_msg:readonly",
      "im:message.p2p_msg:readonly",
      "im:message:send_as_bot",
      "im:resource"
    ],
    "user": []
  }
}
```

This JSON should be versioned in WorkClaw rather than hardcoded in multiple places.

## Error Handling and Recovery

The workflow must be resumable.

Typical failure cases:

- user not logged in
- Feishu page layout drift
- enterprise admin approval required
- credential collection failed
- local bind failed
- long connection save failed because credentials were not bound successfully
- browser tab closed mid-flow

Recovery behavior:

- keep session state locally
- allow `Retry current step`
- allow `Open required page and continue`
- allow `Re-read credentials`
- show actionable explanations tied to the current failed step

The system should optimize for recoverability rather than full hidden automation.

## UX Principles

- do not open Feishu inside the desktop app
- do not surprise the user with silent credential capture
- do not ask the user to understand internal terms like routing or callback internals during the guided flow
- use plain language such as `正在创建飞书应用`, `请先登录飞书`, `正在读取 App ID 和 App Secret`
- when manual input is required, freeze automation clearly and explain what the user needs to do

## Suggested Implementation Units

1. Chrome extension shell and host permissions
2. Native Messaging host registration and message schema
3. Local setup session store and orchestrator
4. Feishu page step detector
5. Credential extraction and secure handoff
6. Existing local bind integration
7. Resume and retry UX
8. E2E test harness with mocked Feishu pages

## Testing Strategy

### Unit Tests

- state transitions
- message schema validation
- step detectors
- log redaction

### Integration Tests

- extension-to-host message exchange
- host-to-WorkClaw orchestration loop
- credential return and local bind success/failure paths

### End-to-End Tests

Use mocked Feishu management pages rather than live Feishu as the CI baseline.

Cover:

- login-required pause
- happy path from app creation to publish
- bind failure and retry
- tab closed and resumed
- page mismatch requiring manual user correction

Manual regression on real Feishu pages should remain a release checklist item rather than a required CI dependency.

## Open Questions

- exact secure storage path for `App Secret` if current secure storage is insufficient
- whether Chrome is guaranteed to be the system default browser on all supported Windows installs
- whether enterprise-admin review states need a dedicated paused state in MVP
- how much in-browser overlay UI is needed versus WorkClaw desktop status updates

## Recommendation

Proceed with a Chrome-only MVP using:

- Chrome extension on `open.feishu.cn`
- Native Messaging host
- local WorkClaw setup orchestrator
- explicit credential extraction consent
- resumable deterministic workflow for the enterprise self-built app path only

This gives the strongest fit for the requested UX while keeping sensitive handling local and auditable.
