# Frontend AI Guardrails Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce concise root guidance, frontend-specific AI development guardrails, and lightweight large-file reporting for `apps/runtime/src/` without changing runtime behavior.

**Architecture:** Keep the root `AGENTS.md` short and cross-repo, then add a local `apps/runtime/src/AGENTS.md` that owns frontend runtime layering and file-budget rules. Add a simple report script and a curated backlog so agents and maintainers can see which files are in warning or split-plan territory without turning the policy into a hard gate.

**Tech Stack:** Markdown docs, repo-local `AGENTS.md`, Node.js reporting scripts, pnpm workspace scripts, React, TypeScript.

---

### Task 1: Add the validated design record

**Files:**
- Create: `docs/plans/2026-03-23-frontend-ai-guardrails-design.md`

**Step 1: Write the design doc**

Write the agreed frontend AI guardrail design with:
- threshold rationale for `300 / 500`
- app/scene/component/hook/service landing-zone guidance
- anti-micro-file guidance
- a reporting-and-backlog approach that does not introduce CI failure policy

**Step 2: Review the document for scope discipline**

Confirm it does not silently expand into a frontend refactor or lint-enforcement rollout.

**Step 3: Commit**

```bash
git add docs/plans/2026-03-23-frontend-ai-guardrails-design.md
git commit -m "docs: add frontend ai guardrails design"
```

### Task 2: Add a frontend runtime local guidance file

**Files:**
- Create: `apps/runtime/src/AGENTS.md`

**Step 1: Draft local guidance**

Include:
- scope of the frontend runtime area
- default landing zones for `App.tsx`, `scenes`, `components`, `hooks`, `api/services`, and `lib/utils`
- file-budget thresholds
- what may stay in a root component versus what should move out
- responsibility-trigger guidance for files that are not yet large but already mix concerns

**Step 2: Review for brevity**

Confirm the file is concise enough to serve as agent memory, not a frontend architecture book.

**Step 3: Commit**

```bash
git add apps/runtime/src/AGENTS.md
git commit -m "docs: add frontend runtime agent guardrails"
```

### Task 3: Link the new local guidance from the root file

**Files:**
- Modify: `AGENTS.md`

**Step 1: Add a short frontend guidance section**

Add only the minimal cross-repo text needed to:
- say frontend runtime work should follow the closer local guidance in `apps/runtime/src/AGENTS.md`
- mention the `300 / 500` trigger model at a high level

**Step 2: Review for duplication**

Avoid repeating detailed frontend layering rules that now belong in the local guidance file.

**Step 3: Commit**

```bash
git add AGENTS.md
git commit -m "docs: route frontend work to local runtime guidance"
```

### Task 4: Add lightweight frontend large-file reporting

**Files:**
- Create: `scripts/report-frontend-large-files.mjs`
- Modify: `package.json`

**Step 1: Write the report script**

Create a Node script that:
- walks `apps/runtime/src/`
- includes `.ts` and `.tsx`
- excludes `__tests__`, `*.test.*`, `*.spec.*`, and build output folders
- classifies files using `warn=300` and `plan=500`
- prints `WARN` and `PLAN` rows in descending line-count order

**Step 2: Add the package script**

Expose the report as:

```json
"report:frontend-large-files": "node scripts/report-frontend-large-files.mjs"
```

**Step 3: Add a focused test**

Create a small node test similar to the Rust report test to verify:
- the package script exists
- the default thresholds are `300` and `500`
- the script scopes itself to `apps/runtime/src/`

**Step 4: Run the focused test**

Run:

```bash
node --test scripts/report-frontend-large-files.test.mjs
```

Expected: PASS

**Step 5: Commit**

```bash
git add scripts/report-frontend-large-files.mjs scripts/report-frontend-large-files.test.mjs package.json
git commit -m "chore: add frontend large file reporting"
```

### Task 5: Add the first frontend large-file backlog

**Files:**
- Create: `docs/plans/2026-03-23-frontend-large-file-backlog.md`

**Step 1: Run the report**

Run:

```bash
pnpm report:frontend-large-files
```

Expected: a sorted list of runtime frontend files in `WARN` or `PLAN`

**Step 2: Curate the backlog**

Write a backlog document that:
- explains prioritization rules
- lists P1 giant page or component files first
- distinguishes `PLAN` files from `WARN` files
- names a first split direction for the biggest files
- defines what counts as backlog progress

**Step 3: Commit**

```bash
git add docs/plans/2026-03-23-frontend-large-file-backlog.md
git commit -m "docs: add frontend large file backlog"
```

### Task 6: Verify the guidance set

**Files:**
- Verify: `AGENTS.md`
- Verify: `apps/runtime/src/AGENTS.md`
- Verify: `docs/plans/2026-03-23-frontend-ai-guardrails-design.md`
- Verify: `docs/plans/2026-03-23-frontend-ai-guardrails-plan.md`
- Verify: `scripts/report-frontend-large-files.mjs`
- Verify: `docs/plans/2026-03-23-frontend-large-file-backlog.md`

**Step 1: Check file diffs**

Run:

```bash
git diff -- AGENTS.md apps/runtime/src/AGENTS.md docs/plans/2026-03-23-frontend-ai-guardrails-design.md docs/plans/2026-03-23-frontend-ai-guardrails-plan.md scripts/report-frontend-large-files.mjs scripts/report-frontend-large-files.test.mjs docs/plans/2026-03-23-frontend-large-file-backlog.md package.json
```

Expected: documentation, guidance, and reporting-only changes

**Step 2: Sanity-check the guidance files**

Run:

```bash
Get-Content AGENTS.md
Get-Content apps/runtime/src/AGENTS.md
```

Expected: guidance is short, readable, and non-contradictory

**Step 3: Re-run the frontend report**

Run:

```bash
pnpm report:frontend-large-files
```

Expected: the report runs cleanly and the backlog aligns with the current output

**Step 4: Commit**

```bash
git add AGENTS.md apps/runtime/src/AGENTS.md docs/plans/2026-03-23-frontend-ai-guardrails-design.md docs/plans/2026-03-23-frontend-ai-guardrails-plan.md scripts/report-frontend-large-files.mjs scripts/report-frontend-large-files.test.mjs docs/plans/2026-03-23-frontend-large-file-backlog.md package.json
git commit -m "docs: establish frontend ai development guardrails"
```
