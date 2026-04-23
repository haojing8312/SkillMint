use crate::im::{
    build_conversation_id, build_parent_conversation_candidates, ImConversationScope,
    ImConversationSurface, ImPeerKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeishuConversationMetadata {
    pub conversation_id: String,
    pub base_conversation_id: String,
    pub parent_conversation_candidates: Vec<String>,
    pub conversation_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeishuConversationInput<'a> {
    pub account_id: Option<&'a str>,
    pub tenant_id: Option<&'a str>,
    pub chat_id: &'a str,
    pub chat_type: Option<&'a str>,
    pub sender_id: Option<&'a str>,
    pub message_id: Option<&'a str>,
    pub root_id: Option<&'a str>,
    pub thread_id: Option<&'a str>,
}

fn normalized_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalized_required(value: &str, fallback: &str) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized.to_string()
    }
}

fn resolved_account_id(input: &FeishuConversationInput<'_>) -> String {
    normalized_non_empty(input.account_id)
        .or_else(|| normalized_non_empty(input.tenant_id))
        .unwrap_or_else(|| "default".to_string())
}

fn peer_kind_from_chat_type(chat_type: Option<&str>) -> ImPeerKind {
    match chat_type.map(str::trim) {
        Some("p2p") | Some("direct") => ImPeerKind::Direct,
        _ => ImPeerKind::Group,
    }
}

fn topic_id_from_input(input: &FeishuConversationInput<'_>) -> Option<String> {
    normalized_non_empty(input.root_id).or_else(|| normalized_non_empty(input.thread_id))
}

fn scope_from_input(input: &FeishuConversationInput<'_>) -> ImConversationScope {
    if topic_id_from_input(input).is_some() {
        ImConversationScope::Topic
    } else {
        ImConversationScope::Peer
    }
}

fn peer_surface(surface: &ImConversationSurface) -> ImConversationSurface {
    ImConversationSurface {
        channel: surface.channel.clone(),
        account_id: surface.account_id.clone(),
        tenant_id: surface.tenant_id.clone(),
        peer_kind: surface.peer_kind,
        peer_id: surface.peer_id.clone(),
        topic_id: None,
        sender_id: None,
        scope: ImConversationScope::Peer,
        message_id: surface.message_id.clone(),
        raw_thread_id: surface.raw_thread_id.clone(),
        raw_root_id: surface.raw_root_id.clone(),
    }
}

pub(crate) fn build_feishu_conversation_surface(
    input: &FeishuConversationInput<'_>,
) -> ImConversationSurface {
    ImConversationSurface {
        channel: "feishu".to_string(),
        account_id: resolved_account_id(input),
        tenant_id: normalized_non_empty(input.tenant_id),
        peer_kind: peer_kind_from_chat_type(input.chat_type),
        peer_id: normalized_required(input.chat_id, "unknown"),
        topic_id: topic_id_from_input(input),
        sender_id: normalized_non_empty(input.sender_id),
        scope: scope_from_input(input),
        message_id: normalized_non_empty(input.message_id),
        raw_thread_id: normalized_non_empty(input.thread_id),
        raw_root_id: normalized_non_empty(input.root_id),
    }
}

pub(crate) fn build_feishu_conversation_metadata(
    input: &FeishuConversationInput<'_>,
) -> FeishuConversationMetadata {
    let surface = build_feishu_conversation_surface(input);
    let base_conversation_id = build_conversation_id(&peer_surface(&surface));
    FeishuConversationMetadata {
        conversation_id: build_conversation_id(&surface),
        base_conversation_id,
        parent_conversation_candidates: build_parent_conversation_candidates(&surface),
        conversation_scope: surface.scope.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_feishu_conversation_metadata, build_feishu_conversation_surface,
        FeishuConversationInput,
    };
    use crate::im::{ImConversationScope, ImPeerKind};

    #[test]
    fn builds_topic_conversation_for_group_threads() {
        let input = FeishuConversationInput {
            account_id: Some("default"),
            tenant_id: Some("tenant-1"),
            chat_id: "oc_group_1",
            chat_type: Some("group"),
            sender_id: Some("ou_sender"),
            message_id: Some("om_msg_1"),
            root_id: Some("om_root_1"),
            thread_id: Some("omt_topic_1"),
        };

        let surface = build_feishu_conversation_surface(&input);
        assert_eq!(surface.peer_kind, ImPeerKind::Group);
        assert_eq!(surface.scope, ImConversationScope::Topic);
        assert_eq!(surface.topic_id.as_deref(), Some("om_root_1"));

        let metadata = build_feishu_conversation_metadata(&input);
        assert_eq!(
            metadata.conversation_id,
            "feishu:default:group:oc_group_1:topic:om_root_1"
        );
        assert_eq!(
            metadata.base_conversation_id,
            "feishu:default:group:oc_group_1"
        );
        assert_eq!(
            metadata.parent_conversation_candidates,
            vec!["feishu:default:group:oc_group_1".to_string()]
        );
    }

    #[test]
    fn builds_peer_conversation_for_direct_messages() {
        let metadata = build_feishu_conversation_metadata(&FeishuConversationInput {
            account_id: Some("default"),
            tenant_id: Some("tenant-1"),
            chat_id: "ou_user_1",
            chat_type: Some("p2p"),
            sender_id: Some("ou_user_1"),
            message_id: Some("om_msg_2"),
            root_id: None,
            thread_id: None,
        });

        assert_eq!(metadata.conversation_id, "feishu:default:direct:ou_user_1");
        assert_eq!(
            metadata.base_conversation_id,
            "feishu:default:direct:ou_user_1"
        );
        assert!(metadata.parent_conversation_candidates.is_empty());
        assert_eq!(metadata.conversation_scope, "peer");
    }

    #[test]
    fn falls_back_to_thread_id_when_root_id_is_missing() {
        let metadata = build_feishu_conversation_metadata(&FeishuConversationInput {
            account_id: Some("default"),
            tenant_id: Some("tenant-1"),
            chat_id: "oc_group_2",
            chat_type: Some("group"),
            sender_id: Some("ou_sender"),
            message_id: Some("om_msg_3"),
            root_id: None,
            thread_id: Some("omt_topic_fallback"),
        });

        assert_eq!(
            metadata.conversation_id,
            "feishu:default:group:oc_group_2:topic:omt_topic_fallback"
        );
        assert_eq!(
            metadata.base_conversation_id,
            "feishu:default:group:oc_group_2"
        );
        assert_eq!(
            metadata.parent_conversation_candidates,
            vec!["feishu:default:group:oc_group_2".to_string()]
        );
        assert_eq!(metadata.conversation_scope, "topic");
    }

    #[test]
    fn blank_account_id_falls_back_to_non_empty_tenant_id() {
        let surface = build_feishu_conversation_surface(&FeishuConversationInput {
            account_id: Some("   "),
            tenant_id: Some("tenant-fallback"),
            chat_id: "oc_group_3",
            chat_type: Some("group"),
            sender_id: Some("ou_sender"),
            message_id: Some("om_msg_4"),
            root_id: None,
            thread_id: None,
        });

        assert_eq!(surface.account_id, "tenant-fallback");
        assert_eq!(surface.tenant_id.as_deref(), Some("tenant-fallback"));
    }
}
