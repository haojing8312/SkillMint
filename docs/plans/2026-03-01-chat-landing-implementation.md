# Chat Landing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a no-session landing page in chat view that introduces product capabilities for general users and starts a new session with optional initial task input.

**Architecture:** Keep existing sidebar + chat architecture. Add a dedicated `NewSessionLanding` React component rendered only when `activeMainView === "chat"` and no session is selected. Reuse existing session creation backend commands and extend app-level orchestration to optionally auto-send the first message after session creation.

**Tech Stack:** React 18, TypeScript, Vite, Tailwind CSS, Framer Motion, Tauri invoke API, Vitest + Testing Library (to be added).

---

### Task 1: Add Frontend Test Infrastructure

**Files:**
- Modify: `apps/runtime/package.json`
- Create: `apps/runtime/vitest.config.ts`
- Create: `apps/runtime/src/test/setup.ts`
- Modify: `apps/runtime/tsconfig.json`

**Step 1: Write the failing test command expectation (no test runner yet)**

Run: `cd apps/runtime && pnpm test`  
Expected: command not found / missing script

**Step 2: Add minimal test tooling**

- Add scripts in `apps/runtime/package.json`:
  - `"test": "vitest run"`
  - `"test:watch": "vitest"`
- Add dev dependencies:
  - `vitest`
  - `@testing-library/react`
  - `@testing-library/jest-dom`
  - `jsdom`
- Create `apps/runtime/vitest.config.ts`:
```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    globals: true,
  },
});
```
- Create `apps/runtime/src/test/setup.ts`:
```ts
import "@testing-library/jest-dom";
```
- Update `apps/runtime/tsconfig.json` `compilerOptions.types` to include `vitest/globals`.

**Step 3: Install dependencies**

Run: `pnpm install`  
Expected: lockfile updated and new test deps installed

**Step 4: Verify test runner works**

Run: `cd apps/runtime && pnpm test`  
Expected: PASS with `0 tests` (or no test files found, command succeeds)

**Step 5: Commit**

```bash
git add apps/runtime/package.json apps/runtime/vitest.config.ts apps/runtime/src/test/setup.ts apps/runtime/tsconfig.json pnpm-lock.yaml
git commit -m "test(runtime): add vitest and testing-library setup"
```

### Task 2: Build `NewSessionLanding` Component with Capability Intro

**Files:**
- Create: `apps/runtime/src/components/NewSessionLanding.tsx`
- Create: `apps/runtime/src/components/__tests__/NewSessionLanding.test.tsx`

**Step 1: Write failing component tests**

Create tests covering:
- renders hero title and subtitle for general users
- does not render quick-action chips
- submits on button click with entered text
- supports Enter submit and Shift+Enter newline
- shows recent session list and empty-state text

Example test skeleton:
```tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { NewSessionLanding } from "../NewSessionLanding";

test("submits entered task", () => {
  const onCreate = vi.fn();
  render(<NewSessionLanding sessions={[]} onSelectSession={() => {}} onCreateSessionWithInitialMessage={onCreate} creating={false} />);
  fireEvent.change(screen.getByPlaceholderText(/输入/i), { target: { value: "整理我的下载目录" } });
  fireEvent.click(screen.getByRole("button", { name: /开始新会话/i }));
  expect(onCreate).toHaveBeenCalledWith("整理我的下载目录");
});
```

**Step 2: Run test to verify failure**

Run: `cd apps/runtime && pnpm test -- NewSessionLanding`  
Expected: FAIL because component file does not exist

**Step 3: Implement minimal component**

Create `NewSessionLanding.tsx` with:
- hero title/subtitle (no "Skill" wording)
- static capability display blocks
- textarea + primary submit button
- keyboard handling (`Enter` submit, `Shift+Enter` newline)
- recent sessions section (max 6), empty state
- optional inline error text area

**Step 4: Run test to verify pass**

Run: `cd apps/runtime && pnpm test -- NewSessionLanding`  
Expected: PASS all tests in `NewSessionLanding.test.tsx`

**Step 5: Commit**

```bash
git add apps/runtime/src/components/NewSessionLanding.tsx apps/runtime/src/components/__tests__/NewSessionLanding.test.tsx
git commit -m "feat(runtime): add no-session landing component for chat view"
```

### Task 3: Integrate Landing into `App.tsx` View State

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Create: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`

**Step 1: Write failing integration tests**

Test cases:
- when chat view + no selected session: landing is rendered
- when session selected: `ChatView` is rendered
- when packaging view selected: `PackagingView` remains rendered

**Step 2: Run test to verify failure**

Run: `cd apps/runtime && pnpm test -- App.chat-landing`  
Expected: FAIL because conditions not implemented

**Step 3: Implement conditional rendering changes**

In `App.tsx`:
- import `NewSessionLanding`
- replace current no-session placeholder button branch
- render landing under: `selectedSkill && models.length > 0 && !selectedSessionId && activeMainView === "chat"`
- pass sessions and handlers required by landing

**Step 4: Run tests**

Run: `cd apps/runtime && pnpm test -- App.chat-landing`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx
git commit -m "feat(runtime): render chat landing page in no-session state"
```

### Task 4: Support Create Session with Optional Initial Message

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Create: `apps/runtime/src/__tests__/App.session-create-flow.test.tsx`

**Step 1: Write failing flow tests**

Test cases:
- `onCreateSessionWithInitialMessage("...")` creates session then invokes `send_message`
- empty input creates session only (no `send_message`)
- workspace picker cancel returns without state mutation

**Step 2: Run failing tests**

Run: `cd apps/runtime && pnpm test -- App.session-create-flow`  
Expected: FAIL because handler signature does not support initial message

**Step 3: Implement orchestration**

In `App.tsx`:
- refactor `handleCreateSession` to accept optional `initialMessage?: string`
- after `create_session` succeeds:
  - set selected session
  - reload sessions
  - if `initialMessage.trim()` exists, invoke `send_message` with created `sessionId`
- catch `send_message` error separately and keep session selected

**Step 4: Run tests**

Run: `cd apps/runtime && pnpm test -- App.session-create-flow`  
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.session-create-flow.test.tsx
git commit -m "feat(runtime): support initial task auto-send after session creation"
```

### Task 5: UX Polishing and Regression Verification

**Files:**
- Modify: `apps/runtime/src/components/NewSessionLanding.tsx`
- Optional modify: `apps/runtime/src/index.css`

**Step 1: Write failing UI detail tests**

Test:
- button shows `正在创建...` when `creating=true`
- error text visible when `error` prop exists
- recent sessions capped at 6 items

**Step 2: Run tests (fail)**

Run: `cd apps/runtime && pnpm test -- NewSessionLanding`  
Expected: FAIL before UI detail implementation

**Step 3: Implement polish**

- loading button state
- inline error style
- stable spacing and responsive widths matching existing blue system

**Step 4: Run all frontend tests**

Run: `cd apps/runtime && pnpm test`  
Expected: PASS all tests

**Step 5: Build verification**

Run: `cd apps/runtime && pnpm build`  
Expected: successful TypeScript + Vite build

**Step 6: Commit**

```bash
git add apps/runtime/src/components/NewSessionLanding.tsx apps/runtime/src/index.css
git commit -m "style(runtime): polish chat landing states and responsiveness"
```

### Task 6: End-to-End Manual Verification and Docs Sync

**Files:**
- Modify: `README.md` (if user-visible behavior notes are needed)
- Modify: `README.zh-CN.md` (same)

**Step 1: Manual verification checklist**

Run app:
```bash
pnpm app
```

Verify:
- chat/no-session shows landing with capability intro
- input task -> create -> auto-send -> chat enters correctly
- empty input -> empty session opens
- cancel folder picker -> no crash, remains landing
- recent session click navigates to chat
- packaging/settings unaffected

**Step 2: Docs update (minimal)**

- Add one short section: no-session landing entry and initial task behavior.

**Step 3: Final verification**

Run:
```bash
cd apps/runtime && pnpm test
cd apps/runtime && pnpm build
```
Expected: both pass

**Step 4: Commit**

```bash
git add README.md README.zh-CN.md
git commit -m "docs: describe new chat landing and first-task flow"
```

