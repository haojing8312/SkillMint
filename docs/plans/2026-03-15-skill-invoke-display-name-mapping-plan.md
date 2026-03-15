# Skill Invoke Display Name Mapping Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 `skill` 工具既能引导模型使用稳定的内部可调用标识，也能在收到展示名时自动映射到正确的技能目录。

**Architecture:** 先在工作区技能提示词里加入显式 `invoke_name` 元数据，并在系统提示词里强调 `skill_name` 的合法来源。然后在 `SkillInvokeTool` 中增加按 `SKILL.md` frontmatter `name` 的兜底映射，保证历史 prompt 和人工输入也能成功解析。

**Tech Stack:** Rust, Tauri runtime, serde_json, anyhow, cargo test

---

### Task 1: Prompt Metadata Regression Test

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat_runtime_io.rs`

**Step 1: Write the failing test**

为工作区技能 prompt 测试新增断言，要求输出包含稳定的可调用标识字段。

**Step 2: Run test to verify it fails**

Run: `cargo test workspace_skill_projection_tests::build_workspace_skill_prompt_entry_includes_location -- --exact`

Expected: FAIL，因为当前 prompt 不包含新字段。

**Step 3: Write minimal implementation**

更新 prompt entry 结构与序列化内容，输出 `invoke_name`。

**Step 4: Run test to verify it passes**

Run: `cargo test workspace_skill_projection_tests::build_workspace_skill_prompt_entry_includes_location -- --exact`

Expected: PASS

### Task 2: Skill Display Name Fallback Test

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_skill_permission_narrowing.rs`
- Modify: `apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`

**Step 1: Write the failing test**

新增一个测试，用中文展示名调用 `skill` 工具，期望命中对应目录并成功返回子技能内容。

**Step 2: Run test to verify it fails**

Run: `cargo test skill_tool_accepts_display_name_via_frontmatter_mapping --test test_skill_permission_narrowing -- --exact`

Expected: FAIL，当前实现会报 `INVALID_SKILL_NAME`。

**Step 3: Write minimal implementation**

在 `SkillInvokeTool` 中增加展示名到目录名的解析流程，并保持现有 ASCII/路径行为不变。

**Step 4: Run test to verify it passes**

Run: `cargo test skill_tool_accepts_display_name_via_frontmatter_mapping --test test_skill_permission_narrowing -- --exact`

Expected: PASS

### Task 3: Prompt Guidance Copy Update

**Files:**
- Modify: `packages/runtime-chat-app/src/service.rs`

**Step 1: Write the failing test**

更新系统 prompt 组装测试，要求说明 `skill` 工具调用应使用 `invoke_name` 或 `location`。

**Step 2: Run test to verify it fails**

Run: `cargo test compose_system_prompt_includes_execution_guidance_and_optional_sections -p runtime-chat-app -- --exact`

Expected: FAIL，因为当前文案未说明新的调用约束。

**Step 3: Write minimal implementation**

调整系统 prompt 文案，明确禁止把展示名当成 `skill_name`。

**Step 4: Run test to verify it passes**

Run: `cargo test compose_system_prompt_includes_execution_guidance_and_optional_sections -p runtime-chat-app -- --exact`

Expected: PASS

### Task 4: Targeted Verification

**Files:**
- Modify: none

**Step 1: Run focused regression suite**

Run: `cargo test workspace_skill_projection_tests --lib`

Expected: PASS

**Step 2: Run skill tool integration tests**

Run: `cargo test --test test_skill_permission_narrowing`

Expected: PASS

**Step 3: Run runtime-chat-app prompt tests**

Run: `cargo test -p runtime-chat-app prompt_assembly -- --exact`

Expected: PASS
