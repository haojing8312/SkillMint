---
name: workclaw-repo-hygiene-review
description: Use when reviewing WorkClaw repo hygiene findings from pnpm review:repo-hygiene, classifying candidates, and recommending the smallest safe cleanup batch without deleting files.
---

# WorkClaw Repo Hygiene Review

## Overview
Use this skill to triage repo hygiene findings and turn them into a small, reviewable cleanup recommendation. Keep the work read-only.

## When to Use
- `pnpm review:repo-hygiene` has findings to interpret
- Temporary artifacts, dead code, duplicates, stale docs, or orphan files need triage
- A cleanup batch needs to be scoped before any destructive edit

## Workflow
1. Run `pnpm review:repo-hygiene`.
2. Read `.artifacts/repo-hygiene/report.json` and `.artifacts/repo-hygiene/summary.md`.
3. Group findings by category and confidence.
4. Separate high-risk or uncertain items from confirmed cleanup candidates.
5. Recommend the smallest cleanup batch that is still well supported.
6. Stop after recommendation; do not delete, move, or rewrite files.

## Output Shape
- Findings grouped by category
- Confidence and risk noted for each group
- Smallest cleanup batch recommendation
- Any follow-up checks needed before execution

## Safety Rules
- Treat generated, runtime-owned, config-driven, and dynamically discovered files as high risk.
- Prefer compatibility fallbacks or deprecation when removal is not clearly safe.
- Do not perform cleanup actions in this skill.
