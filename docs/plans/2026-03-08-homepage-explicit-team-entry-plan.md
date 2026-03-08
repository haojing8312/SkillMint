# Homepage Explicit Team Entry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the homepage always create general single-assistant sessions, and only enter multi-employee team collaboration from explicit team entry points.

**Architecture:** Introduce explicit session modes in the runtime data model, route team orchestration only for `team_entry` sessions, and split homepage launch UX from employee/team launch UX in the React shell. Homepage becomes a general assistant launcher with optional team shortcut cards; employee pages keep explicit direct-chat and team-entry actions.

**Tech Stack:** React, TypeScript, Tauri, Rust, SQLx, Vitest, Cargo tests

---

### Task 1: Add explicit session mode fields in the runtime model

**Files:**
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src-tauri/tests/test_employee_groups_db.rs`

**Step 1: Write the failing database/runtime test**

Add a test in `apps/runtime/src-tauri/tests/test_employee_groups_db.rs` that creates a session and asserts the stored row contains:

```rust
assert_eq!(session["session_mode"], "general");
assert_eq!(session["team_id"], "");
```

Also add a second assertion path for explicit team entry:

```rust
assert_eq!(session["session_mode"], "team_entry");
assert_eq!(session["team_id"], group_id);
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_employee_groups_db
```

Expected: FAIL because `sessions` does not expose `session_mode` / `team_id`.

**Step 3: Write minimal schema and command changes**

In `apps/runtime/src-tauri/src/db.rs`:

- add `session_mode TEXT NOT NULL DEFAULT 'general'`
- add `team_id TEXT NOT NULL DEFAULT ''`
- keep dev-data assumptions simple; no complex migration layer

In `apps/runtime/src-tauri/src/commands/chat.rs`:

- extend `create_session(...)` to accept `session_mode: Option<String>` and `team_id: Option<String>`
- normalize values to:

```rust
match session_mode.unwrap_or_default().trim() {
    "employee_direct" => "employee_direct",
    "team_entry" => "team_entry",
    _ => "general",
}
```

- if mode is not `team_entry`, always store empty `team_id`

In `apps/runtime/src/types.ts` add:

```ts
session_mode?: "general" | "employee_direct" | "team_entry";
team_id?: string;
```

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_employee_groups_db
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src/types.ts apps/runtime/src-tauri/tests/test_employee_groups_db.rs
git commit -m "feat: add explicit session modes"
```

### Task 2: Gate team orchestration by explicit team-entry sessions only

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`

**Step 1: Write the failing backend behavior tests**

Add tests in `apps/runtime/src-tauri/tests/test_im_employee_agents.rs` for:

```rust
assert!(maybe_handle_team_entry_session_message_with_pool(...general_session...).await?.is_none());
assert!(maybe_handle_team_entry_session_message_with_pool(...employee_direct_session...).await?.is_none());
assert_eq!(maybe_handle_team_entry_session_message_with_pool(...team_entry_session...).await?.unwrap().group_id, expected_group_id);
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents
```

Expected: FAIL because the current implementation infers team entry from `sessions.employee_id`.

**Step 3: Write minimal backend implementation**

In `apps/runtime/src-tauri/src/commands/employee_agents.rs`:

- change `maybe_handle_team_entry_session_message_with_pool(...)` to query:

```sql
SELECT COALESCE(session_mode, 'general'), COALESCE(team_id, '')
FROM sessions
WHERE id = ?
```

- return `Ok(None)` unless:

```rust
session_mode.eq_ignore_ascii_case("team_entry") && !team_id.trim().is_empty()
```

- load the exact team by `team_id`, not by matching `entry_employee_id`

In `apps/runtime/src-tauri/src/commands/chat.rs` keep the `send_message(...)` hook, but it now depends on explicit mode instead of implicit employee matching.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/tests/test_im_employee_agents.rs
git commit -m "fix: require explicit team entry for group runs"
```

### Task 3: Split homepage launch mode from employee launch mode in the app shell

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/types.ts`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`
- Test: `apps/runtime/src/__tests__/App.employee-chat-entry.test.tsx`

**Step 1: Write the failing React tests**

Add assertions covering:

- homepage `handleCreateSession(...)` sends `sessionMode: "general"` and empty `teamId`
- employee direct entry sends `sessionMode: "employee_direct"`
- explicit team entry sends `sessionMode: "team_entry"` and correct `teamId`

Example expectation:

```ts
expect(invoke).toHaveBeenCalledWith("create_session", expect.objectContaining({
  sessionMode: "general",
  teamId: "",
}));
```

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.chat-landing.test.tsx src/__tests__/App.employee-chat-entry.test.tsx
```

Expected: FAIL because `create_session` calls do not include explicit mode fields.

**Step 3: Write minimal app-shell implementation**

In `apps/runtime/src/App.tsx`:

- introduce explicit launch context, for example:

```ts
type SessionLaunchMode = "general" | "employee_direct" | "team_entry";
const [pendingLaunchMode, setPendingLaunchMode] = useState<SessionLaunchMode>("general");
const [pendingLaunchTeamId, setPendingLaunchTeamId] = useState("");
```

- homepage `handleCreateSession(...)` should always call:

```ts
sessionMode: "general",
teamId: "",
employeeId: "",
```

- employee direct path should call:

```ts
sessionMode: "employee_direct",
teamId: "",
employeeId: employee.employee_id || employee.role_id || "",
```

- explicit team entry path should call:

```ts
sessionMode: "team_entry",
teamId,
employeeId: teamEntryEmployeeId,
```

- stop reusing `selectedEmployeeId` as homepage launch source

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.chat-landing.test.tsx src/__tests__/App.employee-chat-entry.test.tsx
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/types.ts apps/runtime/src/__tests__/App.chat-landing.test.tsx apps/runtime/src/__tests__/App.employee-chat-entry.test.tsx
git commit -m "feat: separate homepage and employee session launch modes"
```

### Task 4: Add homepage team shortcut cards and explicit team-launch UX

**Files:**
- Modify: `apps/runtime/src/components/NewSessionLanding.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/components/__tests__/NewSessionLanding.test.tsx`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`

**Step 1: Write the failing component tests**

Add tests for:

- homepage renders a “团队协作入口” section when teams exist
- clicking a team shortcut calls the new app callback with the chosen `teamId`
- homepage normal submit does not call the team callback

Example:

```ts
expect(screen.getByText("团队协作入口")).toBeInTheDocument();
fireEvent.click(screen.getByRole("button", { name: /交给团队处理/i }));
expect(onCreateTeamEntrySession).toHaveBeenCalledWith(expect.objectContaining({ teamId: "group-1" }));
```

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/NewSessionLanding.test.tsx src/__tests__/App.chat-landing.test.tsx
```

Expected: FAIL because the landing page has no team shortcut section yet.

**Step 3: Write minimal UI implementation**

In `apps/runtime/src/components/NewSessionLanding.tsx`:

- add props for a lightweight team list:

```ts
teams: Array<{ id: string; name: string; description?: string; memberCount?: number }>;
onCreateTeamEntrySession: (input: { teamId: string; initialMessage: string }) => void;
```

- render a compact “团队协作入口” block below the main composer
- each card uses explicit CTA text like `交给团队处理`

In `apps/runtime/src/App.tsx`:

- pass seeded/custom teams to `NewSessionLanding`
- wire the CTA to explicit `team_entry` session creation

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/NewSessionLanding.test.tsx src/__tests__/App.chat-landing.test.tsx
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/NewSessionLanding.tsx apps/runtime/src/App.tsx apps/runtime/src/components/__tests__/NewSessionLanding.test.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx
git commit -m "feat: add explicit homepage team shortcuts"
```

### Task 5: Make employee/team entry labels explicit and verify end-to-end behavior

**Files:**
- Modify: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Modify: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx`
- Modify: `apps/runtime/src/__tests__/App.employee-chat-entry.test.tsx`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`

**Step 1: Write the failing UI and flow tests**

Add tests for:

- employee page direct-chat CTA remains single-employee
- team CTA uses explicit label like `以团队模式发起任务`
- created team sessions still open the collaboration panel

Example:

```ts
expect(screen.getByRole("button", { name: "以团队模式发起任务" })).toBeInTheDocument();
```

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx src/__tests__/App.employee-chat-entry.test.tsx
```

Expected: FAIL because the current employee entry wording and session mode wiring are not explicit enough.

**Step 3: Write minimal implementation**

In `apps/runtime/src/components/employees/EmployeeHubView.tsx`:

- separate direct employee launch CTA from team launch CTA
- keep labels explicit:
  - `与该员工开始对话`
  - `以团队模式发起任务`

In app/runtime glue:

- keep existing ability to open historical team sessions
- do not auto-convert ordinary employee sessions into team sessions

**Step 4: Run full targeted regression**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.chat-landing.test.tsx src/__tests__/App.employee-chat-entry.test.tsx src/components/__tests__/NewSessionLanding.test.tsx src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_employee_groups_db
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/employees/EmployeeHubView.tsx apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx apps/runtime/src/__tests__/App.employee-chat-entry.test.tsx
git commit -m "feat: make team collaboration entry explicit"
```

### Task 6: Final verification and cleanup

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/NewSessionLanding.tsx`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Test: targeted suites above

**Step 1: Run desktop-facing regression suites**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/__tests__/App.chat-landing.test.tsx src/__tests__/App.employee-chat-entry.test.tsx src/components/__tests__/NewSessionLanding.test.tsx src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx src/__tests__/App.model-setup-hint.test.tsx
```

Expected: PASS

**Step 2: Run runtime regressions**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_employee_groups_db
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents
```

Expected: PASS

**Step 3: Manual verification**

- Homepage input `你好` opens a general chat, not a team run
- Homepage team shortcut opens team collaboration
- Employee page direct entry opens employee chat
- Employee page explicit team CTA opens team collaboration
- History list can still reopen an old team session

**Step 4: Final commit**

```bash
git add apps/runtime/src docs/plans apps/runtime/src-tauri/src apps/runtime/src-tauri/tests
git commit -m "feat: require explicit team entry for multi-employee collaboration"
```
