# WorkClaw AGENTS and Skills Adoption Design

**Date:** 2026-03-17
**Status:** Approved

## Goal

Adopt the OpenAI-style `AGENTS.md` + repo-local `skills` pattern in WorkClaw so the coding agent can consistently choose the right workflow for architecture-sensitive work, required verification, and release-sensitive changes without relying on ad hoc prompts.

## Scope

- Restructure the root `AGENTS.md` so project workflow rules appear before environment-specific notes
- Add a repo-local `.agents/skills/` directory with three initial skills
- Reuse existing repo commands and scripts instead of introducing new automation in phase 1
- Encode trigger conditions for WorkClaw's mixed Tauri, Rust, TypeScript, sidecar, and release workflows
- Preserve the current safety rules and Windows startup guidance already present in `AGENTS.md`

## Out of Scope

- GitHub Actions automation for the new skills
- Subdirectory `AGENTS.md` files
- New wrapper scripts around existing verification commands
- Refactoring the current app, runtime, or packaging code
- Moving personal or global Codex skill instructions out of the repo in this phase

## Current Repo Shape

The design should match how WorkClaw is already organized:

- `apps/runtime/` holds the React app and desktop shell frontend
- `apps/runtime/src-tauri/` holds the Tauri backend and platform integration tests
- `apps/runtime/sidecar/` holds the sidecar runtime, adapters, and browser automation bridge
- `packages/*` holds Rust runtime libraries, routing, policy, model, executor, and skill-related crates
- the root `package.json` already exposes the main operational commands needed for verification and release checks

This makes WorkClaw a strong fit for project-local skills because many tasks are not language-specific. They cross React, Rust, Tauri, vendor sync, packaging, and release boundaries.

## Decision

Use a balanced adoption model:

1. Keep the existing root `AGENTS.md`
2. Move project workflow rules to the top of that file
3. Add `.agents/skills/` for repo-specific workflows
4. Keep deterministic execution in existing repo commands and scripts
5. Reserve CI integration for a later phase once local workflows prove stable

This is the closest fit to OpenAI's published approach without over-rotating into a large repo process rewrite.

## Root AGENTS.md Role

The root `AGENTS.md` should become the single place that answers four questions:

1. What parts of the repo matter for implementation work?
2. Which workflows are mandatory and when must they be used?
3. Which commands define "verified" for this repo?
4. Which safety or compatibility constraints must never be bypassed?

The root file should not try to hold the detailed workflow bodies. Those belong in `.agents/skills/`.

## Proposed Root AGENTS.md Structure

The first sections of the root `AGENTS.md` should look like this:

- `Project overview`
- `Mandatory skill usage`
- `Build and test commands`
- `Release-sensitive commands`
- `Compatibility and safety rules`
- existing Windows startup and process-safety sections

The key change is ordering. Today the file is dominated by skill discovery and local environment instructions. After adoption, the first thing the agent should see is the WorkClaw-specific workflow contract.

## Initial Skill Set

### 1. `workclaw-implementation-strategy`

**Purpose**

Front-load architecture and compatibility thinking before editing risky areas.

**Trigger conditions**

- runtime behavior changes
- routing or provider selection changes
- tool permission changes
- sidecar protocol or bridge changes
- vendor sync boundary changes
- changes that affect desktop, IM, browser automation, or policy interactions

**Expected output**

- affected subsystems
- compatibility and rollout risks
- recommended smallest safe implementation path
- required verification follow-up
- notes for release impact if relevant

**Why this belongs in a skill**

These tasks require reasoning across several layers of the repo and are exactly the kind of work where OpenAI recommends a repo-specific workflow instead of a generic instruction.

### 2. `workclaw-change-verification`

**Purpose**

Define when code changes count as "verified" in WorkClaw.

**Trigger conditions**

- runtime code changes
- Tauri backend changes
- sidecar changes
- Rust package changes
- tests or build behavior changes
- builtin skill asset changes
- UI flows with meaningful behavioral impact

**Expected output**

- exact commands run
- pass or fail status
- what remains unverified
- whether it is acceptable to claim completion

**Phase 1 command mapping**

- `pnpm test:sidecar`
- `pnpm test:rust-fast`
- `pnpm test:builtin-skills`
- `pnpm test:e2e:runtime`
- `pnpm build:runtime`

The skill should route people toward the smallest command set that matches the changed area instead of always running the entire repo.

### 3. `workclaw-release-readiness`

**Purpose**

Separate release-sensitive checks from ordinary code verification.

**Trigger conditions**

- version changes
- installer branding changes
- release docs changes
- packaging changes
- vendor lane changes
- artifacts or release metadata changes

**Expected output**

- release verdict: `GREEN`, `YELLOW`, or `RED`
- explicit blocking issues
- release-note or documentation follow-ups
- recommendation on whether the branch is safe to ship

**Phase 1 command mapping**

- `pnpm release:check-version`
- `pnpm test:release`
- `pnpm test:installer`
- `pnpm test:release-docs`
- `pnpm test:openclaw-vendor-lane`
- `pnpm build:runtime` when packaging output changed

## Proposed Trigger Rules in AGENTS.md

The mandatory usage section should encode simple if/then rules:

- Use `$workclaw-implementation-strategy` before editing runtime behavior, routing, provider integration, tool permissions, sidecar bridge behavior, or vendor sync boundaries.
- Use `$workclaw-change-verification` when changes affect code, tests, examples, skill assets, or build/test behavior before claiming the work is complete.
- Use `$workclaw-release-readiness` when changes affect versioning, release documentation, installer branding, packaging, or vendor release lanes.

These rules are intentionally narrow and actionable. They tell the agent when to route into the repo-local workflow without stuffing full procedures into the root file.

## Why Reuse Existing Commands

Phase 1 should not create new wrapper scripts unless the repo clearly needs them. WorkClaw already exposes a useful command surface in the root `package.json`:

- `pnpm app`
- `pnpm build:runtime`
- `pnpm test:sidecar`
- `pnpm test:rust-fast`
- `pnpm test:e2e:runtime`
- `pnpm test:builtin-skills`
- `pnpm test:release`
- `pnpm test:installer`
- `pnpm test:release-docs`
- `pnpm test:openclaw-vendor-lane`

That means the first version of the skills can be mostly documentation and routing. We can add supporting scripts later only if the repeated shell choreography becomes too annoying or error-prone.

## Compatibility and Safety Rules to Keep Visible

The redesigned `AGENTS.md` should keep several current rules prominent:

- never kill processes by image name
- only kill verified PIDs related to the active task
- do not run multiple `pnpm app` sessions in parallel
- preserve Windows startup guidance for local contributors

In addition, the WorkClaw-specific top section should add:

- packaging and installer changes require release-readiness review
- vendor sync changes must preserve upstream lane tracking
- verification claims must be backed by actual command runs

## Rollout Plan

### Phase 1

- add the three repo-local skills
- restructure the root `AGENTS.md`
- document exact trigger rules
- reuse existing commands only

### Phase 2

Add two optional but high-value skills after the first set proves useful:

- `workclaw-platform-docs-check`
- `workclaw-pr-draft-summary`

### Phase 3

If local use is stable, mirror selected workflows into CI with GitHub Actions.

## Success Criteria

This adoption is successful if:

- a new session can discover the right WorkClaw workflow from the repo alone
- high-risk changes route through an explicit strategy step before editing
- completion claims consistently include the right verification commands
- release-sensitive changes stop being treated like ordinary code edits
- the repo gains project-specific workflow guidance without losing existing local safety notes

## Risks

- If the root `AGENTS.md` stays too dominated by local environment notes, routing quality will remain weak
- If skill descriptions are too vague, the agent will not select them reliably
- If skills over-prescribe giant command stacks, they will be ignored in practice
- If too many skills are added at once, the repo will gain complexity before the pattern proves useful

## Recommendation

Proceed with the balanced adoption plan:

- keep the existing root `AGENTS.md`
- move project workflow rules to the top
- add exactly three repo-local skills in phase 1
- reuse existing commands
- defer CI integration until local usage is stable

This gives WorkClaw an OpenAI-style workflow contract without forcing a large process migration up front.
