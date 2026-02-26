export interface SkillManifest {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  recommended_model: string;
  tags: string[];
  created_at: string;
  username_hint?: string;
}

export interface ModelConfig {
  id: string;
  name: string;
  api_format: string;
  base_url: string;
  model_name: string;
  is_default: boolean;
}

/// 有序的流式输出项：文字和工具调用按发生顺序排列
export interface StreamItem {
  type: "text" | "tool_call";
  content?: string;          // type === "text" 时的文字内容
  toolCall?: ToolCallInfo;   // type === "tool_call" 时的工具信息
}

export interface Message {
  role: "user" | "assistant";
  content: string;
  created_at: string;
  toolCalls?: ToolCallInfo[];
  /// 有序的展示项（新格式），优先使用此字段渲染
  streamItems?: StreamItem[];
}

export interface ToolCallInfo {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output?: string;
  status: "running" | "completed" | "error";
}

export interface SessionInfo {
  id: string;
  title: string;
  created_at: string;
  model_id: string;
  work_dir?: string;
}

/// 文件附件（用于 File Upload 功能）
export interface FileAttachment {
  name: string;
  size: number;
  type: string;
  content: string; // 文件文本内容或 base64
}
