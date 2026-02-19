# LLM Adapter Provider Presets Design

**Date:** 2026-02-19
**Status:** Approved

## Problem

The current adapter layer has two issues:

1. **`anthropic.rs` hardcodes `api.anthropic.com`** — third-party Anthropic-compatible providers (e.g., MiniMax `https://api.minimax.io/anthropic/v1`) cannot be configured.
2. **Reasoning model output is polluted** — DeepSeek returns chain-of-thought in `delta.reasoning_content`; MiniMax OpenAI-compat wraps thinking in `<think>...</think>` inside `delta.content`. Both bleed raw internal monologue into the chat UI.
3. **No user-facing provider guidance** — users must manually look up correct base URLs and model names for every provider.

## Decision: Protocol + Provider Preset Separation (Option B)

**Core principle:** Rust only recognizes two protocols (`openai` / `anthropic`). Provider brand identity lives entirely in the frontend as presets. No new Rust enums per provider.

## Architecture

```
Frontend SettingsView
  └── Provider preset dropdown (fills form only, not stored)
        └── form: { api_format, base_url, model_name, api_key }
              └── invoke("test_connection_cmd") / invoke("save_model_config")

Rust adapters/
  ├── openai.rs   — handles all OpenAI-compatible providers
  │   └── filters out reasoning_content + <think>...</think> blocks
  └── anthropic.rs — handles all Anthropic-compatible providers
      └── already correct: only emits text_delta, ignores thinking_delta
```

`ModelConfig` struct is unchanged. No new database columns.

## Rust Changes: `adapters/openai.rs`

Two filtering rules applied to every SSE chunk in `chat_stream`:

1. **Skip `reasoning_content`**: If `choices[0].delta.reasoning_content` is present and non-empty, do not call `on_token`. Only emit `choices[0].delta.content`.
2. **Strip `<think>…</think>` blocks**: After extracting `content`, use a state machine (`in_think: bool`) to discard bytes between `<think>` and `</think>` tags before calling `on_token`. This handles cases where a provider embeds chain-of-thought inline in the content string.

The filtering is stateful across chunks (a `<think>` block may span multiple SSE chunks), so a `bool` flag is maintained in the stream loop.

## Rust Changes: `adapters/anthropic.rs`

No changes needed. The current implementation extracts only `v["delta"]["text"]`, which corresponds to `text_delta` events. MiniMax's `thinking_delta` events carry their content in `v["delta"]["thinking"]`, not `v["delta"]["text"]`, so they are naturally ignored.

## Frontend Changes: `SettingsView.tsx`

Add a `PROVIDER_PRESETS` constant and a "快速选择" select above the form. Selecting a preset calls `applyPreset(preset)` which sets `form.api_format`, `form.base_url`, and `form.model_name`. The name and api_key fields are not touched by presets.

### Provider Preset Table

| Label | api_format | base_url | default model_name |
|---|---|---|---|
| OpenAI | openai | `https://api.openai.com/v1` | `gpt-4o-mini` |
| Claude (Anthropic) | anthropic | `https://api.anthropic.com/v1` | `claude-3-5-haiku-20241022` |
| MiniMax (OpenAI 兼容) | openai | `https://api.minimax.io/v1` | `MiniMax-M2.5` |
| MiniMax (Anthropic 兼容) | anthropic | `https://api.minimax.io/anthropic/v1` | `MiniMax-M2.5` |
| DeepSeek | openai | `https://api.deepseek.com/v1` | `deepseek-chat` |
| Qwen (国际) | openai | `https://dashscope-intl.aliyuncs.com/compatible-mode/v1` | `qwen-max` |
| Qwen (国内) | openai | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-max` |
| Moonshot/Kimi | openai | `https://api.moonshot.ai/v1` | `kimi-k2` |
| Yi | openai | `https://api.lingyiwanwu.com/v1` | `yi-large` |
| 自定义 | — | (no-op) | — |

## Provider Quirks Reference (not implemented, informational)

| Provider | Known quirk | Impact |
|---|---|---|
| DeepSeek | `delta.reasoning_content` field in stream | Filtered by openai.rs |
| MiniMax OpenAI | `<think>…</think>` in `delta.content` | Filtered by openai.rs |
| MiniMax Anthropic | `thinking_delta` event type | Already ignored by anthropic.rs |
| Qwen | Region-locked keys (intl vs CN key) | Handled via two separate presets |
| Moonshot | `temperature` max is 1.0, not 2.0 | No impact (we don't set temperature) |
| DeepSeek reasoner | No `temperature` param support | No impact (we don't set temperature) |
| GLM/Zhipu | JWT auth, not Bearer | **Not supported** in this iteration |

## Scope

- Rust: only `adapters/openai.rs` changes (add thinking filter)
- Frontend: only `SettingsView.tsx` changes (add preset dropdown)
- No DB migration, no new commands, no new Rust files
