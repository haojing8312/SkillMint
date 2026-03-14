# Workspace Skills Projection Design

**Date:** 2026-03-14

**Goal:** Align WorkClaw skill execution with the OpenClaw workspace-skills model so third-party skills can rely on stable on-disk skill directories and explicit `SKILL.md` locations during a session run.

## Problem

WorkClaw currently loads a skill by reading its `SKILL.md` content and injecting the prompt text into the system prompt. This is sufficient for prompt-only skills, but it breaks skills that depend on files adjacent to `SKILL.md`, such as:

- `scripts/*.py`
- `assets/*`
- `references/*`

The failure mode is structural:

1. the model sees the skill instructions
2. the model does not see a stable skill root path
3. the session `work_dir` is not the skill install directory
4. relative paths in the skill instructions become unreliable

This is why skills like `Auto-Redbook-Skills` degrade or invent unrelated explanations instead of executing their bundled scripts correctly.

## OpenClaw Reference Model

Reference implementation lives under:

- [skills.ts](/e:/code/yzpd/workclaw/reference/openclaw/src/agents/skills.ts)
- [workspace.ts](/e:/code/yzpd/workclaw/reference/openclaw/src/agents/skills/workspace.ts)
- [system-prompt.ts](/e:/code/yzpd/workclaw/reference/openclaw/src/agents/system-prompt.ts)
- [workspace-run.ts](/e:/code/yzpd/workclaw/reference/openclaw/src/agents/workspace-run.ts)

OpenClaw does not rely on hidden install paths in prompt text. Instead it:

1. resolves a concrete `workspaceDir` for the run
2. builds a workspace-local skills view
3. injects an `<available_skills>` block with explicit `SKILL.md` locations
4. keeps the actual skill directories available inside the workspace

That model is what WorkClaw should mirror.

## Desired Behavior

Before every skill-driven session run, WorkClaw should project all visible skills into the current session workspace:

```text
<session_work_dir>/
  skills/
    <skill-dir>/
      SKILL.md
      scripts/
      assets/
      references/
```

Then WorkClaw should inject a skills section into the system prompt that tells the model:

- what skills are available
- what each skill does
- where its `SKILL.md` is located
- that it should read the chosen `SKILL.md` from that explicit path

## Scope

Included in this change:

- local skills imported from a directory
- encrypted skills imported from `.skillpack`
- builtin skills
- workspace-local projected `skills/` directory under the session `work_dir`
- skills prompt generation for the current run

Explicitly out of scope for the first pass:

- changing `allowed_tools` semantics
- new skill syntax/frontmatter features
- online syncing from remote repositories during run startup
- deduplicating prompt entries across teams/employees beyond current install state

## WorkClaw Design

### 1. Resolve a Workspace Skills View

WorkClaw should add a new runtime step that resolves all visible installed skills for the session and converts them into a normalized internal structure:

- `skill_id`
- `display_name`
- `description`
- `source_type`
- `source_path` or decrypted files

This should happen in runtime preparation, not inside the model prompt builder.

### 2. Project Skills into Session Workspace

For each visible skill, WorkClaw should materialize a runtime copy under:

- `<session_work_dir>/skills/<stable-skill-dir>/...`

Projection rules:

- local skill: copy the full directory tree
- encrypted skill: decrypt/unpack and write the full file tree
- builtin skill: create a directory containing at least `SKILL.md`; future builtin assets can be added to the same mechanism

The projected tree is a runtime copy, not the source of truth.

### 3. Build a Skills Prompt from Projected Entries

After projection, WorkClaw should generate a prompt block similar to OpenClaw's workspace skills prompt:

- one entry per available skill
- include `name`
- include `description`
- include the projected `SKILL.md` location

The prompt should instruct the model to:

- scan the available skills first
- choose the most specific applicable skill
- read the selected `SKILL.md` from the listed path

### 4. Keep Session Tool Context Pointing at Session Work Dir

`ToolContext.work_dir` should remain the session workspace root. This is important because:

- file tools already expect a session work directory
- bash already executes relative to `work_dir`
- the projected `skills/` folder becomes naturally reachable from the existing execution model

This means skills can reliably use:

- `skills/<skill-id>/scripts/...`
- or `cd skills/<skill-id>` and run bundled commands from there

### 5. Prefer Runtime Copies Over Original Install Paths

Do not expose original `pack_path` or raw install locations as the main execution contract.

Reasons:

- avoids mutating user-installed skills by accident
- keeps a single stable path model per run
- matches the OpenClaw workspace-centric design
- makes encrypted and builtin skills behave like local directories at runtime

## Directory Naming

Projected skill directories should use a stable filesystem-safe key derived from installed skill id, not display name.

Recommended rule:

- base key: installed `skill_id`
- normalize to safe directory name
- prompt still shows human-readable display name

This avoids collisions and display-name churn.

## Prompt Contract

WorkClaw system prompt should gain a dedicated skills section with behavior equivalent to OpenClaw:

- inspect available skill descriptions before replying
- if one skill clearly applies, read its `SKILL.md` from the listed location
- do not rely on hidden knowledge of install paths

The session's primary skill prompt still remains the base instruction for the run. The new skills prompt is an explicit discovery-and-location layer.

## Sync Strategy

Initial implementation should prefer correctness over optimization:

- remove `<session_work_dir>/skills`
- rebuild it from the resolved visible skills set
- regenerate the skills prompt

This avoids stale runtime copies and keeps the first implementation simple.

## Safety and Isolation

Projected skill directories are runtime copies.

Benefits:

- editing a file under `work_dir/skills/...` does not corrupt the original installed skill
- local, builtin, and encrypted skills all share the same runtime contract
- execution becomes deterministic inside a single workspace boundary

## Risks

### Prompt Growth

Adding all visible skills to prompt increases token usage.

Mitigation:

- cap skill count
- cap character budget
- truncate with a visible warning in diagnostics/tests

### Startup Cost

Full directory projection before each run adds I/O.

Mitigation:

- accept full rebuild in v1
- optimize incrementally later if needed

### Builtin Skill Resource Gaps

Some builtin skills may only exist as embedded markdown today.

Mitigation:

- support `SKILL.md`-only projection first
- extend builtin asset materialization later without changing the contract

## Success Criteria

The design is successful when:

1. a third-party skill with bundled scripts/resources can be executed from a normal WorkClaw session without needing its original install directory as cwd
2. the model can see an explicit `SKILL.md` path for available skills
3. session workspace contains a projected `skills/` directory for the run
4. Auto-Redbook-Skills-style skills can reference bundled files through the projected workspace layout
