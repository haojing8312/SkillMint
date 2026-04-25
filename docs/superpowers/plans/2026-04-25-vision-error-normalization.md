# Vision Error Normalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make WorkClaw classify model context/media/token-budget failures into actionable user-facing errors while preventing old image payloads from repeatedly consuming model context.

**Architecture:** Extend the existing model error taxonomy in `apps/runtime/src-tauri/src/model_errors.rs` and the matching frontend display helpers instead of adding a new error UI. Reuse the existing attachment policy/validation and transcript reconstruction boundaries to validate image payloads and replace historical image blocks with lightweight placeholders.

**Tech Stack:** Rust/Tauri runtime, serde_json, existing WorkClaw attachment platform, React/TypeScript/Vitest frontend tests, existing `pnpm` verification commands.

---

## File Structure

- Modify `apps/runtime/src-tauri/src/model_errors.rs`: add `context_overflow`, `invalid_token_budget`, and `media_too_large` categories, user copy, and Rust unit tests.
- Modify `apps/runtime/src-tauri/src/commands/chat_policy.rs`: map new normalized model errors to route error keys.
- Modify `apps/runtime/src-tauri/src/agent/runtime/failover.rs`: map new normalized model errors to runtime failover keys without making them retryable.
- Modify `apps/runtime/src/types.ts`: extend frontend `ModelErrorKind`.
- Modify `apps/runtime/src/lib/model-error-display.ts`: add frontend copy and inference patterns.
- Modify `apps/runtime/src/lib/model-error-display.test.ts`: add frontend inference/display tests.
- Modify `apps/runtime/src-tauri/src/commands/chat_attachment_validation.rs`: strengthen image payload/MIME validation and add `media_too_large` wording.
- Modify `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`: add attachment validation regression tests.
- Modify `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`: preserve current-turn images, replace historical user images with placeholders.
- Add or extend unit tests in `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`.

No model settings vision test, technical details panel, doctor flow, or image re-reference UI is part of this plan.

---

### Task 1: Extend Backend Model Error Taxonomy

**Files:**
- Modify: `apps/runtime/src-tauri/src/model_errors.rs`

- [ ] **Step 1: Write failing Rust tests for new classifications**

Add these tests inside the existing `#[cfg(test)] mod tests` block in `apps/runtime/src-tauri/src/model_errors.rs`:

```rust
#[test]
fn normalize_model_error_detects_context_overflow_from_prompt_too_long() {
    let result = normalize_model_error("prompt is too long: 277403 tokens > 200000 maximum");
    assert_eq!(result.kind, ModelErrorKind::ContextOverflow);
}

#[test]
fn normalize_model_error_detects_context_overflow_from_max_tokens_context_limit() {
    let result = normalize_model_error(
        "input length and `max_tokens` exceed context limit: 188059 + 20000 > 200000",
    );
    assert_eq!(result.kind, ModelErrorKind::ContextOverflow);
}

#[test]
fn normalize_model_error_detects_invalid_token_budget_without_claiming_context() {
    let result = normalize_model_error("max_tokens must be at least 1, got -1024");
    assert_eq!(result.kind, ModelErrorKind::InvalidTokenBudget);
}

#[test]
fn normalize_model_error_detects_media_size_errors() {
    let result = normalize_model_error("image exceeds 5 MB maximum: 5316852 bytes > 5242880 bytes");
    assert_eq!(result.kind, ModelErrorKind::MediaTooLarge);
}

#[test]
fn normalize_model_error_keeps_tpm_413_as_rate_limit_not_context_overflow() {
    let result = normalize_model_error("413 tokens per minute limit exceeded");
    assert_eq!(result.kind, ModelErrorKind::RateLimit);
}

#[test]
fn model_error_copy_includes_invalid_token_budget_action() {
    assert_eq!(model_error_title(ModelErrorKind::InvalidTokenBudget), "模型输出空间不足");
    assert_eq!(
        model_error_message(ModelErrorKind::InvalidTokenBudget),
        "模型请求没有剩余空间生成回复。请减少当前会话上下文、压缩图片，或使用更大上下文的模型后重试。"
    );
}
```

- [ ] **Step 2: Run the focused Rust test and verify it fails**

Run:

```powershell
pnpm test:rust-fast -- model_errors
```

Expected: the new tests fail because `ModelErrorKind::ContextOverflow`, `InvalidTokenBudget`, and `MediaTooLarge` do not exist.

- [ ] **Step 3: Add the new enum variants and copy**

Update `ModelErrorKind` in `apps/runtime/src-tauri/src/model_errors.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelErrorKind {
    Billing,
    Auth,
    RateLimit,
    Timeout,
    Network,
    ContextOverflow,
    InvalidTokenBudget,
    MediaTooLarge,
    Unknown,
}
```

Update `model_error_title`:

```rust
pub(crate) fn model_error_title(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Billing => "模型余额不足",
        ModelErrorKind::Auth => "鉴权失败",
        ModelErrorKind::RateLimit => "请求过于频繁",
        ModelErrorKind::Timeout => "请求超时",
        ModelErrorKind::Network => "网络连接失败",
        ModelErrorKind::ContextOverflow => "上下文过长",
        ModelErrorKind::InvalidTokenBudget => "模型输出空间不足",
        ModelErrorKind::MediaTooLarge => "附件或图片过大",
        ModelErrorKind::Unknown => "连接失败",
    }
}
```

Update `model_error_message`:

```rust
pub(crate) fn model_error_message(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Billing => {
            "当前模型平台返回余额或额度不足，请到对应服务商控制台充值或检查套餐额度。"
        }
        ModelErrorKind::Auth => "请检查 API Key、组织权限或接口访问范围是否正确。",
        ModelErrorKind::RateLimit => "模型平台当前触发限流，请稍后重试或降低并发频率。",
        ModelErrorKind::Timeout => "模型平台响应超时，请稍后重试，或检查网络和所选模型是否可用。",
        ModelErrorKind::Network => "无法连接到模型接口，请检查 Base URL、网络环境或代理配置。",
        ModelErrorKind::ContextOverflow => {
            "当前会话内容超过了模型可处理的上下文。请减少历史内容、开启新会话，或使用更大上下文的模型。"
        }
        ModelErrorKind::InvalidTokenBudget => {
            "模型请求没有剩余空间生成回复。请减少当前会话上下文、压缩图片，或使用更大上下文的模型后重试。"
        }
        ModelErrorKind::MediaTooLarge => {
            "上传的图片或附件超过了当前模型请求限制。请压缩图片、减少附件数量，或移除不必要的附件后重试。"
        }
        ModelErrorKind::Unknown => "模型平台返回了未识别错误，可查看详细信息进一步排查。",
    }
}
```

- [ ] **Step 4: Add helper predicates and update classification order**

Add these helper functions below `normalized_error_search_text`:

```rust
fn has_tpm_rate_limit_hint(lower: &str) -> bool {
    lower.contains("tokens per minute") || lower.contains(" tpm") || lower.contains("tpm ")
}

fn is_context_overflow_error(lower: &str) -> bool {
    if has_tpm_rate_limit_hint(lower) {
        return false;
    }
    lower.contains("prompt is too long")
        || lower.contains("prompt too long")
        || lower.contains("context length exceeded")
        || lower.contains("maximum context length")
        || lower.contains("context window exceeded")
        || lower.contains("context_window_exceeded")
        || lower.contains("model_context_window_exceeded")
        || lower.contains("exceeds model context window")
        || lower.contains("model token limit")
        || lower.contains("exceed context limit")
        || lower.contains("exceeds the model's maximum context")
        || (lower.contains("input length") && lower.contains("exceed") && lower.contains("context"))
        || (lower.contains("max_tokens") && lower.contains("exceed") && lower.contains("context"))
        || lower.contains("上下文过长")
        || lower.contains("上下文超出")
        || lower.contains("上下文长度超")
        || lower.contains("超出最大上下文")
        || lower.contains("请压缩上下文")
}

fn is_invalid_token_budget_error(lower: &str) -> bool {
    lower.contains("max_tokens must be at least 1")
        || (lower.contains("max_tokens") && lower.contains("got -"))
}

fn is_media_too_large_error(lower: &str) -> bool {
    lower.contains("image exceeds")
        || lower.contains("image dimensions exceed")
        || lower.contains("media too large")
        || lower.contains("payload too large")
        || lower.contains("request too large")
        || lower.contains("request size exceeds")
        || lower.contains("request exceeds the maximum size")
        || lower.contains("附件或图片过大")
        || lower.contains("图片附件总大小")
}
```

In `normalize_model_error`, classify new categories after billing/auth/rate-limit and before timeout/network:

```rust
    } else if is_context_overflow_error(&lower) {
        ModelErrorKind::ContextOverflow
    } else if is_invalid_token_budget_error(&lower) {
        ModelErrorKind::InvalidTokenBudget
    } else if is_media_too_large_error(&lower) {
        ModelErrorKind::MediaTooLarge
```

Keep rate-limit checks before context overflow so `413 tokens per minute limit exceeded` remains `RateLimit`.

- [ ] **Step 5: Run focused test and commit**

Run:

```powershell
pnpm test:rust-fast -- model_errors
```

Expected: the `model_errors` tests pass.

Commit:

```powershell
git add apps/runtime/src-tauri/src/model_errors.rs
git commit -m "feat: classify context and media model errors"
```

---

### Task 2: Propagate New Error Kinds Through Runtime Route Failures

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_policy.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/failover.rs`

- [ ] **Step 1: Write failing route/failover tests**

In `apps/runtime/src-tauri/src/commands/chat_policy.rs`, add tests to the existing test module:

```rust
#[test]
fn classify_model_route_error_uses_invalid_token_budget_key() {
    let kind = classify_model_route_error("max_tokens must be at least 1, got -1024");
    assert_eq!(model_route_error_kind_key(kind), "invalid_token_budget");
}

#[test]
fn classify_model_route_error_uses_context_overflow_key() {
    let kind = classify_model_route_error("prompt is too long: 277403 tokens > 200000 maximum");
    assert_eq!(model_route_error_kind_key(kind), "context_overflow");
}

#[test]
fn classify_model_route_error_uses_media_too_large_key() {
    let kind = classify_model_route_error("image exceeds 5 MB maximum");
    assert_eq!(model_route_error_kind_key(kind), "media_too_large");
}
```

In `apps/runtime/src-tauri/src/agent/runtime/failover.rs`, add tests to the existing test module:

```rust
#[test]
fn runtime_failover_error_kind_maps_invalid_token_budget() {
    let kind = runtime_failover_error_kind_from_error_text("max_tokens must be at least 1, got -1024");
    assert_eq!(runtime_failover_error_kind_key(kind), "invalid_token_budget");
}

#[test]
fn runtime_failover_error_kind_maps_context_overflow() {
    let kind = runtime_failover_error_kind_from_error_text("context window exceeded");
    assert_eq!(runtime_failover_error_kind_key(kind), "context_overflow");
}

#[test]
fn runtime_failover_error_kind_maps_media_too_large() {
    let kind = runtime_failover_error_kind_from_error_text("payload too large");
    assert_eq!(runtime_failover_error_kind_key(kind), "media_too_large");
}
```

- [ ] **Step 2: Run focused Rust tests and verify they fail**

Run:

```powershell
pnpm test:rust-fast -- chat_policy failover
```

Expected: tests fail because route/failover enums do not expose the new keys.

- [ ] **Step 3: Extend `ModelRouteErrorKind`**

In `apps/runtime/src-tauri/src/commands/chat_policy.rs`, add variants:

```rust
pub(crate) enum ModelRouteErrorKind {
    Billing,
    Auth,
    RateLimit,
    Timeout,
    Network,
    ContextOverflow,
    InvalidTokenBudget,
    MediaTooLarge,
    PolicyBlocked,
    MaxTurns,
    LoopDetected,
    NoProgress,
    Unknown,
}
```

Update the `normalize_model_error(error_message).kind` match:

```rust
        crate::model_errors::ModelErrorKind::ContextOverflow => ModelRouteErrorKind::ContextOverflow,
        crate::model_errors::ModelErrorKind::InvalidTokenBudget => ModelRouteErrorKind::InvalidTokenBudget,
        crate::model_errors::ModelErrorKind::MediaTooLarge => ModelRouteErrorKind::MediaTooLarge,
```

Update `model_route_error_kind_key`:

```rust
        ModelRouteErrorKind::ContextOverflow => "context_overflow",
        ModelRouteErrorKind::InvalidTokenBudget => "invalid_token_budget",
        ModelRouteErrorKind::MediaTooLarge => "media_too_large",
```

Do not include these new variants in `should_retry_same_candidate`.

- [ ] **Step 4: Extend `RuntimeFailoverErrorKind`**

In `apps/runtime/src-tauri/src/agent/runtime/failover.rs`, add variants:

```rust
pub(crate) enum RuntimeFailoverErrorKind {
    Billing,
    Auth,
    RateLimit,
    Timeout,
    Network,
    ContextOverflow,
    InvalidTokenBudget,
    MediaTooLarge,
    DeferredTools,
    PolicyBlocked,
    MaxTurns,
    LoopDetected,
    NoProgress,
    Unknown,
}
```

Update `runtime_failover_kind_from_key`:

```rust
        "context_overflow" => RuntimeFailoverErrorKind::ContextOverflow,
        "invalid_token_budget" => RuntimeFailoverErrorKind::InvalidTokenBudget,
        "media_too_large" => RuntimeFailoverErrorKind::MediaTooLarge,
```

Update the `normalize_model_error(error_message).kind` match:

```rust
        crate::model_errors::ModelErrorKind::ContextOverflow => RuntimeFailoverErrorKind::ContextOverflow,
        crate::model_errors::ModelErrorKind::InvalidTokenBudget => RuntimeFailoverErrorKind::InvalidTokenBudget,
        crate::model_errors::ModelErrorKind::MediaTooLarge => RuntimeFailoverErrorKind::MediaTooLarge,
```

Update `runtime_failover_error_kind_key`:

```rust
        RuntimeFailoverErrorKind::ContextOverflow => "context_overflow",
        RuntimeFailoverErrorKind::InvalidTokenBudget => "invalid_token_budget",
        RuntimeFailoverErrorKind::MediaTooLarge => "media_too_large",
```

Do not include these variants in `runtime_should_retry_same_candidate`.

- [ ] **Step 5: Run focused tests and commit**

Run:

```powershell
pnpm test:rust-fast -- chat_policy failover
```

Expected: focused tests pass.

Commit:

```powershell
git add apps/runtime/src-tauri/src/commands/chat_policy.rs apps/runtime/src-tauri/src/agent/runtime/failover.rs
git commit -m "feat: propagate context media model error kinds"
```

---

### Task 3: Extend Frontend Error Display Copy

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/lib/model-error-display.ts`
- Modify: `apps/runtime/src/lib/model-error-display.test.ts`

- [ ] **Step 1: Write failing frontend tests**

Append tests in `apps/runtime/src/lib/model-error-display.test.ts`:

```ts
test.each([
  ["context overflow", "prompt is too long: 277403 tokens > 200000 maximum", "context_overflow"],
  ["invalid token budget", "max_tokens must be at least 1, got -1024", "invalid_token_budget"],
  ["media too large", "image exceeds 5 MB maximum", "media_too_large"],
])("infers %s model errors from raw transport messages", (_label, raw, expected) => {
  expect(inferModelErrorKindFromMessage(raw)).toBe(expected);
});

test("maps invalid token budget to careful non-vision-specific copy", () => {
  expect(getModelErrorDisplay("invalid_token_budget")).toEqual(
    expect.objectContaining({
      kind: "invalid_token_budget",
      title: "模型输出空间不足",
      message: "模型请求没有剩余空间生成回复。请减少当前会话上下文、压缩图片，或使用更大上下文的模型后重试。",
    }),
  );
});
```

- [ ] **Step 2: Run the focused frontend test and verify it fails**

Run:

```powershell
pnpm --dir apps/runtime test -- src/lib/model-error-display.test.ts
```

Expected: tests fail because the frontend union and display table do not include the new categories.

- [ ] **Step 3: Extend frontend types**

Update `ModelErrorKind` in `apps/runtime/src/types.ts`:

```ts
export type ModelErrorKind =
  | "billing"
  | "auth"
  | "rate_limit"
  | "timeout"
  | "network"
  | "context_overflow"
  | "invalid_token_budget"
  | "media_too_large"
  | "unknown";
```

- [ ] **Step 4: Extend frontend copy and inference**

Add entries to `MODEL_ERROR_DISPLAY_COPY` in `apps/runtime/src/lib/model-error-display.ts`:

```ts
  context_overflow: {
    title: "上下文过长",
    message: "当前会话内容超过了模型可处理的上下文。请减少历史内容、开启新会话，或使用更大上下文的模型。",
  },
  invalid_token_budget: {
    title: "模型输出空间不足",
    message: "模型请求没有剩余空间生成回复。请减少当前会话上下文、压缩图片，或使用更大上下文的模型后重试。",
  },
  media_too_large: {
    title: "附件或图片过大",
    message: "上传的图片或附件超过了当前模型请求限制。请压缩图片、减少附件数量，或移除不必要的附件后重试。",
  },
```

Update `isModelErrorKind`:

```ts
    value === "context_overflow" ||
    value === "invalid_token_budget" ||
    value === "media_too_large" ||
```

Add helper predicates near `normalizeErrorSearchText`:

```ts
function hasTpmRateLimitHint(lower: string): boolean {
  return lower.includes("tokens per minute") || lower.includes(" tpm") || lower.includes("tpm ");
}

function isContextOverflowError(lower: string): boolean {
  if (hasTpmRateLimitHint(lower)) return false;
  return (
    lower.includes("prompt is too long") ||
    lower.includes("prompt too long") ||
    lower.includes("context length exceeded") ||
    lower.includes("maximum context length") ||
    lower.includes("context window exceeded") ||
    lower.includes("context_window_exceeded") ||
    lower.includes("model_context_window_exceeded") ||
    lower.includes("exceeds model context window") ||
    lower.includes("model token limit") ||
    lower.includes("exceed context limit") ||
    lower.includes("exceeds the model's maximum context") ||
    (lower.includes("input length") && lower.includes("exceed") && lower.includes("context")) ||
    (lower.includes("max_tokens") && lower.includes("exceed") && lower.includes("context")) ||
    lower.includes("上下文过长") ||
    lower.includes("上下文超出") ||
    lower.includes("上下文长度超") ||
    lower.includes("超出最大上下文") ||
    lower.includes("请压缩上下文")
  );
}

function isInvalidTokenBudgetError(lower: string): boolean {
  return lower.includes("max_tokens must be at least 1") || (lower.includes("max_tokens") && lower.includes("got -"));
}

function isMediaTooLargeError(lower: string): boolean {
  return (
    lower.includes("image exceeds") ||
    lower.includes("image dimensions exceed") ||
    lower.includes("media too large") ||
    lower.includes("payload too large") ||
    lower.includes("request too large") ||
    lower.includes("request size exceeds") ||
    lower.includes("request exceeds the maximum size") ||
    lower.includes("附件或图片过大") ||
    lower.includes("图片附件总大小")
  );
}
```

In `inferModelErrorKindFromMessage`, classify the new kinds after billing/auth/rate-limit and before timeout/network:

```ts
  if (isContextOverflowError(lower)) {
    return "context_overflow";
  }

  if (isInvalidTokenBudgetError(lower)) {
    return "invalid_token_budget";
  }

  if (isMediaTooLargeError(lower)) {
    return "media_too_large";
  }
```

- [ ] **Step 5: Run focused frontend test and commit**

Run:

```powershell
pnpm --dir apps/runtime test -- src/lib/model-error-display.test.ts
```

Expected: focused frontend tests pass.

Commit:

```powershell
git add apps/runtime/src/types.ts apps/runtime/src/lib/model-error-display.ts apps/runtime/src/lib/model-error-display.test.ts
git commit -m "feat: show context and media model errors"
```

---

### Task 4: Strengthen Image Attachment Preflight

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_attachment_validation.rs`
- Modify: `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`

This task implements strict preflight validation without adding a new native image-resize dependency. It rejects unsupported or oversized image payloads before the provider call and uses wording that the model error normalizer can map to `media_too_large`.

- [ ] **Step 1: Write failing attachment validation tests**

Add these helper and tests to `apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs`:

```rust
fn image_attachment_with_payload(name: &str, mime: &str, payload: &str, size_bytes: Option<u64>) -> runtime_lib::commands::chat::AttachmentInput {
    runtime_lib::commands::chat::AttachmentInput {
        id: format!("att-{name}"),
        kind: "image".to_string(),
        name: name.to_string(),
        declared_mime_type: Some(mime.to_string()),
        size_bytes,
        source_type: "browser_file".to_string(),
        source_payload: Some(payload.to_string()),
        extracted_text: None,
        transcript: None,
        summary: None,
        thumbnail: None,
        warnings: Vec::new(),
        truncated: false,
    }
}

#[test]
fn validates_image_payload_mime_type() {
    let policy = default_attachment_policy();
    let attachment = image_attachment_with_payload(
        "not-image.txt",
        "text/plain",
        "data:text/plain;base64,aGVsbG8=",
        Some(5),
    );

    let err = validate_attachment_input(&policy, &attachment).expect_err("text payload should be rejected as image");
    assert!(err.contains("不支持的图片 MIME 类型"));
}

#[test]
fn validates_image_payload_is_valid_base64_data_url() {
    let policy = default_attachment_policy();
    let attachment = image_attachment_with_payload(
        "broken.png",
        "image/png",
        "data:image/png;base64,***not-base64***",
        None,
    );

    let err = validate_attachment_input(&policy, &attachment).expect_err("invalid base64 should be rejected");
    assert!(err.contains("图片附件 broken.png 读取失败"));
}

#[test]
fn oversize_image_payload_uses_media_too_large_friendly_wording() {
    let policy = default_attachment_policy();
    let payload = format!("data:image/png;base64,{}", "a".repeat((5 * 1024 * 1024 + 4) * 4 / 3));
    let attachment = image_attachment_with_payload("huge.png", "image/png", &payload, None);

    let err = validate_attachment_input(&policy, &attachment).expect_err("oversize image should be rejected");
    assert!(err.contains("附件或图片过大"));
}
```

- [ ] **Step 2: Run focused attachment tests and verify they fail**

Run:

```powershell
pnpm test:rust-fast -- test_chat_attachment_platform
```

Expected: at least the MIME/base64 wording assertions fail.

- [ ] **Step 3: Add image MIME and payload validation**

In `apps/runtime/src-tauri/src/commands/chat_attachment_validation.rs`, add:

```rust
fn validate_image_payload(attachment: &AttachmentInput) -> Result<(), String> {
    let declared = attachment
        .declared_mime_type
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !declared.starts_with("image/") {
        return Err(format!(
            "不支持的图片 MIME 类型 {}: {}",
            attachment.name,
            declared
        ));
    }

    let payload = attachment
        .source_payload
        .as_deref()
        .ok_or_else(|| format!("图片附件 {} 缺少 sourcePayload", attachment.name))?;
    decode_base64_payload_len(payload)
        .map(|_| ())
        .map_err(|err| format!("图片附件 {} 读取失败: {err}", attachment.name))
}
```

Call it inside the `"image"` branch after the existing `source_payload` check:

```rust
            validate_image_payload(attachment)?;
```

Update `validate_size_limit` for image attachments:

```rust
    if size_bytes > max_bytes {
        if attachment.kind == "image" {
            Err(format!(
                "附件或图片过大：{} 超过 {} 字节限制",
                attachment.name, max_bytes
            ))
        } else {
            Err(format!(
                "附件 {} 超过 {} 字节限制",
                attachment.name, max_bytes
            ))
        }
    } else {
        Ok(())
    }
```

Update `validate_total_image_payload_size` overflow wording:

```rust
return Err(format!(
    "附件或图片过大：图片附件总大小 {total_bytes} 超过 {} 字节限制",
    policy.image.max_total_bytes
));
```

- [ ] **Step 4: Run focused attachment tests and commit**

Run:

```powershell
pnpm test:rust-fast -- test_chat_attachment_platform
```

Expected: attachment tests pass.

Commit:

```powershell
git add apps/runtime/src-tauri/src/commands/chat_attachment_validation.rs apps/runtime/src-tauri/tests/test_chat_attachment_platform.rs
git commit -m "feat: validate image payloads before model requests"
```

---

### Task 5: Remove Historical Image Payloads From Model Context

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`

- [ ] **Step 1: Write failing transcript tests**

Add these tests to the existing `#[cfg(test)]` module in `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`. If the file does not have a test module, add one at the bottom.

```rust
#[test]
fn current_turn_image_is_preserved_for_openai_compatible_requests() {
    let parts = vec![
        json!({"type": "text", "text": "看这张图"}),
        json!({"type": "image", "name": "screen.png", "mimeType": "image/png", "data": "aGVsbG8="}),
    ];

    let message = RuntimeTranscript::build_current_turn_message("openai", &parts).expect("message");
    let content = message["content"].as_array().expect("content blocks");
    assert!(content.iter().any(|block| block["type"] == "image_url"));
}

#[test]
fn history_user_image_is_replaced_with_placeholder() {
    let history = vec![(
        "user".to_string(),
        "看这张图".to_string(),
        Some(
            serde_json::to_string(&vec![
                json!({"type": "text", "text": "看这张图"}),
                json!({"type": "image", "name": "screen.png", "mimeType": "image/png", "data": "data:image/png;base64,aGVsbG8="}),
            ])
            .unwrap(),
        ),
    )];

    let messages = RuntimeTranscript::reconstruct_history_messages(&history, "openai");
    let content = messages[0]["content"].as_array().expect("content blocks");
    assert!(content.iter().all(|block| block["type"] != "image_url"));
    let text = content[0]["text"].as_str().unwrap_or_default();
    assert!(text.contains("[历史图片 screen.png 已从模型上下文移除]"));
}

#[test]
fn history_text_only_message_is_unchanged() {
    let history = vec![(
        "user".to_string(),
        "hello".to_string(),
        Some(serde_json::to_string(&vec![json!({"type": "text", "text": "hello"})]).unwrap()),
    )];

    let messages = RuntimeTranscript::reconstruct_history_messages(&history, "openai");
    let content = messages[0]["content"].as_array().expect("content blocks");
    assert_eq!(content[0]["text"], "hello");
}
```

- [ ] **Step 2: Run focused transcript tests and verify they fail**

Run:

```powershell
pnpm test:rust-fast -- transcript
```

Expected: `history_user_image_is_replaced_with_placeholder` fails because history currently reuses `build_current_turn_message` and preserves image payloads.

- [ ] **Step 3: Add an image retention policy to transcript reconstruction**

In `apps/runtime/src-tauri/src/agent/runtime/transcript.rs`, add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImagePayloadMode {
    Preserve,
    Placeholder,
}
```

Change `build_current_turn_message` to call a private helper:

```rust
pub(crate) fn build_current_turn_message(api_format: &str, parts: &[Value]) -> Option<Value> {
    Self::build_user_message_with_image_mode(api_format, parts, ImagePayloadMode::Preserve)
}
```

Add the private helper with the current `build_current_turn_message` body and this image branch behavior:

```rust
fn build_user_message_with_image_mode(
    api_format: &str,
    parts: &[Value],
    image_mode: ImagePayloadMode,
) -> Option<Value> {
    if parts.is_empty() {
        return None;
    }
    let mut combined_text_parts = Vec::new();
    let mut content_blocks = Vec::new();
    let mut attachment_blocks = Vec::new();

    for part in parts {
        match part.get("type").and_then(Value::as_str).unwrap_or_default() {
            "text" => {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    if !text.trim().is_empty() {
                        combined_text_parts.push(text.trim().to_string());
                    }
                }
            }
            "image" => {
                if image_mode == ImagePayloadMode::Placeholder {
                    let name = part
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or("图片");
                    combined_text_parts.push(format!("[历史图片 {name} 已从模型上下文移除]"));
                    continue;
                }

                let mime_type = part
                    .get("mimeType")
                    .and_then(Value::as_str)
                    .unwrap_or("image/png");
                let data = part.get("data").and_then(Value::as_str).unwrap_or_default();
                if data.is_empty() {
                    continue;
                }
                if api_format == "anthropic" {
                    let base64_data = data
                        .split_once("base64,")
                        .map(|(_, payload)| payload)
                        .unwrap_or(data);
                    content_blocks.push(json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": mime_type,
                            "data": base64_data,
                        }
                    }));
                } else {
                    let data_url = if data.starts_with("data:") {
                        data.to_string()
                    } else {
                        format!("data:{mime_type};base64,{data}")
                    };
                    content_blocks.push(json!({
                        "type": "image_url",
                        "image_url": { "url": data_url }
                    }));
                }
            }
            "attachment" => {
                attachment_blocks.push(part.clone());
                content_blocks.push(part.clone());
            }
            _ => {}
        }
    }

    if let Some(file_context) = Self::build_attachment_context_text(parts) {
        combined_text_parts.push(file_context);
    }
    if let Some(attachment_context) =
        Self::build_attachment_context_text_from_attachment_blocks(&attachment_blocks)
    {
        combined_text_parts.push(attachment_context);
    }
    let combined_text = combined_text_parts.join("\n\n").trim().to_string();
    if !combined_text.is_empty() {
        content_blocks.insert(0, json!({ "type": "text", "text": combined_text }));
    }

    if content_blocks.is_empty() {
        None
    } else {
        Some(json!({
            "role": "user",
            "content": content_blocks,
        }))
    }
}
```

Replace the history reconstruction call:

```rust
if let Some(message) = Self::build_user_message_with_image_mode(
    api_format,
    parts_array,
    ImagePayloadMode::Placeholder,
) {
    return vec![message];
}
```

- [ ] **Step 4: Run focused transcript tests and commit**

Run:

```powershell
pnpm test:rust-fast -- transcript
```

Expected: transcript tests pass.

Commit:

```powershell
git add apps/runtime/src-tauri/src/agent/runtime/transcript.rs
git commit -m "feat: remove historical image payloads from context"
```

---

### Task 6: Integrated Verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Run Rust fast path**

Run:

```powershell
pnpm test:rust-fast
```

Expected: all Rust fast-path tests pass.

- [ ] **Step 2: Run frontend targeted tests**

Run:

```powershell
pnpm --dir apps/runtime test -- src/lib/model-error-display.test.ts
```

Expected: frontend model error display tests pass.

- [ ] **Step 3: Run attachment platform test directly if Rust fast path did not include it**

Run:

```powershell
pnpm test:rust-fast -- test_chat_attachment_platform
```

Expected: attachment platform tests pass.

- [ ] **Step 4: Check worktree and summarize**

Run:

```powershell
git status --short
```

Expected: clean worktree after the task commits, or only intentionally uncommitted changes from the current implementation batch.

Summarize:

- commands run;
- pass/fail status;
- any unverified areas;
- confirmation that no model settings vision test, technical details panel, doctor flow, or image re-reference UI was added.

---

## Plan Self-Review

- Spec coverage: Task 1 and Task 2 cover backend error normalization and runtime propagation. Task 3 covers frontend copy. Task 4 covers image preflight. Task 5 covers historical image context removal. Task 6 covers verification.
- Scope check: the plan does not include model-page probing, technical-details UI, doctor diagnostics, provider-specific vLLM settings, automatic retry, or model failover.
- Type consistency: the new keys are consistently named `context_overflow`, `invalid_token_budget`, and `media_too_large` across Rust serde output, frontend union types, route keys, and failover keys.
- Implementation boundary: image compression through a new dependency is not included; this phase implements strict preflight rejection and context hygiene, which the approved design allows as the minimum safe implementation.
