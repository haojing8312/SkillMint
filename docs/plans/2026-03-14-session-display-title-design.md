# Session Display Title Design

**Date:** 2026-03-14

**Context**

WorkClaw runtime currently persists `sessions.title` with a default of `New Chat`. The only auto-rename path is in `send_message`, where the first user message is truncated to 20 characters and written back to the session title if the title is still `New Chat`.

That is too weak for the current product:

- new sessions remain `New Chat` until the first user message is sent
- generic opening messages such as `你好`, `继续`, `帮我一下` create poor titles
- team and employee sessions have structured context that should produce better display names
- the sidebar has no `openclaw`-style derived display fallback, so bad persisted titles directly leak into the UI

## Goal

Adopt an `openclaw`-style display chain for session naming:

- use structured labels first when available
- derive a display title when persisted title is generic
- keep persisted `title` for storage compatibility
- expose `display_title` for sidebar rendering

## Reference Mapping

In `reference/openclaw/ui/src/ui/app-render.helpers.ts`, the display name flow is:

- prefer `label`
- then prefer `displayName`
- then fall back to a derived human-readable name from session key semantics

WorkClaw does not have the same session key model, but it does have structured session metadata:

- `session_mode`
- `employee_id`
- `team_id`
- `title`

So the equivalent WorkClaw design is:

- derive `display_title` from structured metadata first
- fall back to user-message-derived title for general chats
- fall back to persisted title if it is meaningful
- only show `New Chat` as the final fallback

## Naming Strategy

### 1. Team sessions

For `session_mode = team_entry`:

- prefer team name
- fall back to persisted `title`
- fall back to `New Chat`

Expected result:

- `市场协作`
- `售前支持组`

### 2. Employee sessions

For `session_mode = employee_direct` and employee-assistant-like sessions:

- prefer employee display name
- fall back to persisted `title`
- fall back to `New Chat`

Expected result:

- `张三`
- `智能体员工助手`

### 3. General sessions

For `session_mode = general`:

- if persisted `title` is meaningful and not generic, use it
- otherwise derive a title from the first meaningful user message
- otherwise fall back to `New Chat`

Expected result:

- `修复登录接口超时`
- `整理本周销售周报`

## Meaningful Title Heuristic

The first version should stay deterministic and cheap. No LLM title generation.

### Generic titles to reject

Reject empty or low-information messages such as:

- `你好`
- `hi`
- `hello`
- `在吗`
- `继续`
- `开始`
- `帮我一下`
- `帮我处理`
- `继续上次`
- `继续刚才`

### Normalization

Before title extraction:

- trim whitespace
- collapse repeated spaces
- replace newlines with spaces
- remove leading punctuation noise where practical

### Truncation

Use a UI-friendly maximum length instead of the current hard `20 chars`.

Recommendation:

- derive up to `28` visible characters
- trim trailing punctuation
- do not append ellipsis in stored `title`
- optional UI ellipsis remains a rendering concern

### Multi-message fallback

If the first user message is generic, scan subsequent user messages until the first meaningful one.

This should only be used for derived `display_title` initially.

Persisted `title` may continue to be updated from the first meaningful message encountered during early conversation flow, but the display layer should not depend on that write succeeding.

## Data Contract

Extend the session list payload with:

- `display_title: string`

Keep existing `title` unchanged for compatibility.

The sidebar should render:

- `session.display_title || session.title`

Optimistic session creation should mirror the same priority rules so the user does not see a temporary `New Chat` flash.

## Backend Responsibilities

Implement in session listing:

- load session rows
- join team / employee context when available
- derive `display_title`

Implement title utility helpers in Rust:

- `is_generic_session_title`
- `normalize_candidate_session_title`
- `derive_session_display_title`

Keep `maybe_update_session_title_from_first_user_message_with_pool` but strengthen it to reject generic titles and normalize useful ones.

## Frontend Responsibilities

Update `SessionInfo` to include optional `display_title`.

Use `display_title` across:

- sidebar list
- optimistic session insert paths in `App.tsx`
- any list/search render path that currently displays raw `title`

## Search and Sorting

This first pass should keep search behavior unchanged unless the current sidebar search explicitly filters by displayed title.

If search only uses backend `title`, that is acceptable for the first iteration. A later follow-up can decide whether `display_title` should also participate in search indexing.

## Testing Scope

### Backend

- general session with meaningful first user message derives a display title
- generic first user message falls through to next meaningful user message
- team session returns team name as `display_title`
- employee session returns employee name as `display_title`
- persisted custom title wins over derived general fallback

### Frontend

- sidebar renders `display_title` when present
- optimistic created general session uses the first message when available
- optimistic team session uses team name
- fallback to `title` still works when `display_title` is absent

## Non-Goals

- LLM-generated titles
- retroactive migration of all historical stored titles
- changing database schema unless required for performance
- redesigning search or sorting semantics beyond displayed label correctness

## Recommended Rollout

Phase 1:

- add backend `display_title`
- update sidebar rendering
- improve general title heuristic

Phase 2, only if needed:

- align search with `display_title`
- add a lightweight background backfill for persisted titles
