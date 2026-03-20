# Feishu Official Plugin Onboarding Design

## Strategy Summary
- Change surface: WorkClaw 的飞书接入主入口、官方插件安装/配置体验、官方插件运行时启动与诊断展示。
- Affected modules: `apps/runtime/src/components/SettingsView.tsx`、新的安装向导前端组件、`apps/runtime/src-tauri/src/commands/openclaw_plugins.rs`、`apps/runtime/plugin-host/scripts/run-feishu-host.mjs`、以及必要的 plugin-host install/setup helper。
- Main risk: 继续把旧飞书连接器表单当主路径，会和飞书官方插件的真实使用方式冲突，导致“看起来已配置，实际消息不通”。
- Recommended smallest safe path: 先把“安装飞书官方插件向导”做成唯一主入口，再把现有设置页降级为“状态面板 + 高级配置 + 诊断”。
- Required verification: 前端组件测试、`openclaw_plugins` Rust 测试、`pnpm test:rust-fast`，以及至少一次真实飞书消息联调。
- Release impact: 中高。该变更直接改变用户接入飞书官方插件的主流程，属于用户可见行为调整。

## Background

当前 WorkClaw 已经具备这些能力：

- 安装并识别 `@larksuite/openclaw-lark`
- 保存官方插件所需的基础飞书配置
- 启动官方插件兼容宿主并显示 `运行中`
- 基础配对审批 UI 与官方插件状态展示

但当前主路径仍然偏向“本地设置页表单”，而不是 OpenClaw 官方插件的真实使用路径。

根据飞书官方文档和本地源码，OpenClaw 的标准流程是：

1. 运行安装器：
   `npx -y @larksuite/openclaw-lark install`
2. 安装器内直接选择：
   - 新建机器人
   - 关联已有机器人
3. 安装期即时校验 `App ID / App Secret`
4. 安装完成后自动让 gateway/插件运行态可用
5. 去飞书会话中用 `/feishu start`、`/feishu auth`、`/feishu doctor` 完成验证与授权

官方文档：
- <https://bytedance.larkoffice.com/docx/MFK7dDFLFoVlOGxWCv5cTXKmnMh#M0usd9GLwoiBxtx1UyjcpeMhnRe>

关键源码依据：
- OpenClaw 插件安装/启用：`references/openclaw/src/cli/plugins-cli.ts`
- OpenClaw channel setup：`references/openclaw/src/wizard/setup.ts`
- 官方飞书 channel 配置/网关：`references/openclaw-lark/src/channel/plugin.ts`
- 官方飞书 monitor：`references/openclaw-lark/src/channel/monitor.ts`
- 官方飞书 onboarding：`references/openclaw-lark/src/channel/onboarding.ts`

## Product Decision

### Recommended Direction

WorkClaw 要对齐成：

- 主入口：安装向导
- 次入口：状态页 / 高级配置页

而不是：

- 主入口：设置页手工填表
- 次入口：命令或飞书内操作

### Why

这样更接近飞书官方插件在 OpenClaw 中的真实形态：

- 用户先“装好机器人”
- 再“在飞书里跟机器人互动”
- 本地设置页主要负责：
  - 查看当前状态
  - 调整高级配置
  - 看诊断日志

## Proposed UX

### 1. Add Feishu Official Plugin Install Wizard

在 `渠道连接器 > 飞书` 页顶部增加主 CTA：

- `安装飞书官方插件`
- 如果已安装则显示 `重新绑定/重跑安装向导`

点击后进入向导弹层或独立步骤面板。

#### Wizard Steps

1. 选择模式
   - `新建机器人`
   - `关联已有机器人`

2. 新建机器人模式
   - 显示二维码或外部跳转
   - 轮询安装器/后端状态
   - 成功后展示“机器人已创建”

3. 关联已有机器人模式
   - 输入 `App ID`
   - 输入 `App Secret`
   - 即时校验凭证
   - 验证通过后写入官方插件配置

4. 安装完成
   - 自动安装/升级 `@larksuite/openclaw-lark`
   - 自动刷新官方插件状态
   - 自动启动官方插件 runtime
   - 引导用户去飞书发一条消息

5. 飞书内下一步提示
   - `/feishu start`
   - `/feishu auth`
   - `/feishu doctor`
   - “学习一下我安装的新飞书插件，列出有哪些能力”

### 2. Reframe Current Settings Page

现有飞书页不再强调“表单配置就是主流程”，而调整为：

- `连接配置`
  这里显示“官方插件安装状态 / 运行状态 / 最近事件 / 最近日志 / 高级字段”

- `官方插件`
  展示插件包名、版本、默认账号、配置模式、能力摘要

- `配对与授权`
  展示配对请求审批，以及引导用户在飞书里执行 `/feishu auth`

### 3. Treat Verification Token / Encrypt Key as Advanced

默认文案明确说明：

- 当前官方插件主路径按 websocket 模式运行
- `Verification Token` / `Encrypt Key` 仅在切换到 webhook 方案时才需要

这些字段保留在高级配置里，不再当作主流程阻塞项。

## Runtime Alignment

### Target Runtime Flow

WorkClaw 应对齐到以下主线：

1. 安装官方插件并写入启用配置
2. 把飞书业务配置写入 `channels.feishu...`
3. 调用官方插件 `gateway.startAccount`
4. 进入官方插件 `monitorFeishuProvider`
5. 由 websocket 接收飞书消息
6. 通过 `/feishu start`、`/feishu doctor`、`/feishu auth` 做用户级验证

### Important Constraint

从当前官方插件源码看，真实 monitor 主线是 websocket，不是 webhook：

- `connectionMode` 虽有 `webhook`
- 但 `monitor.ts` 中明确只实现 websocket monitor

所以 WorkClaw 现阶段不应该把 webhook 当成“官方标准路径”来设计。

## Gaps Versus Current WorkClaw

1. 缺少安装期交互向导
- 现在只有“安装插件”和“设置页保存”
- 缺少“新建机器人 / 关联已有机器人 / 即时校验凭证”

2. 设置页承担了过多主流程责任
- 当前 UI 更像旧连接器页
- 应改为官方插件状态与高级配置页

3. 飞书内命令闭环没有成为主引导
- `/feishu start`
- `/feishu auth`
- `/feishu doctor`

4. 向导完成后缺少“自动拉起 runtime + 即刻验证”

5. 诊断路径尚未完全可视化
- 已有最近日志/最近事件基础
- 还需要更明确区分：
  - 宿主未运行
  - websocket 未建连
  - 消息已收到但处理失败

## Recommended Rollout

### Phase 1

把安装向导做成主入口，但保留现有设置页能力。

### Phase 2

将现有手工表单降级为高级配置区，默认折叠或弱化。

### Phase 3

在 UI 中增加一段“飞书内验证清单”，让用户知道成功标准不是“页面显示运行中”，而是：

- `/feishu start` 返回版本
- 机器人能响应私聊
- `/feishu auth` 能完成授权

## Acceptance Criteria

- 用户可以在 WorkClaw 内完成“新建机器人”或“关联已有机器人”
- 安装器能即时校验已有机器人凭证
- 安装成功后官方插件 runtime 自动进入运行态
- 飞书页能展示最近事件、最近日志、最近错误
- 页面明确引导用户在飞书对话里执行 `/feishu start`、`/feishu auth`、`/feishu doctor`
- 设置页不再被误解为旧飞书 sidecar 的主入口
