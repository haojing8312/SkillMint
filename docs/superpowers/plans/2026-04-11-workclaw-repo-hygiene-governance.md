# WorkClaw Repo Hygiene Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first production-ready WorkClaw repo hygiene workflow: governance docs, non-blocking repo hygiene reports, and dedicated review and cleanup skills.

**Architecture:** Build a thin repo hygiene layer around existing WorkClaw workflow conventions. The implementation should add a deterministic reporting command, repo-local governance docs, and two skills that sit above the existing verification and release-readiness skills rather than replacing them.

**Tech Stack:** Node.js scripts, `package.json` scripts, Markdown docs, repo-local `SKILL.md` workflows, PowerShell-friendly CLI execution

---

## File Structure

### Create

- `docs/maintenance/repo-hygiene.md`
- `scripts/review-repo-hygiene.mjs`
- `scripts/review-repo-hygiene.test.mjs`
- `scripts/lib/repo-hygiene/collect-deadcode-signals.mjs`
- `scripts/lib/repo-hygiene/collect-structure-signals.mjs`
- `scripts/lib/repo-hygiene/collect-drift-signals.mjs`
- `scripts/lib/repo-hygiene/write-report.mjs`
- `.agents/skills/workclaw-repo-hygiene-review/SKILL.md`
- `.agents/skills/workclaw-cleanup-execution/SKILL.md`

### Modify

- `AGENTS.md`
- `package.json`

### Validation Targets

- `scripts/review-repo-hygiene.test.mjs`
- `pnpm test:builtin-skills` is not required unless implementation expands into builtin skill assets

---

### Task 1: Add Repo Hygiene Governance Rules

**Files:**
- Modify: `AGENTS.md`
- Create: `docs/maintenance/repo-hygiene.md`

- [ ] **Step 1: Add a repo hygiene section to `AGENTS.md`**

Add a compact policy section near the workflow and safety guidance. Keep it procedural rather than essay-like.

```md
## Repo Hygiene Governance

- Treat orphan files, dead code, stale docs, duplicate implementations, and temporary artifacts as a maintenance surface, not a one-off cleanup task.
- Prefer repo hygiene review before deletion. Do not remove suspicious files or code only because they appear unused in one static signal.
- Use `pnpm review:repo-hygiene` for non-blocking repo hygiene reporting when the task is cleanup-focused or when a large feature leaves likely follow-up debris.
- Use `workclaw-repo-hygiene-review` to classify candidates before destructive edits.
- Use `workclaw-cleanup-execution` only for a reviewed cleanup batch. Cleanup changes still require `workclaw-change-verification` when code, tests, docs, or skill files change.
- Treat generated, runtime-owned, dynamically discovered, or config-driven files as high-risk cleanup surfaces unless a rule explicitly marks them safe.
```

- [ ] **Step 2: Write the maintenance guide**

Create `docs/maintenance/repo-hygiene.md` with these sections:

```md
# Repo Hygiene

## Why This Exists

WorkClaw uses long-running AI-assisted development across runtime, sidecar, Rust, skills, and docs. Repo hygiene review exists to stop temporary artifacts, dead code, duplicated implementations, and stale references from silently becoming part of the long-term source of truth.

## Finding Categories

- temporary-artifacts
- orphan-files
- dead-code
- duplicate-implementations
- stale-docs-and-skills

## Confidence Levels

- confirmed
- probable
- uncertain

## Allowed Actions

- delete
- deprecate
- relocate
- merge-duplicate
- ignore-with-rationale

## Default Workflow

1. Run `pnpm review:repo-hygiene`.
2. Review the generated report under `.artifacts/repo-hygiene/`.
3. Use `workclaw-repo-hygiene-review` to classify findings.
4. Execute only a scoped approved batch.
5. Run the required verification commands for the touched surface.

## High-Risk Surfaces

- generated outputs
- runtime-discovered assets
- config-driven entrypoints
- sidecar protocol boundaries
- startup-critical runtime code
```

- [ ] **Step 3: Sanity-check the new docs**

Run: `rg -n "Repo Hygiene Governance|workclaw-repo-hygiene-review|workclaw-cleanup-execution|pnpm review:repo-hygiene" AGENTS.md docs/maintenance/repo-hygiene.md`

Expected: matches in both files with no placeholder text.

- [ ] **Step 4: Commit the governance docs**

```bash
git add AGENTS.md docs/maintenance/repo-hygiene.md
git commit -m "docs: add repo hygiene governance"
```

---

### Task 2: Add the Aggregate Repo Hygiene Command

**Files:**
- Create: `scripts/review-repo-hygiene.mjs`
- Create: `scripts/lib/repo-hygiene/write-report.mjs`
- Modify: `package.json`
- Test: `scripts/review-repo-hygiene.test.mjs`

- [ ] **Step 1: Write the failing script test**

Create a focused Node test that checks the command writes a stable report structure even when scanners only emit stub findings.

```js
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

test("review-repo-hygiene writes summary and json outputs", async () => {
  const tmp = await fs.mkdtemp(path.join(os.tmpdir(), "repo-hygiene-"));
  const proc = spawn(process.execPath, [
    "scripts/review-repo-hygiene.mjs",
    "--output-dir",
    tmp,
    "--mode",
    "test",
  ], { cwd: process.cwd(), stdio: "pipe" });

  let stderr = "";
  proc.stderr.on("data", (chunk) => {
    stderr += String(chunk);
  });

  const exitCode = await new Promise((resolve) => proc.on("close", resolve));
  assert.equal(exitCode, 0, stderr);

  const summary = await fs.readFile(path.join(tmp, "summary.md"), "utf8");
  const json = JSON.parse(await fs.readFile(path.join(tmp, "report.json"), "utf8"));

  assert.match(summary, /Repo Hygiene Report/);
  assert.equal(Array.isArray(json.findings), true);
  assert.equal(typeof json.generatedAt, "string");
});
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `node --test scripts/review-repo-hygiene.test.mjs`

Expected: FAIL because `scripts/review-repo-hygiene.mjs` does not exist yet.

- [ ] **Step 3: Implement the report writer**

Create `scripts/lib/repo-hygiene/write-report.mjs`:

```js
import fs from "node:fs/promises";
import path from "node:path";

export async function writeRepoHygieneReport(outputDir, report) {
  await fs.mkdir(outputDir, { recursive: true });

  const summary = [
    "# Repo Hygiene Report",
    "",
    `Generated: ${report.generatedAt}`,
    "",
    `Total findings: ${report.findings.length}`,
    "",
    "## Counts By Category",
    ...Object.entries(report.countsByCategory).map(([key, value]) => `- ${key}: ${value}`),
    "",
  ].join("\n");

  await fs.writeFile(path.join(outputDir, "summary.md"), summary);
  await fs.writeFile(path.join(outputDir, "report.json"), JSON.stringify(report, null, 2) + "\n");
}
```

- [ ] **Step 4: Implement the aggregate command**

Create `scripts/review-repo-hygiene.mjs`:

```js
import path from "node:path";
import { collectDeadcodeSignals } from "./lib/repo-hygiene/collect-deadcode-signals.mjs";
import { collectStructureSignals } from "./lib/repo-hygiene/collect-structure-signals.mjs";
import { collectDriftSignals } from "./lib/repo-hygiene/collect-drift-signals.mjs";
import { writeRepoHygieneReport } from "./lib/repo-hygiene/write-report.mjs";

function parseArgs(argv) {
  const args = { outputDir: ".artifacts/repo-hygiene", mode: "normal" };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--output-dir") args.outputDir = argv[i + 1];
    if (argv[i] === "--mode") args.mode = argv[i + 1];
  }
  return args;
}

function buildCounts(findings) {
  return findings.reduce((acc, finding) => {
    acc[finding.category] = (acc[finding.category] ?? 0) + 1;
    return acc;
  }, {});
}

const args = parseArgs(process.argv.slice(2));
const deadcode = await collectDeadcodeSignals({ mode: args.mode });
const structure = await collectStructureSignals({ mode: args.mode });
const drift = await collectDriftSignals({ mode: args.mode });
const findings = [...deadcode, ...structure, ...drift];

await writeRepoHygieneReport(path.resolve(args.outputDir), {
  generatedAt: new Date().toISOString(),
  findings,
  countsByCategory: buildCounts(findings),
});
```

- [ ] **Step 5: Add package scripts**

Add these entries under `scripts` in `package.json`:

```json
"review:repo-hygiene": "node scripts/review-repo-hygiene.mjs",
"review:repo-hygiene:deadcode": "node scripts/review-repo-hygiene.mjs --mode deadcode",
"review:repo-hygiene:drift": "node scripts/review-repo-hygiene.mjs --mode drift",
"review:repo-hygiene:artifacts": "node scripts/review-repo-hygiene.mjs --mode artifacts"
```

- [ ] **Step 6: Run the script test and verify it passes**

Run: `node --test scripts/review-repo-hygiene.test.mjs`

Expected: PASS with `summary.md` and `report.json` created in the temp directory.

- [ ] **Step 7: Smoke-test the package script**

Run: `pnpm review:repo-hygiene`

Expected: PASS and `.artifacts/repo-hygiene/report.json` plus `.artifacts/repo-hygiene/summary.md` exist.

- [ ] **Step 8: Commit the aggregate command**

```bash
git add package.json scripts/review-repo-hygiene.mjs scripts/review-repo-hygiene.test.mjs scripts/lib/repo-hygiene/write-report.mjs
git commit -m "tooling: add repo hygiene report command"
```

---

### Task 3: Add the First Deterministic Signal Collectors

**Files:**
- Create: `scripts/lib/repo-hygiene/collect-deadcode-signals.mjs`
- Create: `scripts/lib/repo-hygiene/collect-structure-signals.mjs`
- Create: `scripts/lib/repo-hygiene/collect-drift-signals.mjs`
- Modify: `scripts/review-repo-hygiene.test.mjs`

- [ ] **Step 1: Expand the test to cover categories**

Append category assertions to `scripts/review-repo-hygiene.test.mjs`:

```js
assert.equal(
  json.findings.some((item) => item.category === "temporary-artifacts" || item.category === "dead-code" || item.category === "stale-docs-and-skills"),
  true,
);
```

- [ ] **Step 2: Implement dead code signal collection**

Create `scripts/lib/repo-hygiene/collect-deadcode-signals.mjs`:

```js
import { spawnSync } from "node:child_process";

export async function collectDeadcodeSignals({ mode }) {
  if (mode === "drift" || mode === "artifacts") return [];
  if (mode === "test") {
    return [{
      category: "dead-code",
      confidence: "confirmed",
      action: "ignore-with-rationale",
      source: "test-fixture",
      detail: "Synthetic dead-code finding for test mode",
    }];
  }

  const result = spawnSync("pnpm", ["exec", "knip", "--production", "--no-progress"], {
    encoding: "utf8",
    shell: true,
  });

  if (result.status !== 0 && !result.stdout && !result.stderr) {
    return [];
  }

  return (result.stdout || "")
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(0, 50)
    .map((line) => ({
      category: "dead-code",
      confidence: "probable",
      action: "ignore-with-rationale",
      source: "knip",
      detail: line,
    }));
}
```

- [ ] **Step 3: Implement structure anomaly collection**

Create `scripts/lib/repo-hygiene/collect-structure-signals.mjs`:

```js
import fs from "node:fs/promises";

const SUSPICIOUS_NAMES = [/tmp/i, /debug/i, /copy/i, /bak/i, /draft/i, /old/i];

export async function collectStructureSignals({ mode }) {
  if (mode === "deadcode" || mode === "drift") return [];
  if (mode === "test") {
    return [{
      category: "temporary-artifacts",
      confidence: "probable",
      action: "ignore-with-rationale",
      source: "test-fixture",
      detail: "Synthetic temporary artifact finding for test mode",
    }];
  }

  const entries = await fs.readdir(process.cwd(), { withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && SUSPICIOUS_NAMES.some((re) => re.test(entry.name)))
    .map((entry) => ({
      category: "temporary-artifacts",
      confidence: "probable",
      action: "ignore-with-rationale",
      source: "root-structure-scan",
      detail: entry.name,
    }));
}
```

- [ ] **Step 4: Implement docs and skill drift collection**

Create `scripts/lib/repo-hygiene/collect-drift-signals.mjs`:

```js
import fs from "node:fs/promises";

export async function collectDriftSignals({ mode }) {
  if (mode === "deadcode" || mode === "artifacts") return [];
  if (mode === "test") {
    return [{
      category: "stale-docs-and-skills",
      confidence: "probable",
      action: "ignore-with-rationale",
      source: "test-fixture",
      detail: "Synthetic docs drift finding for test mode",
    }];
  }

  const packageJson = JSON.parse(await fs.readFile("package.json", "utf8"));
  const docs = await fs.readFile("docs/maintenance/repo-hygiene.md", "utf8").catch(() => "");
  const findings = [];

  if (!packageJson.scripts?.["review:repo-hygiene"]) {
    findings.push({
      category: "stale-docs-and-skills",
      confidence: "confirmed",
      action: "ignore-with-rationale",
      source: "package-json-check",
      detail: "Missing review:repo-hygiene package script",
    });
  }

  if (docs && !docs.includes("workclaw-repo-hygiene-review")) {
    findings.push({
      category: "stale-docs-and-skills",
      confidence: "confirmed",
      action: "ignore-with-rationale",
      source: "docs-check",
      detail: "Repo hygiene doc is missing review skill reference",
    });
  }

  return findings;
}
```

- [ ] **Step 5: Run the script test suite**

Run: `node --test scripts/review-repo-hygiene.test.mjs`

Expected: PASS with all three categories observable in `test` mode.

- [ ] **Step 6: Run the real report command again**

Run: `pnpm review:repo-hygiene`

Expected: PASS with refreshed report artifacts and at least zero-or-more findings, not a crash.

- [ ] **Step 7: Commit the deterministic collectors**

```bash
git add scripts/lib/repo-hygiene/collect-deadcode-signals.mjs scripts/lib/repo-hygiene/collect-structure-signals.mjs scripts/lib/repo-hygiene/collect-drift-signals.mjs scripts/review-repo-hygiene.test.mjs
git commit -m "tooling: add repo hygiene signal collectors"
```

---

### Task 4: Add the Repo Hygiene Review and Cleanup Skills

**Files:**
- Create: `.agents/skills/workclaw-repo-hygiene-review/SKILL.md`
- Create: `.agents/skills/workclaw-cleanup-execution/SKILL.md`
- Modify: `docs/maintenance/repo-hygiene.md`

- [ ] **Step 1: Write the review skill**

Create `.agents/skills/workclaw-repo-hygiene-review/SKILL.md`:

```md
---
name: workclaw-repo-hygiene-review
description: Use when WorkClaw needs a structured review of orphan files, dead code, stale docs, duplicate implementations, or temporary artifacts before cleanup.
---

# WorkClaw Repo Hygiene Review

## Overview
Use this skill to classify repo hygiene findings before destructive edits. Prefer deterministic scanner output first, then summarize the highest-value cleanup batches.

## When to Use
- The user asks for cleanup, dead code review, orphan file review, or repo hygiene review.
- A large feature likely left follow-up artifacts or duplicate implementations.
- Docs or skills may have drifted from actual commands and paths.

## Workflow
1. Run `pnpm review:repo-hygiene`.
2. Read `.artifacts/repo-hygiene/report.json` and `summary.md`.
3. Group findings into:
   - safe-to-delete
   - likely-dead-needs-confirmation
   - duplicate-or-misplaced-needs-review
   - stale-doc-or-skill-reference
   - generated-or-runtime-owned-ignore
4. Recommend the smallest cleanup batch.
5. Do not delete files directly unless the user explicitly asks to execute an approved batch.
```

- [ ] **Step 2: Write the cleanup execution skill**

Create `.agents/skills/workclaw-cleanup-execution/SKILL.md`:

```md
---
name: workclaw-cleanup-execution
description: Use when WorkClaw has an approved repo hygiene cleanup batch and the work now needs to be executed safely with scoped verification.
---

# WorkClaw Cleanup Execution

## Overview
Use this skill only after a reviewed cleanup batch exists. Keep the batch narrow, preserve behavior unless removal is intentional, and route verification through `workclaw-change-verification`.

## Workflow
1. Confirm the approved candidate set.
2. Apply only the reviewed cleanup edits.
3. Avoid mixing unrelated refactors into the cleanup batch.
4. Run the smallest honest verification command set for the touched surface.
5. Report deleted items, downgraded items, retained uncertain items, and any still-unverified areas.

## Safety Rules
- Do not broaden scope while executing.
- Do not delete generated, runtime-owned, or dynamic surfaces without explicit evidence.
- Prefer deprecation or relocation over deletion when compatibility is unclear.
```

- [ ] **Step 3: Link the new skills from the maintenance doc**

Append this section to `docs/maintenance/repo-hygiene.md`:

```md
## Repo-Local Skills

- `workclaw-repo-hygiene-review`: classify findings and propose cleanup batches
- `workclaw-cleanup-execution`: execute an approved cleanup batch with scoped verification
```

- [ ] **Step 4: Verify skill discovery paths and docs**

Run: `rg -n "workclaw-repo-hygiene-review|workclaw-cleanup-execution" docs/maintenance/repo-hygiene.md .agents/skills/workclaw-repo-hygiene-review/SKILL.md .agents/skills/workclaw-cleanup-execution/SKILL.md`

Expected: matches in all three files.

- [ ] **Step 5: Commit the skills**

```bash
git add docs/maintenance/repo-hygiene.md .agents/skills/workclaw-repo-hygiene-review/SKILL.md .agents/skills/workclaw-cleanup-execution/SKILL.md
git commit -m "skills: add repo hygiene review workflows"
```

---

### Task 5: Final Verification and Maintainer Handoff

**Files:**
- Modify only if verification uncovers drift: `package.json`, `AGENTS.md`, `docs/maintenance/repo-hygiene.md`, `scripts/review-repo-hygiene*.mjs`, `scripts/lib/repo-hygiene/*.mjs`, `.agents/skills/workclaw-*/SKILL.md`

- [ ] **Step 1: Run the scoped script tests**

Run: `node --test scripts/review-repo-hygiene.test.mjs`

Expected: PASS.

- [ ] **Step 2: Run the aggregate hygiene report**

Run: `pnpm review:repo-hygiene`

Expected: PASS with output under `.artifacts/repo-hygiene/`.

- [ ] **Step 3: Run docs and skill smoke checks**

Run: `rg -n "review:repo-hygiene|workclaw-repo-hygiene-review|workclaw-cleanup-execution" AGENTS.md docs/maintenance/repo-hygiene.md .agents/skills`

Expected: all key references present and consistent.

- [ ] **Step 4: Decide whether `workclaw-change-verification` applies**

Use this rule:

```md
- Scripts changed: treat `node --test scripts/review-repo-hygiene.test.mjs` plus `pnpm review:repo-hygiene` as the minimum honest coverage.
- Repo-local skill docs changed only: no runtime command beyond the script test and smoke checks is required.
- If implementation expands into builtin skill assets, runtime code, or sidecar code, stop and add the matching WorkClaw verification commands before landing.
```

- [ ] **Step 5: Summarize rollout status in the final handoff**

Use this structure:

```md
## Rollout Summary
- Governance docs added:
- Package commands added:
- Signal collectors added:
- Skills added:
- Commands run:
- Output location:
- Remaining future work:
```

- [ ] **Step 6: Commit the final verification cleanups if needed**

```bash
git add AGENTS.md package.json docs/maintenance/repo-hygiene.md scripts/review-repo-hygiene.mjs scripts/review-repo-hygiene.test.mjs scripts/lib/repo-hygiene .agents/skills/workclaw-repo-hygiene-review/SKILL.md .agents/skills/workclaw-cleanup-execution/SKILL.md
git commit -m "chore: finalize repo hygiene governance rollout"
```

---

## Self-Review

### Spec Coverage

- Governance rules: covered by Task 1.
- Deterministic scanners and aggregate report: covered by Tasks 2 and 3.
- Review and cleanup skills: covered by Task 4.
- Verification and rollout posture: covered by Task 5.
- Non-blocking first release with no early CI hard-fails: preserved throughout the plan.

### Placeholder Scan

- No `TBD`, `TODO`, or deferred implementation placeholders remain.
- Every task names exact files and concrete commands.

### Consistency Check

- The command family consistently uses `review:repo-hygiene`.
- Skill names consistently use `workclaw-repo-hygiene-review` and `workclaw-cleanup-execution`.
- The plan keeps review separate from execution, matching the approved spec.
