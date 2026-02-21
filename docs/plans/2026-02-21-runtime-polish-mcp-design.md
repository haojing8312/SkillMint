# Runtime 完善 + MCP 集成设计文档

**日期**: 2026-02-21
**状态**: 已批准
**范围**: Dead code 清理 + Tool call 持久化 + 会话历史管理 + MCP 协议集成

---

## 1. 背景与目标

### 1.1 当前状态

Agent UI + OpenAI Tool Calling 已完成（7 个任务）：
- OpenAI function calling 流式解析
- AgentExecutor 始终走 Agent 模式
- 前端 ToolCallCard 可折叠卡片

### 1.2 存在的问题

1. **Dead code**：`chat_stream` 函数在两个适配器中已无引用（2 个编译警告）
2. **消息持久化不完整**：后端保存了 tool call 消息但前端不加载不解析
3. **无会话历史**：`get_messages` 命令存在但前端从不调用，Sidebar 无会话列表
4. **MCP 未集成**：Sidecar MCPManager 完整但未注册到 ToolRegistry

### 1.3 目标

1. 清理未使用的代码，消除编译警告
2. 前端能加载历史消息，包含 tool call 信息
3. Sidebar 展示会话列表，支持切换和管理
4. MCP 服务器可配置，工具动态注册到 Agent

---

## 2. 功能 1：Dead Code 清理

### 2.1 要删除的函数

| 文件 | 函数 | 行数 | 原因 |
|------|------|------|------|
| `adapters/anthropic.rs` | `chat_stream()` (第 6-54 行) | ~49 行 | 被 `chat_stream_with_tools()` 替代 |
| `adapters/openai.rs` | `chat_stream()` (第 6-63 行) | ~58 行 | 被 `chat_stream_with_tools()` 替代 |

删除后 `cargo check` 应无警告。

---

## 3. 功能 2：Tool Call 持久化与消息格式统一

### 3.1 问题分析

`send_message` 中保存消息的逻辑（`chat.rs:152-169`）：
```rust
for msg in final_messages.iter().skip(history.len()) {
    let role = msg["role"].as_str().unwrap_or("assistant");
    let content = serde_json::to_string(&msg["content"]).unwrap_or_default();
    // INSERT INTO messages
}
```

这会保存所有中间消息，包括：
- `role: "assistant"` + content 为 tool_use 数组（Anthropic）或 tool_calls 字段（OpenAI）
- `role: "user"` + content 为 tool_result 数组（Anthropic）
- `role: "tool"` + content 为工具结果（OpenAI）
- `role: "assistant"` + content 为最终文本

### 3.2 `get_messages` 改造

**当前**：返回原始 `(role, content, created_at)`
**改造后**：返回结构化消息，解析 tool call 信息

```rust
#[tauri::command]
pub async fn get_messages(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT role, content, created_at FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(&session_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = vec![];
    for (role, content, created_at) in &rows {
        // 跳过中间工具消息（tool_result, tool 角色）
        if role == "tool" { continue; }

        // 尝试解析 content
        let parsed: Value = serde_json::from_str(content).unwrap_or(json!(content));

        if role == "assistant" {
            // 检查是否包含 tool_use（Anthropic 格式）
            if let Some(arr) = parsed.as_array() {
                if arr.iter().any(|v| v["type"] == "tool_use") {
                    // 这是中间工具调用消息，提取 tool calls
                    let tool_calls: Vec<Value> = arr.iter()
                        .filter(|v| v["type"] == "tool_use")
                        .map(|v| json!({
                            "id": v["id"],
                            "name": v["name"],
                            "input": v["input"],
                            "status": "completed",
                        }))
                        .collect();
                    // 不作为独立消息返回，而是附加到下一条 assistant 消息
                    // （或合并到最终的 assistant 文本消息）
                    continue; // 跳过，后续合并
                }
            }
            // 检查 OpenAI 格式的 tool_calls
            // ...类似处理
        }

        if role == "user" {
            // 检查是否为 tool_result（Anthropic 格式）
            if let Some(arr) = parsed.as_array() {
                if arr.iter().any(|v| v["type"] == "tool_result") {
                    continue; // 跳过
                }
            }
        }

        // 普通消息
        let content_str = if parsed.is_string() {
            parsed.as_str().unwrap_or("").to_string()
        } else {
            content.clone()
        };

        result.push(json!({
            "role": role,
            "content": content_str,
            "created_at": created_at,
        }));
    }
    Ok(result)
}
```

**简化方案**（推荐）：不在 `get_messages` 中做复杂解析，而是改造 `send_message` 的保存逻辑：
- 只保存用户消息和最终 assistant 文本消息
- tool call 信息作为 JSON 字段保存在最终 assistant 消息的 content 中

**保存格式**：
```json
{
  "text": "最终回复文本",
  "tool_calls": [
    {"name": "read_file", "input": {"path": "test.txt"}, "output": "文件内容...", "status": "completed"}
  ]
}
```

当 content 是纯文本时，直接是字符串；当包含 tool calls 时，是 JSON 对象。

### 3.3 `send_message` 保存逻辑改造

替换当前的 `for msg in final_messages.iter().skip(history.len())` 逻辑：

```rust
// 收集本轮的 tool call 信息（从 tool-call-event 中已有）
// 保存最终 assistant 消息
let final_text = /* 从 final_messages 中提取最后一条 assistant 文本 */;
let tool_calls_json = /* 从 tool-call-event 收集 */;

let content = if tool_calls_json.is_empty() {
    final_text.clone()
} else {
    serde_json::to_string(&json!({
        "text": final_text,
        "tool_calls": tool_calls_json,
    })).unwrap_or(final_text.clone())
};

sqlx::query("INSERT INTO messages ...")
    .bind(&content)
    // ...
```

### 3.4 前端 `get_messages` 解析

```typescript
// ChatView.tsx - 加载历史消息
async function loadMessages(sessionId: string) {
  const rows = await invoke<{ role: string; content: string; created_at: string }[]>(
    "get_messages", { sessionId }
  );
  const msgs: Message[] = rows.map(r => {
    // 尝试解析 JSON 格式的 content
    try {
      const parsed = JSON.parse(r.content);
      if (parsed.text !== undefined) {
        return {
          role: r.role as "user" | "assistant",
          content: parsed.text,
          created_at: r.created_at,
          toolCalls: parsed.tool_calls?.map((tc: any) => ({
            id: `${tc.name}-${Date.now()}`,
            ...tc,
          })),
        };
      }
    } catch {}
    // 普通字符串
    return {
      role: r.role as "user" | "assistant",
      content: r.content,
      created_at: r.created_at,
    };
  });
  setMessages(msgs);
}
```

---

## 4. 功能 3：会话历史管理

### 4.1 后端新增命令

**`get_sessions(skill_id)`**：
```rust
#[tauri::command]
pub async fn get_sessions(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, title, created_at, model_id FROM sessions WHERE skill_id = ? ORDER BY created_at DESC"
    )
    .bind(&skill_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|(id, title, created_at, model_id)| json!({
        "id": id, "title": title, "created_at": created_at, "model_id": model_id
    })).collect())
}
```

**`delete_session(session_id)`**：
```rust
#[tauri::command]
pub async fn delete_session(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(&session_id).execute(&db.0).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(&session_id).execute(&db.0).await.map_err(|e| e.to_string())?;
    Ok(())
}
```

**会话标题自动更新**：在 `send_message` 中，首次发消息时更新标题：
```rust
// 检查是否为第一条消息
let msg_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE session_id = ?")
    .bind(&session_id).fetch_one(&db.0).await.map_err(|e| e.to_string())?;

if msg_count.0 <= 1 { // 刚保存的用户消息是第一条
    let title = if user_message.len() > 20 {
        format!("{}...", &user_message[..20])
    } else {
        user_message.clone()
    };
    sqlx::query("UPDATE sessions SET title = ? WHERE id = ?")
        .bind(&title).bind(&session_id).execute(&db.0).await.ok();
}
```

### 4.2 前端架构改造

**App.tsx 状态提升**：
```typescript
const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
const [sessions, setSessions] = useState<SessionInfo[]>([]);

// 加载会话列表
async function loadSessions(skillId: string) {
  const list = await invoke<SessionInfo[]>("get_sessions", { skillId });
  setSessions(list);
}
```

**Sidebar.tsx 改造**：
```
┌──────────────────┐
│ 已安装 Skill      │
├──────────────────┤
│ ▸ Skill A  v1.0  │  ← Skill 列表
│ ▸ Skill B  v2.1  │
├──────────────────┤
│ 会话历史          │
├──────────────────┤
│ 今天的对话...     │  ← 当前 Skill 的会话列表
│ 读取文件测试      │
│ 代码重构助手      │
├──────────────────┤
│ [+ 新建会话]      │
│ [+ 安装 Skill]    │
│ [设置]            │
└──────────────────┘
```

**ChatView.tsx 改造**：
- 接收 `sessionId` 作为 prop（而非内部创建）
- `sessionId` 变化时调用 `get_messages` 加载历史
- "新建会话" 按钮通过回调通知 App.tsx

### 4.3 类型定义

```typescript
// types.ts
export interface SessionInfo {
  id: string;
  title: string;
  created_at: string;
  model_id: string;
}
```

---

## 5. 功能 4：MCP 集成

### 5.1 数据库新增表

```sql
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    args TEXT NOT NULL DEFAULT '[]',
    env TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
)
```

### 5.2 ToolRegistry 改造

当前 `ToolRegistry` 使用 `HashMap<String, Arc<dyn Tool>>`，`register(&mut self)` 需要可变引用。
由于 `Arc<ToolRegistry>` 在多个地方共享，需要支持内部可变性。

**改造**：
```rust
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn register(&self, tool: Arc<dyn Tool>) {
        self.tools.write().unwrap().insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().unwrap().get(name).cloned()
    }

    pub fn unregister(&self, name: &str) {
        self.tools.write().unwrap().remove(name);
    }

    pub fn get_tool_definitions(&self) -> Vec<Value> {
        self.tools.read().unwrap().values().map(|t| json!({...})).collect()
    }
}
```

### 5.3 MCP 工具注册流程

```rust
// 新文件: commands/mcp.rs

#[tauri::command]
pub async fn add_mcp_server(
    name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    db: State<'_, DbState>,
    registry: State<'_, Arc<ToolRegistry>>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // 1. 保存到数据库
    sqlx::query("INSERT INTO mcp_servers (id, name, command, args, env, enabled, created_at) VALUES (?, ?, ?, ?, ?, 1, ?)")
        .bind(&id).bind(&name)
        .bind(&command)
        .bind(serde_json::to_string(&args).unwrap())
        .bind(serde_json::to_string(&env).unwrap())
        .bind(&now)
        .execute(&db.0).await.map_err(|e| e.to_string())?;

    // 2. 通知 Sidecar 连接 MCP 服务器
    let client = reqwest::Client::new();
    client.post("http://localhost:8765/api/mcp/add-server")
        .json(&json!({ "name": name, "command": command, "args": args, "env": env }))
        .send().await.map_err(|e| e.to_string())?;

    // 3. 获取工具列表并注册
    let tools_resp = client.post("http://localhost:8765/api/mcp/list-tools")
        .json(&json!({ "serverName": name }))
        .send().await.map_err(|e| e.to_string())?;
    let tools: Value = tools_resp.json().await.map_err(|e| e.to_string())?;

    if let Some(tool_list) = tools["tools"].as_array() {
        for tool in tool_list {
            let tool_name = tool["name"].as_str().unwrap_or_default();
            let tool_desc = tool["description"].as_str().unwrap_or_default();
            let schema = tool["inputSchema"].clone();

            registry.register(Arc::new(SidecarBridgeTool::new(
                "http://localhost:8765".to_string(),
                "/api/mcp/call-tool".to_string(),
                format!("mcp_{}_{}", name, tool_name),  // 前缀避免冲突
                tool_desc.to_string(),
                schema,
            )));
        }
    }

    Ok(id)
}

#[tauri::command]
pub async fn list_mcp_servers(
    db: State<'_, DbState>,
) -> Result<Vec<Value>, String> { ... }

#[tauri::command]
pub async fn remove_mcp_server(
    id: String,
    db: State<'_, DbState>,
    registry: State<'_, Arc<ToolRegistry>>,
) -> Result<(), String> {
    // 1. 从 DB 获取 server name
    // 2. 从 registry 反注册所有 mcp_{name}_* 工具
    // 3. 从 DB 删除
}
```

### 5.4 SidecarBridgeTool 适配 MCP

当前 `SidecarBridgeTool.execute()` 发送 `input` 到 endpoint。对于 MCP 工具调用，endpoint 是 `/api/mcp/call-tool`，需要包含 `serverName` 和 `toolName`：

```rust
fn execute(&self, input: Value) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}{}", self.sidecar_url, self.endpoint);

    // 如果是 MCP 工具，包装请求体
    let body = if self.endpoint == "/api/mcp/call-tool" {
        json!({
            "serverName": self.mcp_server_name,
            "toolName": self.mcp_tool_name,
            "arguments": input,
        })
    } else {
        input
    };

    let resp = client.post(&url).json(&body).send()?;
    // ...
}
```

或者更简单：为 MCP 创建一个独立的 `McpTool` 结构体，继承 `Tool` trait。

### 5.5 应用启动时恢复 MCP 连接

在 `lib.rs` 的 `setup` 中，数据库初始化后加载已保存的 MCP 服务器配置并连接：

```rust
// lib.rs setup
let mcp_servers = sqlx::query_as::<_, (String, String, String, String)>(
    "SELECT name, command, args, env FROM mcp_servers WHERE enabled = 1"
).fetch_all(&pool).await.ok().unwrap_or_default();

for (name, command, args_json, env_json) in mcp_servers {
    // 连接 Sidecar MCP 并注册工具
}
```

### 5.6 前端 SettingsView MCP 管理

在 SettingsView 中，模型配置区域下方添加 MCP 服务器管理区域：

```
┌─────────────────────────────────┐
│ 模型配置                         │
│ [已有模型列表]                    │
│ [添加模型表单]                    │
├─────────────────────────────────┤
│ MCP 服务器                       │
│ ┌───────────────────────────┐   │
│ │ filesystem  npx @anthropic│   │ ← 已配置列表
│ │ [删除]                     │   │
│ └───────────────────────────┘   │
│ [添加 MCP 服务器]                │
│   名称: ________               │
│   命令: ________               │
│   参数: ________               │
│   [保存]                        │
└─────────────────────────────────┘
```

---

## 6. 不在范围内

- MCP HTTP/WebSocket transport（仅支持 stdio）
- MCP 服务器连接状态实时监控
- 会话搜索功能
- 消息导出功能
- 会话分享功能
