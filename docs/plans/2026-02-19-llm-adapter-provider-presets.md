# LLM Adapter Provider Presets Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix reasoning model output pollution and add provider preset UI so users can configure MiniMax, DeepSeek, Qwen, Moonshot, Yi with zero manual URL lookup.

**Architecture:** Two changes in parallel — (1) Rust `openai.rs` gets a stateful `<think>` stripper and `reasoning_content` filter; (2) `SettingsView.tsx` gets a provider preset dropdown that auto-fills the form. No DB changes, no new Rust files, no new commands.

**Tech Stack:** Rust (reqwest SSE streaming), React 18 + TypeScript + Tailwind, Tauri 2

---

### Task 1: Fix `openai.rs` — filter `reasoning_content` and `<think>` blocks

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs`

**Context:**
DeepSeek returns chain-of-thought in `choices[0].delta.reasoning_content` alongside `choices[0].delta.content`. MiniMax OpenAI-compat wraps thinking inline: `<think>I'm thinking...</think>Final answer`. Both bleed into the chat UI.

The fix: in the SSE stream loop, skip tokens from `reasoning_content`, and strip any `<think>…</think>` spans from `content` using a stateful bool (`in_think`).

**Step 1: Read current file**

Open `apps/runtime/src-tauri/src/adapters/openai.rs` and understand the existing stream parsing loop (around line 38–52).

**Step 2: Rewrite `chat_stream` with thinking filter**

Replace the entire `chat_stream` function body with the version below. Key changes:
- Add `let mut in_think = false;` before the stream loop
- For each `content` token, run it through `filter_thinking(token, &mut in_think)` before calling `on_token`
- Explicitly skip if `reasoning_content` is non-empty

```rust
pub async fn chat_stream(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<Value>,
    mut on_token: impl FnMut(String) + Send,
) -> Result<()> {
    let client = Client::new();
    let mut all_messages = vec![json!({"role": "system", "content": system_prompt})];
    all_messages.extend(messages);

    let body = json!({
        "model": model,
        "messages": all_messages,
        "stream": true
    });

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(anyhow!("API error: {text}"));
    }

    let mut stream = resp.bytes_stream();
    let mut in_think = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let delta = &v["choices"][0]["delta"];
                    // Skip DeepSeek reasoning_content tokens entirely
                    if delta["reasoning_content"].as_str().map(|s| !s.is_empty()).unwrap_or(false) {
                        continue;
                    }
                    if let Some(token) = delta["content"].as_str() {
                        let filtered = filter_thinking(token, &mut in_think);
                        if !filtered.is_empty() {
                            on_token(filtered);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Strip <think>…</think> spans from a streaming token chunk.
/// `in_think` carries state across chunk boundaries.
fn filter_thinking(input: &str, in_think: &mut bool) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut buf = String::new();

    while let Some(c) = chars.next() {
        buf.push(c);
        if *in_think {
            // Look for </think>
            if buf.ends_with("</think>") {
                *in_think = false;
                buf.clear();
            }
            // Keep buf bounded so it doesn't grow unbounded on large thinking blocks
            if buf.len() > 16 { buf = buf[buf.len()-16..].to_string(); }
        } else {
            // Look for <think>
            if buf.ends_with("<think>") {
                *in_think = true;
                // Remove the <think> prefix we may have already added to out
                let clean_len = out.len().saturating_sub(6); // len("<think>") - 1
                out.truncate(clean_len);
                buf.clear();
            } else {
                // Safe to emit everything except the last 6 chars (potential partial tag)
                if buf.len() > 7 {
                    let safe = buf.len() - 7;
                    out.push_str(&buf[..safe]);
                    buf = buf[safe..].to_string();
                }
            }
        }
    }
    // Flush remaining buffer if not in a thinking block
    if !*in_think {
        out.push_str(&buf);
    }
    out
}
```

**Step 3: Verify it compiles**

```bash
cd apps/runtime/src-tauri
cargo check 2>&1 | tail -5
```

Expected: `Finished dev profile` with no errors.

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/adapters/openai.rs
git commit -m "fix(runtime): filter reasoning_content and <think> blocks in openai adapter"
```

---

### Task 2: Add provider presets to `SettingsView.tsx`

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Context:**
Add a "快速选择" select at the top of the "添加模型" panel. When a provider is chosen, `api_format`, `base_url`, and `model_name` are auto-filled. The user still sets name and api_key manually. Selecting "自定义" is a no-op.

**Step 1: Add the PROVIDER_PRESETS constant**

Add this near the top of the file, before the component function:

```typescript
const PROVIDER_PRESETS = [
  { label: "— 快速选择 —", value: "" },
  { label: "OpenAI", value: "openai", api_format: "openai", base_url: "https://api.openai.com/v1", model_name: "gpt-4o-mini" },
  { label: "Claude (Anthropic)", value: "anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com/v1", model_name: "claude-3-5-haiku-20241022" },
  { label: "MiniMax (OpenAI 兼容)", value: "minimax-oai", api_format: "openai", base_url: "https://api.minimax.io/v1", model_name: "MiniMax-M2.5" },
  { label: "MiniMax (Anthropic 兼容)", value: "minimax-ant", api_format: "anthropic", base_url: "https://api.minimax.io/anthropic/v1", model_name: "MiniMax-M2.5" },
  { label: "DeepSeek", value: "deepseek", api_format: "openai", base_url: "https://api.deepseek.com/v1", model_name: "deepseek-chat" },
  { label: "Qwen (国际)", value: "qwen-intl", api_format: "openai", base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", model_name: "qwen-max" },
  { label: "Qwen (国内)", value: "qwen-cn", api_format: "openai", base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1", model_name: "qwen-max" },
  { label: "Moonshot / Kimi", value: "moonshot", api_format: "openai", base_url: "https://api.moonshot.ai/v1", model_name: "kimi-k2" },
  { label: "Yi", value: "yi", api_format: "openai", base_url: "https://api.lingyiwanwu.com/v1", model_name: "yi-large" },
  { label: "自定义", value: "custom" },
] as const;
```

**Step 2: Add `applyPreset` handler inside the component**

Add this function inside `SettingsView`, after the `handleTest` function:

```typescript
function applyPreset(value: string) {
  const preset = PROVIDER_PRESETS.find((p) => p.value === value);
  if (!preset || !("api_format" in preset)) return;
  setForm((f) => ({
    ...f,
    api_format: preset.api_format,
    base_url: preset.base_url,
    model_name: preset.model_name,
  }));
}
```

**Step 3: Add the preset select to JSX**

Inside the `<div className="bg-slate-800 rounded-lg p-4 space-y-3">` block, add the preset select **before** the "名称" field:

```tsx
<div>
  <label className={labelCls}>快速选择 Provider</label>
  <select
    className={inputCls}
    defaultValue=""
    onChange={(e) => applyPreset(e.target.value)}
  >
    {PROVIDER_PRESETS.map((p) => (
      <option key={p.value} value={p.value}>{p.label}</option>
    ))}
  </select>
</div>
```

**Step 4: Verify frontend builds**

```bash
cd apps/runtime
pnpm build 2>&1 | tail -10
```

Expected: `✓ built in` with no TypeScript errors.

**Step 5: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx
git commit -m "feat(runtime): add provider preset dropdown to settings (9 providers)"
```

---

### Task 3: End-to-end verification

**Step 1: Check Runtime hot-reloads**

The running `cargo tauri dev` in `apps/runtime` should auto-detect the Rust change and recompile. Watch for:
```
Finished dev profile
Running target\debug\runtime.exe
```

If it doesn't pick it up after ~60s, the tauri dev watcher may need a restart.

**Step 2: Test DeepSeek or MiniMax thinking filter**

In the Runtime window:
1. Go to 设置 → 快速选择 → "MiniMax (Anthropic 兼容)" or "DeepSeek"
2. Fill name + API key → 测试连接 → should show 连接成功
3. Save → select a skill → send a message that triggers reasoning (e.g. "solve: 2+2×3")
4. Verify the response shows only the final answer, no `<think>` tags or raw chain-of-thought

**Step 3: Test preset fills**

1. Go to 设置 → 快速选择 → "Qwen (国际)"
2. Verify base_url auto-fills to `https://dashscope-intl.aliyuncs.com/compatible-mode/v1`
3. Verify model_name fills to `qwen-max`
4. Verify api_format selector switches to "OpenAI 兼容"

**Step 4: Commit if any fixes needed, otherwise done**

```bash
git add -A
git commit -m "fix(runtime): address e2e verification issues"
```
