use crate::traits::ChatSettingsRepository;
use crate::types::{
    ChatPermissionMode, ChatPreparationRequest, ModelRouteErrorKind, PreparedChatExecution,
    PreparedSessionCreation, SessionCreationRequest,
};
use serde_json::Value;

pub struct ChatPreparationService;

impl ChatPreparationService {
    pub fn new() -> Self {
        Self
    }

    pub fn prepare_session_creation(
        &self,
        request: SessionCreationRequest,
    ) -> PreparedSessionCreation {
        let permission_mode_storage =
            normalize_permission_mode_for_storage(request.permission_mode.as_deref()).to_string();
        let session_mode_storage =
            normalize_session_mode_for_storage(request.session_mode.as_deref()).to_string();
        let normalized_team_id =
            normalize_team_id_for_storage(&session_mode_storage, request.team_id.as_deref());
        let normalized_title = {
            let title = request.title.unwrap_or_default().trim().to_string();
            if title.is_empty() {
                "New Chat".to_string()
            } else {
                title
            }
        };

        PreparedSessionCreation {
            permission_mode_storage,
            session_mode_storage,
            normalized_team_id,
            normalized_title,
            normalized_work_dir: request.work_dir.unwrap_or_default().trim().to_string(),
            normalized_employee_id: request.employee_id.unwrap_or_default().trim().to_string(),
        }
    }

    pub async fn prepare_chat_execution<R: ChatSettingsRepository>(
        &self,
        repo: &R,
        request: ChatPreparationRequest,
    ) -> Result<PreparedChatExecution, String> {
        let routing = repo.load_routing_settings().await?;
        let chat_route = repo.load_chat_routing().await?;
        let capability = infer_capability_from_user_message(&request.user_message).to_string();
        let permission_mode_storage =
            normalize_permission_mode_for_storage(request.permission_mode.as_deref()).to_string();
        let session_mode_storage =
            normalize_session_mode_for_storage(request.session_mode.as_deref()).to_string();
        let normalized_team_id =
            normalize_team_id_for_storage(&session_mode_storage, request.team_id.as_deref());
        let permission_label = permission_mode_label(&permission_mode_storage).to_string();

        let (primary_provider_id, primary_model, fallback_targets) = match chat_route {
            Some(route) if route.enabled => (
                Some(route.primary_provider_id),
                Some(route.primary_model),
                parse_fallback_chain_targets(&route.fallback_chain_json),
            ),
            _ => (None, None, Vec::new()),
        };

        Ok(PreparedChatExecution {
            capability,
            permission_mode_storage,
            session_mode_storage,
            normalized_team_id,
            permission_label,
            max_call_depth: routing.max_call_depth,
            node_timeout_seconds: routing.node_timeout_seconds,
            retry_count: routing.retry_count,
            primary_provider_id,
            primary_model,
            fallback_targets,
            default_model_id: repo.resolve_default_model_id().await?,
            default_usable_model_id: repo.resolve_default_usable_model_id().await?,
        })
    }

    pub async fn prepare_route_candidates<R: ChatSettingsRepository>(
        &self,
        repo: &R,
        model_id: &str,
        request: &ChatPreparationRequest,
    ) -> Result<crate::types::PreparedRouteCandidates, String> {
        let session_model = repo.load_session_model(model_id).await?;
        let requested_capability = infer_capability_from_user_message(&request.user_message);

        let mut retry_count_per_candidate = 0usize;
        let mut route_policy = repo
            .load_route_policy(requested_capability)
            .await?
            .filter(|policy| policy.enabled);
        if route_policy.is_none() && requested_capability != "chat" {
            route_policy = repo
                .load_route_policy("chat")
                .await?
                .filter(|policy| policy.enabled);
        }

        let mut candidates = Vec::new();
        if let Some(policy) = route_policy {
            retry_count_per_candidate = policy.retry_count.clamp(0, 3) as usize;
            let mut provider_targets =
                vec![(policy.primary_provider_id, policy.primary_model.clone())];
            provider_targets.extend(parse_fallback_chain_targets(&policy.fallback_chain_json));

            for (provider_id, preferred_model) in provider_targets {
                if let Some(provider) = repo.get_provider_connection(&provider_id).await? {
                    if is_supported_protocol(&provider.protocol_type)
                        && !provider.api_key.trim().is_empty()
                    {
                        candidates.push(crate::types::PreparedRouteCandidate {
                            protocol_type: provider.protocol_type,
                            base_url: provider.base_url,
                            model_name: if preferred_model.trim().is_empty() {
                                session_model.model_name.clone()
                            } else {
                                preferred_model
                            },
                            api_key: provider.api_key,
                        });
                    }
                }
            }
        }

        if !session_model.api_key.trim().is_empty() {
            candidates.push(crate::types::PreparedRouteCandidate {
                protocol_type: session_model.api_format,
                base_url: session_model.base_url,
                model_name: session_model.model_name,
                api_key: session_model.api_key,
            });
        }

        Ok(crate::types::PreparedRouteCandidates {
            candidates,
            retry_count_per_candidate,
        })
    }
}

pub fn normalize_permission_mode_for_storage(permission_mode: Option<&str>) -> &'static str {
    match permission_mode.unwrap_or("").trim() {
        "standard" | "default" | "accept_edits" => "standard",
        "full_access" | "unrestricted" => "full_access",
        _ => "standard",
    }
}

pub fn normalize_session_mode_for_storage(session_mode: Option<&str>) -> &'static str {
    match session_mode.unwrap_or("").trim() {
        "employee_direct" => "employee_direct",
        "team_entry" => "team_entry",
        "general" => "general",
        _ => "general",
    }
}

pub fn normalize_team_id_for_storage(session_mode: &str, team_id: Option<&str>) -> String {
    if session_mode == "team_entry" {
        team_id.unwrap_or("").trim().to_string()
    } else {
        String::new()
    }
}

pub fn parse_permission_mode_for_runtime(permission_mode: &str) -> ChatPermissionMode {
    match permission_mode {
        "standard" | "default" | "accept_edits" => ChatPermissionMode::AcceptEdits,
        "full_access" | "unrestricted" => ChatPermissionMode::Unrestricted,
        _ => ChatPermissionMode::AcceptEdits,
    }
}

pub fn permission_mode_label(permission_mode: &str) -> &'static str {
    match permission_mode {
        "standard" => "标准模式",
        "full_access" => "全自动模式",
        "default" => "标准模式",
        "unrestricted" => "全自动模式",
        _ => "标准模式",
    }
}

pub fn infer_capability_from_user_message(message: &str) -> &'static str {
    let m = message.to_ascii_lowercase();
    if m.contains("识图")
        || m.contains("看图")
        || m.contains("图片理解")
        || m.contains("vision")
        || m.contains("analyze image")
    {
        return "vision";
    }
    if m.contains("生图")
        || m.contains("画图")
        || m.contains("生成图片")
        || m.contains("image generation")
        || m.contains("generate image")
    {
        return "image_gen";
    }
    if m.contains("语音转文字")
        || m.contains("语音识别")
        || m.contains("stt")
        || m.contains("transcribe")
        || m.contains("speech to text")
    {
        return "audio_stt";
    }
    if m.contains("文字转语音")
        || m.contains("tts")
        || m.contains("text to speech")
        || m.contains("语音合成")
    {
        return "audio_tts";
    }
    "chat"
}

pub fn classify_model_route_error(error_message: &str) -> ModelRouteErrorKind {
    let lower = error_message.to_ascii_lowercase();
    if lower.contains("api key")
        || lower.contains("unauthorized")
        || lower.contains("invalid_api_key")
        || lower.contains("authentication")
        || lower.contains("permission denied")
        || lower.contains("forbidden")
    {
        return ModelRouteErrorKind::Auth;
    }
    if lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("429")
        || lower.contains("quota")
    {
        return ModelRouteErrorKind::RateLimit;
    }
    if lower.contains("timeout") || lower.contains("timed out") || lower.contains("deadline") {
        return ModelRouteErrorKind::Timeout;
    }
    if lower.contains("connection")
        || lower.contains("network")
        || lower.contains("dns")
        || lower.contains("connect")
        || lower.contains("socket")
        || lower.contains("error sending request for url")
        || lower.contains("sending request for url")
    {
        return ModelRouteErrorKind::Network;
    }
    ModelRouteErrorKind::Unknown
}

pub fn should_retry_same_candidate(kind: ModelRouteErrorKind) -> bool {
    matches!(
        kind,
        ModelRouteErrorKind::RateLimit
            | ModelRouteErrorKind::Timeout
            | ModelRouteErrorKind::Network
    )
}

pub fn retry_budget_for_error(kind: ModelRouteErrorKind, configured_retry_count: usize) -> usize {
    if kind == ModelRouteErrorKind::Network {
        configured_retry_count.max(1)
    } else {
        configured_retry_count
    }
}

pub fn retry_backoff_ms(kind: ModelRouteErrorKind, attempt_idx: usize) -> u64 {
    let base_ms = match kind {
        ModelRouteErrorKind::RateLimit => 1200u64,
        ModelRouteErrorKind::Timeout => 700u64,
        ModelRouteErrorKind::Network => 400u64,
        _ => 0u64,
    };
    if base_ms == 0 {
        return 0;
    }
    let exp = attempt_idx.min(3) as u32;
    base_ms.saturating_mul(1u64 << exp).min(5000)
}

pub fn parse_fallback_chain_targets(raw: &str) -> Vec<(String, String)> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|item| {
            let provider_id = item.get("provider_id")?.as_str()?.to_string();
            let model = item
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            Some((provider_id, model))
        })
        .collect()
}

fn is_supported_protocol(protocol: &str) -> bool {
    matches!(protocol, "openai" | "anthropic")
}
