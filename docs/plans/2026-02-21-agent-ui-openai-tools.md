# Agent UI + OpenAI Tool Calling 实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 OpenAI function calling、移除 enable_tools 开关（默认 Agent 模式）、前端添加工具调用可折叠卡片 UI。

**Architecture:** 后端 AgentExecutor 始终走 Agent 模式，根据 api_format 调用对应的 `chat_stream_with_tools`。工具调用过程通过 Tauri `tool-call-event` 事件推送到前端。前端用可折叠卡片渲染工具调用。

**Tech Stack:** Rust (Tauri, reqwest, serde_json), TypeScript (React 18, Tailwind CSS), Tauri Events

---

## Phase 1: OpenAI Tool Calling 后端 (Tasks 1-3)

### Task 1: Implement OpenAI `chat_stream_with_tools`

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs`
- Create: `apps/runtime/src-tauri/tests/test_openai_tools.rs`

**Step 1: Write the failing test**

```rust
// apps/runtime/src-tauri/tests/test_openai_tools.rs
use runtime_lib::agent::{AgentExecutor, ToolRegistry};
use serde_json::json;
use std::sync::Arc;

/// 验证 execute_turn 对 OpenAI 格式不再报 "not yet implemented"
/// （而是返回网络错误，因为 URL 无效）
#[tokio::test]
async fn test_openai_tool_calling_network_error() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "hello"})];

    let result = executor
        .execute_turn(
            "openai",
            "http://invalid-openai-mock-url",
            "mock-key",
            "gpt-4",
            "You are a helpful assistant.",
            messages,
            |_token| {},
        )
        .await;

    // 应返回网络错误（不是 "not yet implemented"）
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        !err_msg.contains("not yet implemented"),
        "OpenAI tool calling 应该已实现，但得到: {}",
        err_msg
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_openai_tools -- -v`
Expected: FAIL — 当前 `execute_turn` 对 openai 格式返回 `"OpenAI tool calling not yet implemented"`

**Step 3: Write implementation**

在 `apps/runtime/src-tauri/src/adapters/openai.rs` 末尾（`test_connection` 函数之前）添加：

```rust
/// OpenAI function calling 流式响应解析
/// 支持 tool_calls delta 增量拼接
pub async fn chat_stream_with_tools(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<Value>,
    tools: Vec<Value>,
    mut on_token: impl FnMut(String) + Send,
) -> Result<crate::agent::types::LLMResponse> {
    use crate::agent::types::{LLMResponse, ToolCall};

    let client = Client::new();
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // 将 Anthropic 格式的工具定义转换为 OpenAI 格式
    let openai_tools: Vec<Value> = tools
        .iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t["name"],
                    "description": t["description"],
                    "parameters": t["input_schema"],
                }
            })
        })
        .collect();

    let mut all_messages = vec![json!({"role": "system", "content": system_prompt})];
    all_messages.extend(messages);

    let body = json!({
        "model": model,
        "messages": all_messages,
        "tools": openai_tools,
        "stream": true,
    });

    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(anyhow!("OpenAI API error: {}", text));
    }

    let mut stream = resp.bytes_stream();
    let mut text_content = String::new();
    let mut in_think = false;

    // OpenAI tool_calls 增量解析状态
    // key: tool call index, value: (id, name, arguments_buffer)
    let mut tool_call_map: std::collections::HashMap<u64, (String, String, String)> =
        std::collections::HashMap::new();
    let mut has_tool_calls = false;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    break;
                }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let delta = &v["choices"][0]["delta"];
                    let finish_reason = v["choices"][0]["finish_reason"].as_str();

                    // 跳过 DeepSeek reasoning_content
                    if delta["reasoning_content"]
                        .as_str()
                        .map(|s| !s.is_empty())
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    // 处理文本 content
                    if let Some(token) = delta["content"].as_str() {
                        let filtered = filter_thinking(token, &mut in_think);
                        if !filtered.is_empty() {
                            text_content.push_str(&filtered);
                            on_token(filtered);
                        }
                    }

                    // 处理 tool_calls delta
                    if let Some(tool_calls_arr) = delta["tool_calls"].as_array() {
                        has_tool_calls = true;
                        for tc in tool_calls_arr {
                            let index = tc["index"].as_u64().unwrap_or(0);
                            let entry = tool_call_map
                                .entry(index)
                                .or_insert_with(|| (String::new(), String::new(), String::new()));

                            // id 只在第一个 delta 出现
                            if let Some(id) = tc["id"].as_str() {
                                entry.0 = id.to_string();
                            }
                            // function.name 只在第一个 delta 出现
                            if let Some(name) = tc["function"]["name"].as_str() {
                                entry.1 = name.to_string();
                            }
                            // function.arguments 增量拼接
                            if let Some(args) = tc["function"]["arguments"].as_str() {
                                entry.2.push_str(args);
                            }
                        }
                    }

                    // finish_reason = "tool_calls" 表示需要执行工具
                    if finish_reason == Some("tool_calls") {
                        has_tool_calls = true;
                    }
                }
            }
        }
    }

    if has_tool_calls && !tool_call_map.is_empty() {
        let mut tool_calls: Vec<ToolCall> = tool_call_map
            .into_iter()
            .map(|(_index, (id, name, args))| ToolCall {
                id,
                name,
                input: serde_json::from_str(&args).unwrap_or(json!({})),
            })
            .collect();
        // 按 id 排序保持稳定顺序
        tool_calls.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(LLMResponse::ToolCalls(tool_calls))
    } else {
        Ok(LLMResponse::Text(text_content))
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_openai_tools -- -v`
Expected: PASS — 网络错误（不是 "not yet implemented"）

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/adapters/openai.rs apps/runtime/src-tauri/tests/test_openai_tools.rs
git commit -m "feat(agent): 实现 OpenAI function calling 流式 tool_calls 解析"
```

---

### Task 2: Update AgentExecutor to support OpenAI format

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/tests/test_react_loop.rs`

**Step 1: Write the failing test**

在 `tests/test_react_loop.rs` 末尾添加：

```rust
#[tokio::test]
async fn test_react_loop_openai_format_network_error() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "hello"})];

    let result = executor
        .execute_turn(
            "openai",
            "http://invalid-openai-url",
            "mock-key",
            "gpt-4",
            "You are a helpful assistant.",
            messages,
            |_token| {},
        )
        .await;

    // OpenAI 格式应返回网络错误（不是 "not yet implemented"）
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(!err_msg.contains("not yet implemented"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_react_loop test_react_loop_openai_format -- -v`
Expected: FAIL — 当前返回 "OpenAI tool calling not yet implemented"

**Step 3: Update executor.rs**

替换 `execute_turn` 中的 `else` 分支（第 63-66 行）：

```rust
            } else {
                // OpenAI 兼容格式
                adapters::openai::chat_stream_with_tools(
                    base_url,
                    api_key,
                    model,
                    system_prompt,
                    messages.clone(),
                    tools,
                    on_token.clone(),
                )
                .await?
            };
```

在 `ToolCalls` 分支中（第 101-123 行），添加 OpenAI 格式的消息构造：

```rust
                    // 添加工具调用和结果到消息历史
                    if api_format == "anthropic" {
                        // Anthropic 格式（保持不变）
                        messages.push(json!({
                            "role": "assistant",
                            "content": tool_calls.iter().map(|tc| json!({
                                "type": "tool_use",
                                "id": tc.id,
                                "name": tc.name,
                                "input": tc.input,
                            })).collect::<Vec<_>>()
                        }));
                        messages.push(json!({
                            "role": "user",
                            "content": tool_results.iter().map(|tr| json!({
                                "type": "tool_result",
                                "tool_use_id": tr.tool_use_id,
                                "content": tr.content,
                            })).collect::<Vec<_>>()
                        }));
                    } else {
                        // OpenAI 格式
                        messages.push(json!({
                            "role": "assistant",
                            "tool_calls": tool_calls.iter().map(|tc| json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": serde_json::to_string(&tc.input).unwrap_or_default(),
                                }
                            })).collect::<Vec<_>>()
                        }));
                        // OpenAI: 每个工具结果是独立的 "tool" 角色消息
                        for tr in &tool_results {
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": tr.tool_use_id,
                                "content": tr.content,
                            }));
                        }
                    }
```

**Step 4: Run tests**

Run: `cd apps/runtime/src-tauri && cargo test --test test_react_loop -- -v`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/tests/test_react_loop.rs
git commit -m "feat(agent): AgentExecutor 支持 OpenAI 格式的工具调用消息"
```

---

### Task 3: Simplify send_message — remove enable_tools, always Agent

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`

**Step 1: Rewrite send_message**

移除 `enable_tools: bool` 参数，移除 `else` 分支（普通聊天模式），始终走 `agent_executor.execute_turn()`。

完整替换 `send_message` 函数：

```rust
#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    session_id: String,
    user_message: String,
    db: State<'_, DbState>,
    agent_executor: State<'_, Arc<AgentExecutor>>,
) -> Result<(), String> {
    // 保存用户消息
    let msg_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&msg_id)
    .bind(&session_id)
    .bind("user")
    .bind(&user_message)
    .bind(&now)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // 加载会话信息
    let (skill_id, model_id) = sqlx::query_as::<_, (String, String)>(
        "SELECT skill_id, model_id FROM sessions WHERE id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("会话不存在 (session_id={session_id}): {e}"))?;

    // 加载 Skill 信息
    let (manifest_json, username, pack_path) = sqlx::query_as::<_, (String, String, String)>(
        "SELECT manifest, username, pack_path FROM installed_skills WHERE id = ?"
    )
    .bind(&skill_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("Skill 不存在 (skill_id={skill_id}): {e}"))?;

    // 获取 system prompt
    let system_prompt = match skillpack_rs::verify_and_unpack(&pack_path, &username) {
        Ok(unpacked) => {
            String::from_utf8_lossy(
                unpacked.files.get("SKILL.md").map(|v| v.as_slice()).unwrap_or_default()
            ).to_string()
        }
        Err(_) => {
            let manifest: skillpack_rs::SkillManifest = serde_json::from_str(&manifest_json)
                .map_err(|e| e.to_string())?;
            manifest.description
        }
    };

    // 加载消息历史
    let history = sqlx::query_as::<_, (String, String)>(
        "SELECT role, content FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(&session_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    let messages: Vec<Value> = history.iter()
        .map(|(role, content)| json!({"role": role, "content": content}))
        .collect();

    // 加载模型配置
    let (api_format, base_url, model_name, api_key) = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT api_format, base_url, model_name, api_key FROM model_configs WHERE id = ?"
    )
    .bind(&model_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("模型配置不存在 (model_id={model_id}): {e}"))?;

    if api_key.is_empty() {
        return Err(format!("模型 API Key 为空，请在设置中重新配置 (model_id={model_id})"));
    }

    // 始终走 Agent 模式
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let final_messages = agent_executor
        .execute_turn(
            &api_format,
            &base_url,
            &api_key,
            &model_name,
            &system_prompt,
            messages,
            move |token: String| {
                let _ = app_clone.emit("stream-token", StreamToken {
                    session_id: session_id_clone.clone(),
                    token,
                    done: false,
                });
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    // 发送结束事件
    let _ = app.emit("stream-token", StreamToken {
        session_id: session_id.clone(),
        token: String::new(),
        done: true,
    });

    // 保存所有新消息到数据库
    for msg in final_messages.iter().skip(history.len()) {
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let role = msg["role"].as_str().unwrap_or("assistant");
        let content = serde_json::to_string(&msg["content"]).unwrap_or_default();

        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&msg_id)
        .bind(&session_id)
        .bind(role)
        .bind(&content)
        .bind(&now)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

**Step 2: Remove unused imports**

`commands/chat.rs` 顶部：移除 `use crate::adapters;`（不再直接使用适配器）。

**Step 3: Verify compilation**

Run: `cd apps/runtime/src-tauri && cargo check`
Expected: SUCCESS

**Step 4: Run all tests**

Run: `cd apps/runtime/src-tauri && cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "refactor(chat): 移除 enable_tools 参数，始终走 Agent 模式"
```

---

## Checkpoint 1: 后端完成

在继续前端之前，验证：
1. `cargo test` 全部通过
2. `cargo check` 编译无警告
3. OpenAI 和 Anthropic 格式都不再返回 "not yet implemented"

---

## Phase 2: 工具调用事件 (Task 4)

### Task 4: Add tool-call-event emission from AgentExecutor

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`

**Step 1: Add ToolCallEvent struct to executor.rs**

在 `executor.rs` 顶部添加：

```rust
use tauri::{AppHandle, Emitter};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ToolCallEvent {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_output: Option<String>,
    pub status: String, // "started" | "completed" | "error"
}
```

**Step 2: Update execute_turn signature**

添加 `app_handle` 和 `session_id` 参数：

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
    ) -> Result<Vec<Value>> {
```

**Step 3: Emit events during tool execution**

在工具执行循环中，调用前后各 emit 一次：

```rust
                    for call in &tool_calls {
                        eprintln!("[agent] Calling tool: {}", call.name);

                        // 发送工具开始事件
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit("tool-call-event", ToolCallEvent {
                                session_id: sid.to_string(),
                                tool_name: call.name.clone(),
                                tool_input: call.input.clone(),
                                tool_output: None,
                                status: "started".to_string(),
                            });
                        }

                        let result = match self.registry.get(&call.name) {
                            Some(tool) => match tool.execute(call.input.clone()) {
                                Ok(output) => output,
                                Err(e) => format!("工具执行错误: {}", e),
                            },
                            None => format!("工具不存在: {}", call.name),
                        };

                        // 发送工具完成事件
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit("tool-call-event", ToolCallEvent {
                                session_id: sid.to_string(),
                                tool_name: call.name.clone(),
                                tool_input: call.input.clone(),
                                tool_output: Some(result.clone()),
                                status: "completed".to_string(),
                            });
                        }

                        tool_results.push(ToolResult {
                            tool_use_id: call.id.clone(),
                            content: result,
                        });
                    }
```

**Step 4: Update call sites**

在 `commands/chat.rs` 中，更新 `execute_turn` 调用：

```rust
    let final_messages = agent_executor
        .execute_turn(
            &api_format,
            &base_url,
            &api_key,
            &model_name,
            &system_prompt,
            messages,
            move |token: String| {
                let _ = app_clone.emit("stream-token", StreamToken {
                    session_id: session_id_clone.clone(),
                    token,
                    done: false,
                });
            },
            Some(&app),
            Some(&session_id),
        )
        .await
        .map_err(|e| e.to_string())?;
```

**Step 5: Update tests — pass None for app_handle/session_id**

在所有测试文件中，`execute_turn` 调用末尾添加 `None, None`：

- `tests/test_react_loop.rs` — 3 处调用
- `tests/test_anthropic_tools.rs` — 2 处调用
- `tests/test_openai_tools.rs` — 1 处调用

示例：
```rust
        .execute_turn(
            "anthropic",
            "http://mock-url",
            "mock-key",
            "mock-model",
            "You are a helpful assistant.",
            messages,
            |_token| {},
            None,
            None,
        )
```

**Step 6: Verify compilation and tests**

Run: `cd apps/runtime/src-tauri && cargo check && cargo test`
Expected: All pass

**Step 7: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/tests/
git commit -m "feat(agent): 工具执行时 emit tool-call-event 到前端"
```

---

## Checkpoint 2: 后端事件完成

验证 `cargo test` 全部通过。

---

## Phase 3: 前端 Agent UI (Tasks 5-7)

### Task 5: Update types.ts and fix send_message call

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: Add ToolCallInfo to types.ts**

```typescript
// apps/runtime/src/types.ts — 末尾添加

export interface ToolCallInfo {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output?: string;
  status: "running" | "completed" | "error";
}
```

**Step 2: Update Message interface**

```typescript
export interface Message {
  role: "user" | "assistant";
  content: string;
  created_at: string;
  toolCalls?: ToolCallInfo[];  // 新增
}
```

**Step 3: Fix send_message invoke call**

在 `ChatView.tsx` 第 87 行，移除 `enableTools` 参数（如果有的话），确保只传 `sessionId` 和 `userMessage`：

```typescript
await invoke("send_message", { sessionId, userMessage: msg });
```

**Step 4: Verify**

Run: `cd apps/runtime && pnpm build` 或 `pnpm dev` 确认前端编译通过。

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): 添加 ToolCallInfo 类型，修复 send_message 调用"
```

---

### Task 6: Create ToolCallCard component

**Files:**
- Create: `apps/runtime/src/components/ToolCallCard.tsx`

**Step 1: Create the component**

```tsx
// apps/runtime/src/components/ToolCallCard.tsx
import { useState } from "react";
import { ToolCallInfo } from "../types";

const TOOL_ICONS: Record<string, string> = {
  read_file: "\u{1F4C2}",
  write_file: "\u{1F4DD}",
  glob: "\u{1F50D}",
  grep: "\u{1F50E}",
  bash: "\u{1F4BB}",
  sidecar_bridge: "\u{1F310}",
};

interface Props {
  toolCall: ToolCallInfo;
}

export function ToolCallCard({ toolCall }: Props) {
  const [expanded, setExpanded] = useState(false);
  const icon = TOOL_ICONS[toolCall.name] || "\u{1F527}";

  const statusLabel =
    toolCall.status === "running" ? (
      <span className="text-blue-400 text-xs animate-pulse">执行中...</span>
    ) : toolCall.status === "completed" ? (
      <span className="text-green-400 text-xs">完成</span>
    ) : (
      <span className="text-red-400 text-xs">错误</span>
    );

  // 格式化输入参数为简短摘要
  const inputSummary = Object.entries(toolCall.input)
    .map(([k, v]) => `${k}: ${typeof v === "string" ? v : JSON.stringify(v)}`)
    .join(", ");
  const shortSummary = inputSummary.length > 60 ? inputSummary.slice(0, 60) + "..." : inputSummary;

  return (
    <div className="my-1 border border-slate-600 rounded-md overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-1.5 text-xs bg-slate-800 hover:bg-slate-750 transition-colors text-left"
      >
        <span>{icon}</span>
        <span className="font-medium text-slate-200">{toolCall.name}</span>
        <span className="text-slate-400 truncate flex-1">{shortSummary}</span>
        {statusLabel}
        <span className="text-slate-500">{expanded ? "\u25BC" : "\u25B6"}</span>
      </button>
      {expanded && (
        <div className="px-3 py-2 bg-slate-900 text-xs space-y-2">
          <div>
            <div className="text-slate-400 mb-1">参数:</div>
            <pre className="bg-slate-950 rounded p-2 overflow-x-auto text-slate-300">
              {JSON.stringify(toolCall.input, null, 2)}
            </pre>
          </div>
          {toolCall.output && (
            <div>
              <div className="text-slate-400 mb-1">结果:</div>
              <pre className="bg-slate-950 rounded p-2 overflow-x-auto text-slate-300 max-h-40 overflow-y-auto">
                {toolCall.output}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

**Step 2: Verify compilation**

Run: `cd apps/runtime && pnpm build`
Expected: SUCCESS (组件未被引用，但应编译通过)

**Step 3: Commit**

```bash
git add apps/runtime/src/components/ToolCallCard.tsx
git commit -m "feat(ui): 创建 ToolCallCard 可折叠工具调用卡片组件"
```

---

### Task 7: Integrate tool-call-event listener and ToolCallCard into ChatView

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: Add imports**

在 `ChatView.tsx` 顶部添加：
```typescript
import { ToolCallInfo } from "../types";
import { ToolCallCard } from "./ToolCallCard";
```

**Step 2: Add state for current tool calls**

在 `ChatView` 组件内部，现有 state 声明之后添加：
```typescript
const [currentToolCalls, setCurrentToolCalls] = useState<ToolCallInfo[]>([]);
```

**Step 3: Add tool-call-event listener**

在 `sessionId` 的 `useEffect` 中（监听 `stream-token` 的同一个 `useEffect`），添加第二个 listener。或者新建一个独立的 `useEffect`：

```typescript
useEffect(() => {
  const unlistenPromise = listen<{
    session_id: string;
    tool_name: string;
    tool_input: Record<string, unknown>;
    tool_output: string | null;
    status: string;
  }>("tool-call-event", ({ payload }) => {
    if (payload.session_id !== sessionId) return;
    if (payload.status === "started") {
      setCurrentToolCalls((prev) => [
        ...prev,
        {
          id: `${payload.tool_name}-${Date.now()}`,
          name: payload.tool_name,
          input: payload.tool_input,
          status: "running",
        },
      ]);
    } else {
      setCurrentToolCalls((prev) =>
        prev.map((tc) =>
          tc.name === payload.tool_name && tc.status === "running"
            ? {
                ...tc,
                output: payload.tool_output ?? undefined,
                status: (payload.status === "completed" ? "completed" : "error") as "completed" | "error",
              }
            : tc
        )
      );
    }
  });
  return () => {
    unlistenPromise.then((fn) => fn());
  };
}, [sessionId]);
```

**Step 4: Update stream-token done handler**

在现有 `stream-token` 监听器的 `done` 分支中，将 `currentToolCalls` 合并到消息中：

```typescript
if (payload.done) {
  const finalContent = streamBufferRef.current;
  setMessages((prev) => [
    ...prev,
    {
      role: "assistant" as const,
      content: finalContent,
      created_at: new Date().toISOString(),
      toolCalls: currentToolCalls.length > 0 ? [...currentToolCalls] : undefined,
    },
  ]);
  setCurrentToolCalls([]);
  streamBufferRef.current = "";
  setStreamBuffer("");
  setStreaming(false);
}
```

注意：`currentToolCalls` 在闭包中捕获。需要使用 ref 或函数式更新来获取最新值。改用 ref：

```typescript
const currentToolCallsRef = useRef<ToolCallInfo[]>([]);
```

并在 `tool-call-event` 监听器中同步更新：
```typescript
setCurrentToolCalls((prev) => {
  const next = [...prev, newToolCall];
  currentToolCallsRef.current = next;
  return next;
});
```

在 `stream-token` done 分支中使用 ref：
```typescript
toolCalls: currentToolCallsRef.current.length > 0 ? [...currentToolCallsRef.current] : undefined,
```

在重置时也清空 ref：
```typescript
currentToolCallsRef.current = [];
setCurrentToolCalls([]);
```

**Step 5: Update message rendering**

替换消息渲染部分，支持 `toolCalls`：

```tsx
{messages.map((m, i) => (
  <div key={i} className={"flex " + (m.role === "user" ? "justify-end" : "justify-start")}>
    <div
      className={
        "max-w-2xl rounded-lg px-4 py-2 text-sm " +
        (m.role === "user"
          ? "bg-blue-600 text-white"
          : "bg-slate-700 text-slate-100")
      }
    >
      {m.role === "assistant" && m.toolCalls && (
        <div className="mb-2">
          {m.toolCalls.map((tc) => (
            <ToolCallCard key={tc.id} toolCall={tc} />
          ))}
        </div>
      )}
      {m.role === "assistant" ? (
        <ReactMarkdown>{m.content}</ReactMarkdown>
      ) : (
        m.content
      )}
    </div>
  </div>
))}
```

**Step 6: Update streaming area**

替换流式输出区域，显示当前工具调用和流式文本：

```tsx
{(currentToolCalls.length > 0 || streamBuffer) && (
  <div className="flex justify-start">
    <div className="max-w-2xl bg-slate-700 rounded-lg px-4 py-2 text-sm text-slate-100">
      {currentToolCalls.map((tc) => (
        <ToolCallCard key={tc.id} toolCall={tc} />
      ))}
      {streamBuffer && <ReactMarkdown>{streamBuffer}</ReactMarkdown>}
      <span className="animate-pulse">|</span>
    </div>
  </div>
)}
```

**Step 7: Verify compilation**

Run: `cd apps/runtime && pnpm build`
Expected: SUCCESS

**Step 8: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): 集成 tool-call-event 监听和 ToolCallCard 渲染"
```

---

## Checkpoint 3: 全部完成

验证清单：
1. `cd apps/runtime/src-tauri && cargo test` — 全部通过
2. `cd apps/runtime && pnpm build` — 前端编译成功
3. `cd apps/runtime/src-tauri && cargo check` — 无错误

运行 `pnpm runtime` 启动应用进行手动验证：
- 配置一个 Anthropic 模型 → 发送消息 → 验证 Agent 工具调用卡片显示
- 配置一个 OpenAI 兼容模型 → 发送消息 → 验证流式响应正常
- 没有工具调用时应退化为普通文本输出
