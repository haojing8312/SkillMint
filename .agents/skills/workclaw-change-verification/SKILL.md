---
name: workclaw-change-verification
description: Use when changes affect runtime code, tests, skill assets, or build and test behavior in WorkClaw and verification is required before claiming completion.
---

# WorkClaw Change Verification

## Overview
Use this skill to decide what "verified" means for the changed area in WorkClaw. Reuse the repo's existing commands and run the smallest command set that honestly covers the touched surface.

## When to Use
- React runtime code changed
- Tauri backend code changed
- Sidecar runtime or adapters changed
- Rust package logic changed
- Builtin skill assets or skill-related crates changed
- Build or test behavior changed
- UI flows changed in a way that can affect user behavior

Do not use this for docs-only changes that cannot affect runtime, tests, packaging, or build behavior.

## Command Routing
- Use `pnpm test:sidecar` for `apps/runtime/sidecar/` changes.
- Use `pnpm test:rust-fast` for `apps/runtime/src-tauri/` or `packages/*` Rust logic changes.
- Use `pnpm test:builtin-skills` for builtin skill asset or skill crate changes.
- Use `pnpm test:e2e:runtime` for user-facing runtime flows that need end-to-end coverage.
- Use `pnpm build:runtime` when desktop packaging, build output, or integration boundaries may be affected.

When multiple surfaces changed, combine the relevant commands instead of picking only one.

## Required Output
- Commands run
- Pass or fail results
- Which changed areas were covered by each command
- Any areas still unverified
- Whether it is valid to claim the work is verified

## Output Template
Use this shape:

```md
## Verification Summary
- Changed surface:
- Commands run:
- Results:
- Covered areas:
- Still unverified:
- Verification verdict:
```

## Core Pattern
1. Map changed files to runtime surface areas.
2. Select the smallest honest command set.
3. Run the commands before claiming completion.
4. Report verification concretely, not vaguely.

## Common Mistakes
- Saying "verified" after reading tests without running them.
- Running only frontend tests for a change that also affects Tauri or sidecar behavior.
- Forgetting `pnpm test:builtin-skills` when skill assets or skill-related crates changed.
