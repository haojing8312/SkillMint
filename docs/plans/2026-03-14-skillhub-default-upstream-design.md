# SkillHub Default Upstream Design

**Date:** 2026-03-14

**Goal:** Make SkillHub the default upstream for expert skill discovery and installation, with ClawHub retained only as a fallback path.

## Context

The current expert skill flows in WorkClaw are built around ClawHub:

- `技能库` reads from the ClawHub library command path.
- `找技能` reads from the ClawHub recommendation/search command path.
- installation still depends on ClawHub/GitHub metadata and download conventions.

That model is now unstable for two separate reasons:

1. ClawHub detail responses no longer reliably expose `github_url`.
2. The old ClawHub download proxy can return `400 Bad Request` even when the skill exists.

SkillHub provides a stronger China-friendly distribution model:

- a static catalog JSON with structured skill metadata
- a slug-based download endpoint that redirects to Tencent COS

This makes SkillHub a better default upstream for both discovery and installation.

## Decision

Adopt a **SkillHub-first** architecture:

- SkillHub becomes the default upstream for:
  - skill library listing
  - find-skills search/recommendation
  - skill installation download
- ClawHub remains available only as a fallback when:
  - SkillHub catalog data is unavailable
  - a slug is missing from SkillHub
  - SkillHub download fails

We will keep existing frontend command names and most UI copy in the first implementation phase to minimize surface-area changes. The behavioral change will happen behind the current Tauri command boundary.

## Recommended Approach

### Approach A: Full SkillHub-first behind existing commands

Use the current frontend commands and route them to SkillHub internally:

- `list_clawhub_library` -> SkillHub catalog by default
- `recommend_clawhub_skills` -> local recommendation over SkillHub catalog
- `install_clawhub_skill` -> SkillHub download by slug first, then fallback

**Why this is recommended**

- keeps frontend churn low
- gives us a stable rollout path
- fixes both search and install reliability together
- lets us defer renaming commands and UI copy until behavior is proven

### Approach B: Download-only migration

Keep ClawHub for list/search but use SkillHub only for downloads.

**Why not recommended**

- partial reliability improvement only
- still exposed to ClawHub metadata/search changes
- does not match the desired “SkillHub as default upstream” product goal

### Approach C: Dual-source aggregation

Merge SkillHub and ClawHub results in the default experience.

**Why not recommended now**

- duplicate resolution and ranking complexity
- harder debugging
- unnecessary for the current goal

## Architecture

### 1. Catalog Source

Add a SkillHub catalog fetcher in the runtime Tauri layer. It will read the published SkillHub JSON and normalize entries into the existing internal skill summary model.

Required source fields from SkillHub:

- `slug`
- `name`
- `description`
- `description_zh`
- `homepage`
- `owner`
- `downloads`
- `stars`
- `version`
- `tags`

Normalization rules:

- `slug` stays the canonical install identifier
- `homepage` maps to `source_url`
- GitHub URL is optional and should not be assumed
- `description_zh` may be preferred for Chinese UI fallback if present

### 2. Library Listing

The current library command should return SkillHub-backed results by default.

Behavior:

- sort by downloads first, then stars, then name
- derive visible tags from catalog tags
- support local caching so the expert library still opens when the network is temporarily unavailable

The command name can stay unchanged in the first phase even though the upstream changes.

### 3. Search / Find Skills

SkillHub does not need a dedicated search API for the first implementation because the catalog JSON already contains the searchable corpus.

We should do local scoring over:

- `name`
- `slug`
- `description`
- `description_zh`
- `tags`
- `owner`

Recommendation scoring should preserve the existing UX:

- return a small ranked list
- expose stars/downloads
- produce a short reason string based on matched fields

ClawHub search remains fallback only.

### 4. Installation

Installation should become:

1. try SkillHub download by slug
2. if that fails, try current ClawHub/GitHub fallback path
3. if fallback also fails, return a merged error with the most useful cause

The SkillHub install path should use:

- `https://lightmake.site/api/v1/download?slug=<slug>`

Expected behavior:

- endpoint returns redirect to COS zip
- zip is downloaded and extracted
- current local import pipeline remains unchanged after zip bytes are obtained

This is important: SkillHub should be treated as the default **artifact source**, not a replacement for the post-download import logic.

### 5. Fallback Policy

Fallback should be narrow and explicit:

- if SkillHub catalog fetch fails -> fall back to cached SkillHub catalog if present, otherwise ClawHub
- if SkillHub search has no slug match -> optionally fall back to ClawHub recommendation/search
- if SkillHub download fails -> fall back to existing ClawHub/GitHub install logic

The fallback should be observable in logs but not noisy in UI.

## UI / Product Behavior

Phase 1 should preserve current user-facing structure:

- `专家技能`
- `技能库`
- `找技能`
- current install button placement

We should avoid a branding rewrite in this phase. Users should simply experience:

- more stable lists
- more stable search
- more successful installs

Possible later phase:

- rename “ClawHub” wording where it is now misleading
- add a subtle “SkillHub 加速” indicator in install flows

## Error Handling

We should separate source failures from install failures:

- Catalog fetch failure:
  - use cache if available
  - otherwise fallback upstream
- Download failure:
  - log SkillHub failure reason
  - attempt fallback
- Import failure after successful download:
  - report as import/skill package issue, not upstream issue

User-facing messaging should remain short. Internal logs should include:

- slug
- attempted upstream
- resolved download URL
- fallback activation

## Testing Strategy

### Backend

Add tests for:

- SkillHub catalog normalization
- SkillHub-first list command behavior
- local recommendation over SkillHub catalog
- SkillHub download redirect handling
- fallback to ClawHub when SkillHub download fails

### Frontend

Add tests for:

- library view rendering SkillHub-derived fields through existing commands
- find-skills recommendations driven by SkillHub-backed command responses
- install flows still sending the correct slug and source metadata

### Verification

Manual verification should cover:

- `技能库` opens with real SkillHub-backed data
- `找技能` returns relevant recommendations
- install succeeds for a slug known to fail under old ClawHub proxy behavior

## Non-Goals

This phase does not include:

- replacing the local import pipeline
- rebranding all ClawHub terminology in UI
- building a combined SkillHub + ClawHub merged market
- introducing a mandatory external SkillHub CLI dependency

## Rollout Plan

Phase 1:

- SkillHub-first backend implementation
- fallback preserved
- existing frontend command shape retained

Phase 2:

- rename commands/types to source-neutral names
- refresh UI copy away from ClawHub-specific wording
- optionally expose source attribution in the UI
