# Milestone 1 收尾功能实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现三个功能：1) File Upload 支持 2) Secure Workspace 配置 UI 3) 手动触发压缩

**Architecture:**
- File Upload: 前端处理文件选择和读取，附加到消息内容发送
- Secure Workspace: 复用已有的 `work_dir` 列，添加 UI 选择器
- 手动压缩: 添加 Tauri 命令 + UI 按钮/命令触发

**Tech Stack:** React (ChatView.tsx), Tauri (commands), Rust (compactor.rs)

---

## 模块 1: File Upload 支持

### Task 1: 添加文件上传类型定义

**Files:**
- Modify: `apps/runtime/src/types.ts`

**Step 1: 添加 FileAttachment 类型**

```typescript
// apps/runtime/src/types.ts 添加
export interface FileAttachment {
  name: string;
  size: number;
  type: string;
  content: string;  // 文件文本内容或 base64
}
```

**Step 2: 提交**

```bash
git add apps/runtime/src/types.ts
git commit -m "feat(ui): 添加 FileAttachment 类型定义"
```

---

### Task 2: ChatView 添加附件状态和文件选择

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx:19-42`

**Step 1: 添加状态变量**

在 `ChatView` 组件中添加:
```typescript
const [attachedFiles, setAttachedFiles] = useState<FileAttachment[]>([]);
```

**Step 2: 添加文件选择处理函数**

在 `handleSend` 函数前添加:
```typescript
const MAX_FILE_SIZE = 5 * 1024 * 1024; // 5MB
const MAX_FILES = 5;

const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
  const files = Array.from(e.target.files || []);

  if (attachedFiles.length + files.length > MAX_FILES) {
    alert(`最多只能上传 ${MAX_FILES} 个文件`);
    return;
  }

  const newFiles: FileAttachment[] = [];
  for (const file of files) {
    if (file.size > MAX_FILE_SIZE) {
      alert(`文件 ${file.name} 超过 5MB 限制`);
      continue;
    }

    const content = await readFileAsText(file);
    newFiles.push({
      name: file.name,
      size: file.size,
      type: file.type,
      content,
    });
  }

  setAttachedFiles(prev => [...prev, ...newFiles]);
  e.target.value = ''; // 重置 input
};

// 辅助函数：读取文件为文本
const readFileAsText = (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsText(file);
  });
};
```

**Step 3: 添加文件删除函数**

```typescript
const removeAttachedFile = (index: number) => {
  setAttachedFiles(prev => prev.filter((_, i) => i !== index));
};
```

**Step 4: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): ChatView 添加文件上传状态和处理函数"
```

---

### Task 3: ChatView 添加附件按钮和附件列表 UI

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx:576-610`

**Step 1: 添加隐藏的文件 input 和附件按钮**

在 `<div className="flex items-center justify-between px-3 pb-2.5">` 之前添加:
```tsx
{/* 附件列表展示 */}
{attachedFiles.length > 0 && (
  <div className="flex flex-wrap gap-2 px-4 pt-3">
    {attachedFiles.map((file, index) => (
      <div
        key={index}
        className="flex items-center gap-2 px-3 py-1.5 bg-gray-100 rounded-full text-xs text-gray-700"
      >
        <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
        </svg>
        <span className="max-w-[150px] truncate">{file.name}</span>
        <span className="text-gray-400">({(file.size / 1024).toFixed(1)}KB)</span>
        <button
          onClick={() => removeAttachedFile(index)}
          className="ml-1 hover:text-red-500"
        >
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    ))}
  </div>
)}

{/* 隐藏的文件输入 */}
<input
  type="file"
  multiple
  onChange={handleFileSelect}
  className="hidden"
  id="file-upload"
/>
```

**Step 2: 在工具栏添加附件按钮**

在发送按钮前添加:
```tsx
<label
  htmlFor="file-upload"
  className="h-8 px-3 flex items-center justify-center gap-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 active:scale-[0.97] text-gray-600 text-xs font-medium transition-all cursor-pointer"
>
  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
  </svg>
  附件
</label>
```

**Step 3: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): ChatView 添加附件按钮和附件列表展示"
```

---

### Task 4: 修改 handleSend 附加文件内容

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx:249-277`

**Step 1: 修改 handleSend 函数**

```typescript
async function handleSend() {
  if (!input.trim() && attachedFiles.length === 0) return;
  if (streaming || !sessionId) return;

  // 构建消息内容：用户输入 + 附件
  let msg = input.trim();
  let fullContent = msg;

  if (attachedFiles.length > 0) {
    const attachmentsText = attachedFiles.map(f => {
      const ext = f.name.split('.').pop()?.toLowerCase() || '';
      const isImage = f.type.startsWith('image/');
      if (isImage) {
        return `## ${f.name}\n![${f.name}](${f.content})`;
      }
      return `## ${f.name}\n\`\`\`${ext}\n${f.content}\n\`\`\``;
    }).join('\n\n');

    fullContent = msg
      ? `${msg}\n\n---\n\n附件文件：\n${attachmentsText}`
      : `附件文件：\n${attachmentsText}`;
  }

  setInput("");
  setAttachedFiles([]);  // 发送后清空附件
  setMessages((prev) => [
    ...prev,
    { role: "user", content: fullContent, created_at: new Date().toISOString() },
  ]);
  setStreaming(true);
  // ... 其余代码保持不变
}
```

**Step 2: 更新发送按钮的 disabled 条件**

```tsx
disabled={!input.trim() && attachedFiles.length === 0}
```

**Step 3: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): handleSend 附加文件内容到消息"
```

---

## 模块 2: Secure Workspace 配置

### Task 5: 检查现有 work_dir 实现

**Step 1: 验证现有功能**

确认以下功能已存在:
- 数据库有 `work_dir` 列（已确认）
- `create_session` 命令接受 `work_dir` 参数（已确认）
- `send_message` 加载并使用 `work_dir`（已确认）

**Step 2: 提交（如果需要）**

如果无需修改数据库，跳过此任务。

---

### Task 6: 前端获取和更新会话 work_dir

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: 添加 workspace 状态**

```typescript
const [workspace, setWorkspace] = useState<string>("");
```

**Step 2: 在 sessionId 变化时加载 workspace**

```typescript
useEffect(() => {
  loadMessages(sessionId);
  // 新增：加载 workspace
  loadWorkspace(sessionId);
  // ... 其他重置逻辑
}, [sessionId]);

// 添加 loadWorkspace 函数
const loadWorkspace = async (sid: string) => {
  try {
    const sessions = await invoke<any[]>("get_sessions", { skillId: skill.id });
    const current = sessions.find((s: any) => s.id === sid);
    if (current) {
      setWorkspace(current.work_dir || "");
    }
  } catch (e) {
    console.error("加载工作空间失败:", e);
  }
};
```

**Step 3: 添加更新 workspace 的函数**

```typescript
const updateWorkspace = async (newWorkspace: string) => {
  try {
    await invoke("update_session_workspace", {
      sessionId,
      workspace: newWorkspace
    });
    setWorkspace(newWorkspace);
  } catch (e) {
    console.error("更新工作空间失败:", e);
  }
};
```

**Step 4: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): ChatView 添加 workspace 状态和加载/更新函数"
```

---

### Task 7: 添加 update_session_workspace 命令

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`

**Step 1: 添加命令**

```rust
#[tauri::command]
pub async fn update_session_workspace(
    session_id: String,
    workspace: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query("UPDATE sessions SET work_dir = ? WHERE id = ?")
        .bind(&workspace)
        .bind(&session_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 2: 在 lib.rs 中注册命令**

```rust
// 确保命令已导出
pub use commands::chat::{
    create_session,
    send_message,
    get_sessions,
    // ... 其他
    update_session_workspace,  // 新增
};
```

**Step 3: 提交**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs
git commit -m "feat(api): 添加 update_session_workspace 命令"
```

---

### Task 8: ChatView 头部添加 workspace 选择器

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: 找到头部区域**

在 `<div className="flex items-center gap-2 px-6 py-2">` 附近找到头部。

**Step 2: 添加 workspace 选择器**

```tsx
{/* Workspace 选择器 */}
{workspace !== undefined && (
  <button
    onClick={() => {
      // 打开目录选择器
      invoke<string | null>("select_directory", {
        defaultPath: workspace || undefined
      }).then((newDir) => {
        if (newDir) {
          updateWorkspace(newDir);
        }
      });
    }}
    className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-gray-100 hover:bg-gray-200 text-xs text-gray-600 transition-colors"
  >
    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
    </svg>
    <span className="max-w-[200px] truncate">
      {workspace || "选择工作目录"}
    </span>
  </button>
)}
```

**Step 3: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): ChatView 头部添加 workspace 选择器"
```

---

### Task 9: 添加 select_directory 命令

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/dialog.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`

**Step 1: 创建 dialog.rs**

```rust
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_directory(
    app: AppHandle,
    default_path: Option<String>,
) -> Result<Option<String>, String> {
    let mut builder = app.dialog().file();

    if let Some(path) = default_path {
        builder = builder.set_directory(&path);
    }

    let result = builder.blocking_pick_folder();

    Ok(result.map(|p| p.to_string()))
}
```

**Step 2: 在 Cargo.toml 添加依赖**

```toml
tauri-plugin-dialog = "2"
```

**Step 3: 在 lib.rs 注册插件和命令**

```rust
// 插件
plugin::Builder::new()
    .plugin(tauri_plugin_dialog::init())
    .build()

// 命令
mod dialog;
pub use dialog::select_directory;
```

**Step 4: 提交**

```bash
git add apps/runtime/src-tauri/src/commands/dialog.rs
git add apps/runtime/src-tauri/src/lib.rs
git add apps/runtime/src-tauri/Cargo.toml
git commit -m "feat(api): 添加 select_directory 命令用于选择工作目录"
```

---

## 模块 3: 手动触发压缩

### Task 10: 添加 compact_context 命令

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/agent/compactor.rs`

**Step 1: 在 chat.rs 添加 compact_context 命令**

```rust
#[derive(serde::Serialize)]
pub struct CompactionResult {
    original_tokens: usize,
    new_tokens: usize,
    summary: String,
}

#[tauri::command]
pub async fn compact_context(
    session_id: String,
    db: State<'_, DbState>,
    app: AppHandle,
) -> Result<CompactionResult, String> {
    // 1. 获取会话消息
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT role, content FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(&session_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    let messages: Vec<Value> = rows
        .iter()
        .map(|(role, content)| json!({ "role": role, "content": content }))
        .collect();

    // 2. 估算原始 token 数
    let original_tokens = estimate_tokens(&messages);

    // 3. 获取模型配置
    let (model_id,): (String,) = sqlx::query_as(
        "SELECT model_id FROM sessions WHERE id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    let (api_format, base_url, api_key, model_name) = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT api_format, base_url, api_key, model_name FROM model_configs WHERE id = ?"
    )
    .bind(&model_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // 4. 创建 transcript 目录
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let transcript_dir = app_data_dir.join("transcripts");
    std::fs::create_dir_all(&transcript_dir).map_err(|e| e.to_string())?;

    // 5. 保存完整记录并压缩
    let transcript_path = save_transcript(&transcript_dir, &session_id, &messages)
        .map_err(|e| e.to_string())?;

    let compacted = auto_compact(
        &api_format,
        &base_url,
        &api_key,
        &model_name,
        &messages,
        &transcript_path.to_string_lossy(),
    )
    .await
    .map_err(|e| e.to_string())?;

    // 6. 更新会话消息（删除旧消息，插入压缩后的消息）
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(&session_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    for msg in &compacted {
        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&session_id)
        .bind(msg["role"].as_str().unwrap_or("user"))
        .bind(msg["content"].as_str().unwrap_or(""))
        .bind(&now)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 7. 返回结果
    let new_tokens = estimate_tokens(&compacted);
    let summary = compacted
        .iter()
        .find(|m| m["role"] == "user")
        .and_then(|m| m["content"].as_str())
        .unwrap_or("")
        .to_string();

    Ok(CompactionResult {
        original_tokens,
        new_tokens,
        summary,
    })
}
```

**Step 2: 导出 estimate_tokens 函数**

在 compactor.rs 中将 `estimate_tokens` 改为 public:

```rust
pub fn estimate_tokens(messages: &[Value]) -> usize {
    // ... 现有代码
}
```

**Step 3: 提交**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs
git add apps/runtime/src-tauri/src/agent/compactor.rs
git commit -m "feat(api): 添加 compact_context 命令"
```

---

### Task 11: 前端添加压缩按钮

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: 添加压缩状态**

```typescript
const [compacting, setCompacting] = useState(false);
```

**Step 2: 添加压缩处理函数**

```typescript
const handleCompact = async () => {
  if (compacting || !sessionId) return;
  setCompacting(true);
  try {
    const result = await invoke<{
      original_tokens: number;
      new_tokens: number;
      summary: string;
    }>("compact_context", { sessionId });

    // 显示压缩结果
    const summaryText = `📦 上下文已压缩：${result.original_tokens} → ${result.new_tokens} tokens`;

    // 添加系统消息
    setMessages(prev => [
      ...prev,
      { role: "system", content: summaryText, created_at: new Date().toISOString() },
      { role: "assistant", content: result.summary, created_at: new Date().toISOString() },
    ]);

    // 刷新消息
    await loadMessages(sessionId);
  } catch (e) {
    console.error("压缩失败:", e);
    alert("压缩失败: " + String(e));
  } finally {
    setCompacting(false);
  }
};
```

**Step 3: 添加 /compact 命令识别**

修改 handleSend 函数，在开头添加:

```typescript
// 检查是否是 /compact 命令
if (input.trim() === "/compact") {
  setInput("");
  handleCompact();
  return;
}
```

**Step 4: 添加压缩按钮到工具栏**

在附件按钮旁添加:

```tsx
<button
  onClick={handleCompact}
  disabled={compacting}
  className="h-8 px-3 flex items-center justify-center gap-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 active:scale-[0.97] disabled:opacity-50 text-gray-600 text-xs font-medium transition-all"
>
  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
  </svg>
  {compacting ? "压缩中..." : "压缩"}
</button>
```

**Step 5: 提交**

```bash
git add apps/runtime/src/components/ChatView.tsx
git commit -m "feat(ui): 添加手动压缩按钮和 /compact 命令支持"
```

---

## 测试验证

### Task 12: 测试 File Upload

**步骤:**
1. 启动应用: `pnpm runtime`
2. 打开浏览器开发者工具
3. 在聊天输入框点击"附件"按钮
4. 选择 1-3 个小文件（< 5MB）
5. 验证附件列表显示正确
6. 点击发送，验证文件内容显示在消息中

---

### Task 13: 测试 Secure Workspace

**步骤:**
1. 新建一个聊天会话
2. 点击头部的工作目录按钮
3. 验证目录选择器打开
4. 选择一个新目录
5. 验证工作目录更新
6. 发送消息，验证 Agent 能访问该目录下的文件

---

### Task 14: 测试手动压缩

**步骤:**
1. 发送多条消息，创造足够的上下文
2. 点击"压缩"按钮 或 输入 `/compact`
3. 验证压缩进行中的状态
4. 验证压缩后的摘要消息显示
5. 验证消息数量减少

---

## 最终提交

```bash
git add -A
git commit -m "feat: 完成 Milestone 1 收尾功能

- File Upload: 支持最多5个文件，单文件≤5MB
- Secure Workspace: 会话级工作目录配置
- Manual Compact: 按钮和 /compact 命令触发压缩

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## 实施顺序

1. Task 1-4: File Upload
2. Task 5-9: Secure Workspace
3. Task 10-11: Manual Compression
4. Task 12-14: 测试验证
