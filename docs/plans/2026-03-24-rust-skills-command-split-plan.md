# Rust Skills Command Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Shrink `apps/runtime/src-tauri/src/commands/skills.rs` by extracting DTOs, pure helpers, local skill flows, and industry bundle flows into focused child modules while preserving the current Tauri command surface.

**Architecture:** Keep the root `skills.rs` as the public command shell. Move shared types into `types.rs`, pure helper logic into `helpers.rs`, local skill flows into `local_skill_service.rs`, and bundle install/update flows into `industry_bundle_service.rs`. Leave behavior unchanged and keep `list_skills` / `delete_skill` stable.

**Tech Stack:** Rust, Tauri commands, sqlx, SQLite, WorkClaw runtime tests

---

### Task 1: Add the child module skeleton

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/skills/types.rs`
- Create: `apps/runtime/src-tauri/src/commands/skills/helpers.rs`
- Create: `apps/runtime/src-tauri/src/commands/skills/local_skill_service.rs`
- Create: `apps/runtime/src-tauri/src/commands/skills/industry_bundle_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`

**Step 1: Add the module declarations**

- Declare the new sibling modules from `skills.rs`
- Re-export the DTOs that other modules already consume through `skills.rs`

**Step 2: Move the pure helper functions**

- Move slug, markdown, tag, semver, and display-name helpers into `helpers.rs`
- Keep the helper APIs private unless the root file or a sibling service needs them

**Step 3: Compile-check the split**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS

### Task 2: Move local skill flows

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/local_skill_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/helpers.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/types.rs`

**Step 1: Move**

- `ensure_skill_display_name_available`
- `render_local_skill_preview`
- `create_local_skill`
- `import_local_skill_to_pool`
- `import_local_skill`
- `refresh_local_skill`

**Step 2: Keep command behavior stable**

- Keep the same return payloads and error strings where practical
- Keep `DbState` usage compatible with existing callers

**Step 3: Add or preserve focused tests**

- keep the existing local-skill command tests green

### Task 3: Move industry bundle flows

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/industry_bundle_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/helpers.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/types.rs`

**Step 1: Move**

- `install_industry_bundle_to_pool`
- `install_industry_bundle`
- `check_industry_bundle_update_from_pool`
- `check_industry_bundle_update`

**Step 2: Preserve comparison semantics**

- keep semver ordering and tag extraction behavior unchanged

**Step 3: Verify**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS

### Task 4: Trim the root command shell

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/types.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/helpers.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/local_skill_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/industry_bundle_service.rs`

**Step 1: Leave only the stable entrypoints**

- keep `list_skills` and `delete_skill`
- keep any command wrappers required for Tauri visibility

**Step 2: Re-export only what callers need**

- do not leave unused re-exports behind if they can be avoided

**Step 3: Run verification**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS
