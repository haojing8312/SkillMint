# Local Skill Fast Path And Exec Design

**Date:** 2026-03-31

## Goal

Improve WorkClaw desktop local-chat skill execution so explicit natural-language skill requests run closer to Codex behavior: fewer model turns, less prompt-only wandering, and more reliable Windows command execution.

## Scope

This design only covers the desktop local chat execution path.

Included:

- explicit natural-language skill targeting before the main model loop
- Windows-native `exec` behavior for deterministic skill commands
- traceable runtime behavior for the new fast path

Excluded for now:

- IM / Feishu / multi-employee orchestration
- fail-fast policy changes after tool failure
- skill asset rewrites outside the runtime compatibility needed for this path

## Problem Summary

The current WorkClaw desktop runtime is slower than Codex for the same PM skill request because two expensive steps happen before useful work starts:

1. Natural-language requests still enter the generic model-routing path, which means the model must discover the correct skill from the prompt catalog.
2. Deterministic skill commands ultimately execute through `exec -> bash -> cmd /C` on Windows, which is a weak fit for PowerShell-heavy skill commands.

The captured Codex session shows the successful path is effectively:

- identify the intended skill immediately
- run one stable PowerShell command
- return the result

The captured WorkClaw session shows a much slower path:

- model reads skill catalog
- model chooses or re-chooses a skill
- shell execution is brittle on Windows
- retries and exploratory behavior consume turns

## Root Causes

### 1. No natural-language skill fast path

`SessionRuntime` only auto-dispatches skills when the user sends slash-command input such as `/pm_summary ...`. Ordinary natural-language requests are still sent through the main model loop with a prompt that asks the model to inspect the available skills.

That means a user message such as “使用 feishu-pm-hub 技能帮我查询谢涛上周日报” still pays the full model-discovery cost before any actual skill work starts.

### 2. `exec` is not a first-class Windows execution tool

Today `exec` is only an alias of `bash`, and on Windows `bash` runs through `cmd /C`. That creates an unnecessary shell hop for deterministic skill commands that are already written as PowerShell invocations.

For PM skills this is especially costly because the stable path is usually a PowerShell script entrypoint.

### 3. Explicit skill intent is not preserved as runtime structure

WorkClaw already has enough data to represent a skill as a structured runtime entry, including `skill_id`, invocation policy, and command dispatch metadata. But the desktop runtime does not yet use that structure to short-circuit the normal chat route when the user explicitly names a skill.

## Target Architecture

### 1. Add a desktop-only explicit skill selector before the model loop

Before normal model execution begins, the local chat runtime should inspect the user message for high-confidence explicit skill references.

The first version should be intentionally conservative:

- only trigger for explicit mentions, not broad intent classification
- only trigger when exactly one workspace skill matches with high confidence
- otherwise fall back to the current model-driven path unchanged

High-confidence signals include:

- exact `skill_id` mention such as `feishu-pm-hub`
- exact slash command reference such as `/pm_summary`
- explicit phrases such as `使用 xxx 技能`, `调用 xxx skill`, `用 xxx skill`

### 2. Split fast-path handling by skill type

If the explicit target skill is prompt-following:

- do not ask the model to rediscover it from the skill catalog
- instead, override the current turn so the selected skill becomes the direct runtime skill context for this request
- preserve the skill's narrowed tool policy and `max_iterations`

If the explicit target skill is dispatchable and the user input is slash-command style:

- keep the existing deterministic dispatch behavior

If the explicit target skill is dispatchable but the user input is natural language:

- do not invent command arguments locally
- fall back to the selected prompt-following skill context if available
- otherwise fall back to the normal model path

This keeps the first version safe and avoids brittle argument extraction.

### 3. Introduce a first-class `exec` tool

`exec` should stop being only an alias to `bash`.

The new `exec` tool should:

- keep the same structured output shape as `bash`
- accept the same input contract used by skill command dispatch
- on Windows, execute commands via PowerShell-compatible semantics instead of `cmd /C`
- preserve work directory, timeout, and background-process support

`bash` should remain available for general shell use, but deterministic skill commands that declare `command-tool: exec` should gain a more reliable runtime path automatically.

### 4. Keep existing slash-command behavior stable

Current `/skill_command ...` dispatch behavior must continue to work exactly as today. The new natural-language fast path is additive and should not break slash-command users.

### 5. Add explicit runtime observability

The runtime should make it obvious when a turn used:

- ordinary model-driven skill discovery
- explicit natural-language skill fast path
- slash-command deterministic dispatch

This observability is required so future regressions can be diagnosed without guessing.

## Affected Surfaces

- `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`
- `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- `apps/runtime/src-tauri/src/agent/runtime/tool_dispatch.rs`
- `apps/runtime/src-tauri/src/agent/tools/mod.rs`
- `apps/runtime/src-tauri/src/agent/tools/bash.rs`
- `apps/runtime/src-tauri/src/agent/types.rs`
- `apps/runtime/src-tauri/src/agent/execution_caps.rs`
- `apps/runtime/src-tauri/tests/test_bash.rs`
- `apps/runtime/src-tauri/tests/test_tool_aliases.rs`
- runtime tests near `session_runtime.rs` and `tool_dispatch.rs`

## Risks

### Behavior risk

A weak explicit-match rule could route normal user requests into the wrong skill. The matcher should remain conservative and require a strong explicit reference.

### Compatibility risk

Some existing skills may assume `exec` behaves exactly like `bash`. The new `exec` tool should preserve the same result format and only change the execution lane.

### Windows quoting risk

PowerShell command execution is sensitive to quoting and escaping. The first implementation should prefer a simple, well-defined PowerShell `-Command` execution path rather than building a broad shell parser.

## Success Criteria

1. A desktop local-chat message that explicitly names one skill no longer depends on prompt-only skill discovery.
2. The selected prompt-following skill executes in a narrower, more direct runtime context.
3. Windows deterministic skill commands run through a first-class `exec` tool instead of `cmd /C`.
4. The PM weekly summary path on WorkClaw gets materially closer to Codex in turns and latency.
5. Existing slash-command behavior still works.

## Verification Expectations

Implementation should include:

- unit tests for explicit natural-language skill matching
- runtime tests for explicit skill fast-path selection
- tests proving slash-command dispatch still works
- Windows-oriented `exec` tests covering PowerShell execution metadata and result shape
- `pnpm test:rust-fast` for touched runtime surfaces

## Release Impact

This is a runtime behavior change for the desktop app. It is not a release-lane or installer change, but it is user-visible and should be treated as a high-sensitivity runtime improvement.
