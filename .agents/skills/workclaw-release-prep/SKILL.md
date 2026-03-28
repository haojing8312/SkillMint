---
name: workclaw-release-prep
description: Use when preparing a WorkClaw desktop release and you need AI to recommend the next version number, summarize changes since the last tag, and draft bilingual Chinese plus English release notes for human confirmation before publishing.
---

# WorkClaw Release Prep

## Overview
Use this skill before cutting a WorkClaw release. It recommends the next version, explains why, and drafts bilingual release notes. It does not publish anything.

## When to Use
- The user wants a release recommendation
- The user wants AI to decide between `patch`, `minor`, or `major`
- The user wants bilingual Chinese + English release notes
- The user wants a confirmation-ready release package before tagging

Do not use this skill to push commits, create tags, or publish releases. Use `workclaw-release-publish` after confirmation.

## Inspect First
- `git tag --sort=-creatordate`
- `git log <last-tag>..HEAD --oneline`
- `git log <last-tag>..HEAD --format=%H%n%s%n%b%n==END==`
- `git diff --name-only <last-tag>..HEAD`
- For high-signal commits, inspect `git show --stat --summary <sha>`
- `apps/runtime/package.json`
- `apps/runtime/src-tauri/Cargo.toml`
- `apps/runtime/src-tauri/tauri.conf.json`
- `.github/release-windows-notes.md`
- Any release-impacting docs or plans the user wants included

Before drafting release notes, build a short `candidate user-visible changes` list from:
- commit titles and commit bodies
- test names that describe user-facing behavior
- provider or model names mentioned explicitly, such as `Qwen`, `DeepSeek`, `OpenAI`, `Claude`
- compatibility, routing, transport, recovery, installer, branding, and skill/runtime keywords

Default rule: if a commit message or test name names a concrete model, provider, or desktop behavior, treat it as a release-note candidate unless there is strong evidence it is purely internal.

## Version Recommendation Rules
- Recommend `patch` for bug fixes, stability hardening, recovery improvements, small UX polish, or non-breaking desktop flow fixes
- Recommend `minor` for clear new user-facing capabilities, new workflows, new navigation models, or meaningful product surface expansion
- Recommend `major` only for breaking changes, migrations, compatibility breaks, or strong user behavior changes that require explicit upgrade communication

When uncertain, provide:
- one recommended version
- one conservative alternative
- a short rationale for both

## Required Output
Use this shape:

```md
## Release Prep
- Last tag:
- Recommended version:
- Alternative version:
- Why:
- Release scope:
- Files or areas reviewed:

## Candidate Highlights
- Included in notes:
- Intentionally omitted:
- Follow-up checks:

## Release Notes Draft
### 中文
- ...

### English
- ...

## Confirmation Needed
- Confirm version:
- Confirm release notes:
- Ready for publish skill: yes | no
```

## Drafting Rules
- Keep release notes concise and user-facing
- Group changes by outcomes, not by files
- Mention desktop installer guidance only if relevant
- Avoid speculative claims not backed by repo changes
- Chinese and English sections should say the same thing, not different things
- Start from the candidate highlight list, then compress; do not draft directly from file paths alone
- When a change improves compatibility for a named provider or model, prefer stating that user-facing outcome plainly
- If a candidate highlight is omitted from the final notes, explain why in `Intentionally omitted`

## Common Mistakes
- Recommending `minor` for pure bugfix bundles without a new capability
- Recommending `patch` when a clearly new workflow shipped
- Mixing internal implementation detail into user-facing release notes
- Reading only changed file paths and missing user-visible gains described in commit bodies or tests
- Treating provider/model compatibility fixes as internal-only by default when they change whether users can actually run that provider
- Proceeding to publish without explicit human confirmation
