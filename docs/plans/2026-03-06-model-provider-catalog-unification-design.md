# 模型服务商目录统一与自定义接入设计

## 1. 目标

解决首次安装“快速配置模型”与设置页“模型连接”列表不一致的问题，并补齐：

1. 首次向导与设置页展示同一份完整服务商清单
2. 选中服务商后，在右侧显示官方 API Key 申请/控制台入口
3. 明确支持两类自定义接入：
   - 自定义 OpenAI 兼容
   - 自定义 Claude (Anthropic)

## 2. 当前问题

当前实现中：

- `apps/runtime/src/App.tsx` 的首次快速向导维护一份 `QUICK_MODEL_PRESETS`，只有 4 个模板
- `apps/runtime/src/components/SettingsView.tsx` 的设置页维护另一份 `PROVIDER_PRESETS`

这导致：

1. 首次接入时可选服务商明显少于设置页
2. 新增服务商时容易漏改其中一处
3. 没有统一的官方申请入口与服务商说明
4. 第三方中转 / 私有网关接入缺少清晰分类

## 3. 设计原则

1. 单一真相源：服务商目录只维护一份
2. 体验一致：首次向导与设置页用同一套服务商定义、默认值和说明
3. 协议优先：第三方中转按接口协议归类，不按厂商硬编码
4. 兼容旧数据：不改数据库结构，不做迁移
5. 渐进扩展：未来新增服务商时，只改统一目录即可生效到两处界面

## 4. 方案选择

### 方案 A：分别维护两份清单，仅补齐首次向导

优点：

- 改动最小

缺点：

- 根因未解决
- 后续仍会再次不一致

### 方案 B：前端统一服务商目录，两个入口共用

优点：

- 一次解决一致性问题
- 改动范围可控
- 不需要改后端数据结构

缺点：

- 目录仍是前端内置数据，未来如果要接后端动态模型发现，需要再演进

### 方案 C：由 Tauri 后端提供内置服务商目录

优点：

- 前后端共享单一目录
- 更利于后续扩展动态发现

缺点：

- 本次范围过大
- 需要同步修改 Rust 命令、前端加载逻辑和测试桩

### 结论

本次采用方案 B：新增前端统一服务商目录模块，由首次向导和设置页共同消费。

## 5. 统一服务商目录

新增文件建议：

- `apps/runtime/src/model-provider-catalog.ts`

每个服务商项包含以下信息：

- `id`：稳定标识，如 `zhipu`、`openai`、`custom-openai`
- `label`：界面展示名称
- `name`：默认连接名称
- `providerKey`：供现有映射和后续扩展复用
- `apiFormat`：`openai` 或 `anthropic`
- `baseUrl`
- `defaultModel`
- `models`：推荐模型列表
- `badge`：如“国内直连”“长文推理”“第三方中转”
- `helper`：服务商说明文案
- `officialConsoleUrl`：官方控制台 / API Key 申请页
- `officialDocsUrl`：官方文档页，可选
- `supportsCustomBaseUrl`
- `supportsCustomModelName`
- `isCustom`

首批目录按当前产品已支持的直连服务商收齐：

- 智谱 GLM
- OpenAI
- Claude (Anthropic)
- MiniMax (OpenAI 兼容)
- MiniMax (Claude 兼容)
- DeepSeek
- Qwen（国际）
- Qwen（国内）
- Moonshot / Kimi
- Yi
- 自定义 OpenAI 兼容
- 自定义 Claude (Anthropic)

## 6. 自定义接入设计

参考 openclaw 的协议视角，自定义接入不再按具体厂商枚举，而是固定为两类：

### 6.1 自定义 OpenAI 兼容

适用场景：

- OpenRouter
- One API / New API
- 其他第三方中转站
- 企业私有 OpenAI 兼容网关

特征：

- `apiFormat = openai`
- `baseUrl` 必须可编辑
- `model_name` 必须可编辑

### 6.2 自定义 Claude (Anthropic)

适用场景：

- Anthropic Messages API 兼容代理
- 第三方 Claude 中转
- 企业私有 Anthropic 兼容网关

特征：

- `apiFormat = anthropic`
- `baseUrl` 必须可编辑
- `model_name` 必须可编辑

### 6.3 自定义项的右侧说明

自定义项不显示“官方购买”按钮，改为说明块：

- 向你的中转 / 代理服务商申请 API Key
- 确认其支持的接口类型、Base URL、模型名
- OpenAI 兼容通常使用 `/v1`
- Claude 兼容应符合 Anthropic Messages API 风格

## 7. UI 交互设计

### 7.1 服务商选择

首次向导与设置页统一使用同一份完整服务商清单：

- 不再区分“快速推荐 4 个”和“完整版”
- 顺序固定：官方直连在前，自定义在后
- 自定义两项作为正式选项直接展示

### 7.2 右侧表单区

选中服务商后，右侧立即展示：

- 服务商名称
- 协议类型标签：`OpenAI 兼容` / `Claude (Anthropic)`
- `获取 API Key` 或 `前往控制台` 按钮
- 可选 `查看接入文档` 按钮
- 服务商说明文案

### 7.3 字段联动

- `连接名称`：默认带出服务商名，允许用户修改
- `Base URL`
  - 官方服务商：默认填充官方地址
  - 自定义服务商：必须允许编辑，显示示例 placeholder
- `模型名`
  - 官方服务商：默认填充推荐模型
  - 允许从推荐列表选择，也允许手工覆盖
  - 自定义服务商：默认值仅作示例，必须允许编辑
- `API 格式`
  - 官方服务商由目录确定
  - 自定义两项分别固定为 `openai` / `anthropic`

### 7.4 首次向导与设置页的关系

两处差异仅保留在外层容器和文案语气：

- 首次向导：强调“测试连接”“保存并开始”
- 设置页：强调“新增 / 编辑 / 设为默认”

但服务商目录、默认值、右侧说明、官方入口按钮、推荐模型全部一致。

## 8. 数据流与兼容性

本次不修改数据库结构，也不修改现有 Tauri 命令签名：

- `save_model_config`
- `test_connection_cmd`

底层仍保存以下字段：

- `name`
- `api_format`
- `base_url`
- `model_name`
- `api_key`

新增的只是前端目录与映射逻辑：

1. 两个入口都从统一目录生成表单默认值
2. 编辑已有配置时，按 `api_format + base_url` 回推目录项
3. 若回推不到：
   - `openai` 归到 `自定义 OpenAI 兼容`
   - `anthropic` 归到 `自定义 Claude (Anthropic)`

这样旧配置无需迁移，用户已有第三方地址仍可正常编辑和保存。

## 9. 错误处理

### 9.1 官方服务商

- 默认提供官方 `Base URL` 和推荐模型
- 用户仍可改写这些字段
- 连接测试失败时继续复用现有错误提示

### 9.2 自定义服务商

在前端增加更明确的校验：

- `Base URL` 为空：禁止测试和保存
- `模型名` 为空：禁止测试和保存
- `API Key` 为空：维持当前禁止测试和保存的逻辑

### 9.3 外链按钮

- 有官方入口时显示按钮
- 无官方入口时不显示按钮，仅显示说明块

## 10. 测试策略

### 10.1 首次向导测试

扩展现有：

- `apps/runtime/src/__tests__/App.model-setup-hint.test.tsx`

覆盖：

- 快速向导展示完整服务商列表
- 切换服务商后表单字段联动正确
- 选择官方服务商时显示官方入口按钮
- 选择自定义 OpenAI 时显示自定义说明
- 选择自定义 Claude 时测试 / 保存走 `anthropic`

### 10.2 设置页测试

新增：

- `apps/runtime/src/components/__tests__/SettingsView.model-providers.test.tsx`

覆盖：

- 设置页服务商清单与首次向导一致
- 右侧展示官方入口按钮
- 编辑已有自定义配置时能正确回落到自定义项并保留原始 `base_url`
- 推荐模型列表随服务商切换更新

### 10.3 纯函数测试

新增目录 helper 测试，避免 `App` 与 `SettingsView` 各自维护一套匹配逻辑：

- 目录项转表单默认值
- 由 `api_format + base_url` 反推目录项

## 11. 验收标准

1. 首次向导与设置页展示完全一致的服务商列表
2. 每个官方服务商在右侧显示官方 API Key 获取入口
3. 支持 `自定义 OpenAI 兼容` 与 `自定义 Claude (Anthropic)` 两项
4. 旧配置可继续编辑，不要求迁移
5. 首次向导测试与设置页新增测试均通过
