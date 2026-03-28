# Builtin Office Skill Runtime Upgrade Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade WorkClaw builtin skills to support full file-tree execution and replace the builtin Office skills with the vendored MiniMax Office skill implementations.

**Architecture:** Extend builtin skill metadata from markdown-only assets to embedded file trees, then project builtin skills into the runtime workspace exactly like local/encrypted skills. Keep the existing builtin Office skill IDs stable while replacing their underlying assets with the MiniMax skill directories so the rest of the product can keep referencing the same IDs.

**Tech Stack:** Rust, Tauri, `runtime-skill-core`, workspace skill projection, vendored skill assets, targeted Rust tests.

---

### Task 1: Define builtin file-tree support in skill core

**Files:**
- Modify: `packages/runtime-skill-core/Cargo.toml`
- Modify: `packages/runtime-skill-core/src/builtin_skills.rs`
- Modify: `packages/runtime-skill-core/src/lib.rs`
- Test: `packages/runtime-skill-core/tests/builtin_skills.rs`

**Step 1: Write the failing tests**

Add tests that assert builtin Office entries expose more than `SKILL.md` and that builtin prompt lookup still returns the embedded `SKILL.md` content.

**Step 2: Run tests to verify they fail**

Run: `cargo test -p runtime-skill-core builtin_skills -- --nocapture`

Expected: FAIL because builtin entries currently expose markdown only.

**Step 3: Implement builtin file-tree metadata**

Add an embedded file-tree representation for builtin skills, expose helper APIs for builtin entry lookup and `SKILL.md` extraction, and keep the existing ID constants stable.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p runtime-skill-core builtin_skills -- --nocapture`

Expected: PASS

### Task 2: Upgrade builtin runtime projection to use full file trees

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`
- Modify: `apps/runtime/src-tauri/src/db/seed.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`

**Step 1: Write the failing tests**

Add or update tests so builtin skill projection expects copied asset directories for builtin Office skills, not just a generated `SKILL.md`.

**Step 2: Run tests to verify they fail**

Run: `cargo test -p runtime workspace_skill_projection -- --nocapture`

Expected: FAIL because builtin runtime projection currently fabricates a one-file tree.

**Step 3: Implement minimal runtime changes**

Wire builtin runtime entry resolution and seeding to the new builtin file-tree APIs while preserving manifest names, descriptions, ids, and `source_type = 'builtin'`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p runtime workspace_skill_projection -- --nocapture`

Expected: PASS

### Task 3: Vendor MiniMax Office skill assets into builtin Office skill directories

**Files:**
- Replace contents under: `apps/runtime/src-tauri/builtin-skills/docx/`
- Replace contents under: `apps/runtime/src-tauri/builtin-skills/xlsx/`
- Replace contents under: `apps/runtime/src-tauri/builtin-skills/pdf/`
- Replace contents under: `apps/runtime/src-tauri/builtin-skills/pptx/`
- Modify as needed: vendored `SKILL.md` frontmatter / references for WorkClaw compatibility

**Step 1: Copy the vendored skill assets**

Replace the current minimal Office builtin directories with the corresponding MiniMax directories while preserving the WorkClaw builtin directory names.

**Step 2: Adjust compatibility details**

Patch only the minimum needed for WorkClaw compatibility, such as path references, invoke metadata, or environment guidance.

**Step 3: Verify the assets exist**

Run targeted directory checks to confirm `SKILL.md`, scripts, assets, templates, and references are present in the vendored builtin directories.

### Task 4: Add targeted coverage for builtin Office assets

**Files:**
- Modify: `packages/runtime-skill-core/tests/builtin_skills.rs`
- Modify: `packages/builtin-skill-checks/tests/builtin_skill_assets.rs`
- Add/modify: runtime tests that assert builtin Office directories project with scripts/assets

**Step 1: Add regression tests**

Assert that builtin Office skills:
- still register under the old builtin ids
- still parse valid metadata from `SKILL.md`
- expose non-trivial file trees
- project scripts/assets into runtime workspaces

**Step 2: Run tests**

Run:
- `cargo test -p runtime-skill-core builtin_skills -- --nocapture`
- `cargo test -p builtin-skill-checks builtin_skill_assets -- --nocapture`
- `cargo test -p runtime workspace_skill_projection -- --nocapture`

Expected: PASS

### Task 5: Run final verification and capture known runtime gaps

**Files:**
- No code changes required unless fixes are needed

**Step 1: Run targeted verification**

Run:
- `cargo test -p runtime-skill-core builtin_skills -- --nocapture`
- `cargo test -p builtin-skill-checks builtin_skill_assets -- --nocapture`
- `cargo test -p runtime workspace_skill_projection -- --nocapture`

**Step 2: Record remaining dependency gaps**

Document any environment constraints discovered during verification, especially `.NET SDK`, Python packages, Node packages, and Playwright requirements for the new builtin Office skills.

**Step 3: Commit**

```bash
git add packages/runtime-skill-core apps/runtime/src-tauri apps/runtime/src-tauri/builtin-skills docs/plans/2026-03-28-builtin-office-skill-runtime-upgrade.md
git commit -m "feat: upgrade builtin office skills to full runtime assets"
```
