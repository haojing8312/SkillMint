# OpenClaw IM Host：Phase 3 External Verification Runbook

本文档用于完成 Phase 3 最后几条执行级验证，并明确区分 Windows 专用回归入口与原始 libtest 入口。

## 背景

当前仓库已经具备以下证据：

- 前端统一渠道设置页测试已通过
- `cargo check -p runtime` 已通过
- `pnpm test:rust-fast` 已通过
- 新增 `im_host` Rust 回归已完成代码落地并编译进入 `runtime`

但本机执行大型 `cargo test --lib ...` 时仍受环境问题阻塞：

- `STATUS_ENTRYPOINT_NOT_FOUND`

为避免把环境缺陷误判成实现失败，仓库已新增 Windows 专用回归入口：

- `pnpm test:im-host-windows-regression`

并且 `pnpm verify:openclaw-im-host:phase3` 在 Windows 下会自动使用该入口。

## 本次要验证的目标

需要确认三类能力在真正执行测试时也成立：

1. WeCom 等待态顺序
2. WeCom 恢复态 lifecycle 路由
3. WeCom final reply 通过 unified host 分发

## 建议执行环境

- Windows：
  - 可直接使用 `pnpm test:im-host-windows-regression`
  - 如需执行原始 libtest，需确认该机器不存在 `STATUS_ENTRYPOINT_NOT_FOUND`
- 非 Windows 或其他能稳定执行 `runtime` Rust test binary 的机器：
  - 可继续执行原始 `cargo test --lib ...` 路径
- 与当前仓库代码保持同一提交或同一工作树状态
- 已完成 `pnpm install`
- Rust / cargo / MSVC toolchain 正常

## 建议执行顺序

如需直接按仓库脚本执行，可优先使用：

```bash
pnpm verify:openclaw-im-host:phase3
```

说明：

- Windows：该脚本会执行 `pnpm test:im-host-windows-regression`
- 其他环境：该脚本会继续执行原始 `cargo test --lib ...` 定向用例

如果当前机器只能接受 compile-level 验证，可先运行：

```bash
pnpm verify:openclaw-im-host:phase3 --compile-only
```

### 1. 基础编译确认

```bash
cargo check -p runtime
pnpm test:rust-fast
```

预期：

- 两条命令都通过

### 2. 执行新增 WeCom interactive waiting-state 回归

Windows 推荐命令：

```bash
pnpm test:im-host-windows-regression
```

非 Windows 或可稳定执行原始 libtest 的机器，可使用：

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib maybe_notify_registered_ask_user_routes_wecom_session_via_unified_host -- --nocapture

cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib maybe_notify_registered_approval_requested_routes_wecom_session_via_unified_host -- --nocapture
```

预期关注点：

- `ask_user_requested` 前会先发 processing stop
- `approval_requested` 前会先发 processing stop
- 顺序应为：
  - `processing_stop -> lifecycle`

### 3. 执行新增 WeCom 恢复态 lifecycle 回归

Windows 推荐命令：

```bash
pnpm test:im-host-windows-regression
```

非 Windows 或可稳定执行原始 libtest 的机器，可使用：

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib host_lifecycle_emit_routes_answer_and_resume_phases_to_wecom_host -- --nocapture
```

预期关注点：

- 能看到以下 phase 经由 unified host 路由到 WeCom host：
  - `ask_user_answered`
  - `approval_resolved`
  - `resumed`

### 4. 执行新增 WeCom final reply dispatch 回归

Windows 推荐命令：

```bash
pnpm test:im-host-windows-regression
```

非 Windows 或可稳定执行原始 libtest 的机器，可使用：

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib host_reply_dispatch_routes_wecom_session_via_unified_host -- --nocapture
```

预期关注点：

- `maybe_dispatch_registered_im_session_reply_with_pool(...)` 返回成功
- WeCom send hook 收到最终回复文本
- 说明 final reply 已走 unified host，而不是绕过宿主层

### 5. 前端统一渠道页回归

```bash
pnpm --dir apps/runtime test -- src/components/__tests__/SettingsView.wecom-connector.test.tsx
```

预期：

- `47 tests` 全通过

## 回填位置

执行完成后，把结果回填到：

- [appendix-b-risk-and-verification.md](/D:/code/WorkClaw/docs/architecture/openclaw-im-host/appendix-b-risk-and-verification.md)
- [06-phase-3-acceptance-summary.md](/D:/code/WorkClaw/docs/architecture/openclaw-im-host/06-phase-3-acceptance-summary.md)
- [08-phase-3-external-verification-result-template.md](/D:/code/WorkClaw/docs/architecture/openclaw-im-host/08-phase-3-external-verification-result-template.md)

建议至少更新以下内容：

- 实际执行命令
- PASS / FAIL
- 若失败，失败是在编译、链接、运行还是断言阶段
- 是否可以把 Phase 3 状态从“93%-95%”提升到“执行级验证完成”

建议按下面模板回填：

```md
## 外部机器验证结果（YYYY-MM-DD）

- 机器 / 环境：
- 执行人：
- 代码基线：

### 执行命令

- `pnpm verify:openclaw-im-host:phase3`

### 结果

- waiting-state order:
- resumed lifecycle routing:
- final reply dispatch:
- frontend WeCom registry/settings:
- Windows 专用回归入口是否通过：
- 原始 libtest 路径是否执行：

### 结论

- 是否可把 Phase 3 状态提升为“执行级验证完成”：
- 仍剩余的问题：
```

## 验收通过标准

如果以下条件全部满足，就可以把第三阶段视为基本完成：

- WeCom waiting-state 顺序测试通过
- WeCom resumed lifecycle 路由测试通过
- WeCom final reply unified host dispatch 测试通过
- WeCom settings / channel registry 前端测试继续全绿
- 没有新增回归导致 Feishu / unified host 基线破坏

其中执行级证据可接受以下任一组合：

- Windows 上 `pnpm test:im-host-windows-regression` 通过
- 非 Windows 或稳定环境上原始 `cargo test --lib ...` 定向用例通过

## 一句话说明

这份 runbook 的目标，不是继续设计，而是把当前已经接近完成的 Phase 3 验证流程标准化：Windows 默认走专用回归入口，其他稳定环境可继续补跑原始 libtest 作为附加证据。
