# 2026-02-27 对话与修改摘要（Skill 互调）

## 1. 对话目标

用户提出当前系统只支持“一个会话调用一个 Skill”，希望支持 Skill 之间互相调用（示例：`.claude/skills/using-superpowers` 调用其他子 Skill）。

核心诉求：
- 讨论可行实现方案；
- 明确如何在现有 Runtime 架构中落地；
- 继续推进并实现第一版代码；
- 最终将本次对话与修改做成文档沉淀。

## 2. 方案结论（本次对话中达成）

采用“新增 Skill 路由工具”的方式实现 Skill 互调，而不是把所有 Skill 一次性注入 system prompt。

分阶段策略：
1. MVP：新增 `skill` 工具，按名称加载子 Skill 的 `SKILL.md`，返回可执行指令。
2. Next：支持 `inline/fork` 两种模式，`fork` 复用子 Agent 执行链路，实现真正隔离执行。

## 3. 本次已完成改动

### 3.1 新增工具

- 新增文件：`apps/runtime/src-tauri/src/agent/tools/skill_invoke.rs`
- 新工具名：`skill`
- 输入参数：
  - `skill_name: string`（必填）
  - `arguments: string[]`（可选）

主要行为：
- 根据 `skill_name` 在本地路径查找 `<skill_name>/SKILL.md`；
- 使用 `SkillConfig::parse` 解析 frontmatter/system prompt；
- 使用 `substitute_arguments` 注入参数；
- 返回结构化说明 + 原始 Skill 指令文本，供主 Agent 继续执行。

### 3.2 工具系统接入

- 修改文件：`apps/runtime/src-tauri/src/agent/tools/mod.rs`
  - 注册模块：`mod skill_invoke;`
  - 导出符号：`pub use skill_invoke::SkillInvokeTool;`

### 3.3 会话执行链接入

- 修改文件：`apps/runtime/src-tauri/src/commands/chat.rs`
  - 在 `send_message` 中注册 `SkillInvokeTool`；
  - 组装 Skill 搜索根目录：
    - `work_dir/.claude/skills`
    - `current_dir/.claude/skills`
    - 本地 Skill 父目录（`source_type == "local"` 时）
    - `USERPROFILE/.claude/skills`
  - 增加辅助函数：`tool_ctx_from_work_dir`。

## 4. 安全与稳定保护（本次已加）

- `skill_name` 合法性校验（只允许字母数字和 `-_.`）；
- 调用深度限制（`max_depth = 4`）；
- 调用栈循环检测（防 A -> B -> A）。

## 5. 编译验证

- 在 `apps/runtime/src-tauri` 执行：
  - `cargo check -q`
- 结果：通过。

## 6. 当前能力边界

已实现的是“指令级互调”（加载并返回子 Skill 指令）。

尚未实现：
- `mode=fork` 的子 Skill 独立执行；
- 父/子 Skill 工具白名单交集约束；
- 更完整的调用链路可观测性（例如层级 trace）。

## 7. 建议的下一步

1. 扩展 `skill` 工具输入：`mode: inline | fork`。
2. `fork` 模式复用 `TaskTool/AgentExecutor`，以独立上下文执行子 Skill。
3. 加入白名单交集策略：`child_allowed = parent_allowed ∩ child_allowed`。
4. 增加互调集成测试（包含循环调用、深度超限、找不到 Skill 等异常路径）。

