# 飞书路由集成说明

本文档面向集成方、二开团队和维护者，说明 WorkClaw 当前内置的 OpenClaw Feishu 路由能力，以及它在统一渠道连接器架构中的位置。

## 能力范围

- 内置 Sidecar 路由引擎：`apps/runtime/sidecar/vendor/openclaw-core/`
- 统一渠道适配器内核：`apps/runtime/sidecar/src/adapters/`
- Feishu 连接器适配器：`apps/runtime/sidecar/src/adapters/feishu/`
- Rust 路由规则持久化：`im_routing_bindings`
- 聊天页路由决策展示：`matched_by` / `session_key` / `agent_id`

## 连接器边界

- WorkClaw 当前将 Feishu 作为第一个 `ChannelAdapter` 接入。
- Sidecar 对外暴露统一 `/api/channels/*` 接口，Feishu 旧入口保留为兼容别名。
- 后续新增 Slack / Discord / Telegram 等渠道时，目标是复用同一连接器边界，而不是继续在业务层增加新的 Feishu 风格专用逻辑。

## 当前产品结构

飞书相关能力在产品上拆成两层：

- `设置中心 > 渠道连接 > 飞书连接`
  - 负责插件、机器人、授权、连接状态与重试
- `智能体员工 > 员工详情 > 飞书接待`
  - 负责哪个员工接待飞书入口
  - 负责默认接待员工与指定群聊/会话范围

这样做的原因是把“飞书是否接通”和“消息由谁处理”拆开表达，避免用户把连接配置和员工分工混淆。

## 路由语义

- 一个飞书连接只能有一个默认接待员工
- 一个飞书连接可以关联多个员工
- 指定群聊/会话规则优先于默认接待员工
- 未命中规则时，回退给默认接待员工
- 员工详情页只管理飞书接待关系，不直接管理飞书插件凭据

## 典型使用场景

- 多员工并行监听与消息分发
- 路由规则可视化配置与模拟验证
- 线程/会话归属一致性追踪
- 连接器状态诊断（重连次数、队列事件、最近事件时间）

## 相关文档

- 员工身份模型：`docs/architecture/employee-identity-model.md`
- OpenClaw 升级维护：`docs/maintainers/openclaw-upgrade.md`
- 飞书 IM 闭环桥接：`docs/integrations/feishu-im-bridge.md`
