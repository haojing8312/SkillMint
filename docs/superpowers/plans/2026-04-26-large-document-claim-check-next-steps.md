# Large Document Claim-Check Next Steps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current media claim-check branch into a safe, extensible attachment pipeline that handles large images without transcript bloat and large Markdown/text/PDF files without losing full-analysis capability.

**Architecture:** Finish and rebase the already-implemented image `mediaRef` claim-check path first. Then split document handling into two explicit modes: bounded model preview for normal chat turns, and full-document analysis through a tool/skill workflow that can read `mediaRef` content in chunks and aggregate results. Do not rely on hard-coded filename words as routing logic.

**Tech Stack:** Rust/Tauri attachment normalization, `chat_media_store`, runtime transcript builders, capability/skill routing, React attachment rendering, Rust integration tests, Vitest.

---

## Current State

- Branch: `feature/media-claim-check`
- Worktree: `.worktrees/media-claim-check`
- Original design: `docs/superpowers/specs/2026-04-24-media-claim-check-design.md`
- Original plan: `docs/superpowers/plans/2026-04-24-media-claim-check.md`
- Completed commit: `31771c8 feat(runtime): implement media claim-check attachments`
- Current uncommitted WIP:
  - `apps/runtime/src-tauri/src/commands/chat_attachments.rs`
  - `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`
  - `apps/runtime/src/components/ChatView.tsx`
  - `apps/runtime/src/types.ts`

The committed branch mostly completes the original large-image claim-check goal. The uncommitted WIP extends the same store to large text/PDF extracted text by saving full content and sending only a 16k preview. That WIP is useful, but it only prevents oversized prompts; it does not yet let the model analyze the full document.

## File Map

- Modify: `apps/runtime/src-tauri/src/commands/chat_media_store.rs`
  - Keep this as the storage boundary for inbound media/document blobs.
- Modify: `apps/runtime/src-tauri/src/commands/chat_attachments.rs`
  - Keep image offload.
  - Convert large text/PDF offload into an intentional document claim-check path.
- Modify: `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`
  - Keep image `mediaRef` hydration.
  - Keep document preview in normal chat turns.
- Modify or create: `apps/runtime/src-tauri/src/agent/tools/document_analyze.rs`
  - Provide a tool boundary for reading document `mediaRef` content in bounded chunks.
- Modify: `apps/runtime/src-tauri/src/agent/runtime/tool_catalog.rs`
  - Register the document analysis tool if the codebase uses this catalog for runtime tools.
- Modify capability/skill routing files under `apps/runtime/src-tauri/src/agent/runtime/skill_routing/` or current capability registry modules.
  - Route explicit full-document analysis intent to the document workflow, not to plain preview.
- Modify: `apps/runtime/src/components/ChatView.tsx`
  - Render saved/truncated document attachments clearly.
- Modify: `apps/runtime/src/types.ts`
  - Make `mediaRef` and `previewChars` explicit on document parts.
- Test: `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`
  - Cover image offload, document offload, preview-only model messages, and cleanup.
- Test: targeted runtime/tool tests for document chunk reads and invalid `mediaRef` rejection.
- Test: existing Vitest attachment rendering coverage.

---

### Task 1: Rebase And Preserve The Existing Image Claim-Check Work

**Files:**
- Review: all files changed by `31771c8`
- Modify only conflict files after rebasing onto `main`
- Test: existing tests listed in the original media claim-check plan

- [ ] **Step 1: Snapshot current WIP before rebase**

Run:

```bash
git -C .worktrees/media-claim-check status --short
git -C .worktrees/media-claim-check diff --stat
```

Expected: the four known WIP files are present. Do not discard them.

- [ ] **Step 2: Create a temporary WIP commit or stash**

Preferred local-only safety commit:

```bash
git -C .worktrees/media-claim-check add apps/runtime/src-tauri/src/commands/chat_attachments.rs apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs apps/runtime/src/components/ChatView.tsx apps/runtime/src/types.ts
git -C .worktrees/media-claim-check commit -m "wip(runtime): extend claim-check to large documents"
```

Expected: worktree becomes clean. This commit can be amended or split later.

- [ ] **Step 3: Rebase branch onto current `main`**

Run:

```bash
git -C .worktrees/media-claim-check fetch origin
git -C .worktrees/media-claim-check rebase main
```

Expected: conflicts are resolved by preserving current `main` capability/vision routing and reapplying only the claim-check behavior that still belongs.

- [ ] **Step 4: Verify image claim-check still works**

Run:

```bash
cargo test --quiet --test test_chat_attachment_platform image_attachments_above_threshold_are_offloaded -- --exact
cargo test --quiet --test test_chat_attachment_platform malformed_later_attachment_cleans_up_offloaded_media -- --exact
```

Expected: both tests pass.

### Task 2: Formalize Document Claim-Check As Preview Plus Saved Full Content

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_attachments.rs`
- Modify: `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/ChatView.tsx`

- [ ] **Step 1: Keep the current failing/coverage tests for large text**

The current WIP already contains useful tests:

```rust
fn large_text_documents_are_offloaded_with_bounded_preview()
fn large_text_document_model_message_uses_preview_only()
```

Keep both tests. Add a PDF extracted-text sibling if PDF support remains in scope:

```rust
#[test]
fn large_pdf_extracted_text_is_offloaded_with_bounded_preview() {
    // Build a ResolvedAttachment path through the public normalization helper
    // with extracted_text longer than 16_000 chars.
    // Assert:
    // - type == "pdf_file"
    // - extractedText length <= 16_000
    // - mediaRef starts with "media://inbound/"
    // - previewChars == 16_000
    // - saved media file contains the full extracted text
}
```

- [ ] **Step 2: Make the document preview constant explicit**

Keep the constant in `chat_attachments.rs`:

```rust
const TEXT_ATTACHMENT_MODEL_PREVIEW_CHARS: usize = 16_000;
```

Use it only for model-bound preview text. Do not confuse it with extraction limits or UI preview limits.

- [ ] **Step 3: Preserve normal-chat behavior**

For `file_text` and `pdf_file`, normal transcript construction should continue to include only:

```json
{
  "text": "<bounded preview>",
  "truncated": true,
  "mediaRef": "media://inbound/<id>",
  "previewChars": 16000
}
```

Expected model behavior: normal chat sees a useful preview and an explicit truncation note, but not the full document.

- [ ] **Step 4: Verify document preview tests**

Run:

```bash
cargo test --quiet --test test_chat_attachment_platform large_text_documents_are_offloaded_with_bounded_preview -- --exact
cargo test --quiet --test test_chat_attachment_platform large_text_document_model_message_uses_preview_only -- --exact
```

Expected: both tests pass.

### Task 3: Add A Full-Document Analysis Tool Boundary

**Files:**
- Create or modify: `apps/runtime/src-tauri/src/agent/tools/document_analyze.rs`
- Modify: `apps/runtime/src-tauri/src/agent/tools/mod.rs`
- Modify: current tool registration file, likely `apps/runtime/src-tauri/src/agent/runtime/tool_catalog.rs` or `apps/runtime/src-tauri/src/agent/runtime/tool_setup.rs`
- Test: targeted Rust tests near the new tool or an existing runtime tool test module

- [ ] **Step 1: Define the tool contract**

Tool input:

```json
{
  "mediaRef": "media://inbound/<id>",
  "mimeType": "text/markdown",
  "analysisGoal": "summarize | extract_structure | answer_question",
  "question": "optional user question",
  "chunkChars": 12000
}
```

Tool output:

```json
{
  "status": "ok",
  "mediaRef": "media://inbound/<id>",
  "chunkCount": 7,
  "analysis": "...",
  "truncated": false
}
```

- [ ] **Step 2: Write invalid-ref tests first**

Test refs:

```text
media://inbound/../evil.md
media://other/id
media://inbound/a/b.md
```

Expected: each fails before any file read.

- [ ] **Step 3: Implement bounded chunk reads**

Read the saved full text from `chat_media_store`, split by character boundaries, and cap each chunk to `chunkChars`. For the first version, keep this as a local deterministic helper that returns chunks; model summarization can be layered through existing agent turns or skill execution.

- [ ] **Step 4: Verify tool tests**

Run:

```bash
cargo test --quiet document_analyze
```

Expected: invalid refs fail, valid refs chunk deterministically, and large documents do not load into one outbound model message.

### Task 4: Route Explicit Full-Document Intent To The Document Workflow

**Files:**
- Modify: capability registry / routing files introduced on current `main`
- Modify: skill-routing adjudicator or tool-selection prompt files as appropriate
- Test: real-agent or targeted route eval scenario

- [ ] **Step 1: Add an intent-level capability, not hard-coded filename words**

Capability concept:

```text
large_document_analysis
```

Positive signals:

```text
analyze the whole attached document
summarize this markdown
read the entire file
extract all sections
answer based on the full document
```

Negative signals:

```text
just upload
save this file
what files are attached
briefly inspect the preview
```

- [ ] **Step 2: Add an eval scenario**

Create a scenario with a large Markdown attachment and a prompt like:

```text
请完整分析这个 Markdown 文件，整理主要章节、关键问题和后续行动建议
```

Expected route:

```text
selected capability/tool includes large_document_analysis or document_analyze
normal chat preview alone is not sufficient
```

- [ ] **Step 3: Verify routing**

Run the smallest relevant eval command for the new scenario. Use local config only; do not commit real provider secrets.

### Task 5: Final Verification And Integration Decision

**Files:**
- No new files expected unless tests reveal missing docs.

- [ ] **Step 1: Run formatting**

Run:

```bash
rustfmt --edition 2024 apps/runtime/src-tauri/src/commands/chat_media_store.rs apps/runtime/src-tauri/src/commands/chat_attachments.rs apps/runtime/src-tauri/src/agent/tools/document_analyze.rs apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs
```

Expected: exit 0.

- [ ] **Step 2: Run targeted verification**

Run:

```bash
cargo test --quiet --test test_chat_attachment_platform
pnpm --dir apps/runtime test -- src/components/__tests__/ChatView.side-panel-redesign.test.tsx src/lib/__tests__/attachmentDrafts.test.ts src/lib/__tests__/attachmentPolicy.test.ts
git diff --check
```

Expected: all pass.

- [ ] **Step 3: Run WorkClaw broader verification**

Run:

```bash
pnpm test:rust-fast
pnpm --dir apps/runtime build
```

Expected: both pass. Vite chunk warnings are acceptable only if build exits 0.

- [ ] **Step 4: Split commits before merge**

Preferred commit shape:

```bash
git commit -m "feat(runtime): claim-check oversized image attachments"
git commit -m "feat(runtime): claim-check large document previews"
git commit -m "feat(runtime): add large document analysis tool"
git commit -m "test(evals): cover large document analysis routing"
```

Expected: each commit is independently reviewable.

## Recommended Execution Order

1. Finish/rebase image claim-check first. This is the original branch goal and should stay small.
2. Keep large document preview offload as a separate commit. It prevents request bloat but should not be sold as full analysis.
3. Add full-document analysis through a tool/skill workflow. This is the actual fix for large Markdown analysis quality.
4. Only after these are verified, decide whether to merge the branch into `main`.

## Completion Criteria

- Large images no longer persist inline base64 in new `contentParts`.
- Existing inline image sessions still work.
- Large text/Markdown/PDF attachments store full content as `mediaRef` and send bounded preview in normal chat.
- Explicit “analyze the whole document” intent routes to a workflow that reads full saved content in chunks.
- Invalid or unsafe `mediaRef` values are rejected.
- Tests and verification commands listed above pass.
