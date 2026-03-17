# WorkClaw AGENTS and Skills Adoption Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce an OpenAI-style project workflow layer in WorkClaw by restructuring the root `AGENTS.md` and adding three repo-local skills for strategy, verification, and release-sensitive work.

**Architecture:** Keep the root `AGENTS.md` as the repo-wide contract, but move project-specific trigger rules to the top and push detailed workflows into `.agents/skills/`. Reuse existing repo commands and scripts rather than adding new automation in the first pass.

**Tech Stack:** Markdown, Codex `AGENTS.md`, repo-local `SKILL.md`, pnpm workspace scripts, Tauri, React, Rust

---

### Task 1: Rewrite the top-level AGENTS contract

**Files:**
- Modify: `AGENTS.md`

**Step 1: Preserve the repo's current safety and startup guidance**

Copy the existing process safety and Windows contributor sections into a working draft so they can be retained verbatim or near-verbatim after the reorder.

**Step 2: Add the new top sections**

Insert these sections before the long local-skill listing:

- `Project overview`
- `Mandatory skill usage`
- `Build and test commands`
- `Release-sensitive commands`
- `Compatibility and safety rules`

**Step 3: Add exact mandatory trigger rules**

Write these trigger bullets into `Mandatory skill usage`:

- Use `$workclaw-implementation-strategy` before editing runtime behavior, routing, provider integration, tool permissions, sidecar bridge behavior, or vendor sync boundaries.
- Use `$workclaw-change-verification` when changes affect code, tests, examples, skill assets, or build/test behavior before claiming completion.
- Use `$workclaw-release-readiness` when changes affect versioning, release documentation, installer branding, packaging, or vendor release lanes.

**Step 4: Review the file ordering**

Verify by reading the first 120 lines of `AGENTS.md` that the WorkClaw-specific workflow contract now appears before environment-specific notes.

Run:

```bash
Get-Content AGENTS.md -TotalCount 120
```

Expected: the first screen includes project overview, mandatory skill triggers, and the main repo commands.

**Step 5: Commit**

```bash
git add AGENTS.md
git commit -m "docs(repo): front-load WorkClaw workflow rules in AGENTS"
```

### Task 2: Add the implementation-strategy skill

**Files:**
- Create: `.agents/skills/workclaw-implementation-strategy/SKILL.md`

**Step 1: Write the frontmatter**

Use this exact frontmatter:

```yaml
---
name: workclaw-implementation-strategy
description: Use when changing runtime behavior, routing, provider integration, tool permissions, sidecar protocols, or vendor sync boundaries in WorkClaw before editing code.
---
```

**Step 2: Write the skill body**

Document:

- when to use it
- the affected repo areas to inspect first
- the required output sections
- the rule to propose the smallest safe implementation path
- the rule to call out follow-on verification and release impact

**Step 3: Sanity-check discoverability**

Run:

```bash
Get-Content .agents/skills/workclaw-implementation-strategy/SKILL.md
```

Expected: the description is trigger-based, not process-summary-based, and the file clearly tells a future agent what to inspect and produce.

**Step 4: Commit**

```bash
git add .agents/skills/workclaw-implementation-strategy/SKILL.md
git commit -m "docs(skills): add WorkClaw implementation strategy skill"
```

### Task 3: Add the change-verification skill

**Files:**
- Create: `.agents/skills/workclaw-change-verification/SKILL.md`

**Step 1: Write the frontmatter**

Use this exact frontmatter:

```yaml
---
name: workclaw-change-verification
description: Use when changes affect runtime code, tests, skill assets, or build and test behavior in WorkClaw and verification is required before claiming completion.
---
```

**Step 2: Encode the verification routing**

Document how to choose among:

- `pnpm test:sidecar`
- `pnpm test:rust-fast`
- `pnpm test:builtin-skills`
- `pnpm test:e2e:runtime`
- `pnpm build:runtime`

The skill should bias toward the smallest command set that matches the changed area.

**Step 3: Encode the output contract**

Require the skill to report:

- commands run
- pass/fail results
- skipped or still-unverified areas
- whether it is valid to claim the work is verified

**Step 4: Sanity-check command references**

Run:

```bash
rg -n "test:sidecar|test:rust-fast|test:builtin-skills|test:e2e:runtime|build:runtime" AGENTS.md .agents/skills/workclaw-change-verification/SKILL.md package.json
```

Expected: all referenced commands exist and the wording is consistent across the repo contract and the skill.

**Step 5: Commit**

```bash
git add .agents/skills/workclaw-change-verification/SKILL.md
git commit -m "docs(skills): add WorkClaw change verification skill"
```

### Task 4: Add the release-readiness skill

**Files:**
- Create: `.agents/skills/workclaw-release-readiness/SKILL.md`

**Step 1: Write the frontmatter**

Use this exact frontmatter:

```yaml
---
name: workclaw-release-readiness
description: Use when changes affect versioning, installer branding, release docs, packaging outputs, or vendor release lanes in WorkClaw before deciding a branch is ready to ship.
---
```

**Step 2: Encode release-sensitive command routing**

Document these commands and when to use them:

- `pnpm release:check-version`
- `pnpm test:release`
- `pnpm test:installer`
- `pnpm test:release-docs`
- `pnpm test:openclaw-vendor-lane`
- `pnpm build:runtime` when packaging output changed

**Step 3: Encode the verdict format**

Require the skill to output:

- `GREEN`, `YELLOW`, or `RED`
- blocking issues
- documentation or release note follow-ups
- a final ship recommendation

**Step 4: Sanity-check the release lane wording**

Run:

```bash
rg -n "release|installer|vendor lane|ship|GREEN|YELLOW|RED" .agents/skills/workclaw-release-readiness/SKILL.md AGENTS.md
```

Expected: the release skill is clearly scoped to shipping-sensitive changes rather than ordinary verification.

**Step 5: Commit**

```bash
git add .agents/skills/workclaw-release-readiness/SKILL.md
git commit -m "docs(skills): add WorkClaw release readiness skill"
```

### Task 5: Verify the repo-local skill layout

**Files:**
- Create: `.agents/skills/workclaw-implementation-strategy/SKILL.md`
- Create: `.agents/skills/workclaw-change-verification/SKILL.md`
- Create: `.agents/skills/workclaw-release-readiness/SKILL.md`
- Modify: `AGENTS.md`

**Step 1: Check file layout**

Run:

```bash
Get-ChildItem .agents/skills -Recurse | ForEach-Object { $_.FullName }
```

Expected: exactly the three new skill directories exist with one `SKILL.md` in each.

**Step 2: Check the root contract references all three skills**

Run:

```bash
rg -n "workclaw-implementation-strategy|workclaw-change-verification|workclaw-release-readiness" AGENTS.md .agents/skills
```

Expected: the root `AGENTS.md` references each skill and each skill has the expected name and trigger language.

**Step 3: Review the final diff**

Run:

```bash
git diff -- AGENTS.md .agents/skills docs/plans/2026-03-17-workclaw-agents-skills-design.md docs/plans/2026-03-17-workclaw-agents-skills-plan.md
```

Expected: only the intended repo-process files changed, with no application code touched.

**Step 4: Final verification note**

Record that this phase intentionally adds workflow guidance only. No runtime behavior, build logic, or CI automation should change.

### Task 6: Record the design and handoff

**Files:**
- Create: `docs/plans/2026-03-17-workclaw-agents-skills-design.md`
- Create: `docs/plans/2026-03-17-workclaw-agents-skills-plan.md`

**Step 1: Save the approved design**

Persist the design rationale, scope, risks, and rollout phases in the design doc.

**Step 2: Save the implementation plan**

Persist this plan file in `docs/plans/` so implementation can happen in a later session.

**Step 3: Final handoff review**

Run:

```bash
Get-Content docs/plans/2026-03-17-workclaw-agents-skills-design.md -TotalCount 80
Get-Content docs/plans/2026-03-17-workclaw-agents-skills-plan.md -TotalCount 120
git diff --stat
```

Expected: the design and plan are readable, and the diff is limited to process documentation files.
