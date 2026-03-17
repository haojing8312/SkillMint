---
name: workclaw-implementation-strategy
description: Use when changing runtime behavior, routing, provider integration, tool permissions, sidecar protocols, or vendor sync boundaries in WorkClaw before editing code.
---

# WorkClaw Implementation Strategy

## Overview
Use this skill to front-load architecture and compatibility thinking before changing risky parts of WorkClaw. The goal is to choose the smallest safe implementation path before code changes begin.

## When to Use
- Runtime behavior or chat orchestration changes
- Provider catalog, model routing, or integration changes
- Tool permission or approval-flow changes
- Sidecar bridge, browser automation, or protocol changes
- Vendor sync or upstream lane boundary changes
- IM, employee, or cross-surface routing changes

Do not use this for isolated copy edits or low-risk docs-only changes.

## Inspect First
- `apps/runtime/src/`
- `apps/runtime/src-tauri/src/`
- `apps/runtime/sidecar/src/`
- `packages/*/src/`
- Related tests in `apps/runtime/src/__tests__/`, `apps/runtime/src-tauri/tests/`, `apps/runtime/sidecar/test/`, and `packages/*/tests/`

## Required Output
- Changed surface area and affected modules
- Compatibility or rollout risks
- Recommended smallest safe implementation path
- Follow-on verification commands that will be required
- Release impact if the change touches packaging, vendors, or externally visible behavior

## Output Template
Use this shape:

```md
## Strategy Summary
- Change surface:
- Affected modules:
- Main risk:
- Recommended smallest safe path:
- Required verification:
- Release impact:
```

## Core Pattern
1. Identify the user-visible behavior or boundary being changed.
2. Trace which frontend, Tauri, sidecar, and Rust modules participate.
3. Prefer the narrowest change that preserves existing contracts.
4. Call out uncertainty before editing instead of guessing.
5. End with the verification and release-readiness follow-ups the change will require.

## Common Mistakes
- Treating sidecar, Tauri, and frontend changes as isolated when the feature crosses all three.
- Editing provider or routing behavior without checking test coverage in the adjacent layers.
- Assuming vendor sync changes are mechanical when they may alter release expectations or upstream compatibility.
