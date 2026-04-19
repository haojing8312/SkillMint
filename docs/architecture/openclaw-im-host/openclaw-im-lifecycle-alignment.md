# OpenClaw IM Lifecycle Alignment

本文档记录 WorkClaw 当前对齐 OpenClaw 官方 IM reply lifecycle 的实现语义。

## 官方顺序

OpenClaw 官方飞书链路强调以下顺序：

1. `dispatchReplyFromConfig`
2. `waitForIdle`
3. `markFullyComplete`
4. `markDispatchIdle`

其中关键点不是“开始发送了最终回复”，而是“所有排队中的 deliver 都已经 flush 完成后，才能宣告 fully complete”。

## WorkClaw 当前对齐

### plugin-host

文件：

- `apps/runtime/plugin-host/src/runtime.ts`
- `apps/runtime/plugin-host/scripts/run-feishu-host.mjs`

当前 host 会显式发出 `reply_lifecycle` 事件：

- `reply_started`
- `processing_started`
- `ask_user_requested`
- `ask_user_answered`
- `approval_requested`
- `approval_resolved`
- `interrupt_requested`
- `resumed`
- `failed`
- `stopped`
- `tool_chunk_queued`
- `block_chunk_queued`
- `final_chunk_queued`
- `wait_for_idle`
- `idle_reached`
- `fully_complete`
- `dispatch_idle`
- `processing_stopped`

### 时序保证

`withReplyDispatcher` 已按以下顺序执行：

1. `run()`
2. `dispatcher.waitForIdle()`
3. `dispatcher.markComplete()`
4. `onSettled()`

这意味着 WorkClaw 不再走“先 complete 再 idle”的危险路径。

另外，当前前端 `latest_reply_completion` 的完成态投影已经进一步收紧：

- `fully_complete` 不再直接投影为 `Completed`
- 只有 `dispatch_idle` 才会被前端和诊断层视为“这条回复真正结束”

这让 WorkClaw 的可观测完成语义更接近 OpenClaw 官方“flush 结束后再宣告完成”的边界。

## Runtime Host Observability

文件：

- `apps/runtime/src-tauri/src/commands/openclaw_plugins/runtime_service.rs`
- `apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`

当前 Tauri runtime 会：

- 解析 `reply_lifecycle` stdout 事件
- 将其写入 `OpenClawPluginFeishuRuntimeStatus.recent_reply_lifecycle`
- 同步写入 `recent_logs`

这样前端和诊断工具已经可以观察到“处理中开始了没有、idle 到了没有、dispatch idle 到了没有”。

同时，前端设置页已开始把恢复态单独展示出来：

- `ask_user_answered`
- `approval_resolved`
- `resumed`

这些 phase 仍然投影在 `running` 大类下，但 UI 不再把它们和普通“处理中”混为一谈，而是显示为“已恢复处理中”，帮助操作者区分“仍在初始处理中”与“已经收到继续执行所需信息，正在恢复推进”。

## 仍未完全对齐的部分

以下能力还需要下一步继续补：

- 在无环境阻塞的机器上真正执行新增的 `im_host` Rust lifecycle 回归，而不只是完成 compile-level 证明
- 继续把 completion / delivery / diagnostics 的统一证据沉淀为更接近阶段验收的总结，而不是分散在多份计划与附录里
- 如果要宣告“完全对齐”，仍需要对 `waitForIdle -> markFullyComplete -> markDispatchIdle` 的最终完成语义做更窄、更强的持续回归

## 新增对齐进展

截至本轮实现：

- `processing_started` / `processing_stopped` 已真正映射到飞书官方 `Typing` reaction 的启停
- Feishu `ask_user` 已改为宿主发送，前端不再直接往飞书线程代发澄清问题
- Feishu `approval_requested` 在发出审批消息前会先结束 processing reaction，避免“还在处理中”与“等待审批”同时显示
- 宿主 stdin 协议已支持通用 `lifecycle_event` 命令，可显式发送 `ask_user_requested / approval_requested`
- `run_failed / run_stopped` 已开始映射为独立 lifecycle phase，而不再只依赖 `processing_stopped + finalState`
- `answer_user_question / resolve_approval` 已开始映射为 `ask_user_answered / approval_resolved`
- `cancel_agent(session_id?)` 已开始映射为 `interrupt_requested`
- `ask_user` 收到回答、`approval_flow` 收到决策后，runtime 恢复执行会映射为 `resumed`
- Feishu interactive 闭环现在已有更硬的宿主级顺序保证：
  - 进入等待时，先 `processing_stopped`，再发 `ask_user_requested / approval_requested`
  - 恢复执行时，`ask_user_answered / approval_resolved / resumed` 会继续路由到注册宿主，而不是只停留在桌面本地状态
- 同一套 `im_host` contract 已进一步在企业微信上获得结构性证明：
  - WeCom `ask_user_requested / approval_requested` 进入等待时，也遵守“先停止 processing、再发 waiting lifecycle”的统一顺序
  - WeCom `ask_user_answered / approval_resolved / resumed` 也能继续经由统一宿主路由到 connector host
  - WeCom final reply 也已具备 `maybe_dispatch_registered_im_session_reply_with_pool(...)` 的 host-level 统一分发路径
- 设置页已经能把恢复态展示为“已恢复处理中”，不再把 `ask_user_answered / approval_resolved / resumed` 压扁成和普通 `running` 完全相同的可见状态
- 前端统一渠道设置页现在也能直接证明：Feishu 与 WeCom 的宿主启停都走同一条 `set_im_channel_host_running` channel host command，而不是保留 WeCom 私有入口

这意味着 WorkClaw 已不只是“Feishu 上 final answer 对齐 OpenClaw”，而是已经开始把 `final / ask_user / approval` 三类关键出站路径收束到统一宿主层，并把这套 contract 向 WeCom 证明为可复用平台能力。

## 当前结论

WorkClaw 现在已经从“只有 send_result 的单点返回模型”，走到“具备官方 lifecycle 语义、idle barrier 与多渠道宿主抽象的基础层”。

按当前证据可把状态概括为：

- Feishu 主线已经基本完成从前端 reply orchestration 向宿主层的迁移
- WeCom 已不再只是概念性接入，而是拿到了 unified `im_host` contract 的等待态、恢复态、final reply 与统一宿主控制证据
- 剩余工作主要是把新增 Rust 回归在无环境问题的机器上真正执行完，并形成更正式的阶段验收结论
