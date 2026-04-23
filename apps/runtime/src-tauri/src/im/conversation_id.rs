use crate::im::conversation_surface::{ImConversationScope, ImConversationSurface, ImPeerKind};

fn peer_kind_label(peer_kind: ImPeerKind) -> &'static str {
    peer_kind.as_str()
}

fn topic_id_for_surface(surface: &ImConversationSurface) -> &str {
    surface.topic_id.as_deref().unwrap_or("unknown")
}

fn sender_id_for_surface(surface: &ImConversationSurface) -> &str {
    surface.sender_id.as_deref().unwrap_or("unknown")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImConversationIdentity {
    pub conversation_id: String,
    pub base_conversation_id: String,
    pub parent_conversation_candidates: Vec<String>,
    pub scope: ImConversationScope,
    pub scope_label: String,
    pub peer_kind_label: String,
}

pub fn build_conversation_id(surface: &ImConversationSurface) -> String {
    match surface.scope {
        ImConversationScope::Peer => format!(
            "{}:{}:{}:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(surface.peer_kind),
            surface.peer_id
        ),
        ImConversationScope::PeerSender => format!(
            "{}:{}:{}:{}:sender:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(surface.peer_kind),
            surface.peer_id,
            sender_id_for_surface(surface)
        ),
        ImConversationScope::Topic => format!(
            "{}:{}:{}:{}:topic:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(surface.peer_kind),
            surface.peer_id,
            topic_id_for_surface(surface)
        ),
        ImConversationScope::TopicSender => format!(
            "{}:{}:{}:{}:topic:{}:sender:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(surface.peer_kind),
            surface.peer_id,
            topic_id_for_surface(surface),
            sender_id_for_surface(surface)
        ),
    }
}

pub fn build_parent_conversation_candidates(surface: &ImConversationSurface) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut scope = surface.scope;

    while let Some(parent_scope) = scope.parent_scope() {
        candidates.push(build_conversation_id(&surface.with_scope(parent_scope)));
        scope = parent_scope;
    }

    candidates
}

pub fn build_base_conversation_id(surface: &ImConversationSurface) -> String {
    build_conversation_id(&surface.peer_surface())
}

pub fn build_conversation_identity(surface: &ImConversationSurface) -> ImConversationIdentity {
    ImConversationIdentity {
        conversation_id: build_conversation_id(surface),
        base_conversation_id: build_base_conversation_id(surface),
        parent_conversation_candidates: build_parent_conversation_candidates(surface),
        scope: surface.scope,
        scope_label: surface.scope.as_str().to_string(),
        peer_kind_label: surface.peer_kind.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::im::{
        build_base_conversation_id, build_conversation_id, build_conversation_identity,
        build_parent_conversation_candidates, ImConversationScope, ImConversationSurface,
        ImPeerKind,
    };

    #[test]
    fn builds_peer_conversation_id() {
        let surface = ImConversationSurface {
            channel: "feishu".to_string(),
            account_id: "default".to_string(),
            tenant_id: Some("tenant-a".to_string()),
            peer_kind: ImPeerKind::Group,
            peer_id: "oc_team".to_string(),
            topic_id: None,
            sender_id: None,
            scope: ImConversationScope::Peer,
            message_id: None,
            raw_thread_id: Some("oc_team".to_string()),
            raw_root_id: None,
        };

        assert_eq!(
            build_conversation_id(&surface),
            "feishu:default:group:oc_team"
        );
    }

    #[test]
    fn builds_topic_sender_conversation_with_parent_candidates() {
        let surface = ImConversationSurface {
            channel: "feishu".to_string(),
            account_id: "default".to_string(),
            tenant_id: Some("tenant-a".to_string()),
            peer_kind: ImPeerKind::Group,
            peer_id: "oc_team".to_string(),
            topic_id: Some("om_root_1".to_string()),
            sender_id: Some("ou_user_1".to_string()),
            scope: ImConversationScope::TopicSender,
            message_id: Some("om_msg_1".to_string()),
            raw_thread_id: Some("oc_team".to_string()),
            raw_root_id: Some("om_root_1".to_string()),
        };

        assert_eq!(
            build_conversation_id(&surface),
            "feishu:default:group:oc_team:topic:om_root_1:sender:ou_user_1"
        );
        assert_eq!(
            build_parent_conversation_candidates(&surface),
            vec![
                "feishu:default:group:oc_team:topic:om_root_1".to_string(),
                "feishu:default:group:oc_team".to_string(),
            ]
        );
    }

    #[test]
    fn falls_back_to_unknown_for_missing_topic_or_sender_components() {
        let surface = ImConversationSurface {
            channel: "dingtalk".to_string(),
            account_id: "robot".to_string(),
            tenant_id: None,
            peer_kind: ImPeerKind::Direct,
            peer_id: "user_1".to_string(),
            topic_id: None,
            sender_id: None,
            scope: ImConversationScope::TopicSender,
            message_id: None,
            raw_thread_id: None,
            raw_root_id: None,
        };

        assert_eq!(
            build_conversation_id(&surface),
            "dingtalk:robot:direct:user_1:topic:unknown:sender:unknown"
        );
        assert_eq!(
            build_parent_conversation_candidates(&surface),
            vec![
                "dingtalk:robot:direct:user_1:topic:unknown".to_string(),
                "dingtalk:robot:direct:user_1".to_string(),
            ]
        );
    }

    #[test]
    fn builds_full_identity_metadata_from_surface() {
        let surface = ImConversationSurface {
            channel: "feishu".to_string(),
            account_id: "default".to_string(),
            tenant_id: Some("tenant-a".to_string()),
            peer_kind: ImPeerKind::Group,
            peer_id: "oc_team".to_string(),
            topic_id: Some("om_root_1".to_string()),
            sender_id: Some("ou_user_1".to_string()),
            scope: ImConversationScope::TopicSender,
            message_id: Some("om_msg_1".to_string()),
            raw_thread_id: Some("oc_team".to_string()),
            raw_root_id: Some("om_root_1".to_string()),
        };

        let identity = build_conversation_identity(&surface);

        assert_eq!(
            identity.conversation_id,
            "feishu:default:group:oc_team:topic:om_root_1:sender:ou_user_1"
        );
        assert_eq!(
            identity.base_conversation_id,
            build_base_conversation_id(&surface)
        );
        assert_eq!(
            identity.parent_conversation_candidates,
            vec![
                "feishu:default:group:oc_team:topic:om_root_1".to_string(),
                "feishu:default:group:oc_team".to_string(),
            ]
        );
        assert_eq!(identity.scope, ImConversationScope::TopicSender);
        assert_eq!(identity.scope_label, "topic_sender");
        assert_eq!(identity.peer_kind_label, "group");
    }
}
