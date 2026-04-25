# Media Claim-Check Attachments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Store oversized inbound image payloads as safe `media://inbound/<id>` references while preserving native vision model behavior.

**Architecture:** Add a small Tauri-side media store under `runtime_root/cache/chat-media/inbound`, offload large image parts during async chat attachment normalization, and hydrate `mediaRef` image parts when constructing model messages. Keep old inline `data` image parts compatible for existing sessions and render new `mediaRef` image history as a non-broken attachment card.

**Tech Stack:** Rust/Tauri, `serde_json`, `base64`, `sha2` or timestamp/id generation from Rust std-compatible crates already present, Vitest/React Testing Library.

---

## File Map

- Create `apps/runtime/src-tauri/src/commands/chat_media_store.rs`: media ref parsing, safe ID generation, write/read/delete helpers, and tests.
- Modify `apps/runtime/src-tauri/src/commands/mod.rs`: expose the new module.
- Modify `apps/runtime/src-tauri/src/runtime_paths.rs`: add or derive the chat media cache path.
- Modify `apps/runtime/src-tauri/src/commands/chat_attachments.rs`: async image offload during normalization; retain sync path for tests/legacy helpers.
- Modify `apps/runtime/src-tauri/src/commands/chat.rs`: pass runtime paths into async normalization from `send_message`.
- Modify `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`: add async current-turn/history message builders that hydrate `mediaRef` image parts.
- Modify `apps/runtime/src-tauri/src/agent/runtime/kernel/turn_preparation.rs`: call async transcript builders during turn preparation.
- Modify `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`: integration coverage for offload and legacy inline compatibility.
- Modify `apps/runtime/src/components/ChatView.tsx`: render persisted `mediaRef` image parts as attachment cards, not broken `<img>`.
- Modify `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`: UI coverage for `mediaRef` history rendering.

---

### Task 1: Media Store Module

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/chat_media_store.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat_media_store.rs`

- [x] **Step 1: Write failing media-store tests**

Add tests that create a temp runtime root, save PNG bytes, read them through `media://inbound/<id>`, and reject unsafe refs like `media://inbound/../evil.png`, `media://inbound/a/b.png`, and `media://other/id.png`.

Run: `cargo test --quiet commands::chat_media_store --lib`

Expected before implementation: compile failure or missing module/function errors.

- [x] **Step 2: Implement minimal media store**

Implement:

```rust
pub const CHAT_MEDIA_REF_PREFIX: &str = "media://inbound/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedChatMedia {
    pub media_ref: String,
    pub id: String,
    pub path: PathBuf,
    pub size_bytes: usize,
    pub mime_type: String,
}

pub fn chat_media_root(runtime_paths: &RuntimePaths) -> PathBuf;
pub fn save_inbound_media(runtime_paths: &RuntimePaths, bytes: &[u8], mime_type: &str, original_name: &str) -> Result<SavedChatMedia, String>;
pub fn resolve_inbound_media_ref(runtime_paths: &RuntimePaths, media_ref: &str) -> Result<PathBuf, String>;
pub fn read_inbound_media_ref(runtime_paths: &RuntimePaths, media_ref: &str, max_bytes: usize) -> Result<Vec<u8>, String>;
pub fn delete_inbound_media_ref(runtime_paths: &RuntimePaths, media_ref: &str) -> Result<(), String>;
```

Use `runtime_paths.cache_dir.join("chat-media").join("inbound")`, `std::fs`, generated IDs based on time plus a counter/hash-safe suffix, and `symlink_metadata` to reject symlinks.

- [ ] **Step 3: Verify media-store tests pass**

Run: `cargo test --quiet commands::chat_media_store --lib`

Expected: media-store tests pass.

Status note: `cargo test --quiet commands::chat_media_store --lib` currently exits before tests with Windows `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` in this worktree. The integration test target is usable, so media-ref safety and hydration regressions are also covered through `test_chat_attachment_platform`.

- [ ] **Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_media_store.rs apps/runtime/src-tauri/src/commands/mod.rs
git commit -m "feat(runtime): add chat media claim-check store"
```

### Task 2: Attachment Offload

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_attachments.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Test: `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`

- [x] **Step 1: Write failing offload test**

Add an async test that builds a 3 MB PNG data URL, calls a new runtime-path-aware normalization helper, and asserts:

```rust
assert_eq!(parts[0]["type"].as_str(), Some("image"));
assert!(parts[0].get("data").is_none());
assert!(parts[0]["mediaRef"].as_str().unwrap().starts_with("media://inbound/"));
```

Also assert the media file exists under the temp runtime root.

Run: `cargo test --quiet --test test_chat_attachment_platform image_attachments_above_threshold_are_offloaded -- --exact`

Expected before implementation: missing helper or inline `data` still present.

- [x] **Step 2: Implement offload in async normalization**

Add `normalize_message_parts_with_pool_and_runtime_paths(parts, pool, runtime_paths)` and use it from `send_message`. Keep `normalize_message_parts_with_pool(parts, pool)` as a compatibility wrapper that does not offload unless paths are available.

Offload only `kind == "image"` when decoded bytes exceed `2 * 1024 * 1024`. Return:

```json
{
  "type": "image",
  "name": "...",
  "mimeType": "...",
  "size": 3145728,
  "mediaRef": "media://inbound/<id>"
}
```

Small images must keep the existing inline `data` field.

- [x] **Step 3: Add failure cleanup coverage**

Add a test with first image offloaded and second malformed image that fails validation/normalization. Assert the first saved media file is deleted.

Run: `cargo test --quiet --test test_chat_attachment_platform malformed_later_attachment_cleans_up_offloaded_media -- --exact`

Expected after implementation: pass.

- [x] **Step 4: Verify attachment platform tests**

Run: `cargo test --quiet --test test_chat_attachment_platform`

Expected: all attachment platform tests pass.

- [ ] **Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat_attachments.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs
git commit -m "feat(runtime): offload oversized chat images"
```

### Task 3: Transcript Hydration

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/kernel/turn_preparation.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/kernel/turn_preparation.rs`

- [x] **Step 1: Write failing transcript hydration tests**

Add tests for OpenAI and Anthropic formats using a temp runtime root and a saved `mediaRef`. Assert OpenAI output includes `image_url.url` as a data URL and Anthropic output includes base64 `source.data`.

Run: `cargo test --quiet agent::runtime::transcript --lib`

Expected before implementation: no async media-ref builder exists or `mediaRef` images are skipped.

- [x] **Step 2: Implement async transcript builders**

Add:

```rust
pub(crate) async fn build_current_turn_message_with_runtime_paths(
    api_format: &str,
    parts: &[Value],
    runtime_paths: Option<&RuntimePaths>,
) -> Result<Option<Value>, String>;
```

Existing sync `build_current_turn_message` should call a shared helper for inline images only. The async helper should hydrate `mediaRef` with `read_inbound_media_ref` and convert it into the same provider-specific image block shape as inline `data`.

- [x] **Step 3: Wire turn preparation**

In `prepare_local_turn`, resolve runtime paths from `params.app`, then call the async transcript builder for current turn and history reconstruction where possible. Preserve existing sync reconstruction for old inline sessions if no runtime paths are available.

- [ ] **Step 4: Verify transcript and turn tests**

Run: `cargo test --quiet agent::runtime::transcript --lib`

Run: `cargo test --quiet agent::runtime::kernel::turn_preparation --lib`

Expected: targeted tests pass.

Status note: lib-target transcript tests are blocked by the same Windows `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` loader failure. OpenAI and Anthropic media hydration are verified through the runnable attachment platform integration tests.

- [ ] **Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/transcript.rs apps/runtime/src-tauri/src/agent/runtime/kernel/turn_preparation.rs
git commit -m "feat(runtime): hydrate media refs for vision turns"
```

### Task 4: Frontend History Rendering

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

- [x] **Step 1: Write failing UI test**

Add a persisted user message with an image part containing `mediaRef` and no `data`. Assert the UI shows the image filename and does not render an `<img src="">` for that part.

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

Expected before implementation: broken or unsupported rendering.

- [x] **Step 2: Implement mediaRef card rendering**

When rendering an attachment-platform image part, use `<img>` only if `sourcePayload` or `data` is present. If only `mediaRef` is present, render the existing generic attachment card metadata with a visible image label.

- [x] **Step 3: Verify UI test**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx
git commit -m "fix(runtime): render claim-checked image history"
```

### Task 5: Final Verification

**Files:**
- No new implementation files expected.

- [x] **Step 1: Format Rust**

Run: `rustfmt --edition 2024 apps/runtime/src-tauri/src/commands/chat_media_store.rs apps/runtime/src-tauri/src/commands/chat_attachments.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/runtime/transcript.rs apps/runtime/src-tauri/src/agent/runtime/kernel/turn_preparation.rs apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`

Expected: exit 0.

- [x] **Step 2: Run required WorkClaw verification**

Run: `cargo test --quiet --test test_chat_attachment_platform`

Run: `pnpm test:rust-fast`

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/ChatView.side-panel-redesign.test.tsx src/lib/__tests__/attachmentDrafts.test.ts src/lib/__tests__/attachmentPolicy.test.ts`

Run: `pnpm --dir apps/runtime build`

Run: `git diff --check`

Expected: all pass. Vite chunk-size warnings are acceptable if build exits 0.

Status note: `rustfmt`, `cargo test --quiet --test test_chat_attachment_platform`, `pnpm test:rust-fast`, targeted Vitest, `pnpm --dir apps/runtime build`, and `git diff --check` pass. The targeted `--lib` Rust test executables for media store/transcript/turn preparation still fail to start on Windows with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` before running assertions.

- [x] **Step 3: Final status**

Report changed surfaces, commands run, pass/fail results, and any unverified areas.
