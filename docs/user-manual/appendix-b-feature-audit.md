# 附录 B. 已实现功能校审记录

## 说明
本手册只写已实现、可在当前代码中确认的功能。  
以下为本次文档编写前的校审依据（代码与现有文档）。

## 校审结果总览
1. 左侧主导航已实现：开始任务 / 专家技能 / 智能体员工 / 设置。
2. 首次启动（无模型时）已实现：快速配置引导层与“快速配置（1分钟）”弹窗。
3. 专家技能页已实现：我的技能 / 技能库 / 找技能 / 安装技能 / 技能打包 / 创建。
4. 安装技能弹窗已实现：加密 `.skillpack` / 本地目录 / ClawHub / 行业包。
5. 智能体员工页已实现：员工助手创建、员工详情、飞书配置、长期记忆导出/清空。
6. 设置页可见能力已实现：模型连接、语言与沉浸式翻译、搜索引擎、MCP 服务器。
7. 文档已明确排除企业能力和未上线能力。

## 代码校审依据
### 1) 主导航与主视图
- `apps/runtime/src/components/Sidebar.tsx`
  - “开始任务 / 专家技能 / 智能体员工 / 设置”按钮文案与行为。
- `apps/runtime/src/App.tsx`
  - 主视图路由：`start-task`、`experts`、`employees`、`settings`。
  - 首次无模型场景：`model-setup-gate`、`quick-model-setup-dialog`、`快速配置（1分钟）`入口。

### 2) 专家技能中心
- `apps/runtime/src/components/experts/ExpertsView.tsx`
  - 顶部按钮：安装技能、技能打包、创建。
  - Tab：我的技能、技能库、找技能。
  - 技能卡片动作：开始任务、刷新、检查更新、更新、移除。

### 3) 安装技能流程
- `apps/runtime/src/components/InstallDialog.tsx`
  - 安装模式：`skillpack`、`local`、`clawhub`、`industry`。
  - `.skillpack` 模式含“用户名（创作者提供）”输入项。

### 4) 技能打包流程
- `apps/runtime/src/components/packaging/PackagingView.tsx`
- `apps/runtime/src/components/packaging/PackForm.tsx`
  - 入口与动作：选择技能目录、导出技能包。

### 5) 智能体员工中心
- `apps/runtime/src/components/employees/EmployeeHubView.tsx`
  - 员工助手入口：打开员工助手 / 新建员工 / 调整员工。
  - 飞书配置与状态：保存飞书配置、重试连接、状态文案。
  - 长期记忆：刷新统计、导出 JSON、清空记忆。
  - 员工操作：设为主员工并进入首页、与该员工对话开始任务、删除员工。

### 6) 设置中心
- `apps/runtime/src/components/SettingsView.tsx`
  - 模型连接：测试连接、保存。
  - 语言与沉浸式翻译：默认语言、翻译显示、触发方式、引擎策略。
  - 搜索引擎：添加/编辑/测试连接/保存。
  - MCP 服务器：预设、命令参数、环境变量、添加服务器。

### 7) 现有文档互证
- `README.md`
  - 产品定位、功能边界、普通用户与维护者文档分层。
- `docs/troubleshooting/skill-installation.md`
  - 技能安装重名冲突排错基线。
- `docs/integrations/feishu-routing.md`
  - 飞书路由能力存在性说明（用于“已实现但不扩展企业叙述”边界）。

## 本次明确不写内容
1. 企业权限体系（RBAC/SSO/多租户）。
2. 尚未在界面暴露的高级路由与健康检查配置细节。
3. 仅维护者使用的二开、发布、底层架构运维流程。

## 模型命名校正说明
1. 本手册新增“推荐模型与命名校正”附录（附录 C）。
2. 校正日期：2026-03-06。
3. 校正原则：优先采用官方页面可确认的公开命名。
