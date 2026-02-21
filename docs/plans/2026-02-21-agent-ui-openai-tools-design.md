# Runtime Agent UI + OpenAI Tool Calling 设计文档

**日期**: 2026-02-21
**状态**: 已批准
**范围**: OpenAI function calling 实现 + Agent 模式默认启用 + 前端工具调用 UI
**参考**: `reference/minimax` 中的 MiniMax Desktop Agent 实现

---

## 1. 背景与目标

### 1.1 当前状态
Agent 后端能力已完成（15 个任务）：
- Tool Trait + Registry（6 个工具：ReadFile, WriteFile, Glob, Grep, Bash, SidecarBridge）
- AgentExecutor ReAct 循环（仅 Anthropic 格式）
- Node.js Sidecar（Playwright + MCP SDK）
- `send_message` 已集成 AgentExecutor（但带 `enable_tools` 开关）

### 1.2 存在的问题
1. **OpenAI tool calling 未实现**：`adapters/openai.rs` 没有 `chat_stream_with_tools`，OpenAI 格式模型无法使用 Agent
2. **前端参数不同步**：`send_message` 需要 `enable_tools` 参数，但前端未传递，导致调用失败
3. **无 Agent UI**：前端无法展示工具调用过程，用户看不到 Agent 的工具操作

### 1.3 目标
1. 实现 OpenAI function calling（`chat_stream_with_tools`）
2. 移除 `enable_tools` 参数，始终走 Agent 模式
3. 前端添加工具调用可折叠卡片 UI

---

## 2. 架构设计

### 2.1 后端数据流

```
send_message (无 enable_tools 参数)
    ↓
AgentExecutor.execute_turn()
    ↓ 根据 api_format 选择适配器
    ├── "anthropic" → anthropic::chat_stream_with_tools()
    └── "openai"    → openai::chat_stream_with_tools()  ← 新增
    ↓
LLMResponse::Text → 结束循环
LLMResponse::ToolCalls → 执行工具 → emit("tool-call-event") → 继续循环
    ↓
emit("stream-token", done: true)
    ↓
保存所有消息到数据库
```

### 2.2 前端事件流

```
Tauri 事件                          前端状态
───────────                         ────────
stream-token (text)  ──→  streamBuffer 累积，Markdown 实时渲染
tool-call-event (started) ──→  toolCalls[] 添加新工具调用卡片（执行中状态）
tool-call-event (completed) ──→  toolCalls[] 更新工具状态和结果
tool-call-event (error) ──→  toolCalls[] 更新为错误状态
stream-token (done: true) ──→  合并 toolCalls + 文本为完整 Message
```

---

## 3. 后端详细设计

### 3.1 OpenAI `chat_stream_with_tools`

**文件**: `adapters/openai.rs`

**OpenAI function calling 请求格式**:
```json
{
  "model": "gpt-4",
  "messages": [...],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "read_file",
        "description": "读取文件内容",
        "parameters": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] }
      }
    }
  ],
  "stream": true
}
```

**SSE 流中的 tool_calls 解析**:
```
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_xxx","function":{"name":"read_file","arguments":""}}]}}]}
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"pa"}}]}}]}
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"th\":\""}}]}}]}
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"test.txt\"}"}}]}}]}
data: {"choices":[{"finish_reason":"tool_calls"}]}
```

**关键点**:
- `tool_calls[].index` 标识工具调用序号（支持并行调用）
- `function.name` 只在第一个 delta 出现
- `function.arguments` 增量拼接为完整 JSON
- `finish_reason` 为 `"tool_calls"` 表示需要执行工具

**工具结果回传格式** (OpenAI):
```json
{
  "role": "tool",
  "tool_call_id": "call_xxx",
  "content": "文件内容..."
}
```

**函数签名**:
```rust
pub async fn chat_stream_with_tools(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<Value>,
    tools: Vec<Value>,
    on_token: impl Fn(String) + Send + Clone,
) -> Result<LLMResponse>
```

**工具定义转换**: AgentExecutor 中的 `get_tool_definitions()` 返回 Anthropic 格式，需要在 `openai::chat_stream_with_tools` 内部转换为 OpenAI 格式：
```
Anthropic: { "name": "x", "description": "y", "input_schema": {...} }
    ↓ 转换
OpenAI: { "type": "function", "function": { "name": "x", "description": "y", "parameters": {...} } }
```

### 3.2 AgentExecutor 改动

**文件**: `agent/executor.rs`

**修改 `execute_turn`**:
- 根据 `api_format` 调用对应的 `chat_stream_with_tools`
- OpenAI 格式的工具结果消息使用 `role: "tool"` + `tool_call_id`
- 添加 `app_handle: Option<&AppHandle>` 参数，用于 emit 工具调用事件

**工具调用事件 emit**:
```rust
#[derive(serde::Serialize, Clone)]
struct ToolCallEvent {
    session_id: String,
    tool_name: String,
    tool_input: Value,
    tool_output: Option<String>,
    status: String,  // "started" | "completed" | "error"
}

// 执行工具前
app.emit("tool-call-event", ToolCallEvent {
    session_id, tool_name, tool_input,
    tool_output: None, status: "started".into()
});

// 执行工具后
app.emit("tool-call-event", ToolCallEvent {
    session_id, tool_name, tool_input,
    tool_output: Some(result), status: "completed".into()
});
```

### 3.3 `send_message` 简化

**文件**: `commands/chat.rs`

**改动**:
- 移除 `enable_tools: bool` 参数
- 移除 `agent_executor: State<'_, Arc<AgentExecutor>>` 参数
- 直接在函数内创建 AgentExecutor 或从全局状态获取
- 始终调用 `agent_executor.execute_turn()`
- 传入 `app.clone()` 供工具调用事件 emit

### 3.4 OpenAI 消息格式适配

AgentExecutor 中需要根据 `api_format` 构造不同的工具调用/结果消息：

| 部分 | Anthropic | OpenAI |
|------|-----------|--------|
| 助手工具调用 | `role: assistant, content: [{ type: tool_use, id, name, input }]` | `role: assistant, tool_calls: [{ id, type: function, function: { name, arguments } }]` |
| 工具结果 | `role: user, content: [{ type: tool_result, tool_use_id, content }]` | `role: tool, tool_call_id, content` |

---

## 4. 前端详细设计

### 4.1 类型定义更新

**文件**: `types.ts`

```typescript
interface ToolCallInfo {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output?: string;
  status: "running" | "completed" | "error";
}

interface Message {
  role: "user" | "assistant";
  content: string;
  created_at: string;
  toolCalls?: ToolCallInfo[];  // 新增
}
```

### 4.2 ToolCallCard 组件

**文件**: `components/ToolCallCard.tsx`（新增）

**功能**:
- 可折叠卡片，展示单个工具调用
- 收起状态：工具图标 + 名称 + 状态标签
- 展开状态：参数（JSON 代码块）+ 执行结果（代码块）

**工具图标映射**:
```typescript
const TOOL_ICONS: Record<string, string> = {
  read_file: "📂",
  write_file: "📝",
  glob: "🔍",
  grep: "🔎",
  bash: "💻",
  sidecar_bridge: "🌐",
};
```

**状态标签**:
- `running`：蓝色脉冲动画 + "执行中..."
- `completed`：绿色 "完成"
- `error`：红色 "错误"

### 4.3 ChatView 改动

**文件**: `components/ChatView.tsx`

**新增状态**:
```typescript
const [currentToolCalls, setCurrentToolCalls] = useState<ToolCallInfo[]>([]);
```

**新增事件监听**: `tool-call-event`
```typescript
useEffect(() => {
  const unlisten = listen<ToolCallEvent>("tool-call-event", ({ payload }) => {
    if (payload.session_id !== sessionId) return;
    if (payload.status === "started") {
      setCurrentToolCalls(prev => [...prev, {
        id: payload.tool_name + Date.now(),
        name: payload.tool_name,
        input: payload.tool_input,
        status: "running",
      }]);
    } else {
      setCurrentToolCalls(prev => prev.map(tc =>
        tc.name === payload.tool_name && tc.status === "running"
          ? { ...tc, output: payload.tool_output, status: payload.status as "completed" | "error" }
          : tc
      ));
    }
  });
  return () => { unlisten.then(fn => fn()); };
}, [sessionId]);
```

**send_message 调用修改**:
```typescript
// 移除 enableTools 参数
await invoke("send_message", { sessionId, userMessage: msg });
```

**stream-token done 处理修改**:
```typescript
if (payload.done) {
  const finalContent = streamBufferRef.current;
  setMessages(prev => [...prev, {
    role: "assistant",
    content: finalContent,
    created_at: new Date().toISOString(),
    toolCalls: currentToolCalls.length > 0 ? [...currentToolCalls] : undefined,
  }]);
  setCurrentToolCalls([]); // 重置
  streamBufferRef.current = "";
  setStreamBuffer("");
  setStreaming(false);
}
```

**消息渲染**:
```tsx
{m.role === "assistant" && (
  <div>
    {m.toolCalls?.map(tc => <ToolCallCard key={tc.id} toolCall={tc} />)}
    <ReactMarkdown>{m.content}</ReactMarkdown>
  </div>
)}
```

**流式区域（执行中的工具调用 + 流式文本）**:
```tsx
{(currentToolCalls.length > 0 || streamBuffer) && (
  <div className="flex justify-start">
    <div className="max-w-2xl bg-slate-700 rounded-lg px-4 py-2 text-sm">
      {currentToolCalls.map(tc => <ToolCallCard key={tc.id} toolCall={tc} />)}
      {streamBuffer && <ReactMarkdown>{streamBuffer}</ReactMarkdown>}
      <span className="animate-pulse">|</span>
    </div>
  </div>
)}
```

---

## 5. 测试策略

### 5.1 后端测试

| 测试 | 文件 | 内容 |
|------|------|------|
| OpenAI 工具定义转换 | `test_openai_tools.rs` | 验证 Anthropic → OpenAI 工具格式转换 |
| OpenAI SSE 解析 | `test_openai_tools.rs` | 验证 tool_calls delta 增量解析 |
| OpenAI 网络错误 | `test_openai_tools.rs` | 无效 URL 返回错误 |
| ReAct 循环 OpenAI 格式 | `test_react_loop.rs` | 验证 OpenAI 格式的消息构造 |
| send_message 无参数 | 编译检查 | 移除 enable_tools 后的编译 |

### 5.2 前端验证

- `cargo check` 确保后端编译通过
- 手动测试：启动应用，发送消息，验证工具调用卡片显示
- 验证无工具调用时退化为普通聊天（Agent 不返回 tool_use 即直接输出文本）

---

## 6. 不在范围内

- OpenAI tool calling 的并行工具调用优化（先实现串行）
- 工具执行权限确认弹框（参考 MiniMax，留待后续）
- 工具执行取消/中断
- 消息中工具调用的数据库持久化格式优化
