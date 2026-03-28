# Unified Vendored Skill Source Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor WorkClaw so builtin skills become preinstalled vendored skills and runtime uses one unified execution path for directory-backed skills.

**Architecture:** Keep vendored third-party skill assets in the repository, sync them into a managed app-data vendor directory at bootstrap, and store them in `installed_skills` as unified directory-backed installs. Remove builtin-specific runtime loading branches so runtime operates on installed content type instead of distribution origin.

**Tech Stack:** Rust, Tauri, SQLite, runtime workspace projection, vendored skill assets, targeted migration and seed tests.

---

### Task 1: Define the unified installed-skill source model

**Files:**
- Modify: `apps/runtime/src-tauri/src/db/schema.rs`
- Modify: `apps/runtime/src-tauri/src/db/migrations.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Test: `apps/runtime/src-tauri/src/db.rs`

**Step 1: Write the failing migration tests**

Add regression tests that describe the desired persistence shape for vendored directory-backed skills and legacy builtin-row compatibility.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib sync_builtin_skills_upserts_manifest_and_source_type -- --nocapture`

Expected: FAIL or reveal that the current schema still encodes builtin as a special runtime type.

**Step 3: Implement the minimal schema evolution**

Introduce backward-compatible fields or interpretation helpers that separate distribution origin from runtime content type. Preserve legacy `source_type` reads during transition if needed.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib sync_builtin_skills_upserts_manifest_and_source_type -- --nocapture`

Expected: PASS

### Task 2: Introduce a vendored skill catalog and sync root

**Files:**
- Modify: `packages/runtime-skill-core/src/builtin_skills.rs`
- Modify: `packages/runtime-skill-core/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/db/seed.rs`
- Add if needed: `apps/runtime/src-tauri/src/commands/skills/vendor_skill_service.rs`
- Test: `packages/runtime-skill-core/tests/builtin_skills.rs`
- Test: `apps/runtime/src-tauri/src/db.rs`

**Step 1: Write the failing tests**

Add tests that assert vendored skills expose catalog metadata needed for sync, and that bootstrap can materialize a vendor-managed directory target deterministically.

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml builtin_skills -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib sync_builtin_skills_is_idempotent -- --nocapture`

Expected: FAIL because builtin metadata currently still assumes runtime embedding semantics.

**Step 3: Implement vendored catalog and sync helpers**

Add vendored-catalog metadata and bootstrap sync logic that copies vendored skills into an app-data vendor root without using builtin-only runtime projection.

**Step 4: Run tests to verify they pass**

Run the same commands and expect PASS.

### Task 3: Migrate legacy builtin rows into vendored directory-backed installs

**Files:**
- Modify: `apps/runtime/src-tauri/src/db/seed.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Test: `apps/runtime/src-tauri/src/db.rs`

**Step 1: Write the failing migration tests**

Add a regression test that seeds an old `source_type = 'builtin'` row, runs bootstrap sync, and expects the row to point at the managed vendor directory using the unified content model while preserving the same skill id.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib sync_builtin_skills_upserts_manifest_and_source_type -- --nocapture`

Expected: FAIL because builtin rows are still persisted as builtin-special records.

**Step 3: Implement the migration**

Rewrite bootstrap seeding so old builtin rows are upgraded in place to vendored directory-backed records. Keep ids stable and make the operation idempotent.

**Step 4: Run test to verify it passes**

Run the same command and expect PASS.

### Task 4: Unify runtime projection for directory-backed skills

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/runtime_inputs.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`

**Step 1: Write the failing runtime tests**

Add tests asserting vendored skills project through the same directory-backed path as local skills, and that builtin-specific runtime file-tree branches are no longer required for Office skill projection.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib resolve_workspace_skill_runtime_entry_for_builtin_docx_includes_vendored_assets -- --nocapture`

Expected: FAIL because runtime still treats builtin as a special content source.

**Step 3: Implement the minimal runtime unification**

Refactor runtime loading so directory-backed installed skills share one projection path regardless of origin. Keep encrypted pack handling separate for now.

**Step 4: Run test to verify it passes**

Run the same command and expect PASS.

### Task 5: Unify readiness and refresh behavior around installed content

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/skills/runtime_status_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills/local_skill_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`
- Test: `apps/runtime/src-tauri/src/commands/skills/runtime_status_service.rs`

**Step 1: Write the failing tests**

Add tests for vendored skills that verify readiness checks resolve their managed directory root without builtin-specific root inference, and that refresh/update entrypoints do not depend on builtin runtime handling.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib builtin_docx_reports_missing_dotnet_sdk_when_not_installed -- --nocapture`

Expected: FAIL or reveal builtin-specific readiness assumptions.

**Step 3: Implement the minimal service changes**

Make readiness and refresh logic operate from installed content metadata and directory paths, not builtin identity.

**Step 4: Run test to verify it passes**

Run the same command and expect PASS.

### Task 6: Trim runtime-skill-core back to bootstrap/catalog responsibilities

**Files:**
- Modify: `packages/runtime-skill-core/src/builtin_skills.rs`
- Modify: `packages/runtime-skill-core/src/lib.rs`
- Modify: `packages/builtin-skill-checks/tests/builtin_skill_assets.rs`

**Step 1: Write the failing tests**

Add assertions that `runtime-skill-core` still exposes the vendored catalog and templates needed for bootstrap while no longer being the primary runtime execution provider.

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --manifest-path packages/runtime-skill-core/Cargo.toml builtin_skills -- --nocapture`
- `cargo test --manifest-path packages/builtin-skill-checks/Cargo.toml -- --nocapture`

Expected: FAIL until responsibilities are clarified.

**Step 3: Implement the reduction**

Keep only the catalog and bootstrap-facing helpers that the app still needs. Remove or de-emphasize APIs that exist only because builtin was a runtime branch.

**Step 4: Run tests to verify they pass**

Run the same commands and expect PASS.

### Task 7: Final verification and rollout notes

**Files:**
- Modify if needed: `docs/plans/2026-03-28-unified-vendored-skill-source-design.md`
- No other code changes required unless fixes are needed

**Step 1: Run verification**

Run:
- `pnpm test:builtin-skills`
- `pnpm test:rust-fast`

**Step 2: Record known gaps**

Document any remaining unverified areas, especially full runtime crate tests blocked by existing Tauri build-script/resource-path issues, and any follow-up needed for encrypted-pack unification.

**Step 3: Commit**

```bash
git add docs/plans/2026-03-28-unified-vendored-skill-source-design.md docs/plans/2026-03-28-unified-vendored-skill-source.md packages/runtime-skill-core apps/runtime/src-tauri
git commit -m "refactor: unify builtin skills as vendored installs"
```
