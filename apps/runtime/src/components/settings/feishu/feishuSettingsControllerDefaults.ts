import type {
  FeishuGatewaySettings,
  OpenClawPluginFeishuAdvancedSettings,
} from "../../../types";

export const DEFAULT_FEISHU_CONNECTOR_SETTINGS: FeishuGatewaySettings = {
  app_id: "",
  app_secret: "",
  ingress_token: "",
  encrypt_key: "",
  sidecar_base_url: "",
};

export const DEFAULT_FEISHU_ADVANCED_SETTINGS: OpenClawPluginFeishuAdvancedSettings = {
  groups_json: "",
  dms_json: "",
  footer_json: "",
  account_overrides_json: "",
  render_mode: "",
  streaming: "",
  text_chunk_limit: "",
  chunk_mode: "",
  reply_in_thread: "",
  group_session_scope: "",
  topic_session_mode: "",
  markdown_mode: "",
  markdown_table_mode: "",
  heartbeat_visibility: "",
  heartbeat_interval_ms: "",
  media_max_mb: "",
  http_timeout_ms: "",
  config_writes: "",
  webhook_host: "",
  webhook_port: "",
  dynamic_agent_creation_enabled: "",
  dynamic_agent_creation_workspace_template: "",
  dynamic_agent_creation_agent_dir_template: "",
  dynamic_agent_creation_max_agents: "",
};
