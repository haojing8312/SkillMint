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

export interface Message {
  role: "user" | "assistant";
  content: string;
  created_at: string;
}
