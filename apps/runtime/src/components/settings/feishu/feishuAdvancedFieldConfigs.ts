import type { FeishuAdvancedFieldConfig } from "./FeishuAdvancedSection.types";

export const FEISHU_ADVANCED_MESSAGE_FIELDS: FeishuAdvancedFieldConfig[] = [
  { key: "footer_json", label: "回复页脚 JSON", description: "定义回复尾部展示的状态、耗时等附加信息。", kind: "textarea", rows: 5 },
  { key: "account_overrides_json", label: "账号覆盖 JSON", description: "按账号覆盖消息展示行为，适合多账号接入时做细分调整。", kind: "textarea", rows: 5 },
  { key: "render_mode", label: "渲染模式", description: "控制回复内容的主要渲染方式。", kind: "input" },
  { key: "streaming", label: "流式输出", description: "决定回复是否边生成边发送。", kind: "input" },
  { key: "text_chunk_limit", label: "文本分块上限", description: "单次消息的最大文本块长度。", kind: "input" },
  { key: "chunk_mode", label: "分块模式", description: "控制长消息按什么策略拆分。", kind: "input" },
  { key: "markdown_mode", label: "Markdown 模式", description: "控制 Markdown 内容如何转换给飞书。", kind: "input" },
  { key: "markdown_table_mode", label: "Markdown 表格模式", description: "控制表格内容的展示方式。", kind: "input" },
];

export const FEISHU_ADVANCED_ROUTING_FIELDS: FeishuAdvancedFieldConfig[] = [
  { key: "groups_json", label: "群聊高级规则 JSON", description: "按群聊配置启用、提及规则等进阶行为。", kind: "textarea", rows: 8 },
  { key: "dms_json", label: "私聊高级规则 JSON", description: "按私聊对象配置启用状态和系统提示。", kind: "textarea", rows: 8 },
  { key: "reply_in_thread", label: "线程内回复", description: "控制消息是否优先在线程中回复。", kind: "input" },
  { key: "group_session_scope", label: "群聊会话范围", description: "决定群聊里如何划分会话上下文。", kind: "input" },
  { key: "topic_session_mode", label: "话题会话模式", description: "决定是否把话题回复视为独立会话。", kind: "input" },
];

export const FEISHU_ADVANCED_RUNTIME_FIELDS: FeishuAdvancedFieldConfig[] = [
  { key: "heartbeat_visibility", label: "心跳可见性", description: "控制连接保活提示是否对外可见。", kind: "input" },
  { key: "heartbeat_interval_ms", label: "心跳间隔毫秒", description: "设置连接保活检测频率。", kind: "input" },
  { key: "media_max_mb", label: "媒体大小上限 MB", description: "限制可处理媒体消息的大小。", kind: "input" },
  { key: "http_timeout_ms", label: "HTTP 超时毫秒", description: "设置外部请求的最大等待时间。", kind: "input" },
  { key: "config_writes", label: "允许插件写回配置", description: "决定插件运行时是否允许自动写回部分配置。", kind: "input" },
];

export const FEISHU_ADVANCED_DYNAMIC_AGENT_FIELDS: FeishuAdvancedFieldConfig[] = [
  { key: "dynamic_agent_creation_enabled", label: "动态 Agent 创建", description: "决定是否允许根据飞书会话动态创建 Agent。", kind: "input" },
  { key: "dynamic_agent_creation_workspace_template", label: "动态工作区模板", description: "定义动态创建工作区时使用的路径模板。", kind: "input" },
  { key: "dynamic_agent_creation_agent_dir_template", label: "动态 Agent 目录模板", description: "定义动态 Agent 目录的生成规则。", kind: "input" },
  { key: "dynamic_agent_creation_max_agents", label: "动态 Agent 数量上限", description: "限制动态创建 Agent 的最大数量。", kind: "input" },
];
