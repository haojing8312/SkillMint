# SkillMint

[English](README.md) | [简体中文](README.zh-CN.md)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://reactjs.org/)

**SkillMint** = Agent Runtime + Encrypted Skill System

An open-source platform to **package, encrypt, and distribute AI Skills** as secure desktop applications. Create once, distribute anywhere.

## What is SkillMint?

SkillMint helps AI Skill creators:
- **Package**: Transform Skills (Markdown with prompts/examples) into encrypted `.skillpack` files
- **Protect**: Military-grade encryption protects your intellectual property
- **Distribute**: Users run Skills in a secure local sandbox environment
- **Monetize**: Control access and distribution of your premium Skills

**For Creators**: Package and sell your Skills securely
**For Users**: Run powerful AI Skills locally without exposing sensitive data

## Architecture

SkillMint consists of two independent applications:

### Runtime (User Application)
The core Agent execution environment where users install and run encrypted Skills:

**Core Agent Capabilities**:
- ✅ **File Operations**: Read, write, edit files with permission control
- ✅ **Code Execution**: Cross-platform Bash/PowerShell command execution
- ✅ **Browser Automation**: Playwright integration for web scraping and automation (via Sidecar)
- ✅ **MCP Integration**: Model Context Protocol server support for extended capabilities
- ✅ **Multi-Agent System**: Sub-Agent task distribution with isolated contexts
- ✅ **Memory Management**: TodoWrite for task tracking, context compression
- ✅ **Web Search**: DuckDuckGo integration for real-time information
- ✅ **Permission System**: Multi-layer security validation

**User Features**:
- Install `.skillpack` files via drag-and-drop or file picker
- Clean chat interface with real-time streaming responses
- Session history with searchable conversation archives
- Multi-model support (Claude 4.6, GPT-4, MiniMax M2.5, GLM-4, DeepSeek)
- Local secure workspace folder configuration
- No command line required

### Studio (Creator Application)
**Status**: Planned for Milestone 3
Professional Skill authoring environment:
- Monaco Editor with Markdown syntax highlighting
- Visual Skill structure editor (SKILL.md + templates/ + examples/)
- Integrated testing chat powered by Claude Code
- One-click packaging with encryption
- Version control and publishing workflow

**Note**: For MVP, creators can use **Claude Code** or **VS Code** to develop Skills, then package them via CLI tool.

## Key Features

### Security & Privacy
- **Military-Grade Encryption**: AES-256-GCM with deterministic key derivation from username
- **Secure Workspace**: Configure trusted local folders for file operations
- **Permission Control**: Multi-layer validation for sensitive operations
- **No Cloud Dependency**: All processing happens locally

### Agent Capabilities
- **ReAct Loop Engine**: Advanced reasoning and action planning
- **Sub-Agent System**: Parallel task execution with isolated contexts
- **Context Compression**: Smart truncation to stay within token limits
- **Tool Registry**: Dynamic tool registration including MCP servers
- **Memory Persistence**: TodoWrite for task tracking across sessions

### Developer Experience
- **Multi-Model Support**: 15+ models across 9 providers
- **Hot Reload**: Real-time Skill updates during development
- **Comprehensive Logging**: Tool call tracing and error diagnostics
- **Cross-Platform**: Windows, macOS, Linux support

## Tech Stack

### Runtime Backend
- **Framework**: Tauri 2.0 (Rust)
- **Database**: SQLite (sqlx)
- **Encryption**: AES-256-GCM (aes-gcm + ring crates)
- **HTTP Client**: reqwest (for LLM APIs)
- **Sidecar**: Node.js 20+ (Playwright, MCP)

### Runtime Frontend
- **UI**: React 18 + TypeScript
- **Components**: shadcn/ui + Tailwind CSS
- **Markdown**: react-markdown + syntax highlighting
- **State**: React hooks (useState, useEffect)

### Shared Packages
- **skillpack-rs**: Encryption, pack/unpack (Rust)
- **model-adapters**: LLM API adapters (future TS package)

## Supported Models

### Latest Cutting-Edge Models (2026)

**Anthropic Claude**:
- Claude 4.6 Sonnet (latest, best reasoning)

**OpenAI**:
- o1 (latest reasoning model)
- GPT-5.3-Codex (latest coding model, 2026)

**Chinese Leading Models**:
- **MiniMax M2.5** (SWE-Bench 80.2%, code generation)
- **GLM-4** (Zhipu AI, strong Chinese comprehension)
- **DeepSeek V3** (math and reasoning)
- **Qwen 2.5** (Alibaba Cloud, multilingual)
- **Moonshot Kimi** (long context)

**Custom Endpoints**: Any OpenAI-compatible API

## Project Structure

```
skillhub/
├── apps/
│   ├── runtime/               # User-facing application
│   │   ├── src/              # React frontend
│   │   ├── src-tauri/        # Rust backend
│   │   │   ├── src/
│   │   │   │   ├── agent/    # Agent system (executor, tools, registry)
│   │   │   │   ├── adapters/ # LLM adapters (Anthropic, OpenAI)
│   │   │   │   ├── commands/ # Tauri commands (skills, chat, models, mcp)
│   │   │   │   └── db.rs     # SQLite schema
│   │   │   └── tests/        # Integration tests
│   │   └── sidecar/          # Node.js sidecar (Playwright, MCP)
│   └── studio/               # Creator application (future)
├── packages/
│   └── skillpack-rs/         # Encryption library (Rust)
├── docs/                     # Documentation
├── reference/                # Open-source project analysis
└── examples/                 # Example Skills
```

## Getting Started

### Prerequisites

- Rust 1.75+
- Node.js 20+
- pnpm

### Development

```bash
# Install dependencies
pnpm install

# Run Runtime in dev mode
pnpm runtime

# Build for production
pnpm build:runtime

# Run tests
cd apps/runtime/src-tauri
cargo test
```

### Installing a Skill

1. Open Runtime application
2. Click "Install Skill" or drag `.skillpack` file to window
3. Enter username (used for decryption key derivation)
4. Configure API keys if needed
5. Start chatting!

## Roadmap

### Milestone 1: Agent Runtime MVP ✨ (Current Focus)

**Core Agent Capabilities** (80% Complete):
- [x] ReAct loop executor with Tool trait abstraction
- [x] File operations: Read, Write, Glob, Grep, Edit
- [x] Bash/PowerShell execution with cross-platform support
- [x] Sub-Agent system (Task tool) for parallel task distribution
- [x] TodoWrite for task management and memory
- [x] Context compression (token budget management)
- [x] Web Search (DuckDuckGo)
- [x] WebFetch for URL content retrieval
- [x] AskUser for interactive user input
- [x] Tool output truncation (30k char limit)
- [x] Permission system (planned, multi-layer validation)
- [ ] Local secure workspace folder configuration
- [ ] MCP server dynamic registration UI (70% - backend done)

**Skill System**:
- [x] Skill YAML frontmatter parsing
- [x] .skillpack encryption/decryption (Rust)
- [x] Install, list, delete Skill commands
- [x] Dynamic Skill loading from `.claude/skills/` directory
- [x] Skill-based system prompt injection
- [ ] Hot reload during development

**Sidecar Integration**:
- [x] Node.js sidecar manager (lifecycle control)
- [x] Hono HTTP server (localhost:8765)
- [ ] Playwright browser automation (15+ tools)
- [x] MCP client integration (connect, list tools, invoke)
- [ ] Browser controller with normalized coordinates

**Multi-Model Support**:
- [x] Anthropic Messages API adapter (Claude models)
- [x] OpenAI-compatible adapter (GPT, MiniMax, DeepSeek, etc.)
- [x] Reasoning content filtering (DeepSeek, MiniMax)
- [x] Model configuration UI (API key, base URL, model name)
- [x] 9 provider presets (Claude, OpenAI, MiniMax, DeepSeek, Qwen, Moonshot, GLM, Yi, Custom)

**User Interface**:
- [x] Chat view with streaming messages
- [x] Markdown rendering with syntax highlighting
- [x] Tool call visualization cards
- [x] Sub-Agent nested display
- [x] Session history sidebar
- [x] Settings view (models, MCP servers)
- [x] AskUser interactive input cards
- [ ] File upload support
- [ ] Secure workspace configuration UI

### Milestone 2: Distribution & Updates 🚀

**Auto-Update**:
- [ ] Application auto-update mechanism (Tauri updater)
- [ ] Update server infrastructure
- [ ] Version check and notification
- [ ] Background download and install

**Skill Version Control**:
- [ ] Skill versioning system (semver)
- [ ] Upgrade/downgrade capabilities
- [ ] Dependency resolution
- [ ] Breaking change detection

**Packaging & Installers**:
- [ ] Windows: NSIS installer + code signing
- [ ] macOS: DMG + notarization
- [ ] Linux: AppImage + deb/rpm packages

**Distribution**:
- [ ] Official download server
- [ ] Mirror CDN setup
- [ ] Update channels (stable, beta, dev)

### Milestone 3: Ecosystem & Enterprise 🏢

**Creator Tools (Studio Application)**:
- [ ] Monaco Editor integration
- [ ] Skill structure visual editor
- [ ] Embedded testing chat (Claude Code integration)
- [ ] One-click packaging UI
- [ ] Template library
- [ ] Publishing workflow

**Marketplace**:
- [ ] Web-based Skill marketplace
- [ ] Search and browse functionality
- [ ] User reviews and ratings
- [ ] Payment integration (Stripe/Alipay)
- [ ] Creator analytics dashboard

**Enterprise Features** (Inspired by enterprise agent architecture):
- [ ] User registration and authentication (JWT)
- [ ] Multi-tenant support (team workspaces)
- [ ] Unified model configuration management
- [ ] Usage quota and billing
- [ ] Admin dashboard with analytics
- [ ] SSO integration (LDAP, OAuth)
- [ ] Audit logging and compliance
- [ ] Private Skill repositories
- [ ] Role-based access control (RBAC)
- [ ] Resource usage monitoring

### Milestone 4: Agent Evolution & Ecosystem Integration 🧬

**EvoMap Integration** (Agent Self-Evolution):
- [ ] GEP (Genome Evolution Protocol) support
- [ ] Gene and Capsule data structures
- [ ] Six-step evolution cycle (Scan → Signal → Intent → Mutate → Validate → Solidify)
- [ ] A2A (Agent-to-Agent) protocol client
- [ ] Automatic capability inheritance from global gene pool
- [ ] Local evolution history and audit logs
- [ ] 70/30 resource allocation (repair vs exploration)

**OpenClaw Ecosystem Integration**:
- [ ] ClawHub Skill marketplace browser
- [ ] One-click Skill import from ClawHub
- [ ] Skill quality scoring and security scanning
- [ ] Community Skill discovery and installation

**Remote Access via IM** (Instant Messaging Integration):
- [ ] WeChat Work / DingTalk bot adapters
- [ ] Secure command relay with authentication
- [ ] Mobile-to-desktop Skill execution
- [ ] Task status notification and streaming results
- [ ] Multi-user permission isolation

## Why "SkillMint"?

**Skill**: The core unit of AI capability - a packaged, reusable instruction set
**Mint**: To create and distribute (like minting coins or NFTs)

Think of it as **"Minting AI Skills"** - create, package, and distribute Skills as easily as npm packages.

## Inspiration

Similar to how Cursor and Claude Code democratized AI-assisted coding, SkillMint aims to democratize AI Skill distribution. Package your expertise once, distribute securely to thousands.

## Future Integration Roadmap

**Agent Evolution**:
- EvoMap's GEP (Genome Evolution Protocol) and A2A communication
- Agent capability inheritance and evolution mechanisms

**Ecosystem Integration**:
- ClawHub marketplace integration strategies
- Community Skill discovery and distribution

## License

Apache 2.0 - see [LICENSE](LICENSE)

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Community

- GitHub Issues: Bug reports and feature requests
- Documentation: [docs/](docs/)
- Examples: [examples/](examples/)
- Reference: [reference/](reference/) - Open-source project analysis

---

**Built with Tauri, React, and Rust** | Inspired by Claude Code, Gemini CLI, and the open-source Agent community
