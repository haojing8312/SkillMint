# Runtime 完善 + MCP 集成实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 清理 dead code、实现 tool call 持久化、添加会话历史管理、集成 MCP 协议支持。

**Architecture:** 分 4 个 Phase：Phase 1 清理 dead code（最简单），Phase 2 改造消息持久化和加载逻辑，Phase 3 重构前端状态管理和 Sidebar 添加会话历史，Phase 4 改造 ToolRegistry 支持动态注册 + MCP 服务器管理全链路。

**Tech Stack:** Rust (Tauri, sqlx, reqwest, serde_json, RwLock), TypeScript (React 18, Tailwind CSS), SQLite, Node.js Sidecar (MCP SDK)

**参考设计文档:** `docs/plans/2026-02-21-runtime-polish-mcp-design.md`

---

## Phase 1: Dead Code 清理 (Task 1)

### Task 1: 删除未使用的 `chat_stream` 函数

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/anthropic.rs:6-54`
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs:6-63`

**Step 1: 删除 `anthropic.rs` 中的 `chat_stream` 函数**

删除第 6-54 行的 `pub async fn chat_stream(...)` 函数（包含完整函数体）。保留 `chat_stream_with_tools` 和 `test_connection`。

**Step 2: 删除 `openai.rs` 中的 `chat_stream` 函数**

删除第 6-63 行的 `pub async fn chat_stream(...)` 函数。保留 `filter_thinking`、`chat_stream_with_tools` 和 `test_connection`。

**Step 3: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功，**无** `chat_stream is never used` 警告。

**Step 4: 运行测试**

Run: `cd apps/runtime/src-tauri && cargo test 2>&1`
Expected: 全部通过（20 passed, 2 ignored）

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/adapters/anthropic.rs apps/runtime/src-tauri/src/adapters/openai.rs
git commit -m "refactor: 删除未使用的 chat_stream 函数，消除编译警告"
```

---

## Phase 2: Tool Call 持久化与消息格式统一 (Tasks 2-4)

### Task 2: 改造 `send_message` 保存逻辑

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs:144-169`

**背景**: 当前 `send_message` 保存所有中间消息（包括 tool_use、tool_result、tool 角色消息）。改为只保存用户可见的消息：用户消息（已在前面保存）和最终 assistant 文本消息（附带 tool_calls 信息）。

**Step 1: 修改 `send_message` 的消息保存逻辑**

将 `chat.rs` 中第 151-169 行的消息保存循环替换为：

```rust
    // 收集本轮工具调用信息
    let mut tool_calls_info: Vec<Value> = vec![];
    for msg in final_messages.iter().skip(history.len()) {
        let role = msg["role"].as_str().unwrap_or("");
        if role == "assistant" {
            // Anthropic 格式: content 是数组，包含 tool_use
            if let Some(arr) = msg["content"].as_array() {
                for item in arr {
                    if item["type"] == "tool_use" {
                        tool_calls_info.push(json!({
                            "name": item["name"],
                            "input": item["input"],
                            "status": "completed",
                        }));
                    }
                }
            }
            // OpenAI 格式: tool_calls 字段
            if let Some(arr) = msg["tool_calls"].as_array() {
                for item in arr {
                    tool_calls_info.push(json!({
                        "name": item["function"]["name"],
                        "input": serde_json::from_str::<Value>(
                            item["function"]["arguments"].as_str().unwrap_or("{}")
                        ).unwrap_or(json!({})),
                        "status": "completed",
                    }));
                }
            }
        }
    }

    // 提取最终文本（最后一条 assistant 消息的纯文本 content）
    let final_text = final_messages.iter().rev()
        .find(|m| m["role"] == "assistant" && m["content"].is_string())
        .and_then(|m| m["content"].as_str())
        .unwrap_or("")
        .to_string();

    // 保存最终 assistant 消息
    let assistant_content = if tool_calls_info.is_empty() {
        final_text.clone()
    } else {
        serde_json::to_string(&json!({
            "text": final_text,
            "tool_calls": tool_calls_info,
        })).unwrap_or(final_text.clone())
    };

    let msg_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&msg_id)
    .bind(&session_id)
    .bind("assistant")
    .bind(&assistant_content)
    .bind(&now)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;
```

**Step 2: 添加会话标题自动更新**

在 `send_message` 中，保存用户消息之后（第 59 行之后），添加：

```rust
    // 自动更新会话标题（首次发消息时）
    let msg_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE session_id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    if msg_count.0 <= 1 {
        let title = if user_message.chars().count() > 20 {
            format!("{}...", user_message.chars().take(20).collect::<String>())
        } else {
            user_message.clone()
        };
        let _ = sqlx::query("UPDATE sessions SET title = ? WHERE id = ?")
            .bind(&title)
            .bind(&session_id)
            .execute(&db.0)
            .await;
    }
```

**Step 3: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "feat(chat): 改造消息保存逻辑，只保存用户可见消息并附带 tool_calls"
```

---

### Task 3: 改造 `get_messages` 返回结构化消息

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`（`get_messages` 函数）

**Step 1: 修改 `get_messages` 函数**

将当前的 `get_messages` 函数（`chat.rs:174-190`）替换为：

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

    Ok(rows.iter().map(|(role, content, created_at)| {
        // 尝试解析 JSON 格式的 content（包含 tool_calls 的 assistant 消息）
        if role == "assistant" {
            if let Ok(parsed) = serde_json::from_str::<Value>(content) {
                if parsed.get("text").is_some() {
                    return json!({
                        "role": role,
                        "content": parsed["text"].as_str().unwrap_or(""),
                        "created_at": created_at,
                        "tool_calls": parsed.get("tool_calls"),
                    });
                }
            }
        }
        // 普通消息（用户消息或纯文本 assistant 消息）
        json!({"role": role, "content": content, "created_at": created_at})
    }).collect())
}
```

**Step 2: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功

**Step 3: 运行测试**

Run: `cd apps/runtime/src-tauri && cargo test 2>&1`
Expected: 全部通过

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "feat(chat): get_messages 返回结构化消息，解析 tool_calls"
```

---

### Task 4: 新增 `get_sessions` 和 `delete_session` 命令

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs:29-40`

**Step 1: 添加 `get_sessions` 命令**

在 `chat.rs` 文件末尾添加：

```rust
#[tauri::command]
pub async fn get_sessions(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, title, created_at, model_id FROM sessions WHERE skill_id = ? ORDER BY created_at DESC"
    )
    .bind(&skill_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|(id, title, created_at, model_id)| json!({
        "id": id,
        "title": title,
        "created_at": created_at,
        "model_id": model_id,
    })).collect())
}
```

**Step 2: 添加 `delete_session` 命令**

在 `chat.rs` 文件末尾添加：

```rust
#[tauri::command]
pub async fn delete_session(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(&session_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(&session_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 3: 注册命令到 Tauri**

在 `lib.rs` 的 `invoke_handler` 中添加两个新命令：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 已有命令 ...
    commands::chat::get_sessions,
    commands::chat::delete_session,
])
```

**Step 4: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(chat): 新增 get_sessions 和 delete_session 命令"
```

---

## Checkpoint 1: 后端功能完成

验证清单：
1. `cd apps/runtime/src-tauri && cargo check` — 无警告（dead code 已清理）
2. `cd apps/runtime/src-tauri && cargo test` — 全部通过

---

## Phase 3: 前端会话历史 (Tasks 5-8)

### Task 5: 更新类型定义

**Files:**
- Modify: `apps/runtime/src/types.ts`

**Step 1: 添加 SessionInfo 类型**

在 `types.ts` 末尾添加：

```typescript
export interface SessionInfo {
  id: string;
  title: string;
  created_at: string;
  model_id: string;
}
```

**Step 2: Commit**

```bash
git add apps/runtime/src/types.ts
git commit -m "feat(ui): 添加 SessionInfo 类型定义"
```

---

### Task 6: 重构 App.tsx 状态管理

**Files:**
- Modify: `apps/runtime/src/App.tsx`

**背景**: 当前 App.tsx 只管理 `selectedSkillId`。需要提升 session 管理到 App 层：新增 `selectedSessionId`、`sessions` 状态，以及 `loadSessions`、`createSession` 函数。ChatView 不再自己创建会话。

**Step 1: 修改 App.tsx**

完整替换 `App.tsx`：

```typescript
import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import { InstallDialog } from "./components/InstallDialog";
import { SettingsView } from "./components/SettingsView";
import { SkillManifest, ModelConfig, SessionInfo } from "./types";

export default function App() {
  const [skills, setSkills] = useState<SkillManifest[]>([]);
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [showInstall, setShowInstall] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    loadSkills();
    loadModels();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 当选中 Skill 变化时，加载会话列表
  useEffect(() => {
    if (selectedSkillId) {
      loadSessions(selectedSkillId);
    } else {
      setSessions([]);
      setSelectedSessionId(null);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedSkillId]);

  async function loadSkills() {
    const list = await invoke<SkillManifest[]>("list_skills");
    setSkills(list);
    if (list.length > 0 && !selectedSkillId) {
      setSelectedSkillId(list[0].id);
    }
  }

  async function loadModels() {
    const list = await invoke<ModelConfig[]>("list_model_configs");
    setModels(list);
  }

  async function loadSessions(skillId: string) {
    try {
      const list = await invoke<SessionInfo[]>("get_sessions", { skillId });
      setSessions(list);
    } catch (e) {
      console.error("加载会话列表失败:", e);
      setSessions([]);
    }
  }

  async function handleCreateSession() {
    const modelId = models[0]?.id;
    if (!selectedSkillId || !modelId) return;
    try {
      const id = await invoke<string>("create_session", {
        skillId: selectedSkillId,
        modelId,
      });
      setSelectedSessionId(id);
      // 重新加载会话列表
      await loadSessions(selectedSkillId);
    } catch (e) {
      console.error("创建会话失败:", e);
    }
  }

  async function handleDeleteSession(sessionId: string) {
    try {
      await invoke("delete_session", { sessionId });
      if (selectedSessionId === sessionId) {
        setSelectedSessionId(null);
      }
      if (selectedSkillId) {
        await loadSessions(selectedSkillId);
      }
    } catch (e) {
      console.error("删除会话失败:", e);
    }
  }

  const handleSessionRefresh = useCallback(() => {
    if (selectedSkillId) loadSessions(selectedSkillId);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedSkillId]);

  const selectedSkill = skills.find((s) => s.id === selectedSkillId) ?? null;

  return (
    <div className="flex h-screen bg-slate-900 text-slate-100 overflow-hidden">
      <Sidebar
        skills={skills}
        selectedSkillId={selectedSkillId}
        onSelectSkill={setSelectedSkillId}
        sessions={sessions}
        selectedSessionId={selectedSessionId}
        onSelectSession={setSelectedSessionId}
        onNewSession={handleCreateSession}
        onDeleteSession={handleDeleteSession}
        onInstall={() => setShowInstall(true)}
        onSettings={() => setShowSettings(true)}
      />
      <div className="flex-1 overflow-hidden">
        {showSettings ? (
          <SettingsView onClose={async () => { await loadModels(); setShowSettings(false); }} />
        ) : selectedSkill && models.length > 0 && selectedSessionId ? (
          <ChatView
            skill={selectedSkill}
            models={models}
            sessionId={selectedSessionId}
            onSessionUpdate={handleSessionRefresh}
          />
        ) : selectedSkill && models.length > 0 ? (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm">
            <button
              onClick={handleCreateSession}
              className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded text-white text-sm"
            >
              新建会话
            </button>
          </div>
        ) : selectedSkill && models.length === 0 ? (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm">
            请先在设置中配置模型和 API Key
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm">
            从左侧选择一个 Skill 开始对话
          </div>
        )}
      </div>
      {showInstall && (
        <InstallDialog
          onInstalled={loadSkills}
          onClose={() => setShowInstall(false)}
        />
      )}
    </div>
  );
}
```

**Step 2: 验证 TypeScript 编译**

Run: `cd apps/runtime && npx tsc --noEmit 2>&1`
Expected: 会有类型错误（因为 Sidebar 和 ChatView 的 props 还没改），这是预期的。

**Step 3: Commit**

```bash
git add apps/runtime/src/App.tsx
git commit -m "refactor(ui): App.tsx 状态提升，管理 session 生命周期"
```

---

### Task 7: 重构 Sidebar 添加会话列表

**Files:**
- Modify: `apps/runtime/src/components/Sidebar.tsx`

**Step 1: 完整替换 Sidebar.tsx**

```typescript
import { SkillManifest, SessionInfo } from "../types";

interface Props {
  skills: SkillManifest[];
  selectedSkillId: string | null;
  onSelectSkill: (id: string) => void;
  sessions: SessionInfo[];
  selectedSessionId: string | null;
  onSelectSession: (id: string) => void;
  onNewSession: () => void;
  onDeleteSession: (id: string) => void;
  onInstall: () => void;
  onSettings: () => void;
}

export function Sidebar({
  skills,
  selectedSkillId,
  onSelectSkill,
  sessions,
  selectedSessionId,
  onSelectSession,
  onNewSession,
  onDeleteSession,
  onInstall,
  onSettings,
}: Props) {
  return (
    <div className="w-56 bg-slate-800 flex flex-col h-full border-r border-slate-700">
      {/* Skill 列表 */}
      <div className="px-4 py-3 text-xs font-medium text-slate-400 border-b border-slate-700">
        已安装 Skill
      </div>
      <div className="overflow-y-auto py-1" style={{ maxHeight: "30%" }}>
        {skills.length === 0 && (
          <div className="px-4 py-3 text-xs text-slate-500">暂无已安装 Skill</div>
        )}
        {skills.map((s) => (
          <button
            key={s.id}
            onClick={() => onSelectSkill(s.id)}
            className={
              "w-full text-left px-4 py-2 text-sm transition-colors " +
              (selectedSkillId === s.id
                ? "bg-blue-600/30 text-blue-300"
                : "text-slate-300 hover:bg-slate-700")
            }
          >
            <div className="font-medium truncate">{s.name}</div>
            <div className="text-xs text-slate-500 truncate">{s.version}</div>
          </button>
        ))}
      </div>

      {/* 会话历史 */}
      {selectedSkillId && (
        <>
          <div className="px-4 py-2 text-xs font-medium text-slate-400 border-t border-b border-slate-700 flex items-center justify-between">
            <span>会话历史</span>
            <button
              onClick={onNewSession}
              className="text-blue-400 hover:text-blue-300 text-xs"
            >
              + 新建
            </button>
          </div>
          <div className="flex-1 overflow-y-auto py-1">
            {sessions.length === 0 && (
              <div className="px-4 py-3 text-xs text-slate-500">暂无会话</div>
            )}
            {sessions.map((s) => (
              <div
                key={s.id}
                className={
                  "group flex items-center px-4 py-2 text-sm cursor-pointer transition-colors " +
                  (selectedSessionId === s.id
                    ? "bg-blue-600/20 text-blue-300"
                    : "text-slate-300 hover:bg-slate-700")
                }
                onClick={() => onSelectSession(s.id)}
              >
                <div className="flex-1 min-w-0">
                  <div className="truncate text-xs">{s.title || "New Chat"}</div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteSession(s.id);
                  }}
                  className="hidden group-hover:block text-red-400 hover:text-red-300 text-xs ml-1 flex-shrink-0"
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        </>
      )}

      {/* 底部按钮 */}
      <div className="p-3 space-y-2 border-t border-slate-700">
        <button
          onClick={onInstall}
          className="w-full bg-blue-600 hover:bg-blue-700 text-sm py-1.5 rounded transition-colors"
        >
          + 安装 Skill
        </button>
        <button
          onClick={onSettings}
          className="w-full bg-slate-700 hover:bg-slate-600 text-sm py-1.5 rounded transition-colors"
        >
          设置
        </button>
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add apps/runtime/src/components/Sidebar.tsx
git commit -m "feat(ui): Sidebar 添加会话历史列表，支持切换和删除"
```

---

### Task 8: 重构 ChatView 接收 sessionId prop 并加载历史

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**背景**: 当前 ChatView 内部管理 sessionId（通过 `create_session`）。重构为接收 `sessionId` 作为 prop，sessionId 变化时加载历史消息。移除内部的 `startNewSession` 和 `selectedModelId`。

**Step 1: 修改 ChatView 的 Props 和状态**

修改 Props 接口和内部状态：

```typescript
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import { SkillManifest, ModelConfig, Message, ToolCallInfo } from "../types";
import { ToolCallCard } from "./ToolCallCard";

interface Props {
  skill: SkillManifest;
  models: ModelConfig[];
  sessionId: string;
  onSessionUpdate?: () => void;
}

export function ChatView({ skill, models, sessionId, onSessionUpdate }: Props) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [streamBuffer, setStreamBuffer] = useState("");
  const [currentToolCalls, setCurrentToolCalls] = useState<ToolCallInfo[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);
  const streamBufferRef = useRef("");
  const currentToolCallsRef = useRef<ToolCallInfo[]>([]);
```

**Step 2: 添加历史消息加载**

在 `sessionId` 变化时加载历史消息：

```typescript
  // 加载历史消息
  useEffect(() => {
    loadMessages();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  async function loadMessages() {
    try {
      const rows = await invoke<{ role: string; content: string; created_at: string; tool_calls?: any[] }[]>(
        "get_messages", { sessionId }
      );
      const msgs: Message[] = rows.map((r) => {
        const msg: Message = {
          role: r.role as "user" | "assistant",
          content: r.content,
          created_at: r.created_at,
        };
        if (r.tool_calls && r.tool_calls.length > 0) {
          msg.toolCalls = r.tool_calls.map((tc: any, i: number) => ({
            id: `${tc.name}-history-${i}`,
            name: tc.name,
            input: tc.input || {},
            output: tc.output,
            status: (tc.status || "completed") as "running" | "completed" | "error",
          }));
        }
        return msg;
      });
      setMessages(msgs);
    } catch (e) {
      console.error("加载消息历史失败:", e);
    }
  }
```

**Step 3: 移除内部的 `startNewSession` 和 `selectedModelId`**

删除所有与 `startNewSession`、`selectedModelId` 相关的代码。移除旧的 `useEffect(() => { startNewSession(); }, [skill.id])` 。

**Step 4: 修改 `handleSend`**

在 `handleSend` 成功后通知父组件刷新会话列表（标题可能更新了）：

```typescript
  async function handleSend() {
    if (!input.trim() || streaming || !sessionId) return;
    const msg = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: msg, created_at: new Date().toISOString() }]);
    setStreaming(true);
    currentToolCallsRef.current = [];
    setCurrentToolCalls([]);
    streamBufferRef.current = "";
    setStreamBuffer("");
    try {
      await invoke("send_message", { sessionId, userMessage: msg });
      onSessionUpdate?.(); // 通知父组件刷新（标题可能更新）
    } catch (e) {
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: "错误: " + String(e), created_at: new Date().toISOString() },
      ]);
    } finally {
      setStreaming(false);
    }
  }
```

**Step 5: 简化头部（移除模型选择器和新建会话按钮）**

头部只显示 Skill 名称和版本：

```tsx
      <div className="flex items-center justify-between px-6 py-3 border-b border-slate-700 bg-slate-800">
        <div>
          <span className="font-medium">{skill.name}</span>
          <span className="text-xs text-slate-400 ml-2">v{skill.version}</span>
        </div>
      </div>
```

**Step 6: 保留 stream-token 和 tool-call-event 监听器**

这两个 useEffect 保持不变，但依赖从 `[sessionId]`（内部 state）改为 prop `sessionId`（已经是 prop 了，不需要改）。

**Step 7: 验证 TypeScript 编译**

Run: `cd apps/runtime && npx tsc --noEmit 2>&1`
Expected: 编译成功，无类型错误

**Step 8: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "refactor(ui): ChatView 接收 sessionId prop，支持加载历史消息"
```

---

## Checkpoint 2: 前端会话历史完成

验证清单：
1. `cd apps/runtime/src-tauri && cargo test` — 全部通过
2. `cd apps/runtime && npx tsc --noEmit` — 前端编译成功
3. `cd apps/runtime/src-tauri && cargo check` — 无错误无警告

---

## Phase 4: MCP 集成 (Tasks 9-14)

### Task 9: ToolRegistry 改造为 RwLock

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/registry.rs`
- Modify: `apps/runtime/src-tauri/tests/test_registry.rs`

**Step 1: 修改 ToolRegistry 使用 RwLock**

将 `registry.rs` 完整替换为：

```rust
use super::tools::{GlobTool, GrepTool, ReadFileTool, WriteFileTool};
use super::types::Tool;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_file_tools() -> Self {
        let registry = Self::new();
        registry.register(Arc::new(ReadFileTool));
        registry.register(Arc::new(WriteFileTool));
        registry.register(Arc::new(GlobTool));
        registry.register(Arc::new(GrepTool));
        registry
    }

    pub fn register(&self, tool: Arc<dyn Tool>) {
        self.tools.write().unwrap().insert(tool.name().to_string(), tool);
    }

    pub fn unregister(&self, name: &str) {
        self.tools.write().unwrap().remove(name);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().unwrap().get(name).cloned()
    }

    pub fn get_tool_definitions(&self) -> Vec<Value> {
        self.tools
            .read()
            .unwrap()
            .values()
            .map(|t| {
                json!({
                    "name": t.name(),
                    "description": t.description(),
                    "input_schema": t.input_schema(),
                })
            })
            .collect()
    }

    /// 返回所有以指定前缀开头的工具名称
    pub fn tools_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.tools.read().unwrap().keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect()
    }
}
```

**Step 2: 更新 executor.rs 中的 `get` 调用**

`executor.rs:119` 中 `self.registry.get(&call.name)` 现在返回 `Option<Arc<dyn Tool>>`（之前是 `Option<&Arc<dyn Tool>>`）。修改：

```rust
let result = match self.registry.get(&call.name) {
    Some(tool) => match tool.execute(call.input.clone()) {
        Ok(output) => output,
        Err(e) => format!("工具执行错误: {}", e),
    },
    None => format!("工具不存在: {}", call.name),
};
```

这个签名变化应该是兼容的（`Arc<dyn Tool>` 可以直接调用 `execute`）。如果编译失败，可能需要调整引用方式。

**Step 3: 运行测试**

Run: `cd apps/runtime/src-tauri && cargo test 2>&1`
Expected: 全部通过（RwLock 是 API 兼容改造）

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/registry.rs apps/runtime/src-tauri/src/agent/executor.rs
git commit -m "refactor(agent): ToolRegistry 改用 RwLock 支持运行时动态注册"
```

---

### Task 10: 数据库新增 mcp_servers 表

**Files:**
- Modify: `apps/runtime/src-tauri/src/db.rs`

**Step 1: 添加 mcp_servers 表创建语句**

在 `db.rs` 的 `init_db` 函数中，`model_configs` 表创建之后、migration 之前添加：

```rust
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            command TEXT NOT NULL,
            args TEXT NOT NULL DEFAULT '[]',
            env TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;
```

**Step 2: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功

**Step 3: Commit**

```bash
git add apps/runtime/src-tauri/src/db.rs
git commit -m "feat(db): 新增 mcp_servers 表存储 MCP 服务器配置"
```

---

### Task 11: 创建 MCP 命令模块

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/mcp.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`

**Step 1: 创建 `commands/mcp.rs`**

```rust
use tauri::State;
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use super::skills::DbState;
use crate::agent::ToolRegistry;
use crate::agent::tools::SidecarBridgeTool;

#[tauri::command]
pub async fn add_mcp_server(
    name: String,
    command: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
    db: State<'_, DbState>,
    registry: State<'_, Arc<ToolRegistry>>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // 保存到数据库
    sqlx::query(
        "INSERT INTO mcp_servers (id, name, command, args, env, enabled, created_at) VALUES (?, ?, ?, ?, ?, 1, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&command)
    .bind(serde_json::to_string(&args).unwrap_or_default())
    .bind(serde_json::to_string(&env).unwrap_or_default())
    .bind(&now)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // 通知 Sidecar 连接 MCP 服务器
    let client = reqwest::Client::new();
    let connect_resp = client.post("http://localhost:8765/api/mcp/add-server")
        .json(&json!({
            "name": name,
            "config": {
                "command": command,
                "args": args,
                "env": env,
            }
        }))
        .send()
        .await
        .map_err(|e| format!("连接 Sidecar 失败: {}", e))?;

    if !connect_resp.status().is_success() {
        return Err("MCP 服务器连接失败".to_string());
    }

    // 获取工具列表并注册
    let tools_resp = client.post("http://localhost:8765/api/mcp/list-tools")
        .json(&json!({ "serverName": name }))
        .send()
        .await
        .map_err(|e| format!("获取工具列表失败: {}", e))?;

    let tools_body: Value = tools_resp.json().await.map_err(|e| e.to_string())?;

    if let Some(tool_list) = tools_body["tools"].as_array() {
        for tool in tool_list {
            let tool_name = tool["name"].as_str().unwrap_or_default();
            let tool_desc = tool["description"].as_str().unwrap_or_default();
            let schema = tool.get("inputSchema").cloned().unwrap_or(json!({"type": "object", "properties": {}}));

            let full_name = format!("mcp_{}_{}", name, tool_name);
            registry.register(Arc::new(SidecarBridgeTool::new(
                "http://localhost:8765".to_string(),
                "/api/mcp/call-tool".to_string(),
                full_name,
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
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i32, String)>(
        "SELECT id, name, command, args, env, enabled, created_at FROM mcp_servers ORDER BY created_at DESC"
    )
    .bind_all(())  // 不需要参数
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // 注意：sqlx 不支持 bind_all(())，需要用无参版本
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i32, String)>(
        "SELECT id, name, command, args, env, enabled, created_at FROM mcp_servers ORDER BY created_at DESC"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|(id, name, command, args, env, enabled, created_at)| json!({
        "id": id,
        "name": name,
        "command": command,
        "args": serde_json::from_str::<Value>(args).unwrap_or(json!([])),
        "env": serde_json::from_str::<Value>(env).unwrap_or(json!({})),
        "enabled": enabled == &1,
        "created_at": created_at,
    })).collect())
}

#[tauri::command]
pub async fn remove_mcp_server(
    id: String,
    db: State<'_, DbState>,
    registry: State<'_, Arc<ToolRegistry>>,
) -> Result<(), String> {
    // 获取 server name
    let (name,): (String,) = sqlx::query_as(
        "SELECT name FROM mcp_servers WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // 从 registry 反注册所有该服务器的工具
    let prefix = format!("mcp_{}_", name);
    let tool_names = registry.tools_with_prefix(&prefix);
    for tool_name in tool_names {
        registry.unregister(&tool_name);
    }

    // 从数据库删除
    sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

**Step 2: 修改 `commands/mod.rs`**

检查当前文件，添加 `pub mod mcp;`。

**Step 3: 注册命令到 lib.rs**

在 `invoke_handler` 中添加：

```rust
commands::mcp::add_mcp_server,
commands::mcp::list_mcp_servers,
commands::mcp::remove_mcp_server,
```

**Step 4: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功（可能需要调整 SidecarBridgeTool 的导入路径）

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/mcp.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(mcp): 新增 add/list/remove MCP 服务器 Tauri 命令"
```

---

### Task 12: 适配 SidecarBridgeTool 支持 MCP 调用

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/tools/sidecar_bridge.rs`

**背景**: 当前 SidecarBridgeTool 直接发送 `input` 到 endpoint。MCP 工具需要包装 `serverName`、`toolName`、`arguments`。

**Step 1: 添加 MCP 相关字段和构造函数**

```rust
use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub struct SidecarBridgeTool {
    sidecar_url: String,
    endpoint: String,
    tool_name: String,
    tool_description: String,
    schema: Value,
    // MCP 特有字段
    mcp_server_name: Option<String>,
    mcp_tool_name: Option<String>,
}

impl SidecarBridgeTool {
    pub fn new(
        sidecar_url: String,
        endpoint: String,
        tool_name: String,
        tool_description: String,
        schema: Value,
    ) -> Self {
        Self {
            sidecar_url,
            endpoint,
            tool_name,
            tool_description,
            schema,
            mcp_server_name: None,
            mcp_tool_name: None,
        }
    }

    pub fn new_mcp(
        sidecar_url: String,
        tool_name: String,
        tool_description: String,
        schema: Value,
        mcp_server_name: String,
        mcp_tool_name: String,
    ) -> Self {
        Self {
            sidecar_url,
            endpoint: "/api/mcp/call-tool".to_string(),
            tool_name,
            tool_description,
            schema,
            mcp_server_name: Some(mcp_server_name),
            mcp_tool_name: Some(mcp_tool_name),
        }
    }
}

impl Tool for SidecarBridgeTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn input_schema(&self) -> Value {
        self.schema.clone()
    }

    fn execute(&self, input: Value) -> Result<String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}{}", self.sidecar_url, self.endpoint);

        // MCP 工具需要包装请求体
        let body = if let (Some(server), Some(tool)) = (&self.mcp_server_name, &self.mcp_tool_name) {
            json!({
                "serverName": server,
                "toolName": tool,
                "arguments": input,
            })
        } else {
            input
        };

        let resp = client.post(&url).json(&body).send()?;

        if !resp.status().is_success() {
            let error_body: Value = resp.json().unwrap_or(json!({}));
            return Err(anyhow!(
                "Sidecar 调用失败: {}",
                error_body["error"].as_str().unwrap_or("Unknown error")
            ));
        }

        let result: Value = resp.json()?;
        // MCP 工具返回的结果在 content 字段
        if let Some(content) = result["content"].as_str() {
            Ok(content.to_string())
        } else if let Some(output) = result["output"].as_str() {
            Ok(output.to_string())
        } else {
            Ok(serde_json::to_string(&result).unwrap_or_default())
        }
    }
}
```

**Step 2: 更新 Task 11 中的 `add_mcp_server` 使用 `new_mcp`**

在 `commands/mcp.rs` 中，将工具注册改为：

```rust
registry.register(Arc::new(SidecarBridgeTool::new_mcp(
    "http://localhost:8765".to_string(),
    full_name,
    tool_desc.to_string(),
    schema,
    name.clone(),
    tool_name.to_string(),
)));
```

**Step 3: 验证编译和测试**

Run: `cd apps/runtime/src-tauri && cargo test 2>&1`
Expected: 全部通过

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tools/sidecar_bridge.rs apps/runtime/src-tauri/src/commands/mcp.rs
git commit -m "feat(mcp): SidecarBridgeTool 支持 MCP 工具调用包装"
```

---

### Task 13: 应用启动时恢复 MCP 连接

**Files:**
- Modify: `apps/runtime/src-tauri/src/lib.rs`

**Step 1: 在 setup 中添加 MCP 初始化**

在 `lib.rs` 的 `setup` 闭包中，`app.manage(agent_executor)` 之后添加：

```rust
            // 恢复已保存的 MCP 服务器连接
            let pool_clone = pool.clone();
            let registry_clone = Arc::clone(&registry);
            tauri::async_runtime::spawn(async move {
                let servers = sqlx::query_as::<_, (String, String, String, String)>(
                    "SELECT name, command, args, env FROM mcp_servers WHERE enabled = 1"
                )
                .fetch_all(&pool_clone)
                .await
                .unwrap_or_default();

                let client = reqwest::Client::new();
                for (name, command, args_json, env_json) in servers {
                    let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
                    let env: std::collections::HashMap<String, String> = serde_json::from_str(&env_json).unwrap_or_default();

                    // 连接 MCP 服务器
                    let connect_result = client.post("http://localhost:8765/api/mcp/add-server")
                        .json(&serde_json::json!({
                            "name": name,
                            "config": { "command": command, "args": args, "env": env }
                        }))
                        .send()
                        .await;

                    if connect_result.is_err() {
                        eprintln!("[mcp] 连接 MCP 服务器 {} 失败（Sidecar 可能未启动）", name);
                        continue;
                    }

                    // 获取工具列表并注册
                    if let Ok(resp) = client.post("http://localhost:8765/api/mcp/list-tools")
                        .json(&serde_json::json!({ "serverName": name }))
                        .send()
                        .await
                    {
                        if let Ok(body) = resp.json::<serde_json::Value>().await {
                            if let Some(tool_list) = body["tools"].as_array() {
                                for tool in tool_list {
                                    let tool_name = tool["name"].as_str().unwrap_or_default();
                                    let tool_desc = tool["description"].as_str().unwrap_or_default();
                                    let schema = tool.get("inputSchema").cloned()
                                        .unwrap_or(serde_json::json!({"type": "object", "properties": {}}));

                                    let full_name = format!("mcp_{}_{}", name, tool_name);
                                    registry_clone.register(Arc::new(
                                        crate::agent::tools::SidecarBridgeTool::new_mcp(
                                            "http://localhost:8765".to_string(),
                                            full_name,
                                            tool_desc.to_string(),
                                            schema,
                                            name.clone(),
                                            tool_name.to_string(),
                                        )
                                    ));
                                }
                                eprintln!("[mcp] 已恢复 MCP 服务器 {} 的工具注册", name);
                            }
                        }
                    }
                }
            });
```

注意：需要将 `registry` 的创建从 `Arc::new(ToolRegistry::with_file_tools())` 改为先创建再 clone：

```rust
            // 初始化 AgentExecutor（包含文件工具）
            let registry = Arc::new(ToolRegistry::with_file_tools());
            let agent_executor = Arc::new(AgentExecutor::new(Arc::clone(&registry)));
            app.manage(agent_executor);
            app.manage(Arc::clone(&registry));  // 也将 registry 作为 State 管理
```

**Step 2: 验证编译**

Run: `cd apps/runtime/src-tauri && cargo check 2>&1`
Expected: 编译成功

**Step 3: Commit**

```bash
git add apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(mcp): 应用启动时自动恢复 MCP 服务器连接和工具注册"
```

---

### Task 14: 前端 SettingsView 添加 MCP 管理

**Files:**
- Modify: `apps/runtime/src/components/SettingsView.tsx`

**Step 1: 添加 MCP 服务器管理 UI**

在 SettingsView 中，模型配置区域下方添加 MCP 服务器管理区域。

在组件内添加新的 state：

```typescript
const [mcpServers, setMcpServers] = useState<any[]>([]);
const [mcpForm, setMcpForm] = useState({ name: "", command: "", args: "" });
const [mcpError, setMcpError] = useState("");

useEffect(() => { loadMcpServers(); }, []);

async function loadMcpServers() {
  try {
    const list = await invoke<any[]>("list_mcp_servers");
    setMcpServers(list);
  } catch (e) {
    console.error("加载 MCP 服务器失败:", e);
  }
}

async function handleAddMcp() {
  setMcpError("");
  try {
    const args = mcpForm.args.split(/\s+/).filter(Boolean);
    await invoke("add_mcp_server", {
      name: mcpForm.name,
      command: mcpForm.command,
      args,
      env: {},
    });
    setMcpForm({ name: "", command: "", args: "" });
    loadMcpServers();
  } catch (e) {
    setMcpError(String(e));
  }
}

async function handleRemoveMcp(id: string) {
  await invoke("remove_mcp_server", { id });
  loadMcpServers();
}
```

在 return JSX 中，模型配置区域的 `</div>` 之后添加：

```tsx
      {/* MCP 服务器管理 */}
      <div className="bg-slate-800 rounded-lg p-4 space-y-3 mt-6">
        <div className="text-xs font-medium text-slate-400 mb-2">MCP 服务器</div>

        {mcpServers.length > 0 && (
          <div className="space-y-2 mb-3">
            {mcpServers.map((s) => (
              <div key={s.id} className="flex items-center justify-between bg-slate-700 rounded px-3 py-2 text-sm">
                <div>
                  <span className="font-medium">{s.name}</span>
                  <span className="text-slate-400 ml-2 text-xs">{s.command} {s.args?.join(" ")}</span>
                </div>
                <button onClick={() => handleRemoveMcp(s.id)} className="text-red-400 hover:text-red-300 text-xs">
                  删除
                </button>
              </div>
            ))}
          </div>
        )}

        <div>
          <label className={labelCls}>名称</label>
          <input className={inputCls} placeholder="例: filesystem" value={mcpForm.name} onChange={(e) => setMcpForm({ ...mcpForm, name: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>命令</label>
          <input className={inputCls} placeholder="例: npx" value={mcpForm.command} onChange={(e) => setMcpForm({ ...mcpForm, command: e.target.value })} />
        </div>
        <div>
          <label className={labelCls}>参数（空格分隔）</label>
          <input className={inputCls} placeholder="例: @anthropic/mcp-server-filesystem /tmp" value={mcpForm.args} onChange={(e) => setMcpForm({ ...mcpForm, args: e.target.value })} />
        </div>
        {mcpError && <div className="text-red-400 text-xs">{mcpError}</div>}
        <button
          onClick={handleAddMcp}
          disabled={!mcpForm.name || !mcpForm.command}
          className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 text-sm py-1.5 rounded transition-colors"
        >
          添加 MCP 服务器
        </button>
      </div>
```

**Step 2: 验证 TypeScript 编译**

Run: `cd apps/runtime && npx tsc --noEmit 2>&1`
Expected: 编译成功

**Step 3: Commit**

```bash
git add apps/runtime/src/components/SettingsView.tsx
git commit -m "feat(ui): SettingsView 添加 MCP 服务器管理面板"
```

---

## Checkpoint 3: 全部完成

验证清单：
1. `cd apps/runtime/src-tauri && cargo check` — 无错误无警告
2. `cd apps/runtime/src-tauri && cargo test` — 全部通过
3. `cd apps/runtime && npx tsc --noEmit` — 前端编译成功

手动验证（`pnpm runtime`）：
- 发送消息 → Agent 工具调用卡片正常显示
- 退出重新打开 → 历史会话列表可见，点击可恢复
- 历史消息中 tool calls 正确渲染
- 设置中添加 MCP 服务器 → 工具自动注册
- 新会话中可调用 MCP 工具
