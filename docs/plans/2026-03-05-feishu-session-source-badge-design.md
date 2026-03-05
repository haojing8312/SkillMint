# Feishu Session Source Badge Design

## Goal
在桌面端会话列表中明确标识“来自飞书同步”的会话，降低用户对会话来源的识别成本。

## UX Decision
- 采用会话项右侧小徽标方案（`飞书`）。
- 会话标题保持原样，不引入前缀污染。
- 仅对飞书同步会话展示徽标，本地会话不占位。

## Data Contract
- `get_sessions` 返回新增字段：
  - `source_channel`: `feishu | local`
  - `source_label`: `飞书 | ""`
- `search_sessions` 返回同样字段，确保搜索态和默认列表态一致。

## UI Rules
- 侧边栏会话项标题右侧显示徽标。
- 徽标文案默认 `飞书`，支持后端下发 `source_label` 覆盖。
- 提示文案：`来自飞书会话同步`。

## Edge Cases
- 历史会话中只要存在 `im_thread_sessions.session_id = sessions.id` 视为飞书来源。
- 删除会话后重建不影响来源标记逻辑，来源由绑定关系动态决定。

## Verification
- 组件测试：`Sidebar` 在 `source_channel=feishu` 时显示徽标，本地会话不显示。
- 类型检查：`tsc --noEmit` 通过。
- 后端回归：`test_search_sessions` 通过。
