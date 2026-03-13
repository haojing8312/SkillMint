# OpenAI Thinking Filter Design

## Context

客户诊断包确认崩溃发生在 `apps/runtime/src-tauri/src/adapters/openai.rs` 的 `filter_thinking()`。当前实现为了在流式输出阶段剥离 `<think>...</think>`，会对 Rust `String` 做按字节切片；一旦流片段中包含中文等多字节 UTF-8 字符，release 构建会因为非法字符边界 panic 并直接退出。

`reference/openclaw` 和 `reference/open-claude-cowork` 的成熟实现给出的共识是：思考内容优先按结构化字段处理，文本标签剥离只做兼容层，且应在文本层完成，不能依赖不安全的字节切片。

## Approaches

### 方案 A：最小止血

仅把当前 `filter_thinking()` 改成字符安全实现，继续保留当前“在流式正文里剥离 `<think>` 标签”的行为。

优点：
- 改动最小，能最快消除闪退
- 不影响现有上层调用接口

缺点：
- 仍然把标签解析放在适配器文本流里，设计上偏脆
- 后续支持 `<thinking>`、`<final>` 等变体时仍要继续扩

### 方案 B：参考 openclaw 的分层方案，推荐

保持短期接口不变，但把实现分成两层：
- 结构化层：优先跳过 `reasoning_content` 等独立思考字段
- 文本兼容层：用字符安全状态机剥离 `<think>...</think>`，只作为 safety net

优点：
- 直接修复崩溃
- 与 `openclaw` / opencode 的处理边界一致
- 为后续把 reasoning 升级为独立事件留出空间

缺点：
- 比最小止血多一点测试和状态机代码

### 方案 C：彻底重构为 reasoning 独立事件

让适配器把思考和正文拆成不同事件，再由上层决定是否展示。

优点：
- 架构最干净
- 与成熟项目最一致

缺点：
- 牵涉消息协议、存储和渲染层，不适合当前线上热修

## Decision

采用方案 B。

第一步先在 `openai.rs` 中实现字符安全的 `<think>` 兼容过滤，保留现有外部接口和行为，优先恢复稳定性。第二步通过测试锁定边界，确保后续再演进到结构化 reasoning 时不会回归。

## Implementation Shape

- 移除 `filter_thinking()` 中所有按字节索引切 `String` 的逻辑
- 改成基于字符迭代的增量状态机
- 只把 ASCII 标签匹配缓存在小窗口中，不对正文做不安全切片
- 继续保留 `reasoning_content` 直接跳过的逻辑

## Required Tests

- 中文正文无 `<think>` 标签时不 panic，且完整透传
- `<think>...</think>` 跨 chunk 时能正确隐藏思考内容
- 标签外的中文正文能正确保留
- 未闭合 `<think>` 块不会把后续正文错误输出
- 普通尖括号文本不会被误判为 thinking 标签

## Success Criteria

- 客户诊断包中的同类输入不再触发崩溃
- 相关 Rust 单测覆盖中文和跨 chunk 场景
- 不改变普通 OpenAI 文本流和 tool call 解析行为
