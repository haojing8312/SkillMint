# AgentExecutor 全面增强设计文档

**日期**：2026-02-23
**状态**：已批准
**范围**：Runtime Agent 系统 — 权限集成、上下文压缩、Bash 超时、状态追踪、子 Agent 流式、工具确认 UI

---

## 背景

17 项 Skill Runtime 增强已完成（15 个工具、Skill 元数据解析、权限模型定义、101 个测试）。但 AgentExecutor 执行流仍有关键缺口：

| 现状 | 缺口 |
|------|------|
| `PermissionMode` 已实现 | 未集成到 `execute_turn()` |
| `trim_messages()` 简单裁剪 | 无 LLM 摘要式上下文压缩 |
| `AgentState` 枚举已定义 | 未在循环中使用或发射 |
| `BashTool` 可执行命令 | 无超时、无危险命令拦截 |
| `TaskTool` 子 Agent 隔离 | 不向前端流式输出 |
| 工具正常执行 | 写操作无用户确认机制 |

---

## 第 1 部分：权限模型集成到执行流

### 设计

在 `execute_turn()` 循环内，工具执行前增加权限检查：

```
LLM 返回 ToolCalls
  ↓
对每个 tool_call:
  ├─ permission_mode.needs_confirmation(tool_name)?
  │   ├─ 否 → 直接执行
  │   └─ 是 → 发射 "tool-confirm-event"
  │            ├─ 前端显示确认卡片
  │            ├─ 用户点击 "允许" / "拒绝"
  │            ├─ 后端通过 mpsc channel 接收结果
  │            ├─ 允许 → 执行工具
  │            └─ 拒绝 → 返回 "用户拒绝了此操作" 作为 tool_result
  ↓
继续循环
```

### 关键决策

- **session 级别权限**：不同 Skill 可以有不同权限模式
- **复用 AskUser 的 mpsc 模式**：新增 `ToolConfirmResponder` 类型（`Arc<Mutex<Option<mpsc::Sender<bool>>>>`）
- **数据库存储**：`sessions` 表新增 `permission_mode TEXT DEFAULT 'default'` 列
- **前端传入**：`create_session` 时指定权限模式

### 涉及文件

| 文件 | 变更 |
|------|------|
| `agent/executor.rs` | `execute_turn` 增加 `permission_mode` 参数，工具执行前检查 |
| `commands/chat.rs` | 传递权限模式、注册 ToolConfirmResponder、新增 `confirm_tool_execution` command |
| `db.rs` | `sessions` 表迁移新增列 |
| `lib.rs` | 注册新 command |
| 前端 `ChatView.tsx` | 监听 `tool-confirm-event`、显示确认卡片 |

---

## 第 2 部分：三层上下文压缩

参考 Claude Code 的三层压缩策略：

### Layer 1 — 微压缩（每轮自动）

在 `execute_turn()` 调用 LLM 前执行：

```rust
fn micro_compact(messages: &mut Vec<Value>, keep_recent: usize) {
    // 找到所有 tool_result / tool 角色的消息
    // 保留最近 keep_recent=3 条完整内容
    // 将更早的替换为 "[已执行: {tool_name}]"
}
```

- 仅修改发送给 LLM 的 `trimmed` 副本，不影响 `messages` 原始数据
- 静默执行，无 LLM 调用开销

### Layer 2 — 自动压缩（token 超阈值）

当 `estimate_tokens(messages) > 50,000` 时触发：

1. **保存**：完整对话导出到 `{app_data_dir}/transcripts/{session_id}_{timestamp}.jsonl`
2. **摘要**：调用当前 LLM 生成结构化摘要
3. **替换**：用 `[对话已压缩。记录: {path}]\n\n{summary}` + 确认消息替换整个 messages

**摘要 Prompt**（适配自 Claude Code compact.prompt.md）：

```
请总结以下对话，确保连续性。输出以下章节：
1. 用户请求与意图
2. 关键技术上下文（技术栈、框架）
3. 已修改文件与代码段（含完整代码片段）
4. 错误与修复记录
5. 问题解决过程
6. 用户原始消息（逐条引用）
7. 待办任务
8. 当前工作状态
9. 建议的下一步
```

### Layer 3 — 手动压缩（compact 工具）

新增 `CompactTool`：
- Agent 可主动调用 `compact` 工具
- 支持 `focus` 参数指定摘要重点（如 "重点保留 TypeScript 相关变更"）
- 触发与 Layer 2 相同的逻辑

### 涉及文件

| 文件 | 变更 |
|------|------|
| `agent/executor.rs` | 添加 `micro_compact()`、阈值检查触发 `auto_compact()` |
| `agent/tools/compact_tool.rs` | 新建，实现 CompactTool |
| `agent/tools/mod.rs` | 导出 CompactTool |
| `commands/chat.rs` | 注册 CompactTool |

---

## 第 3 部分：Bash 超时控制

### 设计

```rust
impl BashTool {
    fn execute(&self, input: Value) -> Result<String> {
        let command = input["command"].as_str()?;
        let timeout_ms = input["timeout_ms"].as_u64().unwrap_or(120_000);

        // 1. 危险命令黑名单检查
        if is_dangerous(command) {
            return Ok("错误: 危险命令已被拦截".to_string());
        }

        // 2. 启动子进程
        let mut child = Command::new(shell).args(args).spawn()?;

        // 3. 超时等待
        match child.wait_timeout(Duration::from_millis(timeout_ms)) {
            Ok(Some(status)) => { /* 正常完成 */ }
            Ok(None) => {
                child.kill()?;
                return Ok(format!("命令执行超时（{}ms），已终止", timeout_ms));
            }
            Err(e) => { /* 错误处理 */ }
        }
    }
}
```

### 危险命令黑名单

```rust
fn is_dangerous(command: &str) -> bool {
    let patterns = [
        "rm -rf /", "rm -rf /*",
        "format c:", "format d:",
        "shutdown", "reboot",
        "> /dev/sda", "dd if=/dev/zero",
        ":(){ :|:& };:",  // fork bomb
    ];
    let lower = command.to_lowercase();
    patterns.iter().any(|p| lower.contains(p))
}
```

### 涉及文件

| 文件 | 变更 |
|------|------|
| `agent/tools/bash.rs` | 添加超时、黑名单检查 |
| `tests/test_bash.rs` | 新增超时和黑名单测试 |

---

## 第 4 部分：AgentState 状态追踪

### 设计

executor 在循环每个阶段发射 `agent-state-event`：

```rust
#[derive(Serialize, Clone)]
struct AgentStateEvent {
    session_id: String,
    state: String,      // "thinking" | "tool_calling" | "finished" | "error"
    detail: Option<String>, // 工具名 or 错误信息
    iteration: usize,
}
```

发射时机：
- 循环开始 → `thinking`
- 收到 ToolCalls → `tool_calling` (detail = 工具名列表)
- 循环结束（Text）→ `finished`
- 错误 → `error` (detail = 错误信息)

### 前端展示

ChatView 顶部添加状态条：
- `thinking` → "思考中..." + 旋转动画
- `tool_calling` → "执行工具: read_file, bash" + 进度动画
- `finished` → 隐藏
- `error` → 红色提示

### 涉及文件

| 文件 | 变更 |
|------|------|
| `agent/executor.rs` | 每阶段发射 `agent-state-event` |
| 前端 `ChatView.tsx` | 监听事件、显示状态条 |

---

## 第 5 部分：子 Agent 流式输出

### 设计

当前 TaskTool 的 `on_token` 是 `|_| {}`，子 Agent 执行过程对用户不可见。

改进方案：
- TaskTool 构造时接收 `AppHandle` 和 `session_id`
- 子 Agent 的 token 通过 `stream-token` 事件发送，额外标记 `sub_agent: true`
- 子 Agent 的 `tool-call-event` 也正常发射

```rust
// TaskTool::execute 内部
let on_token = {
    let app = self.app_handle.clone();
    let sid = self.session_id.clone();
    move |token: String| {
        let _ = app.emit("stream-token", StreamToken {
            session_id: sid.clone(),
            token,
            done: false,
            sub_agent: true,
        });
    }
};
```

### 前端展示

- `ToolCallCard` 中 `task` 类型的卡片内嵌流式文本区域
- 子 Agent 的 tool-call-event 在卡片内部嵌套展示

### 涉及文件

| 文件 | 变更 |
|------|------|
| `agent/tools/task_tool.rs` | 接收 AppHandle/session_id，转发 stream-token |
| `commands/chat.rs` | 构造 TaskTool 时传入 AppHandle |
| `StreamToken` struct | 新增 `sub_agent: bool` 字段 |
| 前端 `ChatView.tsx` | 区分主/子 Agent 流式输出 |
| 前端 `ToolCallCard.tsx` | 内嵌流式文本 |

---

## 第 6 部分：工具确认 UI

### 前端新增组件

`ToolConfirmCard` — 显示待确认的工具调用：

```
┌────────────────────────────────────────┐
│  ⚠️ 需要确认                           │
│  工具: bash                            │
│  命令: rm -rf node_modules             │
│                                        │
│  [允许]  [拒绝]                         │
└────────────────────────────────────────┘
```

### 事件流

```
后端 emit "tool-confirm-event" { session_id, tool_name, tool_input }
  ↓
前端显示 ToolConfirmCard
  ↓
用户点击 → invoke("confirm_tool_execution", { confirmed: bool })
  ↓
后端 mpsc channel 收到结果 → 继续执行
```

### 涉及文件

| 文件 | 变更 |
|------|------|
| 前端新建 `ToolConfirmCard.tsx` | 确认卡片组件 |
| 前端 `ChatView.tsx` | 监听 `tool-confirm-event`、显示确认卡片 |
| `commands/chat.rs` | 新增 `confirm_tool_execution` command |
| `lib.rs` | 注册 command |

---

## 实施顺序（建议）

按依赖关系排序：

1. **Bash 超时 + 黑名单**（独立，无依赖）
2. **AgentState 状态追踪**（独立，修改 executor + 前端）
3. **权限集成 + 工具确认 UI**（依赖 executor 改动 + 新 command + 新前端组件）
4. **Layer 1 微压缩**（修改 executor，独立于权限）
5. **Layer 2 自动压缩**（依赖 Layer 1，需要 LLM 调用）
6. **Layer 3 compact 工具**（依赖 Layer 2 逻辑）
7. **子 Agent 流式输出**（修改 TaskTool + StreamToken + 前端）

---

## 测试策略

| 部分 | 测试方式 |
|------|----------|
| 权限集成 | 单元测试 needs_confirmation + 集成测试（mock channel） |
| 三层压缩 | micro_compact 单元测试、auto_compact mock LLM 测试 |
| Bash 超时 | 超时命令测试、黑名单匹配测试 |
| AgentState | 事件发射验证 |
| 子 Agent 流式 | TaskTool 集成测试 |
| 工具确认 UI | 手动测试 + TypeScript 编译检查 |
