# Model Default Switch On Create Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make newly created model connections automatically become the default model, while keeping edit operations non-disruptive and preserving a single default model at all times.

**Architecture:** Keep model persistence and default switching as separate responsibilities. The frontend saves model configs as before, but on create it immediately invokes a new backend `set_default_model` command. The backend owns default-model exclusivity and deletion fallback so every caller sees a consistent single-default state.

**Tech Stack:** React 18, TypeScript, Tauri 2, Rust, Vitest, sqlx

---

### Task 1: Add failing tests for create-time default switching in settings

**Files:**
- Modify: `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Write the failing test**

- Add a test that renders two existing models where the first one is default.
- Simulate creating a new model from the settings form.
- Assert that the frontend invokes `save_model_config` first and then `set_default_model` with the saved id.
- Assert that the model list refreshes and the new model is shown as default.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- The test fails because `set_default_model` does not exist in the frontend flow and the save command does not return an id.

**Step 3: Write minimal implementation**

- Update the test doubles so `save_model_config` can return a model id.
- Adjust `SettingsView` create flow to call `set_default_model` only when `editingModelId` is empty.

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- The create-flow test passes and proves that new models trigger default switching.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx apps/runtime/src/components/SettingsView.tsx
git commit -m "test: cover create-time model default switching"
```

### Task 2: Add failing backend tests for unique default-model switching

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_models.rs`
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`

**Step 1: Write the failing tests**

- Add a Rust test for `set_default_model` that seeds two non-search model configs, marks the first as default, runs the command for the second, and asserts the default flips exclusively.
- Add a test proving search configs keep their own default flag untouched.

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_models
```

Expected:
- Tests fail because `set_default_model` is not implemented.

**Step 3: Write minimal implementation**

- Add the `set_default_model(model_id)` Tauri command.
- Clear defaults only for model configs where `api_format NOT LIKE 'search_%'`.
- Set the requested model as the single default.
- Register the command in the Tauri command list if needed.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_models
```

Expected:
- Backend tests pass and prove there is only one default non-search model.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/models.rs apps/runtime/src-tauri/tests/test_models.rs
git commit -m "feat: add explicit default model switching"
```

### Task 3: Return saved model ids from the backend save command

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`
- Modify: `apps/runtime/src/App.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`

**Step 1: Write the failing tests**

- Update the quick-setup and settings mocks to expect `save_model_config` to return a string id instead of `void`.
- Add assertions that callers accept the returned id without breaking existing create and edit flows.

**Step 2: Run tests to verify failure**

Run:

```bash
pnpm --filter runtime test -- App.model-setup-hint.test.tsx SettingsView.model-providers.test.tsx
```

Expected:
- Tests fail because the command signature and mock shapes no longer match.

**Step 3: Write minimal implementation**

- Change Rust `save_model_config` to return `Result<String, String>`.
- Return the existing id on update and the generated id on create.
- Update all frontend invocations and tests to consume the returned string.

**Step 4: Run tests to verify they pass**

Run:

```bash
pnpm --filter runtime test -- App.model-setup-hint.test.tsx SettingsView.model-providers.test.tsx
```

Expected:
- Frontend callers keep working with the new return type.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/models.rs apps/runtime/src/App.tsx apps/runtime/src/components/SettingsView.tsx apps/runtime/src/__tests__/App.model-setup-hint.test.tsx apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx
git commit -m "refactor: return saved model ids from model config saves"
```

### Task 4: Add manual default switching to the model list

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`
- Modify: `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`

**Step 1: Write the failing tests**

- Add a test that a non-default model row shows `设为默认`.
- Click the button and assert `set_default_model` is invoked with the correct id.
- Assert the list reloads and the default badge moves.

**Step 2: Run tests to verify failure**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- Tests fail because the model list currently lacks the button and handler.

**Step 3: Write minimal implementation**

- Add a `handleSetDefaultModel` function in `SettingsView`.
- Render the `设为默认` action for non-default models.
- Reuse the existing reload/error-handling pattern from search defaults.

**Step 4: Run tests to verify they pass**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- Model list manual default switching passes.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx
git commit -m "feat: add manual default action for models"
```

### Task 5: Preserve default state on edit and cover regression cases

**Files:**
- Modify: `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Write the failing tests**

- Add a test that editing the current default model keeps it default after save.
- Add a test that editing a non-default model does not change the current default model.

**Step 2: Run tests to verify failure**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- One or both tests fail if the create-time default-switch logic leaks into edit mode.

**Step 3: Write minimal implementation**

- Ensure the save handler gates auto-default behavior strictly on `editingModelId === null`.
- Keep the existing row state when editing.

**Step 4: Run tests to verify they pass**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- Edit flows preserve the previous default state.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx
git commit -m "test: preserve default model on edit"
```

### Task 6: Add deletion fallback so the app never loses a default model

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/models.rs`
- Modify: `apps/runtime/src-tauri/tests/test_models.rs`
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: Write the failing tests**

- Add a backend test that deletes the current default model while at least one other non-search model remains.
- Assert one remaining model is automatically promoted to default.
- Add a frontend smoke assertion if needed that reloading after deletion still shows one default badge.

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_models
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- Tests fail because deletion currently removes the row without promoting a replacement.

**Step 3: Write minimal implementation**

- Update `delete_model_config` to detect whether the deleted config was the default non-search model.
- If so, select one remaining non-search model and mark it default in the same logical flow.
- Keep search configs excluded from this fallback behavior.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_models
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- Deleting the default model leaves exactly one remaining default model when any models remain.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/models.rs apps/runtime/src-tauri/tests/test_models.rs apps/runtime/src/components/SettingsView.tsx apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx
git commit -m "feat: restore a default model after deletion"
```

### Task 7: Run focused verification and update docs if needed

**Files:**
- Modify: `docs/user-manual/06-settings.md`

**Step 1: Run the focused test suite**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx App.model-setup-hint.test.tsx
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_models
```

Expected:
- All targeted frontend and backend tests pass.

**Step 2: Update user-facing settings copy**

- Add one short note to the settings manual that newly added model connections automatically become the default model, and users can manually switch defaults later.

**Step 3: Run the affected test suite again if the docs change touched snapshots or copy-sensitive tests**

Run:

```bash
pnpm --filter runtime test -- SettingsView.model-providers.test.tsx
```

Expected:
- No regressions.

**Step 4: Commit**

```bash
git add docs/user-manual/06-settings.md
git commit -m "docs: document automatic default switching for new models"
```
