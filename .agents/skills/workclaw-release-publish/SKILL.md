---
name: workclaw-release-publish
description: Use when publishing a confirmed WorkClaw desktop release after the version number and bilingual Chinese plus English release notes have already been reviewed and approved by a human.
---

# WorkClaw Release Publish

## Overview
Use this skill only after the release version and bilingual release notes are confirmed. It updates release metadata, commits the release, pushes the branch, creates and pushes the tag, and produces a local Windows package.

## When to Use
- The user has already confirmed the target version
- The user has already confirmed the Chinese + English release notes
- The user wants the tag pushed to trigger the GitHub Windows release workflow
- The user wants a local `.exe` or `.msi` generated as part of the release

If the version or release notes are not yet confirmed, stop and use `workclaw-release-prep`.

## Required Inputs
- Confirmed version in `vX.Y.Z` format
- Confirmed bilingual release notes content

## Files To Update
- `apps/runtime/package.json`
- `apps/runtime/src-tauri/Cargo.toml`
- `apps/runtime/src-tauri/tauri.conf.json`
- `.github/release-windows-notes.md`
- `apps/runtime/src-tauri/Cargo.lock` when refreshed by build tooling

## Execution Order
1. Verify the requested tag format and confirm the version files match the intended release
2. Re-read the approved release notes and sanity-check that they still cover the major user-visible outcomes discovered during release prep
3. Update the release notes template with the approved bilingual content
4. If an important user-facing outcome is missing, stop and ask for note confirmation again instead of publishing a partial summary
5. Run release checks:
   - `pnpm release:check-version <tag>`
   - `pnpm test:release`
   - `pnpm test:installer`
   - `pnpm test:release-docs`
6. Run user-flow or packaging verification when release surface changed:
   - `pnpm test:e2e:runtime`
   - `pnpm build:runtime`
7. Commit the release changes
8. Push `main`
9. Create and push the tag
10. Report local installer paths and note that the remote tag triggers `.github/workflows/release-windows.yml`

## Required Output
Use this shape:

```md
## Publish Summary
- Version:
- Commit:
- Tag:
- Commands run:
- Local artifacts:
- Remote trigger:

## Verification Summary
- Changed surface:
- Results:
- Still unverified:
- Verification verdict:

## Release Readiness
- Verdict:
- Blocking issues:
- Ship recommendation:
```

## Guardrails
- Never auto-publish based only on AI-generated version suggestions
- Never create a release tag before release checks pass
- Never claim GitHub release completion unless the tag push succeeded
- If packaging fails, report the failure and do not describe the release as complete
- Never silently publish if the approved notes are missing a major provider/model compatibility fix or other concrete user-facing outcome discovered during prep

## Common Mistakes
- Publishing from an unconfirmed draft
- Forgetting to update all three runtime version files
- Pushing a tag that does not match app versions
- Reporting only local packaging while skipping release checks
