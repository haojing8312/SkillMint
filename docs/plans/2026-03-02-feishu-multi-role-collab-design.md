# 2026-03-02 Feishu Multi-Role Collaboration Design

## 1. Background and Goal

This upgrade targets one core MVP objective:
- ordinary users install the desktop app
- connect Feishu via enterprise app + callback webhook
- bind multiple role agents into one Feishu group
- see real-time multi-agent discussion in the same group
- interrupt and steer discussion at any time

The first business scenario template is `商机评审`.

Non-goal in this MVP:
- end-to-end autonomous R&D execution pipeline
- multi-IM full rollout (WeCom/DingTalk are reserved by architecture)

## 2. Product Scope (MVP)

### 2.1 Must Have

- Feishu enterprise app callback integration
- one group mapped to multiple role agents
- orchestrated role discussion in group
- real-time process visibility (streaming summaries/events)
- human interruption controls (`@role`, `pause`, `resume`, `override`)
- OpenClaw-style memory with multi-role/multi-session isolation

### 2.2 Initial Role Templates

- Presales Agent
- Project Manager Agent
- Business Consultant Agent
- Architect Agent

## 3. Architecture

Use replaceable 4-layer architecture:

1. `IM Gateway` (Feishu first)
- receives callback events
- emits normalized internal events
- sends text/cards back to group

2. `Conversation Orchestrator`
- maps group thread to role set
- controls turn-taking and discussion stages
- resolves interruption priority and convergence

3. `Agent Runtime Bridge`
- dispatches role tasks to existing runtime (`task_tool`, sub-agent chain)
- consumes runtime streaming events (`stream-token`, `agent-state-event`)
- translates them to IM-ready output

4. `Memory & Role Profile`
- role profile and permissions
- session memory and organizational case memory
- recall/capture pipelines and audit metadata

OpenClaw can be used early as an integration accelerator at gateway boundary, but core orchestration and multi-role memory remain in this project.

## 4. Runtime Flow

1. Feishu callback arrives at gateway
2. gateway normalizes event into `ImEvent`
3. orchestrator updates stage and decides next speaker(s)
4. bridge dispatches task(s) to runtime
5. runtime streams progress/events
6. bridge publishes role updates to Feishu group in real time
7. human instruction can interrupt at any step and preempt automated turn order

## 5. Interruption and Control Policy

Priority from high to low:

1. `human.override` (direct decision/constraint)
2. `pause` / `resume`
3. `@role` priority mention
4. automated stage policy

Required behavior:
- no hidden state changes after pause
- every override is persisted in session memory with author/timestamp
- orchestrator emits explicit state transition event for observability

## 6. Memory Model (OpenClaw-style, upgraded)

Use file-first layered memory:

- `daily` log: append-only timeline
- `session` memory: thread-scoped working memory
- `role` long-term memory: stable role-specific preferences/rules
- `org` casebook: cross-role shared delivery history and case references

### 6.1 Suggested Storage Layout

`<app-data>/memory/`
- `daily/YYYY-MM-DD.md`
- `sessions/<im_thread_id>.md`
- `roles/<role_id>/MEMORY.md`
- `org/CASEBOOK.md`

### 6.2 Recall/Capture

- auto-recall before each role response:
  - role memory + session memory + relevant casebook snippets
- auto-capture after each stage:
  - confirmed facts, decisions, risks, and TODOs
- manual capture command from human:
  - force-write selected statement into chosen memory layer

### 6.3 Write Gate

Only write long-term layers when confidence and confirmation checks pass.
Unconfirmed hypotheses stay in session memory.

All writes carry metadata:
- source message id
- role author
- confidence
- created_at
- optional revision_of id

## 7. Security and Permission Model

- role tools are controlled by allowlist
- effective role permission is intersection of:
  - tenant policy
  - session policy
  - role allowlist
- any denied action must produce explicit user-visible feedback

## 8. UX and Observability

Feishu group:
- concise role messages with structured format:
  - `结论`
  - `依据`
  - `不确定项`
  - `下一步`

Desktop app:
- full trace panel for:
  - role timeline
  - state transitions
  - latency and failure nodes

## 9. Failure Handling

- role timeout: mark degraded, continue with fallback role/human prompt
- permission denied: explicit denial reason in group + desktop trace
- callback transient failure: retry with idempotency key
- duplicate callbacks: dedupe by event id

## 10. Delivery Plan

Week 1:
- gateway + orchestrator + runtime bridge integration
- real-time group updates and interruption controls

Week 2:
- memory layers and recall/capture automation
- `商机评审` scenario template
- reliability hardening and end-to-end tests

