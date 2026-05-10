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
  source_type?: string;
}

export interface SkillOsSourceProjection {
  raw_source_type: string;
  canonical: string;
  immutable_content: boolean;
  directory_backed: boolean;
  requires_unpack_for_view: boolean;
}

export interface SkillOsCapabilities {
  can_list: boolean;
  can_view: boolean;
  can_patch: boolean;
  can_archive: boolean;
  can_reset: boolean;
  can_agent_delete: boolean;
  can_user_uninstall: boolean;
}

export interface SkillOsToolsetPolicy {
  requires_toolsets: string[];
  optional_toolsets: string[];
  denied_toolsets: string[];
  unknown_toolsets: string[];
}

export interface SkillOsUsageTelemetry {
  view_count: number;
  use_count: number;
  patch_count: number;
  last_viewed_at: string;
  last_used_at: string;
  last_patched_at: string;
  pinned: boolean;
}

export interface SkillOsIndexEntry {
  skill_id: string;
  name: string;
  description: string;
  version: string;
  tags: string[];
  source: SkillOsSourceProjection;
  capabilities: SkillOsCapabilities;
  toolset_policy: SkillOsToolsetPolicy;
  lifecycle_state: string;
  usage: SkillOsUsageTelemetry;
}

export interface SkillOsView {
  entry: SkillOsIndexEntry;
  content: string;
  read_only: boolean;
  derived: boolean;
}

export interface SkillOsVersionEntry {
  version_id: string;
  skill_id: string;
  source_type: string;
  action: string;
  summary: string;
  created_at: string;
}

export interface SkillOsMutationResult {
  action: string;
  skill: SkillOsView;
  version_id: string;
  rollback_to_version_id?: string | null;
  reset_to_version_id?: string | null;
  growth_event_id?: string | null;
  diff: string;
}

export interface ClawhubSkillSummary {
  name: string;
  slug: string;
  description: string;
  github_url?: string | null;
  source_url?: string | null;
  stars: number;
}

export interface ClawhubLibraryItem {
  slug: string;
  name: string;
  summary: string;
  github_url?: string | null;
  source_url?: string | null;
  tags: string[];
  stars: number;
  downloads: number;
}

export interface ClawhubLibraryResponse {
  items: ClawhubLibraryItem[];
  next_cursor?: string | null;
  last_synced_at?: string | null;
}

export interface SkillhubCatalogSyncStatus {
  total_skills: number;
  last_synced_at?: string | null;
  refreshed: boolean;
}

export interface ClawhubSkillDetail {
  slug: string;
  name: string;
  summary: string;
  description: string;
  author?: string | null;
  github_url?: string | null;
  source_url?: string | null;
  updated_at?: string | null;
  stars: number;
  downloads: number;
  tags: string[];
  readme?: string | null;
}

export interface ClawhubSkillRecommendation {
  slug: string;
  name: string;
  description: string;
  stars: number;
  score: number;
  reason: string;
  github_url?: string | null;
  source_url?: string | null;
}

export interface ClawhubInstallRequest {
  slug: string;
  githubUrl?: string | null;
  sourceUrl?: string | null;
}

export interface ModelConfig {
  id: string;
  name: string;
  api_format: string;
  base_url: string;
  model_name: string;
  is_default: boolean;
  supports_vision?: boolean;
}

export interface ProviderConfig {
  id: string;
  provider_key: string;
  display_name: string;
  protocol_type: string;
  base_url: string;
  auth_type: string;
  api_key_encrypted: string;
  org_id: string;
  extra_json: string;
  enabled: boolean;
}

export interface ProviderPluginInfo {
  key: string;
  display_name: string;
  capabilities: string[];
}

export interface ChatRoutingPolicy {
  primary_provider_id: string;
  primary_model: string;
  fallback_chain_json: string;
  timeout_ms: number;
  retry_count: number;
  enabled: boolean;
}

export interface CapabilityRoutingPolicy {
  capability: string;
  primary_provider_id: string;
  primary_model: string;
  fallback_chain_json: string;
  timeout_ms: number;
  retry_count: number;
  enabled: boolean;
}

export interface ProviderHealthInfo {
  provider_id: string;
  ok: boolean;
  protocol_type: string;
  message: string;
}

export type ModelErrorKind =
  | "billing"
  | "auth"
  | "rate_limit"
  | "context_overflow"
  | "invalid_token_budget"
  | "media_too_large"
  | "timeout"
  | "network"
  | "unknown";

export interface ModelConnectionTestResult {
  ok: boolean;
  kind: ModelErrorKind;
  title: string;
  message: string;
  raw_message?: string | null;
}

export interface RouteAttemptLog {
  session_id: string;
  capability: string;
  api_format: string;
  model_name: string;
  attempt_index: number;
  retry_index: number;
  error_kind: string;
  success: boolean;
  error_message: string;
  created_at: string;
}

export interface RouteAttemptStat {
  capability: string;
  error_kind: string;
  success: boolean;
  count: number;
}

export interface CapabilityRouteTemplateInfo {
  template_id: string;
  name: string;
  description: string;
  capability: string;
}

export interface FrontMatter {
  name?: string;
  description?: string;
  version?: string;
  model?: string;
}

export interface SkillDirInfo {
  files: string[];
  front_matter: FrontMatter;
}

