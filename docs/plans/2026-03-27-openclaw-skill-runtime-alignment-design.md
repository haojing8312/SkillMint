# OpenClaw Skill Runtime Alignment Design

**Date:** 2026-03-27

## Goal

Rebuild WorkClaw's skill runtime around the OpenClaw skill model so that `SKILL.md` frontmatter becomes the only authoritative skill runtime contract, skill invocation can support structured command dispatch, and script-oriented skills stop degrading into slow prompt-only exploratory runs.

## Why This Change

WorkClaw currently treats local skills primarily as prompt text. The runtime reads `SKILL.md`, extracts a narrow frontmatter subset, and returns instruction text to the model. This works for descriptive skills, but it fails for script-oriented skills because the runtime has no first-class understanding of:

- invocation policy
- dependency requirements
- command dispatch metadata
- deterministic tool routing

That gap causes models to inspect files, probe environments, and write ad hoc debug scripts instead of executing the intended workflow.

## Target Model

Adopt the OpenClaw pattern as the baseline:

1. `SKILL.md` is the single runtime contract.
2. Frontmatter carries machine-readable invocation metadata.
3. The runtime builds a structured skill catalog from skill directories.
4. Prompt assembly uses the catalog for discovery and selection.
5. Skill commands can dispatch directly to tools using frontmatter-defined metadata.

## Non-Goals

- Do not preserve the current `agents/openai.yaml`-driven behavior as a runtime authority.
- Do not add a separate WorkClaw-only execution engine format in this refactor.
- Do not keep "instruction_only because no allowed_tools" as the core skill executability model.

## Architecture Changes

### 1. Expand Skill Frontmatter Parsing

Move WorkClaw `SkillConfig` toward the OpenClaw model:

- invocation policy:
  - `user-invocable`
  - `disable-model-invocation`
- metadata:
  - `metadata.openclaw`
  - `requires`
  - `primaryEnv`
  - `install`
- command dispatch:
  - `command-dispatch`
  - `command-tool`
  - `command-arg-mode`

`allowed_tools` can remain temporarily for internal policy narrowing, but it should no longer define whether a skill is fundamentally "real" or "instruction only".

### 2. Introduce Skill Entry / Snapshot Layer

WorkClaw should stop treating installed skills as raw markdown blobs. Instead, it should build a structured skill entry list from projected workspace skills, similar to OpenClaw's skill workspace loader:

- skill identity
- projected path
- parsed frontmatter
- invocation policy
- metadata
- prompt-visible skill fields
- optional command dispatch spec

This snapshot becomes the canonical source for:

- prompt construction
- display name resolution
- user-invocable command registration
- skill selection and dispatch

### 3. Replace Prompt-Only Skill Invocation

The `skill` tool should no longer only return "here is the raw skill text". It should return structured skill resolution data:

- skill metadata summary
- invocation policy
- command dispatch spec if present
- system prompt body for prompt-following cases

The runtime can then choose between:

- prompt-following skill flow
- structured tool dispatch flow

### 4. Add OpenClaw-Style Command Dispatch

For skills with:

- `command-dispatch: tool`
- `command-tool: <tool-name>`

WorkClaw should support deterministic dispatch without asking the model to infer the command from prose. The first version should mirror OpenClaw's strictness:

- support only `tool` dispatch
- support only `raw` arg forwarding
- reject or ignore unknown dispatch kinds

This directly solves the "existing script but agent still improvises" issue for skill commands that dispatch to `bash`.

### 5. Retire `agents/openai.yaml` as Runtime Authority

OpenClaw does not rely on a parallel `agents/openai.yaml` contract to define core skill runtime behavior. WorkClaw should align:

- keep `SKILL.md` as the single source of truth
- stop depending on `agents/openai.yaml` for behavior selection
- if UI still needs display metadata, derive it from frontmatter or treat yaml as optional display-only metadata

## Affected Modules

- `packages/runtime-skill-core/src/skill_config.rs`
- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`
- `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- `packages/runtime-chat-app/src/prompt_assembly.rs`
- chat/runtime command registration paths that surface workspace skills

## Risks

### Behavior Risk

Changing skill parsing and invocation semantics can alter how existing local skills are surfaced and selected.

### UI / Command Exposure Risk

Once `user-invocable` and command dispatch exist, WorkClaw may expose more skill entrypoints than before unless the filtering layer is explicit.

### Policy Risk

If `command-tool: bash` is introduced without policy checks, skills could bypass current tool narrowing expectations. Dispatch must still respect the active tool policy.

## Recommended Rollout

Implement in phases:

1. frontmatter and skill entry model
2. prompt/catalog alignment
3. structured `skill` tool output
4. command dispatch path
5. cleanup and retirement of yaml runtime dependency

## Verification Expectations

At implementation time, verification should include:

- focused Rust tests for frontmatter parsing
- workspace skill projection tests
- skill invocation / dispatch tests
- prompt assembly tests for skill catalog rendering
- any command-registration tests affected by user-invocable skills

## Release Impact

This is runtime behavior and skill orchestration work, but not installer or vendor-lane work. It is not inherently release-sensitive, though it should be treated as high-risk runtime behavior and verified accordingly.
