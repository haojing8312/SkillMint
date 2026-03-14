# Workspace Skills Projection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an OpenClaw-style workspace skills projection layer to WorkClaw so each session run gets a projected `skills/` directory plus a generated skills prompt with explicit `SKILL.md` paths.

**Architecture:** Extend runtime preparation to resolve installed skills, materialize a session-local `work_dir/skills` tree, and generate a prompt block from those projected paths. Keep execution rooted at the session workspace so existing bash/file tool behavior stays intact while third-party skills gain stable relative paths.

**Tech Stack:** Rust, Tauri runtime, existing WorkClaw chat runtime pipeline, filesystem copy/unpack logic, Rust tests.

---

### Task 1: Add Workspace Skill Projection Types

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Test: `apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs`

**Step 1: Write the failing test**

Add tests covering:

- projected directory name derives from `skill_id`
- workspace snapshot entry includes `name`, `description`, and projected `SKILL.md` path
- generated prompt includes the projected location

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture`

Expected: FAIL because projection types/helpers do not exist yet.

**Step 3: Write minimal implementation**

In `chat_runtime_io.rs`, add minimal internal structs such as:

- `WorkspaceSkillEntry`
- `WorkspaceSkillSnapshot`

Add helper functions for:

- normalizing projected directory names from `skill_id`
- building projected `SKILL.md` paths
- formatting one prompt entry

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs
git commit -m "feat: add workspace skill projection metadata"
```

### Task 2: Resolve Installed Skills into Runtime Entries

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`
- Test: `apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs`

**Step 1: Write the failing test**

Add tests covering resolution of:

- local skill entry from `pack_path`
- builtin skill entry from embedded markdown
- encrypted skill entry from unpacked files

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture`

Expected: FAIL because runtime entries are not resolved yet.

**Step 3: Write minimal implementation**

Add runtime entry resolution helpers that:

- query visible installed skills
- read metadata from manifest/frontmatter
- expose enough info to project full skill content

Do not optimize for reuse yet.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/src/commands/skills.rs apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs
git commit -m "feat: resolve installed skills for runtime projection"
```

### Task 3: Project Skills into Session Workspace

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Test: `apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs`

**Step 1: Write the failing test**

Add tests covering:

- local skill copies `SKILL.md` and `scripts/`
- encrypted skill writes full unpacked tree
- builtin skill creates a projected directory with `SKILL.md`
- sync rebuilds `work_dir/skills`

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture`

Expected: FAIL because projection sync is missing.

**Step 3: Write minimal implementation**

Implement:

- `sync_skills_to_workspace(...)`
- deletion/rebuild of `<work_dir>/skills`
- per-source projection behavior

Use simple full rebuild semantics in v1.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs
git commit -m "feat: project installed skills into session workspace"
```

### Task 4: Generate OpenClaw-Style Skills Prompt

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Modify: `apps/runtime/src-tauri/src/agent/system_prompts/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs`
- Test: `apps/runtime/src-tauri/tests/test_system_prompt.rs`

**Step 1: Write the failing test**

Add tests asserting:

- system prompt includes a skills section when snapshot prompt is non-empty
- prompt tells the model to inspect available skills and read `SKILL.md` by location
- prompt contains projected skill paths

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_system_prompt -- --nocapture`

Expected: FAIL because no workspace skills section exists.

**Step 3: Write minimal implementation**

Add:

- `build_workspace_skills_prompt(...)`
- a system prompt section modeled after OpenClaw's mandatory skills instructions

Keep existing primary skill prompt behavior intact.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_system_prompt -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/src/agent/system_prompts/mod.rs apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs apps/runtime/src-tauri/tests/test_system_prompt.rs
git commit -m "feat: add workspace skills prompt to runtime system prompt"
```

### Task 5: Wire Projection into Session Preparation

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_send_message_flow.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_tool_setup.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`
- Test: `apps/runtime/src-tauri/tests/test_e2e_flow.rs`

**Step 1: Write the failing test**

Add an integration-style test asserting that session preparation:

- resolves a session `work_dir`
- projects skills into `<work_dir>/skills`
- passes generated skills prompt into final system prompt

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_e2e_flow -- --nocapture`

Expected: FAIL because session preparation does not project skills yet.

**Step 3: Write minimal implementation**

Update session preparation to:

- resolve visible skill entries
- sync them into the session workspace
- build a `WorkspaceSkillSnapshot`
- pass the snapshot prompt into system prompt composition

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_e2e_flow -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_send_message_flow.rs apps/runtime/src-tauri/src/commands/chat_tool_setup.rs apps/runtime/src-tauri/src/commands/chat_runtime_io.rs apps/runtime/src-tauri/tests/test_e2e_flow.rs
git commit -m "feat: wire workspace skill projection into session startup"
```

### Task 6: Add Regression Coverage for Bundled Skill Assets

**Files:**
- Test: `apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs`
- Test: `apps/runtime/src-tauri/tests/test_e2e_flow.rs`

**Step 1: Write the failing test**

Add regression coverage for a third-party style skill fixture containing:

- `SKILL.md`
- `scripts/tool.py`
- `assets/template.html`

Verify those files exist in the projected session workspace after preparation.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection test_e2e_flow -- --nocapture`

Expected: FAIL if non-`SKILL.md` assets are not copied.

**Step 3: Write minimal implementation**

Fix any missing recursive copy/unpack behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection test_e2e_flow -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_workspace_skills_projection.rs apps/runtime/src-tauri/tests/test_e2e_flow.rs apps/runtime/src-tauri/src/commands/chat_runtime_io.rs
git commit -m "test: cover projected skill asset availability"
```

### Task 7: Verify and Document

**Files:**
- Modify: `docs/plans/2026-03-14-workspace-skills-projection-design.md`
- Modify: `docs/plans/2026-03-14-workspace-skills-projection-implementation-plan.md`

**Step 1: Run focused verification**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_workspace_skills_projection -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_system_prompt -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_e2e_flow -- --nocapture
```

Expected: PASS

**Step 2: Run formatting if needed**

Run:

```bash
cargo fmt --manifest-path apps/runtime/src-tauri/Cargo.toml --all
```

Expected: no changes or only formatting changes

**Step 3: Update docs if implementation diverged**

Adjust design/plan docs to match final implementation details.

**Step 4: Commit**

```bash
git add docs/plans/2026-03-14-workspace-skills-projection-design.md docs/plans/2026-03-14-workspace-skills-projection-implementation-plan.md
git commit -m "docs: finalize workspace skills projection plan"
```
