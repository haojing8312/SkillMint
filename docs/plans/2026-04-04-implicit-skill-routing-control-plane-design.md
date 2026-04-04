# Implicit Skill Routing Control Plane Design

**Date:** 2026-04-04

## Goal

Make WorkClaw handle natural-language implicit skill requests with a dedicated routing control plane so desktop local chat can select and execute the right skill lane without paying the full generic agent-loop cost on every skill-shaped request.

## Scope

This design only covers WorkClaw desktop local chat runtime behavior.

Included:

- natural-language implicit skill routing before the generic agent loop
- structured routing decisions for prompt skills and deterministic skills
- dedicated execution runners for routed skill requests
- route-level observability and benchmarkability

Excluded:

- changes to `feishu-pm` or any other workspace skill content
- IM / Feishu / multi-employee orchestration changes
- broad fail-fast policy redesign
- packaged installer or release pipeline changes

## Problem Summary

WorkClaw can already execute skills, but it still behaves too much like a generic agent runtime. For natural-language requests that are obviously skill-shaped, the runtime often falls through to the open-ended turn loop before it decides whether a skill should handle the task.

That creates three costs:

1. The model spends time rediscovering a skill that the runtime could have shortlisted locally.
2. Prompt-following skills are treated like regular chat turns too late in the pipeline.
3. Slow-path debugging and route ambiguity are hard to diagnose because routing and execution signals are not first-class runtime outputs.

By contrast, the more complete `close-code` implementation uses a clearer split between:

- input classification
- command / skill routing
- specialized execution lanes
- generic open-task execution

WorkClaw needs the same kind of control plane.

## Design Principles

1. Route first, execute second.
2. Be conservative: only steal traffic from the generic loop when confidence is high.
3. Treat skill routing as structured runtime behavior, not prompt engineering.
4. Keep deterministic skill dispatch fast and isolated.
5. Make route decisions observable enough to compare against Codex-like behavior.

## Target Architecture

### 1. Dedicated Control Plane Before The Main Loop

Every user turn should first pass through a routing layer instead of dropping directly into the generic turn executor.

The new top-level flow should be:

1. user input arrives
2. `InvocationRouter` classifies the request
3. `RouteDecision` is produced
4. the matching runner executes
5. execution and route telemetry are recorded

Only `OpenTask` decisions should enter the generic agent loop.

### 2. Structured Route Decisions

The router should output a structured runtime enum rather than mutate prompts ad hoc.

Recommended decision types:

- `OpenTask`
- `PromptSkillInline`
- `PromptSkillFork`
- `DirectDispatchSkill`

This lets later runtime layers operate on stable semantics instead of rediscovering the request type.

### 3. Skill Route Index

WorkClaw should maintain a startup-built `SkillRouteIndex` derived from workspace skill runtime entries.

The index should contain:

- `skill_id`
- display name
- aliases
- description
- `when_to_use`
- family / domain tags
- invocation policy
- command-dispatch metadata
- execution mode (`inline`, `fork`, `dispatch`)
- allowed tools
- max iterations

The routing layer should consult this index instead of scanning raw skill files during each turn.

## Routing Strategy

### 1. Candidate Recall

First perform a cheap local recall pass without using the main model.

Inputs for recall should include:

- normalized user text
- skill id matches
- display name and alias matches
- `description`
- `when_to_use`
- family / domain vocabulary
- recent successful route history

The goal is not to decide immediately. The goal is to reduce the search space to a small candidate set.

### 2. Route Adjudication

Then perform a focused adjudication step over the recalled candidates.

The adjudicator should only answer one question:

- should this request be routed to a specific skill, or
- should it stay as an `OpenTask`

This stage should remain lightweight. It should not execute tools and should not construct the full generic system prompt.

Recommended behavior:

- high-confidence single winner: route directly
- low-confidence or near-tie: fallback to `OpenTask`
- empty candidate set: fallback to `OpenTask`

### 3. Family Weighting Without Hard Coding Outcomes

Domain words such as “飞书”, “日报”, “任务”, “汇总”, “项管”, “同步”, “多维表格” should be allowed to boost recall scores for relevant skill families.

This weighting should help recall, but should not force a final route by itself.

## Execution Model

### 1. Direct Dispatch Runner

If the routed skill is deterministic and already has a command-dispatch contract, execute it directly through the existing skill command dispatch path.

This lane should bypass the generic agent loop completely.

### 2. Prompt Skill Inline Runner

If the routed skill is a prompt-following inline skill:

- load the skill contract directly
- inject its system prompt
- narrow tools and max iterations
- execute with a dedicated prompt-skill runner

This should not be implemented by rewriting the user request into another text prompt and throwing it back into the generic loop.

### 3. Prompt Skill Fork Runner

If the routed skill is defined to run in a forked execution context, route directly to a dedicated fork runner.

This keeps parity with systems that treat forked skills as a first-class execution mode rather than as an afterthought.

### 4. Open Task Runner

If routing confidence is insufficient, hand off to the existing generic loop unchanged.

This preserves safety while the routing layer remains conservative.

## Fallback Policy

Fallback must be explicit and observable.

Recommended fallback conditions:

- no candidate recalled
- multiple candidates with no clear winner
- invalid or incomplete skill contract
- deterministic skill matched but required arguments cannot be safely resolved

Every fallback should produce a machine-readable reason, for example:

- `no_candidates`
- `ambiguous_candidates`
- `invalid_skill_contract`
- `dispatch_argument_resolution_failed`

## Module Layout

Recommended new runtime module group:

- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/mod.rs`
- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/intent.rs`
- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/index.rs`
- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/recall.rs`
- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/adjudicator.rs`
- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/runner.rs`
- `apps/runtime/src-tauri/src/agent/runtime/skill_routing/observability.rs`

Suggested responsibilities:

- `intent.rs`: route enums, confidence model, route payloads
- `index.rs`: build and cache `SkillRouteIndex`
- `recall.rs`: local candidate recall
- `adjudicator.rs`: final route selection
- `runner.rs`: map route decisions to runtime execution lanes
- `observability.rs`: route metrics, structured logging, reason codes

Existing modules should be simplified rather than expanded:

- `session_runtime.rs`: session orchestration only
- `tool_setup.rs`: execution tool preparation only
- `turn_executor.rs`: generic open-task loop only

## Observability

Route observability is part of the design, not a later enhancement.

Each turn should record:

- `route_ms`
- `candidate_count`
- `candidate_skill_ids`
- `selected_skill`
- `selected_runner`
- `confidence`
- `fallback_reason`
- `route_source` (`explicit_command`, `implicit_skill_route`, `open_task`)

Execution-level data should remain correlated with the route decision so performance analysis can answer:

- did routing happen
- which runner executed
- how much time was spent before first useful work

## Testing Strategy

### 1. Router Unit Tests

Add deterministic tests for:

- clear single-skill routing
- ambiguous competing candidates
- no candidate match
- prompt-skill versus deterministic-skill branching

### 2. Runner Integration Tests

Verify that:

- routed deterministic skills bypass the generic loop
- routed prompt skills receive narrowed runtime context
- `OpenTask` still reaches the generic loop
- fallback reasons are preserved

### 3. Real Benchmark Cases

Use fixed benchmark prompts such as:

- “帮我查询谢涛上周工作日报”
- “给郝敬创建一条测试任务，截至下周末”
- “同步项管部月度日报看板”

Track:

- total latency
- route latency
- first tool latency
- tool count
- turn count
- selected runner
- final route correctness

## Risks

### 1. Over-routing

If the recall layer is too aggressive, ordinary chat requests may get hijacked into a skill lane. The adjudicator and fallback policy must stay conservative.

### 2. Under-routing

If the router is too timid, WorkClaw will keep behaving like a generic loop and miss the point of this architecture. Benchmark prompts must be used to tune the threshold.

### 3. Mixed Responsibility Drift

If routing logic is left partially in `session_runtime.rs`, partially in `tool_setup.rs`, and partially in `turn_executor.rs`, the design will regress into the current muddled state. Routing should live in one module family.

## Success Criteria

1. Natural-language implicit skill requests are classified before the generic agent loop.
2. High-confidence skill-shaped requests reach dedicated runners instead of paying full generic-loop cost.
3. Prompt-following skills no longer depend on prompt-only rediscovery when routing confidence is high.
4. Deterministic skill dispatch remains fast and isolated.
5. Route decisions are observable enough to compare WorkClaw against Codex-like performance.

## Out Of Scope For This Design

- rewriting `feishu-pm` skills
- changing workspace skill content formats
- broad policy changes around fail-fast or error recovery
- release engineering and installer work
