---
name: workclaw-cleanup-execution
description: Use when executing a previously reviewed WorkClaw repo hygiene cleanup batch after review has already classified the candidates and selected the batch.
---

# WorkClaw Cleanup Execution

## Overview
Use this skill only to carry out a cleanup batch that was already reviewed and scoped. Keep execution narrow and traceable.

## When to Use
- A repo hygiene review already selected a cleanup batch
- The batch is limited to confirmed candidates
- Verification must follow the cleanup

## Workflow
1. Confirm the reviewed batch, the exact file list, and the exact reviewed action for each file.
2. Apply only the reviewed cleanup actions.
3. Keep changes limited to the approved batch.
4. Route verification through `workclaw-change-verification`.
5. Report any leftover unreviewed findings instead of expanding scope.

## Safety Rules
- Do not expand beyond the reviewed batch.
- Do not reinterpret findings during execution.
- Do not delete additional files "while here".
- Stop and escalate if the batch depends on uncertain or high-risk candidates.
- Use `workclaw-change-verification` for any required verification path.

## Output Shape
- Reviewed batch executed
- Files changed
- Verification route used
- Any remaining unaddressed findings
