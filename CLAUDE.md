# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

**重要约定**：除代码、命令、文件路径、技术术语等必须使用英文的内容外，所有说明、注释、文档均使用中文。

## 项目概述

**SkillMint** 是一个开源的 AI Skill 打包与桌面应用发布平台。项目由两个 Tauri 桌面应用组成：**Studio**（供创作者打包 Skill）和 **Runtime**（供用户安装和运行 Skill）。Skill 使用 AES-256-GCM 加密打包成 `.skillpack` 文件。

## Monorepo 结构

```
apps/
├── studio/              # 创作者端应用 (Tauri + React)
├── runtime/             # 用户端应用 (Tauri + React + Node.js Sidecar)
│   ├── src-tauri/       # Rust 后端，包含 agent 系统
│   └── sidecar/         # Node.js HTTP 服务器 (Playwright, MCP)
packages/
└── skillpack-rs/        # 核心加密/打包库 (Rust)
```

**包管理器**: pnpm workspaces + Turborepo

## 常用命令

```bash
# 开发
pnpm runtime              # 以开发模式运行 Runtime 应用
pnpm studio               # 以开发模式运行 Studio 应用

# 构建
pnpm build:runtime        # 构建 Runtime 生产版本
pnpm build:studio         # 构建 Studio 生产版本

# 测试
cd apps/runtime/src-tauri
cargo test                # 运行所有 Rust 测试
cargo test --test test_registry  # 运行特定测试文件

# Sidecar
cd apps/runtime/sidecar
pnpm build                # 构建 Node.js sidecar
```

## 核心架构

### 1. 加密系统（基于用户名）

加密模型使用**确定性密钥推导**方式，从用户名生成密钥：

```
username → PBKDF2-HMAC-SHA256 (100k iterations) → AES-256-GCM key
                ↓ salt = SHA256(skill_id + skill_name)
```

**关键特性**：
- 相同用户名 + Skill 始终生成相同密钥
- 创作者无需存储密钥
- 通过 manifest 中加密的 "SKILLHUB_OK" token 验证用户名

**代码位置**: `packages/skillpack-rs/src/crypto.rs`

### 2. SkillPack 格式

`.skillpack` 文件是 ZIP 压缩包：
```
myskill.skillpack
├── manifest.json           # 明文（id, name, version, username_hint, encrypted_verify）
├── icon.png                # 明文（可选）
└── encrypted/              # AES-256-GCM 加密文件
    ├── SKILL.md.enc
    ├── templates/*.md.enc
    └── examples/*.md.enc
```

**打包**: `packages/skillpack-rs/src/pack.rs`
**解包**: `packages/skillpack-rs/src/unpack.rs`

### 3. 多模型系统（双协议架构）

所有 LLM 交互通过两种适配器格式：

**Anthropic Messages API** (`adapters/anthropic.rs`):
- 原生支持：Claude 模型
- 兼容：MiniMax Anthropic 端点

**OpenAI 兼容 API** (`adapters/openai.rs`):
- 原生支持：OpenAI GPT 模型
- 兼容：DeepSeek, Qwen, MiniMax, Moonshot, Yi

**推理内容过滤**：
- DeepSeek：过滤 `delta.reasoning_content`
- MiniMax：移除 `<think>...</think>` 标签
- 防止内部思考过程污染聊天界面

**Provider 预设配置**: 详见 `docs/plans/2026-02-19-llm-adapter-provider-presets-design.md`

### 4. Agent 系统（Rust + Node.js 混合架构）

```
Tauri Rust Backend
├── AgentExecutor (ReAct 循环)
├── ToolRegistry (Tool trait 抽象)
├── Native Tools: ReadFile, WriteFile, Glob, Grep, Bash
└── HTTP Client → Node.js Sidecar (localhost:8765)
                      ├── Playwright (浏览器自动化)
                      └── MCP Client (协议支持)
```

**Tool Trait** (`agent/types.rs`):
```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn execute(&self, input: Value) -> Result<String>;
}
```

**已实现的工具**：
- `agent/tools/read_file.rs` - 读取文件内容
- `agent/tools/write_file.rs` - 写入/创建文件
- `agent/tools/glob_tool.rs` - 文件模式匹配 (`**/*.rs`)
- `agent/tools/grep_tool.rs` - 正则搜索文件
- `agent/tools/bash.rs` - 跨平台 shell 执行（Windows 上使用 PowerShell）

**测试**: `apps/runtime/src-tauri/tests/test_*.rs`

### 5. 数据库结构 (SQLite)

**位置**: `{app_data_dir}/skillhub.db`

```sql
installed_skills (id, manifest, installed_at, last_used_at, username, pack_path)
sessions (id, skill_id, title, created_at, model_id)
messages (id, session_id, role, content, created_at)
model_configs (id, name, api_format, base_url, model_name, is_default, api_key)
```

**Schema 定义**: `apps/runtime/src-tauri/src/db.rs`

### 6. Tauri IPC 模式

**后端 (Rust)**:
```rust
#[tauri::command]
pub async fn install_skill(
    app: AppHandle,
    pack_path: String,
    username: String,
) -> Result<SkillManifest, String> {
    // 实现代码
}
```

**前端 (TypeScript)**:
```typescript
import { invoke } from "@tauri-apps/api/core";

const manifest = await invoke<SkillManifest>("install_skill", {
    packPath: path,
    username: user,
});
```

**事件流** (SSE 风格):
```rust
app.emit("stream-token", StreamToken {
    session_id,
    token: token.to_string(),
    done: false,
})?;
```

### 7. Sidecar 通信

Node.js sidecar (`apps/runtime/sidecar/`) 在 8765 端口运行 Hono HTTP 服务器。

**Rust 端调用**:
```rust
let resp = reqwest::Client::new()
    .post("http://localhost:8765/api/browser/navigate")
    .json(&json!({ "url": "https://example.com" }))
    .send().await?;
```

**Sidecar 端点** (`sidecar/src/index.ts`):
- `POST /api/browser/navigate` - 浏览器自动化
- `POST /api/mcp/connect` - MCP 服务器连接

**生命周期管理**: 由 `src-tauri/src/sidecar.rs` 中的 `SidecarManager` 管理

## 重要配置文件

### Runtime 应用
- **Tauri 配置**: `apps/runtime/src-tauri/tauri.conf.json`
  - App ID: `dev.skillhub.runtime`
  - 开发端口: 5174
  - 窗口尺寸: 1200x750
- **Rust 依赖**: `apps/runtime/src-tauri/Cargo.toml`
  - 关键依赖: aes-gcm, pbkdf2, sqlx, reqwest, tokio
- **前端配置**: `apps/runtime/package.json`
  - React 18, Vite 5, Tailwind CSS

### Studio 应用
- **Tauri 配置**: `apps/studio/src-tauri/tauri.conf.json`
- **命令**: `commands.rs` 中的 `read_skill_dir`, `pack_skill`

### 构建优化
`Cargo.toml` 中的 Release 配置：
```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "s"      # 优化体积
panic = "abort"
strip = true
```

## 关键源文件

### Runtime 后端 (apps/runtime/src-tauri/src)
- `lib.rs` - Tauri 构建器，命令注册
- `db.rs` - SQLite schema 和初始化
- `sidecar.rs` - Node.js sidecar 生命周期管理
- `commands/skills.rs` - 安装/列出/删除 Skill
- `commands/chat.rs` - 发送消息，创建会话
- `commands/models.rs` - 模型配置 CRUD
- `adapters/anthropic.rs` - Anthropic API，包含 tool_use 解析
- `adapters/openai.rs` - OpenAI 兼容 API
- `agent/registry.rs` - 工具注册
- `agent/executor.rs` - ReAct 循环（开发中）
- `agent/tools/` - 各个工具的实现

### Runtime 前端 (apps/runtime/src)
- `App.tsx` - 主组件，状态管理
- `components/ChatView.tsx` - 聊天界面，流式输出
- `components/Sidebar.tsx` - Skill 列表
- `components/SettingsView.tsx` - 模型配置界面
- `components/InstallDialog.tsx` - Skill 安装对话框

### 共享包 (packages/skillpack-rs/src)
- `crypto.rs` - AES-256-GCM 加密/解密
- `pack.rs` - 创建 .skillpack 文件
- `unpack.rs` - 验证和解压
- `types.rs` - PackConfig, SkillManifest, FrontMatter

## 参考开源项目

**重要**：`reference/` 目录包含了三个优秀的 AI Agent 开源项目作为技术参考。在实现 SkillHub 功能时，强烈建议先查阅相关项目的实现方案。

⚠️ **使用原则**：
- ✅ **参考设计思路和架构模式** - 学习其设计理念、架构选择、问题解决方案
- ✅ **借鉴最佳实践** - 错误处理、安全机制、性能优化等经验
- ✅ **理解实现细节** - 深入了解具体功能的实现方式
- ❌ **不要直接复制代码** - 必须根据 SkillHub 的实际需求重新设计和实现
- ❌ **不要照搬架构** - SkillHub 有自己独特的定位（加密 Skill 打包分发平台）
- ⚠️ **注意许可证差异** - WorkAny (Community License), Gemini CLI (Apache 2.0), OpenClaw (MIT)

**正确的参考方式**：
1. 先理解 SkillHub 的需求和约束
2. 查看参考项目如何解决类似问题
3. 分析其方案的优缺点
4. 结合 SkillHub 特点重新设计
5. 用自己的代码实现，不直接复制

### 快速索引

**查看总览**：[reference/README.md](reference/README.md)

**按功能查找参考**：

| 功能需求 | 参考项目 | 文档路径 |
|---------|---------|---------|
| **Tauri 桌面应用架构** | WorkAny | [reference/docs/workany.md](reference/docs/workany.md) |
| **Sidecar 二进制打包** | WorkAny | [reference/docs/workany.md#2-codex-sandbox-隔离执行](reference/docs/workany.md) |
| **Agent Runtime 实现** | Gemini CLI | [reference/docs/gemini-cli.md#1-自研-agent-runtime](reference/docs/gemini-cli.md) |
| **MCP 服务器集成** | Gemini CLI | [reference/docs/gemini-cli.md#2-mcp-model-context-protocol-集成](reference/docs/gemini-cli.md) |
| **Tool 系统设计** | Gemini CLI | [reference/docs/gemini-cli.md#3-内置工具系统](reference/docs/gemini-cli.md) |
| **沙箱代码执行** | WorkAny, Gemini CLI | [reference/docs/workany.md#2-codex-sandbox-隔离执行](reference/docs/workany.md) |
| **首次运行向导** | OpenClaw | [reference/docs/openclaw.md#1-向导式安装onboard](reference/docs/openclaw.md) |
| **系统诊断工具** | OpenClaw | [reference/docs/openclaw.md#2-doctor-诊断工具](reference/docs/openclaw.md) |
| **Skill/插件系统** | OpenClaw | [reference/docs/openclaw.md#5-skills-platform](reference/docs/openclaw.md) |
| **Extensions 扩展** | Gemini CLI | [reference/docs/gemini-cli.md#5-extensions-扩展系统](reference/docs/gemini-cli.md) |
| **多平台构建脚本** | WorkAny | [reference/docs/workany.md#1-跨平台构建脚本](reference/docs/workany.md) |
| **Artifact 预览** | WorkAny | [reference/docs/workany.md#3-artifact-实时预览](reference/docs/workany.md) |
| **浏览器自动化** | MiniMax | [reference/docs/minimax.md#1-browserview-浏览器控制](reference/docs/minimax.md) |
| **反检测技术** | MiniMax | [reference/docs/minimax.md#2-反检测技术-stealthjs](reference/docs/minimax.md) |

### 四个参考项目概览

1. **[WorkAny](reference/docs/workany.md)** (811 ⭐) - 桌面 AI Agent，Tauri + Claude Code
   - ✅ 与 SkillHub 架构最相似（Tauri + React + Rust）
   - ✅ externalBin 打包策略可直接应用于 Sidecar
   - ✅ Artifact 实时预览设计

2. **[Gemini CLI](reference/docs/gemini-cli.md)** (11.4K+ ⭐) - Google 官方 AI Agent，CLI 工具
   - ✅ 完整的 Agent Runtime 实现（自研）
   - ✅ MCP 集成最佳实践（Google 官方）
   - ✅ 丰富的内置工具系统

3. **[OpenClaw](reference/docs/openclaw.md)** - 多渠道 AI 助手 Gateway
   - ✅ Onboard Wizard 交互式安装向导
   - ✅ Doctor 诊断工具
   - ✅ Skills 三级分类（bundled/managed/workspace）

4. **[MiniMax Agent](reference/docs/minimax.md)** - MiniMax 桌面端逆向工程
   - ✅ BrowserView 浏览器控制（15+ 工具）
   - ✅ 反检测技术（stealth.js 18种方法）
   - ✅ Electron 主进程代码可直接复用

### 使用示例

**场景 1：实现 MCP 服务器管理 UI**
```bash
# 1. 查看 Gemini CLI 的 MCP 集成章节
cat reference/docs/gemini-cli.md | grep -A 20 "MCP 集成"

# 2. 查看源码实现
cd reference/gemini-cli
grep -r "mcpServers" packages/cli/src/mcp/
```

**场景 2：优化 Sidecar 打包**
```bash
# 1. 查看 WorkAny 的 externalBin 配置
cat reference/docs/workany.md | grep -A 20 "externalBin"

# 2. 查看源码
cat reference/workany/src-tauri/tauri.conf.json
```

**场景 3：添加首次运行向导**
```bash
# 1. 查看 OpenClaw 的 Onboard Wizard
cat reference/docs/openclaw.md | grep -A 30 "向导式安装"

# 2. 查看源码实现
cd reference/openclaw && cat src/cli/commands/onboard.ts
```

## 文档阅读顺序

按以下顺序阅读文档以获得完整上下文：

1. **README.md** - 项目概述、技术栈、路线图
2. **reference/README.md** - 参考开源项目总览（⭐ 新增）
3. **docs/plans/2026-02-19-skillhub-mvp-design.md** - MVP 架构、数据库 schema
4. **docs/plans/2026-02-19-llm-adapter-provider-presets-design.md** - 多模型设计
5. **docs/plans/2026-02-20-agent-capabilities-design.md** - Agent 系统架构
6. **SkillHub_PRD.md** - 产品需求文档

## 当前开发状态

### 已完成 ✅
- Monorepo 脚手架
- skillpack-rs 核心库
- 多模型适配器系统（9 个 provider）
- Runtime 基础 UI 和流式聊天
- Skill 安装流程
- Tool trait 和 registry
- 文件工具（Read, Write, Glob, Grep）及测试
- Bash 工具，跨平台支持
- Sidecar 管理器基础设施
- Anthropic tool_use SSE 解析

### 进行中 🔄 (feat/agent-capabilities 分支)
- AgentExecutor ReAct 循环
- Playwright 浏览器自动化
- MCP 协议集成

### 未开始 ❌
- Studio: Skill 编辑器（Monaco）
- Studio: 测试聊天界面
- Studio: 打包 UI
- 自动更新机制
- Skill 市场

## Git 工作流

当前分支：`feat/agent-capabilities`（可能）
主分支：`main`

在进行更改前用 `git status` 检查当前分支。

## 开发提示

### 运行整个技术栈
1. 启动 Runtime：`pnpm runtime`（同时启动 Rust 后端和 React 前端）
2. Sidecar 通过 Tauri sidecar 配置自动构建
3. Rust 和 React 都支持热重载

### 调试
- Rust 后端：使用 `println!()` 或 `dbg!()` 宏，输出显示在终端
- React 前端：浏览器开发者工具（开发模式下自动打开）
- Sidecar：检查 `http://localhost:8765` 或终端日志

### 添加新工具
1. 创建 `apps/runtime/src-tauri/src/agent/tools/my_tool.rs`
2. 实现 `Tool` trait
3. 在 `agent/registry.rs` 中注册：`registry.register(Box::new(MyTool));`
4. 在 `tests/test_my_tool.rs` 中添加测试

### 修改 Skill 格式
1. 更新 `packages/skillpack-rs/src/types.rs`（结构体）
2. 更新 `pack.rs` 中的打包逻辑
3. 更新 `unpack.rs` 中的解包逻辑
4. 更新测试

### 测试加密功能
```bash
cd packages/skillpack-rs
cargo test crypto::tests  # 仅运行加密测试
```

### ⚠️ 重要警告：不要杀掉 Node.js 进程

**问题描述**：使用 `taskkill //F //IM node.exe` 或类似命令杀掉所有 Node.js 进程会导致：
- **Claude Code 进程被杀死**（Claude Code 本身运行在 Node.js 上）
- **所有正在进行的任务丢失**
- **需要重新启动 Claude Code**

**正确做法**：
- 如需停止 Sidecar，使用 `Ctrl+C` 在运行 Sidecar 的终端中优雅退出
- 或者通过进程 PID 精确杀掉特定进程：
  ```bash
  # Windows
  netstat -ano | findstr :8765  # 找到占用 8765 端口的进程 PID
  taskkill /PID <PID> /F         # 杀掉特定 PID

  # Linux/macOS
  lsof -ti:8765 | xargs kill     # 杀掉占用 8765 端口的进程
  ```

**错误示例**（会杀掉 Claude Code）：
```bash
❌ taskkill //F //IM node.exe     # 会杀掉所有 Node.js，包括 Claude Code
❌ killall node                    # Linux/macOS 同样会杀掉 Claude Code
```

**正确示例**（仅杀掉 Sidecar）：
```bash
✅ netstat -ano | findstr :8765 → 找到 PID → taskkill /PID <PID> /F
✅ 在 Sidecar 运行的终端按 Ctrl+C
```

## 代码风格

### Rust
- 使用 `anyhow::Result` 进行错误处理
- 在 Tauri 命令中将错误转换为 `String`：`.map_err(|e| e.to_string())?`
- 在 `#[cfg(test)]` 模块或独立的 `tests/` 目录中编写测试
- 当数据库可用时，使用 `sqlx::query!()` 进行编译时 SQL 验证

### TypeScript
- 启用严格模式
- 使用 Tauri `invoke()` 调用后端
- 在 UI 中处理加载/错误状态
- 使用 Tailwind 进行样式设计（不使用 CSS modules）

## 中文优先原则

在编写代码时遵循以下规则：

### 必须使用英文的情况
- 代码本身（变量名、函数名、类名等）
- 命令行命令和参数
- 文件路径
- 技术术语（如 API、HTTP、JSON 等标准术语）
- 配置文件中的键名
- 代码注释中的技术引用

### 应该使用中文的情况
- 代码注释中的说明性文字
- 用户界面文本（UI labels、按钮文字、提示信息等）
- 错误消息和日志（面向中文用户时）
- 文档和说明文档
- Git commit 消息（项目约定）
- 变量和函数的文档字符串

### 示例对比

❌ **不推荐**（说明性内容使用英文）：
```rust
// Create a new skill instance and register it
let skill = Skill::new("my-skill");
registry.register(skill);
```

✅ **推荐**（说明性内容使用中文）：
```rust
// 创建新的 Skill 实例并注册
let skill = Skill::new("my-skill");
registry.register(skill);
```

❌ **不推荐**（UI 文字使用英文）：
```typescript
<button>Install Skill</button>
<p>Please enter your username</p>
```

✅ **推荐**（UI 文字使用中文）：
```typescript
<button>安装 Skill</button>
<p>请输入用户名</p>
```

### 混合使用示例
```rust
/// 安装 Skill 到本地数据库
///
/// # 参数
/// - `pack_path`: .skillpack 文件路径
/// - `username`: 用于解密的用户名
///
/// # 返回
/// 成功时返回 SkillManifest，失败时返回错误信息
#[tauri::command]
pub async fn install_skill(
    app: AppHandle,
    pack_path: String,
    username: String,
) -> Result<SkillManifest, String> {
    // 解压并验证 skillpack
    let manifest = unpack_skillpack(&pack_path, &username)
        .map_err(|e| format!("解包失败: {}", e))?;

    // 保存到数据库
    save_to_db(&app, manifest).await
}
```
