# OpenClaw IM Reuse Rearchitecture Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-architect WorkClaw's IM pipeline so OpenClaw becomes the source of truth for agent/session/conversation behavior, while WorkClaw keeps only the minimum adapter, desktop bridge, and product-specific runtime surfaces.

**Architecture:** Collapse WorkClaw's custom IM session semantics into an OpenClaw-first model. Treat `employee` as `agent`, move channel/session/conversation routing toward OpenClaw-compatible boundaries, shrink `sidecar adapter` into channel normalization plus outbound delivery, and shrink `im host bridge` into a transport bridge from normalized channel events into agent turns. Prefer replacement over compatibility layers when the old WorkClaw design conflicts with OpenClaw semantics.

**Tech Stack:** Rust, Tauri runtime, TypeScript sidecar, OpenClaw reference implementation under `references/openclaw`, SQLite (schema replacement allowed), existing WorkClaw desktop runtime and event bridge

---

## Strategy Summary
- Change surface: IM ingress, agent/session routing, session persistence, desktop event bridge, sidecar channel adapters, database schema, and any UI/runtime code that assumes `employee` is distinct from `agent`.
- Affected modules:
  - `apps/runtime/src-tauri/src/commands/employee_agents*`
  - `apps/runtime/src-tauri/src/commands/im_host/*`
  - `apps/runtime/src-tauri/src/commands/channel_connectors.rs`
  - `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
  - `apps/runtime/src-tauri/src/commands/openclaw_plugins/*`
  - `apps/runtime/src-tauri/src/im/*`
  - `apps/runtime/src-tauri/src/db/*`
  - `apps/runtime/sidecar/src/adapters/*`
  - `apps/runtime/sidecar/src/openclaw-bridge/*`
  - `references/openclaw/docs/*`
  - `references/openclaw/src/*` for session/channel/binding semantics
- Main risk: WorkClaw currently duplicates some OpenClaw concepts in different layers. If we partially merge semantics, we will keep two competing session models and preserve the exact bug class we are trying to remove.
- Recommended smallest safe path: Stop incremental compatibility-first IM session work. First define a target architecture and replacement boundaries, then migrate ingress and persistence in a controlled sequence. Keep only short-lived compatibility shims at the outer edges.
- Required verification:
  - `pnpm --dir apps/runtime/sidecar test`
  - `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_route_session_mapping -- --nocapture`
  - `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_feishu_conversation_identity -- --nocapture`
  - `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_normalized_im_conversation_identity -- --nocapture`
  - Follow-up migration-specific regression tests for new schema/bootstrap
- Release impact: High for IM behavior and session continuity. This is a runtime behavior refactor, not an internal cleanup.

---

## Current-State Diagnosis

### 1. `employee` and `agent` are conceptually overlapping today

WorkClaw currently uses `employee` as its user-facing agent abstraction, but that layer also owns IM dispatch, session binding, and some routing logic.

Evidence:
- [employee_agents.rs](/D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents.rs)
- [employee_agents/service.rs](/D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/service.rs)
- [employee_agents/session_service.rs](/D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs)

Practical effect:
- `employee` is not just persona/config; it also acts like OpenClaw's `agent` plus part of the session router.
- That coupling makes it hard to reuse OpenClaw session semantics directly.

### 2. `sidecar adapter` is a channel integration layer

The sidecar adapter layer is the correct place for channel-specific code. It should:
- connect to Feishu / WeCom / DingTalk / etc.
- normalize inbound payloads
- send outbound messages
- optionally retain replay/debug metadata

It should not own agent/session semantics beyond what is required to carry routing metadata.

Evidence:
- [apps/runtime/sidecar/src/adapters/kernel.ts](/D:/code/WorkClaw/apps/runtime/sidecar/src/adapters/kernel.ts)
- [apps/runtime/sidecar/src/adapters/types.ts](/D:/code/WorkClaw/apps/runtime/sidecar/src/adapters/types.ts)
- [apps/runtime/sidecar/src/adapters/wecom/index.ts](/D:/code/WorkClaw/apps/runtime/sidecar/src/adapters/wecom/index.ts)

### 3. `im host bridge` is a transport bridge into the desktop runtime

The `im_host` layer currently takes normalized channel events and turns them into WorkClaw runtime events.

Evidence:
- [inbound_bridge.rs](/D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs)
- [channel_connectors.rs](/D:/code/WorkClaw/apps/runtime/src-tauri/src/commands/channel_connectors.rs)

Its correct long-term role should be narrow:
- parse normalized channel events
- forward them into the agent runtime
- emit reply/lifecycle events back to the desktop runtime

It should not permanently own a second, WorkClaw-specific IM session model if OpenClaw already has one.

### 4. WorkClaw already has an OpenClaw routing foothold, but not a full semantic takeover

There is already an `openclaw-bridge` lane in the sidecar:
- [route-engine.ts](/D:/code/WorkClaw/apps/runtime/sidecar/src/openclaw-bridge/route-engine.ts)

This is good news: WorkClaw is already structurally willing to import OpenClaw routing logic.

The problem is that the Tauri runtime still reconstructs too much local IM/session behavior after ingress.

---

## Target Architecture

### Core Principle

OpenClaw owns:
- agent identity
- session identity
- conversation identity
- IM/channel routing semantics
- session reset/continuation rules
- compaction semantics

WorkClaw owns:
- desktop runtime shell
- product-specific UI state
- local journal/telemetry surfaces
- thin channel adapters
- thin bridge from normalized channel events to the runtime

### Target Layering

1. **Channel Adapter Layer**
- Input: native channel payloads
- Output: normalized OpenClaw-compatible event envelopes
- Responsibility: channel-specific I/O only

2. **Conversation/Session Resolution Layer**
- Input: normalized channel events
- Output: OpenClaw agent target + session key + conversation binding context
- Responsibility: OpenClaw semantics, not WorkClaw-local heuristics

3. **Runtime Bridge Layer**
- Input: resolved agent/session turn request
- Output: desktop runtime execution and reply delivery signals
- Responsibility: bridge only, not session policy

4. **Persistence Layer**
- Input: OpenClaw-style conversation/session/agent metadata
- Output: durable tables for local startup, reply routing, observability
- Responsibility: storage and projections, not primary behavior definition

---

## What Should Be Reused From OpenClaw

### Reuse directly when possible

- Agent-scoped session key semantics
- Channel conversation binding semantics
- Topic/thread parent linkage semantics
- Reset/continue semantics
- Long-session compaction semantics
- Session-to-channel delivery metadata model

Reference anchors:
- [agent-send.md](/D:/code/WorkClaw/references/openclaw/docs/tools/agent-send.md)
- `references/openclaw/docs/cli/acp.md`
- `references/openclaw/docs/channels/*`
- `references/openclaw/src/auto-reply/reply/*`
- `references/openclaw/src/routing/*`
- `references/openclaw/src/infra/outbound/*`

### Reuse conceptually, but adapt to WorkClaw runtime boundaries

- Gateway/session metadata projections
- Binding-service APIs
- Conversation focus / refocus model
- Channel-specific thread/topic session policies

These may need adaptation because WorkClaw is a desktop runtime with Tauri rather than the full OpenClaw gateway shape.

### Delete or shrink WorkClaw-local semantics when they conflict

- `employee` as a separate session-routing abstraction
- coarse `thread_id`-first IM binding logic
- route-key based session reuse heuristics
- multiple local layers independently deriving conversation/session semantics

---

## Modules To Keep, Shrink, Replace, Or Delete

### Keep, but shrink

#### `apps/runtime/sidecar/src/adapters/*`
Keep as channel adapters.

Shrink to:
- native channel I/O
- normalized event output
- outbound delivery
- optional local capture/debug tooling

Do not let adapters own session policy beyond carrying metadata.

#### `apps/runtime/src-tauri/src/commands/im_host/*`
Keep as runtime bridge.

Shrink to:
- parse normalized event
- forward to the runtime turn entrypoint
- map runtime reply/lifecycle signals back to the channel layer

Remove long-term ownership of WorkClaw-specific IM session policy.

### Replace

#### `employee_agents/session_service.rs`
Replace with an OpenClaw-compatible agent/session binding service.

Current problem:
- it still acts like a local authority for session binding and reuse.

Target:
- agent/session binding should be driven by OpenClaw semantics and persisted locally as a projection.

#### `employee_agents.rs`
Split into:
- agent definition / catalog concerns
- team/group orchestration concerns
- IM routing concerns removed or delegated

Current file still mixes unrelated concerns and should not remain the root of IM semantics.

### Delete or deprecate

#### Legacy thread-first schema surfaces
- `im_thread_sessions` as the primary source of truth
- any route-key-only session lookup
- any fallback that reconstructs WorkClaw-local session semantics after OpenClaw routing already decided the target

We may keep transitional readers briefly, but the architecture target is to remove them.

---

## Proposed Replacement Data Model

Since schema compatibility is no longer a priority, prefer a new set of tables instead of endlessly mutating `im_thread_sessions`.

### 1. `agent_conversation_bindings`

Purpose:
- bind a concrete channel conversation to a concrete agent/session target

Suggested shape:

```sql
CREATE TABLE agent_conversation_bindings (
  conversation_id TEXT NOT NULL,
  channel TEXT NOT NULL,
  account_id TEXT NOT NULL DEFAULT '',
  agent_id TEXT NOT NULL,
  session_key TEXT NOT NULL,
  session_id TEXT NOT NULL DEFAULT '',
  base_conversation_id TEXT NOT NULL DEFAULT '',
  parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]',
  scope TEXT NOT NULL DEFAULT '',
  peer_kind TEXT NOT NULL DEFAULT '',
  peer_id TEXT NOT NULL DEFAULT '',
  topic_id TEXT NOT NULL DEFAULT '',
  sender_id TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (conversation_id, agent_id)
);
```

### 2. `channel_delivery_routes`

Purpose:
- track how replies for a session should be delivered back to a channel

Suggested shape:

```sql
CREATE TABLE channel_delivery_routes (
  session_key TEXT NOT NULL PRIMARY KEY,
  channel TEXT NOT NULL,
  account_id TEXT NOT NULL DEFAULT '',
  conversation_id TEXT NOT NULL,
  reply_target TEXT NOT NULL DEFAULT '',
  updated_at TEXT NOT NULL
);
```

### 3. `agent_session_projection`

Purpose:
- local projection of OpenClaw agent/session state for desktop UI and startup

Suggested shape:

```sql
CREATE TABLE agent_session_projection (
  session_key TEXT NOT NULL PRIMARY KEY,
  agent_id TEXT NOT NULL,
  session_id TEXT NOT NULL DEFAULT '',
  channel TEXT NOT NULL DEFAULT '',
  conversation_id TEXT NOT NULL DEFAULT '',
  last_message_at TEXT NOT NULL DEFAULT '',
  last_reply_at TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'idle',
  updated_at TEXT NOT NULL
);
```

### What to retire

- `im_thread_sessions` as primary session lookup
- `thread_id + employee_id` as identity
- route-session-key-driven recovery

---

## Required Naming Shift

To align the codebase with the architecture:

- `employee` should become a presentation/domain alias of `agent`
- internal runtime/session code should prefer `agent_id`
- `employee_id` should remain only where UI or migration compatibility still needs it temporarily

Recommended transitional rule:
- new core APIs use `agent_id`
- old `employee_id` fields become compatibility aliases or projections

---

## Migration Phases

### Phase 0: Freeze further compatibility-first IM work

Do not add more local session heuristics to:
- `employee_agents/session_service.rs`
- `im_host/inbound_bridge.rs`
- legacy thread-binding tables

Only allow bugfixes or tooling additions until the target architecture lands.

### Phase 1: Introduce a new agent/session binding core

Create a new module set, for example:
- `apps/runtime/src-tauri/src/im/agent_session_binding.rs`
- `apps/runtime/src-tauri/src/im/conversation_binding_store.rs`
- `apps/runtime/src-tauri/src/im/channel_delivery_store.rs`

This layer becomes the only authority for:
- conversation to session binding
- session to channel reply route
- agent/session projection

### Phase 2: Demote `employee_agents` out of IM authority

Move IM binding responsibilities out of `employee_agents`.

Keep `employee_agents` only for:
- agent catalog/persona config
- group/team orchestration
- user-facing configuration surfaces

### Phase 3: Make `im_host` a thin bridge

`inbound_bridge.rs` should:
- accept normalized event
- call the new binding core
- enqueue a runtime turn for the resolved `agent_id/session_key`

It should not derive fallback session semantics beyond minimal normalization.

### Phase 4: Replace the legacy schema

Introduce the new binding tables.

Then:
- migrate or discard old IM thread binding data
- update startup and lifecycle lookups to read new tables only
- remove `im_thread_sessions` as authoritative storage

### Phase 5: Rework compaction after session ownership is stable

Once OpenClaw-compatible session ownership is the source of truth:
- rework WorkClaw compaction semantics to align with OpenClaw
- stop treating compaction as an isolated WorkClaw concern

---

## Implementation Tasks

### Task 1: Write and align the replacement architecture contract

**Files:**
- Create: `docs/architecture/openclaw-im-reuse.md`
- Modify: `docs/superpowers/plans/2026-04-22-openclaw-im-reuse-rearchitecture-plan.md`

- [ ] Write a concise architecture doc that defines:
  - `agent == employee` mapping
  - sidecar adapter responsibility
  - im host bridge responsibility
  - OpenClaw-owned semantics vs WorkClaw-owned surfaces
- [ ] Add a table of “keep / shrink / replace / delete” modules.
- [ ] Add the target storage model and migration intent.
- [ ] Commit with: `docs: define openclaw-first IM architecture`

### Task 2: Introduce agent-first naming in the IM core

**Files:**
- Create: `apps/runtime/src-tauri/src/im/agent_identity.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/types.rs`
- Modify: `apps/runtime/src-tauri/src/im/types.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

- [ ] Add a small agent identity layer that normalizes `agent_id` and explicitly maps `employee_id` as compatibility data.
- [ ] Update new IM/session code paths to prefer `agent_id`.
- [ ] Keep old UI-facing structures compiling via compatibility aliases where necessary.
- [ ] Add regression coverage showing `employee` aliases still resolve to the same agent identity.
- [ ] Commit with: `refactor: introduce agent-first IM identity`

### Task 3: Add the replacement binding store

**Files:**
- Create: `apps/runtime/src-tauri/src/im/agent_session_binding.rs`
- Create: `apps/runtime/src-tauri/src/im/conversation_binding_store.rs`
- Modify: `apps/runtime/src-tauri/src/db/schema.rs`
- Modify: `apps/runtime/src-tauri/src/db/migrations.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

- [ ] Create new `agent_conversation_bindings` and `channel_delivery_routes` tables.
- [ ] Add repository/store methods that read/write only the new tables.
- [ ] Add migration tests that prove a fresh database works and that legacy tables are no longer required for new bindings.
- [ ] Commit with: `feat: add openclaw-style conversation binding store`

### Task 4: Reroute ingress through the new binding core

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs`
- Modify: `apps/runtime/src-tauri/src/commands/channel_connectors.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_host/lifecycle.rs`
- Test: `apps/runtime/src-tauri/tests/test_normalized_im_conversation_identity.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

- [ ] Replace direct `employee_agents` binding resolution with the new IM binding core.
- [ ] Keep `im_host` as a bridge that produces runtime turn dispatches only.
- [ ] Update lifecycle lookups to use the new tables.
- [ ] Commit with: `refactor: thin im host bridge to openclaw-style binding flow`

### Task 5: Demote `employee_agents` from IM routing authority

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

- [ ] Remove IM-primary session resolution responsibilities from `employee_agents`.
- [ ] Keep only catalog/group/team responsibilities.
- [ ] Route all IM session resolution through the new binding store.
- [ ] Commit with: `refactor: remove employee_agents from primary IM session routing`

### Task 6: Reduce sidecar adapters to channel-only concerns

**Files:**
- Modify: `apps/runtime/sidecar/src/adapters/*`
- Modify: `apps/runtime/sidecar/src/openclaw-bridge/*`
- Test: `apps/runtime/sidecar/test/*.test.ts`

- [ ] Ensure adapters only produce normalized events and outbound delivery calls.
- [ ] Keep optional capture/sanitize tooling local to the adapter boundary.
- [ ] Remove any adapter-side logic that tries to become the session authority.
- [ ] Commit with: `refactor: keep sidecar adapters channel-only`

### Task 7: Replace legacy thread-first lookups

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/im_host/lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/commands/wecom_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway/*`
- Test: `apps/runtime/src-tauri/tests/test_im_host_windows_regressions.rs`

- [ ] Replace remaining thread-first reply and lifecycle lookups with conversation/session-key aware lookups.
- [ ] Verify reply routing still works for Feishu and WeCom.
- [ ] Commit with: `refactor: replace thread-first IM reply lookups`

### Task 8: Revisit compaction with OpenClaw semantics

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/*`
- Reference: `references/openclaw/src/auto-reply/reply/*`
- Test: targeted runtime/session tests to be defined during implementation

- [ ] Compare current WorkClaw compaction semantics with OpenClaw's session compaction behavior.
- [ ] Replace the “summary-only overwrite” pattern where it conflicts with OpenClaw semantics.
- [ ] Commit with: `refactor: align compaction with openclaw session semantics`

---

## Open Questions To Decide Before Implementation

1. Should WorkClaw keep the word `employee` in the user-facing UI while the runtime switches fully to `agent_id` internally?
2. Should WorkClaw continue to persist local session projections in SQLite, or should some of that state become a cached mirror of upstream/OpenClaw stores?
3. Do we want a hard cutover migration that discards legacy IM thread bindings, or a short-lived import step into the new tables?
4. Which OpenClaw channel/session modules are stable enough to vendor directly versus only copying semantics from the reference repo?

---

## Recommended Execution Order

1. Approve the architecture direction in this plan.
2. Write the short architecture doc.
3. Build the new binding core and tables.
4. Thin `im_host`.
5. Demote `employee_agents`.
6. Cut over reply/lifecycle lookups.
7. Revisit compaction.

---

## Verification Summary For This Planning Step

- Changed surface: planning/docs only
- Commands run:
  - code inspection over `apps/runtime/src-tauri/src/commands/im_host/*`
  - code inspection over `apps/runtime/src-tauri/src/commands/employee_agents*`
  - code inspection over `apps/runtime/sidecar/src/adapters/*`
  - code inspection over `apps/runtime/sidecar/src/openclaw-bridge/*`
  - code inspection over `references/openclaw/docs/*`
- Results: enough evidence gathered to define a rearchitecture plan
- Covered areas: current WorkClaw ingress/session structure and OpenClaw reference semantics
- Still unverified: exact upstream OpenClaw modules to vendor directly during implementation
- Verification verdict: valid for planning, not an implementation completion claim
