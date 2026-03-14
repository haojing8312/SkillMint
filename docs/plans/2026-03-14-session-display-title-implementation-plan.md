# Session Display Title Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an `openclaw`-style session display title chain so WorkClaw sidebar names prefer team/employee context or meaningful user intent instead of defaulting to `New Chat`.

**Architecture:** Keep persisted `sessions.title` for compatibility, but add a backend-derived `display_title` in `list_sessions`. The frontend will render `display_title` preferentially and optimistic session creation will use the same naming rules. General-chat title derivation remains deterministic and non-LLM-based.

**Tech Stack:** Rust, SQLx, SQLite, React, TypeScript, Vitest

---

### Task 1: Document the naming rules in code-facing tests

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_chat_commands.rs`
- Modify: `apps/runtime/src/__tests__/App.session-create-flow.test.tsx`

**Step 1: Write the failing backend tests**

Add tests covering:

- general session returns `display_title` from a meaningful first user message
- generic first user message does not become the display title
- team session returns team name as `display_title`
- employee session returns employee name as `display_title`

**Step 2: Run backend tests to verify they fail**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_chat_commands -- --nocapture
```

Expected:

- failing assertions because `display_title` does not yet exist or still resolves to `New Chat`

**Step 3: Write the failing frontend tests**

Add tests covering:

- sidebar prefers `display_title` over `title`
- optimistic created session can show a non-generic derived title when input is available

**Step 4: Run frontend tests to verify they fail**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.session-create-flow.test.tsx
```

Expected:

- failing assertions because UI still renders raw `title`

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_chat_commands.rs apps/runtime/src/__tests__/App.session-create-flow.test.tsx
git commit -m "test(runtime): capture session display title rules"
```

### Task 2: Add backend title-derivation helpers

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`

**Step 1: Add deterministic helper functions**

Implement helpers for:

- `is_generic_session_title`
- `normalize_candidate_session_title`
- `derive_meaningful_session_title_from_messages`

Rules:

- trim and normalize whitespace
- reject low-information openings
- truncate to a stable UI-friendly length

**Step 2: Update persisted-title write path**

Strengthen `maybe_update_session_title_from_first_user_message_with_pool` so it does not write generic titles.

**Step 3: Add `display_title` derivation in session listing**

In `list_sessions_with_pool`:

- join or resolve team names for `team_entry`
- join or resolve employee names for employee sessions
- derive `display_title` for general sessions from stored messages when needed

**Step 4: Run backend tests**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_chat_commands -- --nocapture
```

Expected:

- new display-title tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/src/commands/chat_session_io.rs apps/runtime/src-tauri/tests/test_chat_commands.rs
git commit -m "feat(runtime): derive session display titles"
```

### Task 3: Extend the frontend session model and sidebar rendering

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/Sidebar.tsx`
- Modify: `apps/runtime/src/App.tsx`

**Step 1: Extend TypeScript types**

Add optional:

```ts
display_title?: string;
```

to the session shape used by the app.

**Step 2: Update sidebar rendering**

Render:

```ts
session.display_title || session.title
```

where session names are shown.

**Step 3: Align optimistic session creation**

Update optimistic session helpers in `App.tsx` so:

- team sessions prefer team name
- employee sessions prefer employee name when available
- general sessions can use the initial prompt when it is meaningful

Keep fallback behavior deterministic.

**Step 4: Run frontend tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.session-create-flow.test.tsx
```

Expected:

- updated session-name assertions pass

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src/components/Sidebar.tsx apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.session-create-flow.test.tsx
git commit -m "feat(runtime): render derived session display titles"
```

### Task 4: Verify no regressions in session list behavior

**Files:**
- Modify only if needed after verification

**Step 1: Run TypeScript compile**

Run:

```bash
pnpm --dir apps/runtime exec tsc --noEmit
```

Expected:

- exit 0

**Step 2: Run targeted frontend regression tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.session-create-flow.test.tsx src/__tests__/App.session-search-global.test.tsx
```

Expected:

- pass

**Step 3: Run targeted Rust regression tests**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_chat_commands -- --nocapture
```

Expected:

- pass

**Step 4: Manual verification**

Verify in app:

- create a blank general session and send `你好`; sidebar should remain generic until a meaningful prompt appears
- create a general session and send `帮我整理本周销售周报`; sidebar should show a meaningful title
- create a team-entry session; sidebar should show team name immediately

**Step 5: Commit**

```bash
git add -A
git commit -m "test(runtime): verify session display title rollout"
```

### Task 5: Document rollout behavior

**Files:**
- Modify: `docs/plans/2026-03-14-session-display-title-design.md`
- Modify: `docs/plans/2026-03-14-session-display-title-implementation-plan.md`

**Step 1: Record any deviations from the original design**

Update docs if implementation differs from the plan.

**Step 2: Note any follow-up items**

Capture possible future work:

- search by `display_title`
- persisted-title backfill
- session rename command

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-session-display-title-design.md docs/plans/2026-03-14-session-display-title-implementation-plan.md
git commit -m "docs(runtime): document session display title rollout"
```
