# Smoke Test 修复与增强设计文档

**日期**: 2026-02-23
**范围**: Sidebar 折叠修复、安装后自动切换、MCP 预设与依赖、工作目录、内置通用 Skill

---

## 1. Sidebar 折叠修复（Bug Fix）

### 现状问题

折叠后完全隐藏 Sidebar 组件，在主区域左上角用 absolute 定位放一个 `☰` 按钮，该按钮覆盖在主内容上方，遮挡 Skill 名称和其他 UI 元素。

### 方案

折叠后不完全隐藏 Sidebar，改为渲染一个 **窄侧边栏**（宽度 48px）：
- 只显示 3 个图标按钮：展开（`▶`）、安装（`+`）、设置（`⚙`）
- 不显示 Skill 名称、会话列表等文字内容
- `☰` 按钮从 `App.tsx` 的 absolute 定位移除，改为 Sidebar 内部渲染

### 文件变更

| 文件 | 改动 |
|------|------|
| `Sidebar.tsx` | 新增 `collapsed` prop，折叠时渲染窄版布局 |
| `App.tsx` | 移除 absolute `☰` 按钮，改为传递 `collapsed` prop 给 Sidebar |

---

## 2. 安装 Skill 后自动切换会话（UX 改进）

### 现状问题

`InstallDialog` 安装/导入成功后只调用 `onInstalled()` 刷新 Skill 列表，不自动选中新 Skill，也不创建新会话。

### 方案

- `InstallDialog.onInstalled` 回调签名改为 `onInstalled(skillId: string)`
- 安装成功后传入新 Skill 的 ID
- `App.tsx` 收到 skillId 后：
  1. `loadSkills()` 刷新列表
  2. `setSelectedSkillId(skillId)` 选中新 Skill
  3. 自动调用 `handleCreateSession()` 创建新会话

### 文件变更

| 文件 | 改动 |
|------|------|
| `InstallDialog.tsx` | `onInstalled` 改为接收 `skillId: string` 参数 |
| `App.tsx` | `handleInstalled(skillId)` 中自动切换 + 创建会话 |

---

## 3. MCP 预设与 Skill 依赖检查

### 3a. MCP 常用预设

在 `SettingsView.tsx` MCP 表单中增加快速选择下拉框，预置常用配置：

| 预设名称 | command | args | 需要的环境变量 |
|---------|---------|------|---------------|
| filesystem | npx | `["@anthropic/mcp-server-filesystem", "/tmp"]` | — |
| brave-search | npx | `["@anthropic/mcp-server-brave-search"]` | `BRAVE_API_KEY` |
| memory | npx | `["@anthropic/mcp-server-memory"]` | — |
| puppeteer | npx | `["@anthropic/mcp-server-puppeteer"]` | — |
| fetch | npx | `["@anthropic/mcp-server-fetch"]` | — |

选择后自动填充 name/command/args/env 提示，用户可修改后添加。

### 3b. SKILL.md 中声明 MCP 依赖

新增 frontmatter 字段 `mcp-servers`：

```yaml
---
name: web-researcher
mcp-servers:
  - name: brave-search
    command: npx
    args: ["@anthropic/mcp-server-brave-search"]
    env: ["BRAVE_API_KEY"]
---
```

### 3c. 导入时依赖检查

`import_local_skill` 后端命令：
1. 解析 `mcp-servers` 字段
2. 查询 DB 中已有 MCP 服务器的 `name`
3. 返回缺失列表

前端在 InstallDialog 中展示提示："此 Skill 需要以下 MCP 服务器，请先在设置中配置"。

### Rust 结构体变更

```rust
// skill_config.rs
#[derive(Deserialize, Debug, Clone)]
pub struct McpServerDep {
    pub name: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
}

// SkillConfig 新增字段
pub mcp_servers: Vec<McpServerDep>,

// FrontMatter 新增字段
#[serde(alias = "mcp-servers", default)]
mcp_servers: Vec<McpServerDep>,
```

### 后端返回结构变更

`import_local_skill` 返回值改为包含 missing MCP 信息：

```rust
#[derive(serde::Serialize)]
pub struct ImportResult {
    pub manifest: SkillManifest,
    pub missing_mcp: Vec<String>,  // 缺失的 MCP 服务器名称
}
```

### 文件变更

| 文件 | 改动 |
|------|------|
| `skill_config.rs` | 新增 `McpServerDep` 结构体和 `mcp_servers` 字段 |
| `commands/skills.rs` | `import_local_skill` 检查 MCP 依赖，返回 `ImportResult` |
| `SettingsView.tsx` | MCP 表单增加预设下拉框 |
| `InstallDialog.tsx` | 导入后展示缺失 MCP 警告 |

---

## 4. 工作目录选择（每会话独立）

### 数据库变更

```sql
ALTER TABLE sessions ADD COLUMN work_dir TEXT DEFAULT '';
```

### 会话创建流程

1. 用户点击「新建会话」
2. 弹出目录选择器（`open({ directory: true })`）
3. 用户选择目录后创建会话，`work_dir` 存入 DB
4. ChatView 顶部显示当前工作目录路径（可点击重新选择）

### 后端安全沙箱

`send_message` 中将 `work_dir` 传递给 `AgentExecutor`：
- 所有文件工具（ReadFile, WriteFile, Glob, Grep）检查路径前缀
- Bash 工具将 `cwd` 设为 `work_dir`
- 路径检查逻辑：规范化后检查 `canonical_path.starts_with(canonical_work_dir)`

### 工具层改动

每个文件工具的 `execute` 方法新增 `work_dir` 参数（或通过 context 传递）：

```rust
// 方案：通过 ToolContext 传递
pub struct ToolContext {
    pub work_dir: Option<PathBuf>,
}

// Tool trait 扩展
fn execute(&self, input: Value, ctx: &ToolContext) -> Result<String>;
```

### 文件变更

| 文件 | 改动 |
|------|------|
| `db.rs` | migration 新增 `work_dir` 列 |
| `commands/chat.rs` | `create_session` 接收 `work_dir` 参数，`send_message` 读取并传递 |
| `agent/types.rs` | 新增 `ToolContext` 结构体，`Tool` trait 方法签名变更 |
| `agent/tools/*.rs` | 所有工具实现路径检查 |
| `agent/executor.rs` | 传递 `ToolContext` 给工具 |
| `App.tsx` | `handleCreateSession` 先弹目录选择器 |
| `ChatView.tsx` | 顶部显示工作目录 |

---

## 5. 内置通用 Skill

### 方案

在 `init_db` 中自动插入一个内置通用 Skill：

```rust
// db.rs init_db 中
let builtin_manifest = SkillManifest {
    id: "builtin-general".to_string(),
    name: "通用助手".to_string(),
    description: "通用 AI 助手，可以读写文件、执行命令、搜索网页等".to_string(),
    version: "1.0.0".to_string(),
    ..
};

sqlx::query(
    "INSERT OR IGNORE INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type) VALUES (?, ?, ?, '', '', 'builtin')"
)
```

### System Prompt

```
你是一个通用 AI 助手。你可以：
- 读取和编写文件
- 在终端中执行命令
- 搜索文件和代码
- 搜索网页获取信息
- 管理记忆和上下文

请根据用户的需求，自主分析、规划和执行任务。
工作目录为用户指定的目录，所有文件操作限制在该目录范围内。
```

### Sidebar 显示

内置 Skill 使用特殊标签 `[内置]`（类似本地 Skill 的 `[本地]` 标签）。始终显示在 Skill 列表最顶部。

### 文件变更

| 文件 | 改动 |
|------|------|
| `db.rs` | `init_db` 中插入内置通用 Skill |
| `commands/chat.rs` | `send_message` 处理 `source_type = 'builtin'` |
| `Sidebar.tsx` | 内置 Skill 标签样式 |

---

## 实施优先级

```
Phase 1（Bug Fix + 快速改进）
├── 1. Sidebar 折叠修复
├── 2. 安装后自动切换
└── 5. 内置通用 Skill

Phase 2（核心功能）
├── 4. 工作目录选择 + 安全沙箱
└── 3a. MCP 预设

Phase 3（Skill 生态）
├── 3b. SKILL.md MCP 依赖声明
└── 3c. 导入时依赖检查
```
