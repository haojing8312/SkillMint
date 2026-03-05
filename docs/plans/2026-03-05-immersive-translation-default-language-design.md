# 默认语言与沉浸式翻译设计

## 1. 背景与目标

当前 WorkClaw 在“专家技能”域中已经有局部翻译能力（技能库页面），但“找技能”和聊天里的安装候选卡片仍可能直接展示英文元数据，导致中文用户体验割裂。  
本设计目标是在不影响现有安装/检索/路由链路的前提下，新增可配置的“默认语言 + 默认沉浸式翻译”能力，并沉淀为跨场景可复用基础设施。

## 2. 范围

### 2.1 本期范围（MVP）

- 新增全局偏好：
  - `default_language`（默认 `zh-CN`）
  - `immersive_translation_enabled`（默认 `true`）
  - `immersive_translation_display`（默认 `translated_only`，可扩展）
- 抽象通用翻译命令，支持按目标语言批量翻译与缓存。
- 前端新增通用翻译 Hook/渲染策略，统一应用于：
  - `SkillLibraryView`
  - `FindSkillsView`
  - `ChatView` 中由 `clawhub_search`/`clawhub_recommend` 渲染的安装候选卡片

### 2.2 非本期范围

- 完整 i18n 文案体系改造（应用静态文案多语言）
- 所有工具输出全文自动翻译
- 术语库、记忆化风格迁移、人工校对工作流

## 3. 设计原则

1. 翻译仅影响“展示层”，不改变业务标识（slug/id/path/raw query）。
2. 默认自动翻译，失败回退原文，不阻塞关键流程（检索、安装、更新）。
3. 后端统一能力，前端统一接入，避免每个页面重复实现。
4. 与现有数据结构兼容，优先增量演进，避免破坏现有接口和测试。

## 4. 现状与问题定位

## 4.1 已有能力

- 后端已有 `translate_clawhub_texts(texts)`，目标固定 `zh-CN`，并有 `skill_i18n_cache` 缓存表。
- `SkillLibraryView` 已批量调用该命令翻译 `name/summary/tags`。

## 4.2 缺口

- “找技能”结果（`FindSkillsView`）未走统一翻译链路。
- 聊天中“可安装技能”卡片（`ChatView`）未走统一翻译链路。
- 运行时偏好中尚无“默认语言/沉浸式翻译”配置。
- 翻译服务命名与语义绑定 ClawHub 场景，复用边界不清晰。

## 5. 方案对比

### 方案 A：页面各自调用现有翻译接口

- 优点：改造快，风险低。
- 缺点：重复代码多，后续扩展到更多场景会持续复制粘贴。

### 方案 B：统一翻译服务 + 前端通用 Hook（推荐）

- 优点：跨场景复用、开关和语言策略一致、维护成本低。
- 缺点：首轮改造面稍大，需要补齐偏好配置与兼容层。

### 方案 C：仅通过 Agent 回复层“顺带翻译”

- 优点：实现最省。
- 缺点：不可控、结构化字段不稳定、无法保证 UI 一致与缓存复用。

结论：采用方案 B。

## 6. 架构设计

## 6.1 运行时偏好扩展

在 `runtime_preferences` 基础上扩展字段：

- `default_language: String`（默认 `zh-CN`）
- `immersive_translation_enabled: bool`（默认 `true`）
- `immersive_translation_display: String`（默认 `translated_only`，预留 `bilingual_inline`）

偏好持久化仍基于 `app_settings`，新增 key：

- `runtime_default_language`
- `runtime_immersive_translation_enabled`
- `runtime_immersive_translation_display`

## 6.2 后端翻译服务通用化

新增通用命令（示例名）：

- `translate_texts_with_preferences(texts: Vec<String>, scene: Option<String>)`

行为：

1. 读取运行时偏好；
2. 若翻译关闭，直接回传原文；
3. 对每条文本执行：
   - 空文本直接返回；
   - 若已是目标语言（例如 CJK 主体且目标为 `zh-CN`），跳过翻译；
   - 查询缓存（cache key 含 target language + source text hash）；
   - 未命中则调用翻译引擎；
   - 失败回退原文；
4. 返回与输入等长数组，顺序严格对应。

兼容策略：

- 现有 `translate_clawhub_texts` 保留，内部复用新通用函数，避免前端一次性大改。

## 6.3 前端通用接入

新增 Hook（示例）：

- `useImmersiveTranslation(texts: string[], options?)`

返回：

- `translatedMap`
- `isTranslating`
- `mode`（显示策略）

渲染策略：

- `translated_only`：优先译文，缺失回退原文。
- `bilingual_inline`：主文显示译文，次文小字展示原文（后续可切换）。

## 6.4 场景落位

1. `SkillLibraryView`：
   - 替换现有局部翻译逻辑为通用 Hook（行为保持一致）。
2. `FindSkillsView`：
   - 翻译 `name/description/reason`，安装弹窗 title/summary 使用译文主显示。
3. `ChatView` 安装候选卡片：
   - 翻译 `candidate.name/description`，不影响 `slug` 和安装参数。

## 7. 数据流

1. 页面收集候选文本（去重）。
2. 前端调用统一翻译命令。
3. 后端读取偏好并执行缓存/翻译。
4. 前端按显示模式渲染译文（必要时附原文）。
5. 用户执行安装操作时仍传原始业务字段（slug/github_url）。

## 8. 错误处理与回退

- 翻译接口失败：页面静默回退原文，不影响交互可用性。
- 偏好读取异常：使用默认值（`zh-CN + enabled=true`）。
- 缓存写入失败：不阻塞返回，仅记录日志（如后续加日志）。
- 翻译时延：采用分批请求与去重，避免一次大批量阻塞主线程。

## 9. 测试设计

## 9.1 Rust 单元/集成

- `runtime_preferences`：
  - 默认值返回正确；
  - 设置并读取新字段正确；
  - 兼容旧数据（缺失字段）正确回退默认值。
- 翻译服务：
  - 关闭翻译时返回原文；
  - 缓存命中/未命中逻辑正确；
  - 输入输出顺序与长度一致；
  - 非英文文本跳过翻译。

## 9.2 前端测试（Vitest）

- `SkillLibraryView`：开启翻译时展示译文；接口失败回退原文。
- `FindSkillsView`：推荐卡片与安装确认展示译文。
- `ChatView`：工具输出解析后安装卡片展示译文，安装参数仍为原始 slug。

## 10. 风险与缓解

- 风险：翻译 API 不稳定。  
  缓解：缓存 + 失败回退 + 不阻塞流程。

- 风险：显示层翻译误改业务字段。  
  缓解：明确“只翻显示字段”，安装链路参数维持原值，并加测试保护。

- 风险：改造影响现有页面性能。  
  缓解：去重、分批、增量更新 `map`。

## 11. 验收标准

1. 新安装用户默认语言为简体中文，沉浸式翻译默认开启。
2. 技能库、找技能、聊天安装候选卡片出现英文时自动显示中文译文。
3. 翻译失败时不影响检索/安装流程。
4. 安装与更新流程的业务参数未被翻译污染（slug 等保持原始值）。
5. 现有相关测试通过，并新增覆盖默认配置与关键翻译链路。
