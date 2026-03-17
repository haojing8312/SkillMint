---
name: workclaw-release-readiness
description: Use when changes affect versioning, installer branding, release docs, packaging outputs, or vendor release lanes in WorkClaw before deciding a branch is ready to ship.
---

# WorkClaw Release Readiness

## Overview
Use this skill for shipping-sensitive changes. It separates release readiness from ordinary code verification and produces a clear ship recommendation with evidence.

## When to Use
- Version files or release metadata changed
- Installer branding or installer checks changed
- Release docs changed
- Packaging outputs or desktop build expectations changed
- Vendor lane or upstream sync release paths changed
- A branch is being assessed for ship readiness after release-sensitive work

Do not use this for ordinary feature work that does not touch release-sensitive surfaces.

## Command Routing
- Run `pnpm release:check-version` for version and release metadata changes.
- Run `pnpm test:release` for release-path validation.
- Run `pnpm test:installer` for installer branding or installer flow changes.
- Run `pnpm test:release-docs` for release note or release document changes.
- Run `pnpm test:openclaw-vendor-lane` for vendor sync or lane changes.
- Run `pnpm build:runtime` when packaging output or desktop build behavior changed.

## Required Output
- Release verdict: `GREEN`, `YELLOW`, or `RED`
- Blocking issues, if any
- Required release-note or documentation follow-ups
- Final recommendation on whether the branch is safe to ship

## Output Template
Use this shape:

```md
## Release Readiness
- Verdict: GREEN | YELLOW | RED
- Changed release surface:
- Commands run:
- Blocking issues:
- Required follow-ups:
- Ship recommendation:
```

## Core Pattern
1. Identify which release-sensitive surface changed.
2. Run the matching release commands.
3. Start from "safe to ship" and only downgrade when there is concrete evidence.
4. If blocked, provide an explicit unblock checklist.

## Common Mistakes
- Treating release-doc or installer changes as low-risk and skipping dedicated checks.
- Reporting a release verdict without citing commands or evidence.
- Mixing ordinary runtime verification with ship-readiness judgment and losing the release signal.
