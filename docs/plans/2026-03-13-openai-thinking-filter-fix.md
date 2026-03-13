# OpenAI Thinking Filter Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 OpenAI 适配器在过滤 `<think>` 标签时对 UTF-8 字符串按字节切片导致的闪退，同时用测试锁定中文和跨 chunk 场景。

**Architecture:** 保持现有 `process_openai_sse_text()` 和 `LLMResponse` 接口不变，只替换 `filter_thinking()` 的内部实现。新实现使用字符安全的增量状态机，将 `<think>` 标签过滤限制在兼容层，优先保留现有 reasoning 字段跳过逻辑。

**Tech Stack:** Rust, cargo test, Tauri runtime adapter tests

---

### Task 1: Add failing adapter tests

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs`
- Test: `apps/runtime/src-tauri/src/adapters/openai.rs`

**Step 1: Write the failing tests**

在 `#[cfg(test)] mod tests` 中新增：
- `filter_thinking_keeps_multibyte_text_without_panicking`
- `filter_thinking_hides_cross_chunk_think_blocks`
- `filter_thinking_preserves_multibyte_text_after_think_block`

这些测试直接覆盖 `filter_thinking()` 和 `process_openai_sse_text()`。

**Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml filter_thinking -- --nocapture`

Expected:
- 至少一个新测试失败
- 失败原因与当前字节切片 panic 或错误输出一致

**Step 3: Commit**

先不提交，进入最小实现。

### Task 2: Replace byte slicing with a char-safe state machine

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs`

**Step 1: Write minimal implementation**

- 删除 `buf[..safe]` / `buf[safe..]` 一类逻辑
- 引入字符安全状态机
- 保持 `<think>` 跨 chunk 状态通过 `in_think` 继续传递
- 仅缓存少量潜在标签前缀，不缓存整段正文

**Step 2: Run targeted tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml filter_thinking -- --nocapture`

Expected:
- 新增测试通过

**Step 3: Refactor if needed**

- 保持实现简单，不顺带重构其它适配器逻辑

### Task 3: Verify adapter behavior does not regress

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs` if tests reveal issues

**Step 1: Run existing adapter tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib openai -- --nocapture`

Expected:
- OpenAI 适配器相关测试全部通过

**Step 2: Run broader backend verification**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib -- --nocapture`

Expected:
- 全部通过，或只有已知无关 warning

### Task 4: Document the fix in user-facing troubleshooting context

**Files:**
- Modify: `docs/troubleshooting/runtime-diagnostics.md`

**Step 1: Add a short note**

- 记录“如果诊断包出现 char boundary panic，优先检查 thinking 标签过滤逻辑”

**Step 2: Run doc sanity check**

Run: `rg -n "char boundary|thinking" docs/troubleshooting/runtime-diagnostics.md`

Expected:
- 能定位到新增说明

### Task 5: Commit the fix

**Step 1: Commit**

```bash
git add apps/runtime/src-tauri/src/adapters/openai.rs docs/troubleshooting/runtime-diagnostics.md docs/plans/2026-03-13-openai-thinking-filter-design.md docs/plans/2026-03-13-openai-thinking-filter-fix.md
git commit -m "fix(runtime): harden openai thinking filter"
```
