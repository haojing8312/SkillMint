# OpenClaw Skill Runtime Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild WorkClaw's skill runtime around the OpenClaw skill model so `SKILL.md` frontmatter drives discovery, invocation policy, and structured command dispatch instead of prompt-only execution.

**Architecture:** Add an OpenClaw-style skill entry layer on top of projected workspace skills, expand frontmatter parsing in `runtime-skill-core`, teach the runtime to build structured skill snapshots, and update `skill` invocation so it can return and execute command dispatch metadata rather than only raw prompt text.

**Tech Stack:** Rust, Tauri runtime, `runtime-skill-core`, `runtime-chat-app`, SQLite-backed skill projection, existing WorkClaw agent/tool runtime.

---

### Task 1: Lock The New Skill Contract

**Files:**
- Modify: `packages/runtime-skill-core/src/skill_config.rs`
- Test: `packages/runtime-skill-core/tests/skill_config.rs`
- Reference: `references/openclaw/src/agents/skills/frontmatter.ts`
- Reference: `references/openclaw/src/agents/skills/types.ts`

**Step 1: Write failing parser tests for new frontmatter fields**

Add tests covering:

- `user-invocable`
- `disable-model-invocation`
- `metadata`
- `command-dispatch`
- `command-tool`
- `command-arg-mode`

Use exact markdown fixtures in the test file.

**Step 2: Run the parser tests and verify they fail**

Run: `cargo test -p runtime-skill-core --test skill_config`

Expected: new assertions fail because the fields are not parsed yet.

**Step 3: Extend `SkillConfig` with OpenClaw-style parsed fields**

Add new Rust structs for:

- invocation policy
- optional metadata blob or parsed subset
- optional command dispatch spec

Keep `name`, `description`, `allowed_tools`, `model`, and existing fields working.

**Step 4: Parse the new fields from YAML frontmatter**

Implement parsing in `SkillConfig::parse` without breaking existing frontmatter.

**Step 5: Re-run the parser tests**

Run: `cargo test -p runtime-skill-core --test skill_config`

Expected: PASS.

### Task 2: Define Runtime Skill Entry Types

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/types.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`

**Step 1: Add new runtime skill entry fields**

Add fields for:

- parsed frontmatter snapshot
- invocation policy
- optional command dispatch
- optional metadata subset

Do not remove the existing prompt-facing fields yet.

**Step 2: Wire type exports**

Update `runtime_io.rs` exports so the new entry types remain available to runtime callers.

**Step 3: Add or extend tests for runtime entry construction**

Cover local skill projection with frontmatter that includes command dispatch and invocation policy.

**Step 4: Run the targeted tests**

Run: `cargo test -p runtime_lib workspace_skill_projection_tests`

Expected: PASS with new entry shape covered.

### Task 3: Parse Structured Skill Entries During Projection

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`

**Step 1: Make skill projection parse `SKILL.md` instead of only storing raw content**

When building `WorkspaceSkillRuntimeEntry`, parse the projected `SKILL.md` using the expanded `SkillConfig`.

**Step 2: Persist parsed invocation metadata in the runtime entry**

Store:

- frontmatter-derived display info
- invocation policy
- command dispatch spec
- metadata needed for prompt or runtime decisions

**Step 3: Keep prompt projection behavior intact**

`<available_skills>` should still render `name`, `invoke_name`, `description`, and `location`.

**Step 4: Add regression tests**

Verify:

- projected entries still produce prompt blocks
- structured metadata survives projection

**Step 5: Run targeted projection tests**

Run: `cargo test -p runtime_lib workspace_skill_projection_tests`

Expected: PASS.

### Task 4: Align Prompt Assembly With OpenClaw Selection Rules

**Files:**
- Modify: `packages/runtime-chat-app/src/prompt_assembly.rs`
- Test: `packages/runtime-chat-app/tests/prompt_assembly.rs`
- Reference: `references/openclaw/src/agents/system-prompt.ts`

**Step 1: Update the skill instructions in the system prompt**

Change the skill guidance so it matches the OpenClaw pattern more closely:

- scan available skill descriptions first
- read exactly one matching skill up front
- do not read skills if none clearly apply

**Step 2: Preserve WorkClaw-specific constraints only where still necessary**

Do not reintroduce prompt text that implies all skill behavior must remain prompt-only.

**Step 3: Add prompt assembly tests**

Assert the new selection rules are present and that prompt formatting remains stable.

**Step 4: Run prompt assembly tests**

Run: `cargo test -p runtime-chat-app prompt_assembly`

Expected: PASS.

### Task 5: Replace The Current Skill Executability Model

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- Test: `apps/runtime/src-tauri/tests/test_skill_permission_narrowing.rs`
- Test: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`

**Step 1: Write failing tests for structured skill resolution**

Cover cases where:

- a skill is prompt-following only
- a skill has command dispatch metadata
- a skill has `disable-model-invocation`

**Step 2: Remove the current “instruction_only because no allowed_tools” assumption**

A skill without `allowed_tools` is not necessarily non-executable anymore.

**Step 3: Return structured skill resolution output**

The skill tool should include:

- invocation policy
- command dispatch info
- prompt body
- policy-filtered tool scope

**Step 4: Preserve permission denial for actual blocked dispatch**

If a dispatch targets a tool outside policy, reject it explicitly.

**Step 5: Run targeted skill invoke tests**

Run: `cargo test -p runtime_lib skill_tool`

Expected: PASS.

### Task 6: Introduce OpenClaw-Style Command Dispatch

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_dispatch.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- Test: `apps/runtime/src-tauri/tests/test_skill_permission_narrowing.rs`
- Test: add a focused runtime test near skill invocation / tool dispatch coverage

**Step 1: Add a minimal command dispatch model**

Support only:

- `command-dispatch: tool`
- `command-tool: <tool-name>`
- `command-arg-mode: raw`

**Step 2: Define dispatch behavior**

When a skill command is resolved through this path:

- raw user args are forwarded to the target tool
- target tool name is checked against policy
- no freeform intermediate prompt-following step is required

**Step 3: Add tests**

Cover:

- dispatch to allowed tool succeeds
- dispatch to blocked tool fails
- unknown dispatch kind is ignored or rejected deterministically

**Step 4: Run the targeted runtime tests**

Run: `cargo test -p runtime_lib skill_permission_narrowing`

Expected: PASS.

### Task 7: Add User-Invocable Skill Command Specs

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`
- Modify any command-registration path that surfaces slash or quick commands for skills
- Test: add targeted tests for command spec generation
- Reference: `references/openclaw/src/agents/skills/workspace.ts`

**Step 1: Add a WorkClaw equivalent of skill command spec generation**

Generate command specs from projected skill entries that are:

- `user-invocable != false`
- optionally command-dispatch-enabled

**Step 2: Ensure names are sanitized and unique**

Follow the OpenClaw pattern:

- lowercase normalization
- safe command names
- deterministic de-duplication

**Step 3: Add tests for generated command specs**

Verify:

- sanitized names
- de-duplicated names
- dispatch metadata inclusion

**Step 4: Run command-spec-related tests**

Run: `cargo test -p runtime_lib skill_command`

Expected: PASS.

### Task 8: Remove `agents/openai.yaml` From Runtime Authority

**Files:**
- Search: `apps/runtime/src-tauri/src/`
- Search: `packages/*/src/`
- Modify only runtime paths that currently treat yaml as authoritative
- Test: any affected skill-loading tests

**Step 1: Identify runtime behavior still sourced from `agents/openai.yaml`**

Search for code paths that treat yaml as behavior-defining rather than display-only.

**Step 2: Move behavior-driving fields to `SKILL.md` frontmatter**

If UI still needs yaml, leave it as optional display metadata only.

**Step 3: Add regression tests**

Verify runtime behavior is driven by `SKILL.md`, not yaml.

**Step 4: Run the affected tests**

Run the smallest relevant Rust test subset that covers skill loading.

Expected: PASS.

### Task 9: Update One Real Skill Fixture To Prove The Model

**Files:**
- Create or modify: a focused test fixture skill under existing test helpers
- Optionally add a local fixture under `tests/helpers`
- Test: targeted runtime integration tests

**Step 1: Add a realistic fixture**

Use a skill frontmatter example with:

- `name`
- `description`
- `user-invocable`
- `command-dispatch`
- `command-tool`
- `command-arg-mode`

**Step 2: Write an integration-style test**

Prove:

- the skill appears in prompt projection
- the command spec is generated
- dispatch metadata survives invocation

**Step 3: Run the focused test**

Run the single targeted test file.

Expected: PASS.

### Task 10: Full Verification Sweep

**Files:**
- No code changes

**Step 1: Run `runtime-skill-core` tests**

Run: `cargo test -p runtime-skill-core`

Expected: PASS.

**Step 2: Run runtime chat prompt tests**

Run: `cargo test -p runtime-chat-app`

Expected: PASS.

**Step 3: Run focused Tauri runtime tests**

Run: `cargo test -p runtime_lib skill`

Expected: PASS for skill-related coverage.

**Step 4: Run WorkClaw skill verification command**

Run: `pnpm test:builtin-skills`

Expected: PASS, or document exact failures if unrelated.

**Step 5: Commit**

Run:

```bash
git add packages/runtime-skill-core apps/runtime/src-tauri packages/runtime-chat-app docs/plans
git commit -m "feat: align skill runtime with openclaw dispatch model"
```
