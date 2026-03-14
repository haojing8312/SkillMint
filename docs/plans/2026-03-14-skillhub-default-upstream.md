# SkillHub Default Upstream Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make SkillHub the default upstream for expert skill discovery and installation, while keeping ClawHub only as a fallback.

**Architecture:** Keep the current frontend command interface stable and switch the Tauri command implementations to SkillHub-first behavior. Use SkillHub catalog JSON for library/search data, SkillHub slug download for installation, and preserve existing ClawHub/GitHub logic as a fallback path.

**Tech Stack:** Rust + Tauri commands, React + TypeScript, Vitest, reqwest, serde_json, existing local import pipeline.

---

### Task 1: Add SkillHub catalog normalization in Tauri

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Test: `apps/runtime/src-tauri/src/commands/clawhub.rs`

**Step 1: Write the failing tests**

Add Rust unit tests for:

- parsing SkillHub catalog JSON into existing `ClawhubLibraryItem` / recommendation-compatible data
- preserving `slug`, `name`, `description`, `homepage -> source_url`, `downloads`, `stars`, `owner`

Include fixture-style JSON using:

```rust
let body = serde_json::json!({
    "skills": [
        {
            "slug": "self-improving-agent",
            "name": "self-improving-agent",
            "description": "Captures learnings",
            "description_zh": "记录学习",
            "version": "3.0.1",
            "homepage": "https://clawhub.ai/skills/self-improving-agent",
            "downloads": 206819,
            "stars": 1975,
            "owner": "pskoett",
            "tags": ["automation", "latest"]
        }
    ]
});
```

**Step 2: Run the failing tests**

Run:

```bash
cargo test skillhub --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- new tests fail because SkillHub normalization does not exist yet

**Step 3: Implement minimal normalization**

Add:

- SkillHub catalog URL constant
- a `SkillhubCatalogEntry` normalization helper or direct JSON parsing helper
- mapping into the existing frontend-facing structs

Implementation requirements:

- prefer `description_zh` when non-empty for Chinese-friendly summary fallback
- map `homepage` to `source_url`
- synthesize `github_url` only if later needed; do not require it

**Step 4: Run tests to verify pass**

Run:

```bash
cargo test skillhub --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- new SkillHub normalization tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/clawhub.rs
git commit -m "feat: add skillhub catalog normalization"
```

### Task 2: Switch library listing to SkillHub-first with fallback

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Test: `apps/runtime/src-tauri/src/commands/clawhub.rs`

**Step 1: Write the failing tests**

Add backend tests for:

- `list_clawhub_library_with_pool` using SkillHub data when available
- fallback to cached data or ClawHub when SkillHub fetch fails

**Step 2: Run the failing tests**

Run:

```bash
cargo test list_clawhub_library --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- tests fail because list path is still ClawHub-first

**Step 3: Implement minimal SkillHub-first library fetching**

Modify the library fetch flow to:

1. try SkillHub catalog
2. normalize into `ClawhubLibraryResponse`
3. cache the normalized JSON body
4. fallback to existing ClawHub fetch only on failure

Keep existing command names unchanged.

**Step 4: Run tests to verify pass**

Run:

```bash
cargo test list_clawhub_library --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- SkillHub-first tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/clawhub.rs
git commit -m "feat: make skill library skillhub-first"
```

### Task 3: Switch find-skills recommendation to local SkillHub search

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Test: `apps/runtime/src-tauri/src/commands/clawhub.rs`

**Step 1: Write the failing tests**

Add tests for:

- local SkillHub recommendation scoring across `name`, `slug`, `description`, `description_zh`, `tags`, `owner`
- result ordering by score, then stars
- fallback to ClawHub search only when SkillHub fetch fails

**Step 2: Run the failing tests**

Run:

```bash
cargo test recommend_clawhub_skills --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- tests fail because recommendations still come from ClawHub library query

**Step 3: Implement minimal local scoring**

Reuse the existing scoring framework where possible, but feed it SkillHub-normalized summaries.

Requirements:

- preserve output shape
- keep current reason string style
- do not introduce a new frontend API

**Step 4: Run tests to verify pass**

Run:

```bash
cargo test recommend_clawhub_skills --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- recommendation tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/clawhub.rs
git commit -m "feat: use skillhub for find-skills recommendations"
```

### Task 4: Make installation SkillHub-first by slug

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/clawhub.rs`
- Test: `apps/runtime/src-tauri/src/commands/clawhub.rs`

**Step 1: Write the failing tests**

Add tests for:

- slug-based SkillHub download URL generation
- successful zip download path through SkillHub endpoint
- fallback to existing ClawHub/GitHub logic when SkillHub download fails

**Step 2: Run the failing tests**

Run:

```bash
cargo test install_clawhub_skill --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- tests fail because install path does not yet try SkillHub first

**Step 3: Implement minimal SkillHub-first install**

Add:

- SkillHub download endpoint builder: `https://lightmake.site/api/v1/download?slug=<slug>`
- redirect-aware download helper
- install flow order:
  1. SkillHub by slug
  2. current resolved repo flow

Do not replace the extraction/import logic after bytes are downloaded.

**Step 4: Run tests to verify pass**

Run:

```bash
cargo test install_clawhub_skill --manifest-path apps/runtime/src-tauri/Cargo.toml
```

Expected:

- install tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/clawhub.rs
git commit -m "feat: make skill install skillhub-first"
```

### Task 5: Update frontend tests for SkillHub-first behavior

**Files:**
- Modify: `apps/runtime/src/components/experts/__tests__/SkillLibraryView.translation.test.tsx`
- Modify: `apps/runtime/src/components/experts/__tests__/FindSkillsView.translation.test.tsx`
- Modify: `apps/runtime/src/components/InstallDialog.tsx`
- Create or Modify: `apps/runtime/src/components/__tests__/InstallDialog.*.test.tsx`

**Step 1: Write the failing tests**

Add/adjust frontend tests to assert:

- library items can be rendered from SkillHub-backed response fields
- recommendations work with SkillHub-backed responses
- install dialog passes slug correctly even when only SkillHub-derived source metadata exists

**Step 2: Run the failing tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/experts/__tests__/SkillLibraryView.translation.test.tsx src/components/experts/__tests__/FindSkillsView.translation.test.tsx
```

Expected:

- tests fail only where assumptions still reflect ClawHub-first behavior

**Step 3: Write minimal implementation updates**

Only adjust frontend where needed to preserve compatibility with SkillHub-derived payloads. Avoid renaming visible UI text in this phase.

**Step 4: Run tests to verify pass**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/experts/__tests__/SkillLibraryView.translation.test.tsx src/components/experts/__tests__/FindSkillsView.translation.test.tsx
```

Expected:

- targeted frontend tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src/components/experts/__tests__/SkillLibraryView.translation.test.tsx apps/runtime/src/components/experts/__tests__/FindSkillsView.translation.test.tsx apps/runtime/src/components/InstallDialog.tsx
git commit -m "test: cover skillhub-first expert skill flows"
```

### Task 6: Verify end-to-end behavior and document residual gaps

**Files:**
- Modify: `docs/plans/2026-03-14-skillhub-default-upstream-design.md`
- Optional Modify: `docs/user-manual/05-experts-and-skills.md`

**Step 1: Run backend verification**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml skillhub
```

Expected:

- SkillHub-related backend tests pass

**Step 2: Run frontend verification**

Run:

```bash
pnpm --dir apps/runtime exec tsc --noEmit
pnpm --dir apps/runtime exec vitest run src/components/experts/__tests__/SkillLibraryView.translation.test.tsx src/components/experts/__tests__/FindSkillsView.translation.test.tsx
```

Expected:

- TypeScript passes or only reports pre-existing unrelated failures
- targeted frontend tests pass

**Step 3: Manual verification**

Verify in app:

- `专家技能 -> 技能库` shows SkillHub-backed data
- `找技能` returns SkillHub-backed recommendations
- install succeeds for a SkillHub-known slug such as `self-improving-agent`

**Step 4: Update docs**

Record any practical limitation found during verification:

- fallback behavior used
- fields missing from SkillHub catalog
- whether UI copy still references ClawHub

**Step 5: Commit**

```bash
git add docs/plans/2026-03-14-skillhub-default-upstream-design.md docs/user-manual/05-experts-and-skills.md
git commit -m "docs: document skillhub-first expert skill behavior"
```
