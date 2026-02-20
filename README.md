# SkillMint

[English](README.md) | [简体中文](README.zh-CN.md)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://reactjs.org/)

**SkillMint** is an open-source AI Skill packaging and desktop application publishing platform. Transform your Skills into encrypted, distributable desktop applications in minutes.

## What is SkillMint?

SkillMint helps Skill creators:
- **Package**: Write or import existing Skills, encrypt them into `.skillpack` files
- **Protect**: AES-256-GCM encryption protects your intellectual property
- **Distribute**: Users install Skills through the unified Runtime client
- **Monetize**: Control access and distribution of your premium Skills

Think of it as **"Mint your Skills into distributable assets"** - just like a mint turns raw materials into valuable coins.

## Architecture

SkillMint consists of two independent desktop applications:

### Studio (For Creators)
- Import existing Skill directories or write from scratch
- Monaco Editor with Markdown support
- Built-in chat testing interface
- One-click packaging to `.skillpack` with encryption
- Multi-model testing (Claude, GPT, MiniMax, DeepSeek, Qwen, etc.)

### Runtime (For End Users)
- Install `.skillpack` files via drag-and-drop
- Clean chat interface with Markdown rendering
- Session history and management
- Model selection and API key management
- No command line, no technical knowledge required

## Key Features

- **IP Protection**: Skill content encrypted with AES-256-GCM
- **Multi-Model Support**: Anthropic Messages API + OpenAI-compatible APIs
- **Lightweight**: ~30MB Runtime, ~50MB Studio (Tauri-based)
- **Cross-Platform**: Windows, macOS, Linux
- **Secure**: API keys stored in system keychain
- **Open Source**: Apache 2.0 license

## Tech Stack

- **Desktop Framework**: Tauri 2.0
- **Frontend**: React 18 + TypeScript + shadcn/ui + Tailwind CSS
- **Editor**: Monaco Editor
- **Backend**: Rust
- **Database**: SQLite
- **Encryption**: AES-256-GCM (Rust `aes-gcm` + `ring`)

## Supported Models

### Anthropic Messages API
- Claude 3.5 Sonnet
- Claude 3.5 Haiku
- Claude 3 Opus

### OpenAI-Compatible APIs
Configure different base URLs to support:
- OpenAI GPT-4, GPT-3.5
- MiniMax M2.5 (SWE-Bench 80.2%)
- DeepSeek
- Qwen (Alibaba Cloud)
- Moonshot Kimi
- GLM (Zhipu AI)
- Custom endpoints

## Project Structure

```
skillhub/
├── apps/
│   ├── studio/           # Creator-facing desktop app
│   │   ├── src/          # React frontend
│   │   └── src-tauri/    # Rust backend
│   └── runtime/          # User-facing desktop app
│       ├── src/          # React frontend
│       └── src-tauri/    # Rust backend
├── packages/
│   ├── ui/               # Shared UI components
│   ├── skill-core/       # Skill parsing/validation (TypeScript)
│   ├── model-adapters/   # Model adapters (TypeScript)
│   └── skillpack-rs/     # SkillPack format & encryption (Rust)
├── docs/                 # Documentation
└── examples/             # Example Skills
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

# Run Studio in dev mode
pnpm --filter @skillmint/studio tauri dev

# Run Runtime in dev mode
pnpm --filter @skillmint/runtime tauri dev

# Build for production
pnpm build
```

## Roadmap

### Milestone 1: Core MVP
- [x] Monorepo scaffold
- [x] skillpack-rs Rust crate (encryption/decryption)
- [ ] Studio: Import existing Skill directories
- [ ] Studio: Skill editor (Monaco) + Front Matter form
- [ ] Studio: Embedded chat testing
- [ ] Studio: One-click packaging to `.skillpack`
- [ ] Runtime: Install `.skillpack` files
- [ ] Runtime: Basic chat UI with streaming
- [ ] Runtime: Session history persistence
- [ ] Model adapters: Anthropic + OpenAI-compatible

### Milestone 2: Polish
- [ ] Studio: Built-in Skill templates
- [ ] Studio: Version management
- [ ] Runtime: File upload support
- [ ] Runtime: Multi-session management
- [ ] Auto-update mechanism
- [ ] Installers (Windows NSIS, macOS DMG)

### Milestone 3: Ecosystem
- [ ] Official Skill marketplace
- [ ] Studio: Publish to marketplace
- [ ] Runtime: Install from marketplace
- [ ] AI-assisted Skill generation
- [ ] Linux AppImage

## Why "SkillMint"?

**Mint** has a dual meaning:
1. **Mint (noun)**: A facility that produces coins - transforming raw materials into valuable currency
2. **Mint (verb)**: To create/produce (popularized by NFT culture - "minting" digital assets)

SkillMint transforms your Skills (raw intellectual property) into distributable, monetizable desktop applications (valuable assets).

## Inspiration

Similar to how [MoneyPrinterTurbo](https://github.com/harry0703/MoneyPrinterTurbo) helps creators automate video monetization, SkillMint helps Skill creators automate AI application distribution and monetization.

## License

Apache 2.0 - see [LICENSE](LICENSE)

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Community

- GitHub Issues: Bug reports and feature requests
- Documentation: [docs/](docs/)
- Examples: [examples/](examples/)

---

**Made with Tauri, React, and Rust**
