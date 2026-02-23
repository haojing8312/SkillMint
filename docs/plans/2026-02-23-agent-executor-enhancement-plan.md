# AgentExecutor 全面增强实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 AgentExecutor 具备权限确认、三层上下文压缩、Bash 超时、状态追踪、子 Agent 流式输出和工具确认 UI

**Architecture:** 在现有 `execute_turn()` ReAct 循环基础上，增加权限检查层（mpsc channel 阻塞等待前端确认）、三层压缩管线（micro_compact → auto_compact → compact 工具）、AgentState 事件发射。前端监听新事件并渲染对应 UI 组件。

**Tech Stack:** Rust (Tauri 2, tokio, serde_json, wait-timeout), TypeScript (React 18, Tailwind CSS)

---

## Phase 1: Bash 超时与危险命令拦截（独立，无依赖）

### Task 1: Bash 危险命令黑名单

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/bash.rs`
- Test: `apps/runtime/src-tauri/tests/test_bash.rs`

**Step 1: 写测试**

在 `tests/test_bash.rs` 末尾添加：

```rust
#[test]
fn test_bash_dangerous_command_blocked() {
    let tool = BashTool::new();
    let input = json!({"command": "rm -rf /"});
    let result = tool.execute(input).unwrap();
    assert!(result.contains("危险命令"));
}

#[test]
fn test_bash_dangerous_format_blocked() {
    let tool = BashTool::new();
    let input = json!({"command": "format c:"});
    let result = tool.execute(input).unwrap();
    assert!(result.contains("危险命令"));
}

#[test]
fn test_bash_safe_command_not_blocked() {
    let tool = BashTool::new();
    let input = json!({"command": "echo safe"});
    let result = tool.execute(input).unwrap();
    assert!(!result.contains("危险命令"));
    assert!(result.contains("safe"));
}
```

**Step 2: 运行测试验证失败**

```bash
cd apps/runtime/src-tauri && cargo test --test test_bash -- test_bash_dangerous 2>&1
```

Expected: FAIL（`is_dangerous` 函数尚不存在）

**Step 3: 实现危险命令检查**

在 `bash.rs` 的 `impl BashTool` 块内（`get_shell` 方法之后）添加：

```rust
fn is_dangerous(command: &str) -> bool {
    let lower = command.to_lowercase();
    let patterns = [
        "rm -rf /", "rm -rf /*", "rm -rf ~",
        "format c:", "format d:",
        "shutdown", "reboot",
        "> /dev/sda", "dd if=/dev/zero",
        ":(){ :|:& };:",
        "mkfs.", "wipefs",
    ];
    patterns.iter().any(|p| lower.contains(p))
}
```

在 `execute` 方法的 command 解析之后、Command::new 之前添加：

```rust
if Self::is_dangerous(command) {
    return Ok("错误: 危险命令已被拦截。此命令可能造成不可逆损害。".to_string());
}
```

**Step 4: 运行测试验证通过**

```bash
cd apps/runtime/src-tauri && cargo test --test test_bash 2>&1
```

Expected: 5 passed, 0 failed

**Step 5: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/tools/bash.rs apps/runtime/src-tauri/tests/test_bash.rs
git commit -m "feat(agent): Bash 工具添加危险命令拦截"
```

---

### Task 2: Bash 超时控制

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/bash.rs`
- Modify: `apps/runtime/src-tauri/Cargo.toml`（添加 `wait-timeout` 依赖）
- Test: `apps/runtime/src-tauri/tests/test_bash.rs`

**Step 1: 添加依赖**

在 `Cargo.toml` 的 `[dependencies]` 中添加：

```toml
wait-timeout = "0.2"
```

**Step 2: 写测试**

在 `tests/test_bash.rs` 末尾添加：

```rust
#[test]
fn test_bash_timeout() {
    let tool = BashTool::new();
    // Windows: ping -n 10 localhost (等 10 秒)
    // Unix: sleep 10
    let command = if cfg!(target_os = "windows") {
        "ping -n 10 127.0.0.1"
    } else {
        "sleep 10"
    };
    let input = json!({"command": command, "timeout_ms": 1000});
    let result = tool.execute(input).unwrap();
    assert!(result.contains("超时"));
}

#[test]
fn test_bash_no_timeout_fast_command() {
    let tool = BashTool::new();
    let input = json!({"command": "echo fast", "timeout_ms": 5000});
    let result = tool.execute(input).unwrap();
    assert!(result.contains("fast"));
    assert!(!result.contains("超时"));
}
```

**Step 3: 运行测试验证失败**

```bash
cd apps/runtime/src-tauri && cargo test --test test_bash -- test_bash_timeout 2>&1
```

Expected: FAIL（超时逻辑未实现）

**Step 4: 实现超时**

重写 `bash.rs` 的 `execute` 方法使用 `wait_timeout` crate：

```rust
use std::process::{Command, Stdio, Child};
use wait_timeout::ChildExt;
use std::time::Duration;

fn execute(&self, input: Value) -> Result<String> {
    let command = input["command"]
        .as_str()
        .ok_or_else(|| anyhow!("缺少 command 参数"))?;

    // 危险命令检查
    if Self::is_dangerous(command) {
        return Ok("错误: 危险命令已被拦截。此命令可能造成不可逆损害。".to_string());
    }

    let timeout_ms = input["timeout_ms"].as_u64().unwrap_or(120_000);
    let (shell, flag) = Self::get_shell();

    let mut child: Child = Command::new(shell)
        .arg(flag)
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let timeout = Duration::from_millis(timeout_ms);
    match child.wait_timeout(timeout)? {
        Some(status) => {
            let stdout = String::from_utf8_lossy(
                &child.stdout.take().map(|mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).ok();
                    buf
                }).unwrap_or_default()
            ).to_string();
            let stderr = String::from_utf8_lossy(
                &child.stderr.take().map(|mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).ok();
                    buf
                }).unwrap_or_default()
            ).to_string();

            if !status.success() {
                Ok(format!(
                    "命令执行失败（退出码 {}）\nstderr:\n{}",
                    status.code().unwrap_or(-1),
                    stderr
                ))
            } else {
                Ok(format!("stdout:\n{}\nstderr:\n{}", stdout, stderr))
            }
        }
        None => {
            // 超时 — kill 子进程
            let _ = child.kill();
            let _ = child.wait(); // 回收资源
            Ok(format!("命令执行超时（{}ms），已终止", timeout_ms))
        }
    }
}
```

**Step 5: 运行全部 bash 测试**

```bash
cd apps/runtime/src-tauri && cargo test --test test_bash 2>&1
```

Expected: 7 passed, 0 failed

**Step 6: 提交**

```bash
git add apps/runtime/src-tauri/Cargo.toml apps/runtime/src-tauri/src/agent/tools/bash.rs apps/runtime/src-tauri/tests/test_bash.rs
git commit -m "feat(agent): Bash 工具添加超时控制（wait-timeout）"
```

---

## Phase 2: AgentState 状态追踪

### Task 3: AgentState 事件发射

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Test: 编译验证（事件发射依赖 Tauri 运行时，用单元测试覆盖不依赖 Tauri 的部分）

**Step 1: 定义 AgentStateEvent 结构体**

在 `executor.rs` 的 `ToolCallEvent` 结构体之后添加：

```rust
#[derive(serde::Serialize, Clone, Debug)]
pub struct AgentStateEvent {
    pub session_id: String,
    pub state: String,          // "thinking" | "tool_calling" | "finished" | "error"
    pub detail: Option<String>, // 工具名列表 or 错误信息
    pub iteration: usize,
}
```

**Step 2: 在 execute_turn 循环中发射事件**

在循环的 `eprintln!("[agent] Iteration...")` 之后添加：

```rust
// 发射 "thinking" 状态
if let (Some(app), Some(sid)) = (app_handle, session_id) {
    let _ = app.emit("agent-state-event", AgentStateEvent {
        session_id: sid.to_string(),
        state: "thinking".to_string(),
        detail: None,
        iteration,
    });
}
```

在 `LLMResponse::ToolCalls` 分支的 `eprintln!("[agent] Executing...")` 之后添加：

```rust
if let (Some(app), Some(sid)) = (app_handle, session_id) {
    let tool_names: Vec<&str> = tool_calls.iter().map(|tc| tc.name.as_str()).collect();
    let _ = app.emit("agent-state-event", AgentStateEvent {
        session_id: sid.to_string(),
        state: "tool_calling".to_string(),
        detail: Some(tool_names.join(", ")),
        iteration,
    });
}
```

在 `LLMResponse::Text` 分支的 return 之前添加：

```rust
if let (Some(app), Some(sid)) = (app_handle, session_id) {
    let _ = app.emit("agent-state-event", AgentStateEvent {
        session_id: sid.to_string(),
        state: "finished".to_string(),
        detail: None,
        iteration,
    });
}
```

在 max_iterations 错误返回之前添加：

```rust
if let (Some(app), Some(sid)) = (app_handle, session_id) {
    let _ = app.emit("agent-state-event", AgentStateEvent {
        session_id: sid.to_string(),
        state: "error".to_string(),
        detail: Some(format!("达到最大迭代次数 {}", self.max_iterations)),
        iteration,
    });
}
```

**Step 3: 编译验证**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
```

Expected: 编译成功

**Step 4: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs
git commit -m "feat(agent): AgentState 事件发射（thinking/tool_calling/finished/error）"
```

---

### Task 4: 前端 AgentState 状态条

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: 添加状态变量和事件监听**

在 `askUserAnswer` 状态之后添加：

```tsx
const [agentState, setAgentState] = useState<{
  state: string;
  detail?: string;
  iteration: number;
} | null>(null);
```

在 ask-user-event useEffect 之后添加新的 useEffect：

```tsx
// agent-state-event 事件监听
useEffect(() => {
  const unlistenPromise = listen<{
    session_id: string;
    state: string;
    detail: string | null;
    iteration: number;
  }>("agent-state-event", ({ payload }) => {
    if (payload.session_id !== sessionId) return;
    if (payload.state === "finished") {
      setAgentState(null);
    } else {
      setAgentState({
        state: payload.state,
        detail: payload.detail ?? undefined,
        iteration: payload.iteration,
      });
    }
  });
  return () => {
    unlistenPromise.then((fn) => fn());
  };
}, [sessionId]);
```

在 sessionId useEffect 中添加 `setAgentState(null);` 重置。

**Step 2: 渲染状态条**

在消息列表 `<div>` 开头（`{messages.map...}` 之前）添加：

```tsx
{agentState && (
  <div className="sticky top-0 z-10 flex items-center gap-2 bg-slate-800/90 backdrop-blur px-4 py-2 rounded-lg text-xs text-slate-300 border border-slate-700">
    <span className="animate-spin h-3 w-3 border-2 border-blue-400 border-t-transparent rounded-full" />
    {agentState.state === "thinking" && "思考中..."}
    {agentState.state === "tool_calling" && `执行工具: ${agentState.detail}`}
    {agentState.state === "error" && (
      <span className="text-red-400">错误: {agentState.detail}</span>
    )}
    <span className="text-slate-500 ml-auto">迭代 {agentState.iteration}</span>
  </div>
)}
```

**Step 3: TypeScript 编译验证**

```bash
cd apps/runtime && npx tsc --noEmit 2>&1
```

Expected: 无错误

**Step 4: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): 添加 Agent 状态指示条（思考中/执行工具/错误）"
```

---

## Checkpoint 1

运行全量测试确认无回归：

```bash
cd apps/runtime/src-tauri && cargo test 2>&1
cd apps/runtime && npx tsc --noEmit 2>&1
```

Expected: 全部通过

---

## Phase 3: 权限集成与工具确认 UI

### Task 5: 数据库 sessions 表添加 permission_mode 列

**Files:**
- Modify: `apps/runtime/src-tauri/src/db.rs`

**Step 1: 添加迁移**

在 `db.rs` 的 `init_db` 函数末尾（`Ok(pool)` 之前）添加：

```rust
// Migration: add permission_mode column
let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN permission_mode TEXT NOT NULL DEFAULT 'default'")
    .execute(&pool)
    .await;
```

**Step 2: 编译验证**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
```

**Step 3: 提交**

```bash
git add apps/runtime/src-tauri/src/db.rs
git commit -m "feat(db): sessions 表添加 permission_mode 列"
```

---

### Task 6: ToolConfirmResponder 和 confirm_tool_execution command

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`

**Step 1: 在 chat.rs 顶部添加新状态结构**

在 `AskUserState` 之后添加：

```rust
/// 工具确认通道
pub struct ToolConfirmState(pub Arc<std::sync::Mutex<Option<std::sync::mpsc::Sender<bool>>>>);
```

添加 `use crate::agent::permissions::PermissionMode;` 到导入。

**Step 2: 添加 confirm_tool_execution command**

在 `answer_user_question` command 之后添加：

```rust
/// 用户确认或拒绝工具执行
#[tauri::command]
pub async fn confirm_tool_execution(
    confirmed: bool,
    tool_confirm_state: State<'_, ToolConfirmState>,
) -> Result<(), String> {
    let guard = tool_confirm_state
        .0
        .lock()
        .map_err(|e| format!("锁获取失败: {}", e))?;
    if let Some(sender) = guard.as_ref() {
        sender
            .send(confirmed)
            .map_err(|e| format!("发送确认失败: {}", e))?;
        Ok(())
    } else {
        Err("没有等待中的工具确认请求".to_string())
    }
}
```

**Step 3: 在 lib.rs 注册 command**

在 `invoke_handler` 的 `answer_user_question` 之后添加：

```rust
commands::chat::confirm_tool_execution,
```

**Step 4: 编译验证**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
```

**Step 5: 提交**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(agent): 添加 ToolConfirmState 和 confirm_tool_execution command"
```

---

### Task 7: execute_turn 权限检查集成

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: 扩展 execute_turn 签名**

添加两个参数到 `execute_turn`：

```rust
pub async fn execute_turn(
    &self,
    api_format: &str,
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    mut messages: Vec<Value>,
    on_token: impl Fn(String) + Send + Clone,
    app_handle: Option<&AppHandle>,
    session_id: Option<&str>,
    allowed_tools: Option<&[String]>,
    permission_mode: PermissionMode,                                    // 新增
    tool_confirm_tx: Option<Arc<std::sync::Mutex<Option<std::sync::mpsc::Sender<bool>>>>>, // 新增
) -> Result<Vec<Value>>
```

添加导入：`use super::permissions::PermissionMode;`

**Step 2: 在工具执行前添加权限检查**

在 `let result = match self.registry.get(&call.name)` 之前，添加权限确认逻辑：

```rust
// 权限确认检查
if permission_mode.needs_confirmation(&call.name) {
    if let (Some(app), Some(sid)) = (app_handle, session_id) {
        // 发射确认请求事件
        let _ = app.emit("tool-confirm-event", serde_json::json!({
            "session_id": sid,
            "tool_name": call.name,
            "tool_input": call.input,
        }));

        // 创建通道并等待用户确认
        let (tx, rx) = std::sync::mpsc::channel();
        if let Some(ref confirm_state) = tool_confirm_tx {
            if let Ok(mut guard) = confirm_state.lock() {
                *guard = Some(tx);
            }
        }

        let confirmed = rx
            .recv_timeout(std::time::Duration::from_secs(300))
            .unwrap_or(false);

        // 清理
        if let Some(ref confirm_state) = tool_confirm_tx {
            if let Ok(mut guard) = confirm_state.lock() {
                *guard = None;
            }
        }

        if !confirmed {
            // 用户拒绝
            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                let _ = app.emit("tool-call-event", ToolCallEvent {
                    session_id: sid.to_string(),
                    tool_name: call.name.clone(),
                    tool_input: call.input.clone(),
                    tool_output: Some("用户拒绝了此操作".to_string()),
                    status: "error".to_string(),
                });
            }
            tool_results.push(ToolResult {
                tool_use_id: call.id.clone(),
                content: "用户拒绝了此操作".to_string(),
            });
            continue; // 跳过此工具，继续下一个
        }
    }
}
```

**Step 3: 更新所有 execute_turn 调用点**

在 `commands/chat.rs` 的 `send_message` 中，读取 session 的 permission_mode 并传入：

```rust
// 读取会话权限模式
let (skill_id, model_id, perm_str) = sqlx::query_as::<_, (String, String, String)>(
    "SELECT skill_id, model_id, permission_mode FROM sessions WHERE id = ?"
)
// ...
let permission_mode = match perm_str.as_str() {
    "accept_edits" => PermissionMode::AcceptEdits,
    "unrestricted" => PermissionMode::Unrestricted,
    _ => PermissionMode::Default,
};
```

传入 execute_turn：

```rust
.execute_turn(
    // ... existing params ...
    permission_mode,
    Some(tool_confirm_responder.clone()),
)
```

同时在 send_message 中创建 ToolConfirmState 并管理到 app：

```rust
let tool_confirm_responder = Arc::new(std::sync::Mutex::new(None));
app.manage(ToolConfirmState(tool_confirm_responder.clone()));
```

在 TaskTool 的调用处，传入 `PermissionMode::Unrestricted` 和 `None`（子 Agent 不需要权限确认）。

**Step 4: 编译验证**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
```

**Step 5: 运行全部测试**

注意：现有测试调用 `execute_turn` 的地方需要添加新参数。搜索所有 `execute_turn` 调用点并添加 `PermissionMode::Unrestricted, None`。

```bash
cd apps/runtime/src-tauri && cargo test 2>&1
```

**Step 6: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/tools/task_tool.rs
git commit -m "feat(agent): execute_turn 集成权限确认（PermissionMode + mpsc channel）"
```

---

### Task 8: 工具确认前端 UI

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: 添加状态和事件监听**

```tsx
const [toolConfirm, setToolConfirm] = useState<{
  toolName: string;
  toolInput: Record<string, unknown>;
} | null>(null);
```

添加 useEffect 监听 `tool-confirm-event`：

```tsx
useEffect(() => {
  const unlistenPromise = listen<{
    session_id: string;
    tool_name: string;
    tool_input: Record<string, unknown>;
  }>("tool-confirm-event", ({ payload }) => {
    if (payload.session_id !== sessionId) return;
    setToolConfirm({
      toolName: payload.tool_name,
      toolInput: payload.tool_input,
    });
  });
  return () => {
    unlistenPromise.then((fn) => fn());
  };
}, [sessionId]);
```

**Step 2: 添加确认处理函数**

```tsx
async function handleToolConfirm(confirmed: boolean) {
  try {
    await invoke("confirm_tool_execution", { confirmed });
  } catch (e) {
    console.error("工具确认失败:", e);
  }
  setToolConfirm(null);
}
```

**Step 3: 渲染确认卡片**

在 AskUser 卡片之前添加：

```tsx
{toolConfirm && (
  <div className="flex justify-start">
    <div className="max-w-2xl bg-orange-900/40 border border-orange-600/50 rounded-lg px-4 py-3 text-sm">
      <div className="font-medium text-orange-200 mb-1">需要确认执行</div>
      <div className="text-slate-300 mb-2">
        工具: <span className="font-mono text-orange-300">{toolConfirm.toolName}</span>
      </div>
      <pre className="text-xs bg-slate-800 rounded p-2 mb-2 overflow-x-auto max-h-32 text-slate-400">
        {JSON.stringify(toolConfirm.toolInput, null, 2)}
      </pre>
      <div className="flex gap-2">
        <button
          onClick={() => handleToolConfirm(true)}
          className="bg-green-600 hover:bg-green-700 text-white px-4 py-1 rounded text-xs transition-colors"
        >
          允许
        </button>
        <button
          onClick={() => handleToolConfirm(false)}
          className="bg-red-600 hover:bg-red-700 text-white px-4 py-1 rounded text-xs transition-colors"
        >
          拒绝
        </button>
      </div>
    </div>
  </div>
)}
```

**Step 4: TypeScript 编译验证**

```bash
cd apps/runtime && npx tsc --noEmit 2>&1
```

**Step 5: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): 添加工具执行确认卡片（允许/拒绝）"
```

---

## Checkpoint 2

```bash
cd apps/runtime/src-tauri && cargo test 2>&1
cd apps/runtime && npx tsc --noEmit 2>&1
```

Expected: 全部通过

---

## Phase 4: 三层上下文压缩

### Task 9: Layer 1 — 微压缩

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Test: `apps/runtime/src-tauri/tests/test_context_trimming.rs`

**Step 1: 写测试**

在 `tests/test_context_trimming.rs` 末尾添加：

```rust
#[test]
fn test_micro_compact_replaces_old_tool_results() {
    use runtime_lib::agent::executor::micro_compact;

    let messages = vec![
        json!({"role": "user", "content": "start"}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "1", "content": "long output 1 ".repeat(100)}]}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "2", "content": "long output 2 ".repeat(100)}]}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "3", "content": "long output 3"}]}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "4", "content": "long output 4"}]}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "5", "content": "recent output"}]}),
        json!({"role": "assistant", "content": "done"}),
    ];

    let result = micro_compact(&messages, 3);
    // 最近 3 条 tool_result（id 3,4,5）应保留完整内容
    // 旧的（id 1,2）应被替换
    let r1 = serde_json::to_string(&result[1]).unwrap();
    assert!(r1.contains("[已执行]"));
    let r2 = serde_json::to_string(&result[2]).unwrap();
    assert!(r2.contains("[已执行]"));
    // 最近的应保留
    let r5 = serde_json::to_string(&result[5]).unwrap();
    assert!(r5.contains("recent output"));
}

#[test]
fn test_micro_compact_few_messages_no_change() {
    use runtime_lib::agent::executor::micro_compact;

    let messages = vec![
        json!({"role": "user", "content": "hello"}),
        json!({"role": "assistant", "content": "hi"}),
    ];
    let result = micro_compact(&messages, 3);
    assert_eq!(result.len(), 2);
}
```

**Step 2: 运行测试验证失败**

```bash
cd apps/runtime/src-tauri && cargo test --test test_context_trimming -- test_micro_compact 2>&1
```

Expected: FAIL（函数不存在）

**Step 3: 实现 micro_compact**

在 `executor.rs` 的 `trim_messages` 函数之后添加：

```rust
/// Layer 1 微压缩：替换旧的 tool_result 内容为占位符
///
/// 保留最近 `keep_recent` 条 tool_result 的完整内容，
/// 将更早的替换为 "[已执行]" 占位符。
pub fn micro_compact(messages: &[Value], keep_recent: usize) -> Vec<Value> {
    // 找出所有包含 tool_result 的消息索引
    let tool_result_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            // Anthropic: content 是数组且包含 tool_result
            m["content"].as_array().map_or(false, |arr| {
                arr.iter().any(|v| v["type"].as_str() == Some("tool_result"))
            })
            // OpenAI: role == "tool"
            || m["role"].as_str() == Some("tool")
        })
        .map(|(i, _)| i)
        .collect();

    if tool_result_indices.len() <= keep_recent {
        return messages.to_vec();
    }

    let cutoff = tool_result_indices.len() - keep_recent;
    let old_indices: std::collections::HashSet<usize> =
        tool_result_indices[..cutoff].iter().copied().collect();

    messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if old_indices.contains(&i) {
                // 替换为占位符
                if m["role"].as_str() == Some("tool") {
                    // OpenAI 格式
                    json!({
                        "role": "tool",
                        "tool_call_id": m["tool_call_id"],
                        "content": "[已执行]"
                    })
                } else {
                    // Anthropic 格式
                    let replaced = m["content"].as_array().map(|arr| {
                        arr.iter().map(|v| {
                            if v["type"].as_str() == Some("tool_result") {
                                json!({
                                    "type": "tool_result",
                                    "tool_use_id": v["tool_use_id"],
                                    "content": "[已执行]"
                                })
                            } else {
                                v.clone()
                            }
                        }).collect::<Vec<_>>()
                    });
                    match replaced {
                        Some(arr) => json!({"role": "user", "content": arr}),
                        None => m.clone(),
                    }
                }
            } else {
                m.clone()
            }
        })
        .collect()
}
```

**Step 4: 在 execute_turn 中使用**

在 `let trimmed = trim_messages(...)` 之前添加：

```rust
let compacted = micro_compact(&messages, 3);
let trimmed = trim_messages(&compacted, DEFAULT_TOKEN_BUDGET);
```

删除原来的 `let trimmed = trim_messages(&messages, DEFAULT_TOKEN_BUDGET);` 行。

**Step 5: 运行测试**

```bash
cd apps/runtime/src-tauri && cargo test --test test_context_trimming 2>&1
```

Expected: 全部通过

**Step 6: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/tests/test_context_trimming.rs
git commit -m "feat(agent): Layer 1 微压缩 — 替换旧 tool_result 为占位符"
```

---

### Task 10: Layer 2 — 自动压缩（LLM 摘要）

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/compactor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: 创建 compactor.rs**

```rust
use crate::adapters;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;

const AUTO_COMPACT_THRESHOLD: usize = 50_000; // token 阈值

const COMPACT_SYSTEM_PROMPT: &str = "你是一个对话总结助手。请准确、结构化地总结对话内容。";

const COMPACT_USER_PROMPT: &str = r#"请总结以下对话，确保连续性。输出以下章节（每章节用 ## 标题）：

## 用户请求与意图
所有明确的用户请求

## 关键技术上下文
涉及的技术栈、框架、架构

## 已修改文件
文件路径和修改内容（含代码片段）

## 错误与修复
遇到的错误及解决方式

## 待办任务
已请求但未完成的任务

## 当前工作状态
压缩前正在进行的工作

## 下一步
建议的下一个操作

---

对话内容：
"#;

/// 检查是否需要自动压缩
pub fn needs_auto_compact(estimated_tokens: usize) -> bool {
    estimated_tokens > AUTO_COMPACT_THRESHOLD
}

/// 保存完整对话记录到磁盘
pub fn save_transcript(
    transcript_dir: &PathBuf,
    session_id: &str,
    messages: &[Value],
) -> Result<PathBuf> {
    std::fs::create_dir_all(transcript_dir)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.jsonl", session_id, timestamp);
    let path = transcript_dir.join(&filename);

    let content: String = messages
        .iter()
        .map(|m| serde_json::to_string(m).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&path, content)?;
    Ok(path)
}

/// 调用 LLM 生成对话摘要，替换 messages
pub async fn auto_compact(
    api_format: &str,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[Value],
    transcript_path: &str,
) -> Result<Vec<Value>> {
    // 将所有消息序列化为文本
    let conversation_text: String = messages
        .iter()
        .map(|m| {
            let role = m["role"].as_str().unwrap_or("unknown");
            let content = m["content"].as_str().unwrap_or("");
            format!("[{}]: {}", role, content)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = format!("{}\n{}", COMPACT_USER_PROMPT, conversation_text);
    let summary_messages = vec![json!({"role": "user", "content": user_prompt})];

    // 调用 LLM 生成摘要（不使用工具、不流式输出）
    let response = if api_format == "anthropic" {
        adapters::anthropic::chat_stream_with_tools(
            base_url, api_key, model,
            COMPACT_SYSTEM_PROMPT,
            summary_messages, vec![],
            |_| {},
        ).await?
    } else {
        adapters::openai::chat_stream_with_tools(
            base_url, api_key, model,
            COMPACT_SYSTEM_PROMPT,
            summary_messages, vec![],
            |_| {},
        ).await?
    };

    let summary = match response {
        super::types::LLMResponse::Text(text) => text,
        _ => "摘要生成失败".to_string(),
    };

    // 用摘要替换整个消息列表
    Ok(vec![
        json!({
            "role": "user",
            "content": format!(
                "[对话已压缩。完整记录: {}]\n\n{}",
                transcript_path, summary
            )
        }),
        json!({
            "role": "assistant",
            "content": "已了解之前的对话上下文，准备继续工作。"
        }),
    ])
}
```

**Step 2: 在 agent/mod.rs 中添加模块**

```rust
pub mod compactor;
```

**Step 3: 在 execute_turn 中集成**

在循环开头（iteration check 之前）添加自动压缩检查：

```rust
// 自动压缩检查（仅在第二轮及之后）
if iteration > 1 {
    let tokens = estimate_tokens(&messages);
    if super::compactor::needs_auto_compact(tokens) {
        eprintln!("[agent] Token 数 {} 超过阈值，触发自动压缩", tokens);
        if let (Some(app), Some(sid)) = (app_handle, session_id) {
            let transcript_dir = app.path().app_data_dir()
                .unwrap_or_default()
                .join("transcripts");
            if let Ok(path) = super::compactor::save_transcript(&transcript_dir, sid, &messages) {
                let path_str = path.to_string_lossy().to_string();
                match super::compactor::auto_compact(
                    api_format, base_url, api_key, model,
                    &messages, &path_str,
                ).await {
                    Ok(compacted) => {
                        eprintln!("[agent] 自动压缩完成，消息数 {} → {}", messages.len(), compacted.len());
                        messages = compacted;
                    }
                    Err(e) => eprintln!("[agent] 自动压缩失败: {}", e),
                }
            }
        }
    }
}
```

**Step 4: 编译验证**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
```

**Step 5: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/compactor.rs apps/runtime/src-tauri/src/agent/mod.rs apps/runtime/src-tauri/src/agent/executor.rs
git commit -m "feat(agent): Layer 2 自动压缩 — LLM 摘要替换旧消息"
```

---

### Task 11: Layer 3 — compact 工具

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/tools/compact_tool.rs`
- Modify: `apps/runtime/src-tauri/src/agent/tools/mod.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`

**Step 1: 创建 compact_tool.rs**

```rust
use crate::agent::types::Tool;
use anyhow::Result;
use serde_json::{json, Value};

/// 手动上下文压缩工具
///
/// Agent 可主动调用此工具触发上下文压缩。
/// 实际压缩由 executor 在下一轮迭代检测标志后执行。
pub struct CompactTool;

impl CompactTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for CompactTool {
    fn name(&self) -> &str {
        "compact"
    }

    fn description(&self) -> &str {
        "手动触发对话上下文压缩。当对话过长时使用此工具来压缩历史消息，保留关键信息。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "focus": {
                    "type": "string",
                    "description": "压缩时的重点关注方向（可选，如 '重点保留 TypeScript 相关变更'）"
                }
            }
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let focus = input["focus"].as_str().unwrap_or("");
        if focus.is_empty() {
            Ok("已请求上下文压缩。将在下一轮迭代执行。".to_string())
        } else {
            Ok(format!("已请求上下文压缩（重点: {}）。将在下一轮迭代执行。", focus))
        }
    }
}
```

**Step 2: 在 mod.rs 注册**

添加 `mod compact_tool;` 和 `pub use compact_tool::CompactTool;`

**Step 3: 在 chat.rs 的 send_message 中注册**

```rust
let compact_tool = CompactTool::new();
agent_executor.registry().register(Arc::new(compact_tool));
```

**Step 4: 编译验证**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
```

**Step 5: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/tools/compact_tool.rs apps/runtime/src-tauri/src/agent/tools/mod.rs apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "feat(agent): Layer 3 compact 工具 — Agent 手动触发压缩"
```

---

## Checkpoint 3

```bash
cd apps/runtime/src-tauri && cargo test 2>&1
cd apps/runtime && npx tsc --noEmit 2>&1
```

Expected: 全部通过

---

## Phase 5: 子 Agent 流式输出

### Task 12: TaskTool 接收 AppHandle 并转发 stream-token

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/task_tool.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`（StreamToken 添加 sub_agent 字段）
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`（传入 AppHandle）

**Step 1: 扩展 StreamToken**

在 `commands/chat.rs` 中，StreamToken 结构体添加字段：

```rust
#[derive(serde::Serialize, Clone)]
struct StreamToken {
    session_id: String,
    token: String,
    done: bool,
    #[serde(default)]
    sub_agent: bool,
}
```

更新所有现有 StreamToken 构造处添加 `sub_agent: false`。

**Step 2: 扩展 TaskTool 构造函数**

```rust
pub struct TaskTool {
    registry: Arc<ToolRegistry>,
    api_format: String,
    base_url: String,
    api_key: String,
    model: String,
    app_handle: Option<tauri::AppHandle>,
    session_id: Option<String>,
}

impl TaskTool {
    pub fn new(
        registry: Arc<ToolRegistry>,
        api_format: String,
        base_url: String,
        api_key: String,
        model: String,
    ) -> Self {
        Self {
            registry, api_format, base_url, api_key, model,
            app_handle: None,
            session_id: None,
        }
    }

    pub fn with_app_handle(mut self, app: tauri::AppHandle, session_id: String) -> Self {
        self.app_handle = Some(app);
        self.session_id = Some(session_id);
        self
    }
}
```

**Step 3: 在 execute 中转发 token**

替换 `|_| {}` 为：

```rust
let on_token = match (&self.app_handle, &self.session_id) {
    (Some(app), Some(sid)) => {
        let app = app.clone();
        let sid = sid.clone();
        Box::new(move |token: String| {
            let _ = app.emit("stream-token", serde_json::json!({
                "session_id": sid,
                "token": token,
                "done": false,
                "sub_agent": true,
            }));
        }) as Box<dyn Fn(String) + Send>
    }
    _ => Box::new(|_| {}) as Box<dyn Fn(String) + Send>,
};
```

同时传入 `app_handle.as_ref()` 和 `session_id.as_deref()` 给子 executor（用于 tool-call-event）。

**Step 4: 在 chat.rs 中构造 TaskTool 时调用 with_app_handle**

```rust
let task_tool = TaskTool::new(...)
    .with_app_handle(app.clone(), session_id.clone());
```

**Step 5: 编译验证 + 测试**

```bash
cd apps/runtime/src-tauri && cargo build 2>&1
cd apps/runtime/src-tauri && cargo test 2>&1
```

**Step 6: 提交**

```bash
git add apps/runtime/src-tauri/src/agent/tools/task_tool.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/executor.rs
git commit -m "feat(agent): 子 Agent 流式输出 — TaskTool 转发 stream-token"
```

---

### Task 13: 前端区分子 Agent 流式输出

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: 扩展 stream-token 事件监听**

在 stream-token 事件的类型中添加 `sub_agent?: boolean`。

添加状态：

```tsx
const [subAgentBuffer, setSubAgentBuffer] = useState("");
const subAgentBufferRef = useRef("");
```

在 stream-token 处理中区分主/子 Agent：

```tsx
if (payload.sub_agent) {
    subAgentBufferRef.current += payload.token;
    setSubAgentBuffer(subAgentBufferRef.current);
} else {
    // 现有主 Agent 逻辑
}
```

在 `done` 事件中重置子 Agent buffer：

```tsx
subAgentBufferRef.current = "";
setSubAgentBuffer("");
```

**Step 2: 在 ToolCallCard 中显示子 Agent 流式文本**

在流式区域的 ToolCallCard 组件中，对 task 类型的工具卡片传入 `subAgentBuffer` prop。在卡片内部显示实时流式文本。

**Step 3: TypeScript 编译验证**

```bash
cd apps/runtime && npx tsc --noEmit 2>&1
```

**Step 4: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/ToolCallCard.tsx
git commit -m "feat(ui): 子 Agent 流式输出展示"
```

---

## Final Checkpoint

```bash
cd apps/runtime/src-tauri && cargo test 2>&1
cd apps/runtime && npx tsc --noEmit 2>&1
```

Expected: 全部通过，0 错误

---

## 任务汇总

| # | 任务 | Phase | 依赖 |
|---|------|-------|------|
| 1 | Bash 危险命令黑名单 | 1 | 无 |
| 2 | Bash 超时控制 | 1 | Task 1 |
| 3 | AgentState 事件发射 | 2 | 无 |
| 4 | 前端 AgentState 状态条 | 2 | Task 3 |
| 5 | DB sessions 添加 permission_mode | 3 | 无 |
| 6 | ToolConfirmResponder + command | 3 | Task 5 |
| 7 | execute_turn 权限检查集成 | 3 | Task 6 |
| 8 | 工具确认前端 UI | 3 | Task 7 |
| 9 | Layer 1 微压缩 | 4 | 无 |
| 10 | Layer 2 自动压缩 | 4 | Task 9 |
| 11 | Layer 3 compact 工具 | 4 | Task 10 |
| 12 | TaskTool 子 Agent 流式输出 | 5 | 无 |
| 13 | 前端子 Agent 流式展示 | 5 | Task 12 |
