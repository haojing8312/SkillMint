# OpenClaw 升级维护手册

> **历史 / 遗留 vendor lane：** 本文档保留给维护者理解和必要时复现既有 OpenClaw vendor 同步流程。它不再代表 WorkClaw 的前向产品架构，也不应作为新增 IM、routing、browser、MCP、toolset 或 profile-runtime 能力的设计入口。
>
> 当前方向以 Hermes-aligned sidecar removal roadmap 为准：profile runtime、原生 Rust `ToolRegistry` / toolsets、平台 gateway / adapter、profile-owned memory / skills / growth / curator。OpenClaw 相关脚本、vendor 目录和检查在 Batch 3C 后仍保持原样，等待后续批次按 [release/vendor lane replacement plan](../plans/2026-05-11-release-vendor-lane-replacement-plan.md) 引入 neutral checks、迁移文档引用，再执行显式废弃；本批次不删除命令或脚本。

本文档用于维护者执行遗留 OpenClaw 核心同步与回归验证，包括已存在的 routing subset，以及历史上为多 IM 连接器预留的 vendor lane。Batch 3C 已记录这些 vendor lane 的替换检查或废弃路径，详见 [release/vendor lane replacement plan](../plans/2026-05-11-release-vendor-lane-replacement-plan.md)。

## 前置条件

1. 准备 OpenClaw 上游仓库本地副本
2. 设置环境变量 `OPENCLAW_UPSTREAM_PATH` 指向上游仓库路径

## 升级步骤

### 1. 路由核心子集（当前已启用）

1. 执行同步脚本：`node scripts/sync-openclaw-core.mjs`
2. 核对并更新以下文件：
   - `apps/runtime/sidecar/vendor/openclaw-core/UPSTREAM_COMMIT`
   - `apps/runtime/sidecar/vendor/openclaw-core/PATCHES.md`

### 2. IM adapter vendor lane（当前仅预留，不默认启用）

1. 准备 OpenClaw 上游仓库本地副本，并设置：
   - `OPENCLAW_IM_UPSTREAM_PATH`
   - 或沿用 `OPENCLAW_UPSTREAM_PATH`
2. 执行同步脚本：`node scripts/sync-openclaw-im-core.mjs`
3. 核对并更新以下文件：
   - `apps/runtime/sidecar/vendor/openclaw-im-core/UPSTREAM_COMMIT`
   - `apps/runtime/sidecar/vendor/openclaw-im-core/PATCHES.md`
4. 在真正启用第二个渠道前，先补全同步 manifest，再引入对应 adapter 包装层。

## 回归验证

1. Vendor lane 元数据检查：`node --test scripts/check-openclaw-vendor-lane.test.mjs`
2. Sidecar 测试：`pnpm --dir apps/runtime/sidecar test`
3. 路由回归：`cargo test --test test_openclaw_gateway --test test_openclaw_route_regression -- --nocapture`

## 建议补充检查

- Feishu 路由配置页可视化行为
- 聊天页路由决策卡片字段完整性
- 多员工监听与自动恢复流程
- 新渠道接入时，确认所有上游代码仍被限制在 `apps/runtime/sidecar/vendor/` 和 `apps/runtime/sidecar/src/adapters/` 边界内
