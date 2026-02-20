# SkillMint

[English](README.md) | [简体中文](README.zh-CN.md)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://reactjs.org/)

**SkillMint** 是一个开源的 AI Skill 打包与桌面应用发布平台。将你的 Skill 快速转化为加密的、可分发的桌面应用程序。

## 什么是 SkillMint？

SkillMint 帮助 Skill 创作者：
- **打包**：编写或导入现有 Skill，加密打包成 `.skillpack` 文件
- **保护**：AES-256-GCM 加密保护你的知识产权
- **分发**：用户通过统一的 Runtime 客户端安装 Skill
- **变现**：控制你的优质 Skill 的访问权限和分发渠道

就像 **"将你的 Skill 铸造成可分发的资产"** —— 正如铸币厂将原材料转化为有价值的货币。

## 架构

SkillMint 由两个独立的桌面应用组成：

### Studio（创作者端）
- 导入现有 Skill 目录或从零编写
- Monaco 编辑器，支持 Markdown
- 内置对话测试界面
- 一键加密打包为 `.skillpack`
- 多模型测试（Claude、GPT、MiniMax、DeepSeek、Qwen 等）

### Runtime（用户端）
- 通过拖拽安装 `.skillpack` 文件
- 简洁的对话界面，支持 Markdown 渲染
- 会话历史和管理
- 模型选择和 API Key 管理
- 无需命令行，无需技术知识

## 核心特性

- **知识产权保护**：Skill 内容使用 AES-256-GCM 加密
- **多模型支持**：Anthropic Messages API + OpenAI 兼容 API
- **轻量级**：~30MB Runtime，~50MB Studio（基于 Tauri）
- **跨平台**：Windows、macOS、Linux
- **安全**：API Key 存储在系统密钥链中
- **开源**：Apache 2.0 许可证

## 技术栈

- **桌面框架**：Tauri 2.0
- **前端**：React 18 + TypeScript + shadcn/ui + Tailwind CSS
- **编辑器**：Monaco Editor
- **后端**：Rust
- **数据库**：SQLite
- **加密**：AES-256-GCM（Rust `aes-gcm` + `ring`）

## 支持的模型

### Anthropic Messages API
- Claude 3.5 Sonnet
- Claude 3.5 Haiku
- Claude 3 Opus

### OpenAI 兼容 API
通过配置不同的 Base URL 支持：
- OpenAI GPT-4、GPT-3.5
- MiniMax M2.5（SWE-Bench 80.2%）
- DeepSeek
- Qwen / 通义千问（阿里云）
- Moonshot Kimi
- GLM / 智谱清言
- 自定义端点

## 项目结构

```
skillhub/
├── apps/
│   ├── studio/           # 创作者端桌面应用
│   │   ├── src/          # React 前端
│   │   └── src-tauri/    # Rust 后端
│   └── runtime/          # 用户端桌面应用
│       ├── src/          # React 前端
│       └── src-tauri/    # Rust 后端
├── packages/
│   ├── ui/               # 共享 UI 组件
│   ├── skill-core/       # Skill 解析/验证（TypeScript）
│   ├── model-adapters/   # 模型适配器（TypeScript）
│   └── skillpack-rs/     # SkillPack 格式与加密（Rust）
├── docs/                 # 文档
└── examples/             # 示例 Skill
```

## 快速开始

### 前置要求

- Rust 1.75+
- Node.js 20+
- pnpm

### 开发

```bash
# 安装依赖
pnpm install

# 以开发模式运行 Studio
pnpm --filter @skillmint/studio tauri dev

# 以开发模式运行 Runtime
pnpm --filter @skillmint/runtime tauri dev

# 生产构建
pnpm build
```

## 路线图

### 里程碑 1：核心 MVP
- [x] Monorepo 脚手架
- [x] skillpack-rs Rust crate（加密/解密）
- [ ] Studio：导入现有 Skill 目录
- [ ] Studio：Skill 编辑器（Monaco）+ Front Matter 表单
- [ ] Studio：内嵌对话测试
- [ ] Studio：一键打包为 `.skillpack`
- [ ] Runtime：安装 `.skillpack` 文件
- [ ] Runtime：基础对话 UI 和流式输出
- [ ] Runtime：会话历史持久化
- [ ] 模型适配器：Anthropic + OpenAI 兼容

### 里程碑 2：体验完善
- [ ] Studio：内置 Skill 模板
- [ ] Studio：版本管理
- [ ] Runtime：文件上传支持
- [ ] Runtime：多会话管理
- [ ] 自动更新机制
- [ ] 安装包（Windows NSIS、macOS DMG）

### 里程碑 3：生态建设
- [ ] 官方 Skill 市场
- [ ] Studio：发布到市场
- [ ] Runtime：从市场安装
- [ ] AI 辅助生成 Skill
- [ ] Linux AppImage

## 为什么叫 "SkillMint"？

**Mint** 有双重含义：
1. **Mint（名词）**：铸币厂 —— 将原材料转化为有价值的货币
2. **Mint（动词）**：铸造/发行（NFT 文化中流行的术语 —— "铸造"数字资产）

SkillMint 将你的 Skill（原始知识产权）转化为可分发、可变现的桌面应用（有价值的资产）。

## 灵感来源

类似 [MoneyPrinterTurbo](https://github.com/harry0703/MoneyPrinterTurbo) 帮助创作者自动化视频变现，SkillMint 帮助 Skill 创作者自动化 AI 应用分发和变现。

## 开源协议

Apache 2.0 - 详见 [LICENSE](LICENSE)

## 贡献

欢迎贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 社区

- GitHub Issues：Bug 报告和功能请求
- 文档：[docs/](docs/)
- 示例：[examples/](examples/)

---

**使用 Tauri、React 和 Rust 构建**
