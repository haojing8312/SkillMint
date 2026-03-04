# 飞书 IM 闭环桥接（桌面会话 <-> 飞书群）

本文档说明运行时如何把智能体员工在桌面端的执行过程，持续桥接回飞书群，并在飞书侧完成 `ask_user` 闭环。

## 事件闭环

1. 飞书入站消息触发 `im-role-dispatch-request`
2. 桌面端 `App.tsx` 收到后：
   - 普通阶段调用 `send_message`
   - `ask_user` 等待阶段调用 `answer_user_question`
3. 执行过程中的 `stream-token` 被聚合后转发为 `send_feishu_text_message`
4. 收到 `ask-user-event` 时，立即把澄清问题发回飞书群
5. 用户在飞书回复后再次进入 `im-role-dispatch-request`，并回填到 `answer_user_question`

对应实现入口：`apps/runtime/src/App.tsx`

## 流式转发策略（防刷屏 + 保实时）

- 文本聚合阈值：`STREAM_CHUNK_SIZE = 120`
- 时间节流窗口：`STREAM_FLUSH_INTERVAL_MS = 1200`
- 单条飞书消息最大长度：`1800`

策略说明：

- token 持续进入缓冲区，达到阈值或定时器到点会 flush
- 连续 flush 受 1200ms 窗口节流，避免高频调用飞书发送接口
- `done=true` 与 `ask-user-event` 会强制 flush，确保关键节点即时可见

## 子智能体（sub-agent）可见性

为支持“项目经理委派开发团队”场景，`sub_agent=true` 的流式 token 也参与飞书桥接，不再被忽略。

这保证了：

- 桌面端出现委派流式输出时，飞书端也有同步进度
- 飞书用户不会只在最后一步才看到回包

## 自动化覆盖

核心回归测试：`apps/runtime/src/__tests__/App.im-feishu-bridge.test.tsx`

覆盖点：

- `ask_user` 提问转发 + 后续回复走 `answer_user_question`
- 流式 token 转飞书
- `sub_agent` token 转飞书
- 节流窗口内不高频发送，窗口后补刷
- “委派流式 -> 需求澄清 -> 用户回复”闭环

执行命令：

```bash
pnpm --dir apps/runtime test -- App.im-feishu-bridge.test.tsx
```

## 常见问题排查

1. 飞书端完全无流式输出
   - 检查是否收到 `stream-token` 事件
   - 检查 `send_feishu_text_message` 调用是否成功
2. 只在结束时才看到一条消息
   - 检查是否处于节流窗口（默认 1200ms）
   - 检查 token 是否持续写入缓冲区
3. 澄清问题只弹桌面 UI，不回飞书
   - 检查 `ask-user-event` 是否命中 IM 桥接 session
   - 检查 `suppressAskUserPrompt` 是否仅影响桌面提示，不影响飞书转发
