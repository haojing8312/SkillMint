use crate::im::conversation_surface::{ImConversationScope, ImConversationSurface, ImPeerKind};

fn peer_kind_label(peer_kind: ImPeerKind) -> &'static str {
    peer_kind.as_str()
}

fn has_topic_id(surface: &ImConversationSurface) -> bool {
    surface
        .topic_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
}

fn has_sender_id(surface: &ImConversationSurface) -> bool {
    surface
        .sender_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
}

fn effective_scope(surface: &ImConversationSurface) -> ImConversationScope {
    match surface.scope {
        ImConversationScope::Peer => ImConversationScope::Peer,
        ImConversationScope::PeerSender => {
            if has_sender_id(surface) {
                ImConversationScope::PeerSender
            } else {
                ImConversationScope::Peer
            }
        }
        ImConversationScope::Topic => {
            if has_topic_id(surface) {
                ImConversationScope::Topic
            } else {
                ImConversationScope::Peer
            }
        }
        ImConversationScope::TopicSender => match (has_topic_id(surface), has_sender_id(surface)) {
            (true, true) => ImConversationScope::TopicSender,
            (true, false) => ImConversationScope::Topic,
            (false, true) => ImConversationScope::PeerSender,
            (false, false) => ImConversationScope::Peer,
        },
    }
}

pub fn build_conversation_id(surface: &ImConversationSurface) -> String {
    match effective_scope(surface) {
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
            surface
                .sender_id
                .as_deref()
                .expect("effective peer_sender scope requires sender_id")
        ),
        ImConversationScope::Topic => format!(
            "{}:{}:{}:{}:topic:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(surface.peer_kind),
            surface.peer_id,
            surface
                .topic_id
                .as_deref()
                .expect("effective topic scope requires topic_id")
        ),
        ImConversationScope::TopicSender => format!(
            "{}:{}:{}:{}:topic:{}:sender:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(surface.peer_kind),
            surface.peer_id,
            surface
                .topic_id
                .as_deref()
                .expect("effective topic_sender scope requires topic_id"),
            surface
                .sender_id
                .as_deref()
                .expect("effective topic_sender scope requires sender_id")
        ),
    }
}

pub fn build_parent_conversation_candidates(surface: &ImConversationSurface) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut scope = effective_scope(surface);

    while let Some(parent_scope) = scope.parent_scope() {
        candidates.push(build_conversation_id(&surface.with_scope(parent_scope)));
        scope = parent_scope;
    }

    candidates
}

#[cfg(test)]
mod tests {
    use crate::im::{
        build_conversation_id, build_parent_conversation_candidates, ImConversationScope,
        ImConversationSurface, ImPeerKind,
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
    fn degrades_missing_topic_or_sender_components_to_stable_narrower_scopes() {
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
            "dingtalk:robot:direct:user_1"
        );
        assert!(build_parent_conversation_candidates(&surface).is_empty());
    }

    #[test]
    fn degrades_topic_sender_to_peer_sender_when_topic_missing() {
        let surface = ImConversationSurface {
            channel: "feishu".to_string(),
            account_id: "default".to_string(),
            tenant_id: Some("tenant-a".to_string()),
            peer_kind: ImPeerKind::Group,
            peer_id: "oc_team".to_string(),
            topic_id: None,
            sender_id: Some("ou_user_1".to_string()),
            scope: ImConversationScope::TopicSender,
            message_id: Some("om_msg_1".to_string()),
            raw_thread_id: Some("oc_team".to_string()),
            raw_root_id: None,
        };

        assert_eq!(
            build_conversation_id(&surface),
            "feishu:default:group:oc_team:sender:ou_user_1"
        );
        assert_eq!(
            build_parent_conversation_candidates(&surface),
            vec!["feishu:default:group:oc_team".to_string()]
        );
    }

    #[test]
    fn degrades_topic_sender_to_topic_when_sender_missing() {
        let surface = ImConversationSurface {
            channel: "feishu".to_string(),
            account_id: "default".to_string(),
            tenant_id: Some("tenant-a".to_string()),
            peer_kind: ImPeerKind::Group,
            peer_id: "oc_team".to_string(),
            topic_id: Some("om_root_1".to_string()),
            sender_id: None,
            scope: ImConversationScope::TopicSender,
            message_id: Some("om_msg_1".to_string()),
            raw_thread_id: Some("oc_team".to_string()),
            raw_root_id: Some("om_root_1".to_string()),
        };

        assert_eq!(
            build_conversation_id(&surface),
            "feishu:default:group:oc_team:topic:om_root_1"
        );
        assert_eq!(
            build_parent_conversation_candidates(&surface),
            vec!["feishu:default:group:oc_team".to_string()]
        );
    }

    #[test]
    fn degrades_topic_without_topic_id_to_peer() {
        let surface = ImConversationSurface {
            channel: "feishu".to_string(),
            account_id: "default".to_string(),
            tenant_id: Some("tenant-a".to_string()),
            peer_kind: ImPeerKind::Group,
            peer_id: "oc_team".to_string(),
            topic_id: None,
            sender_id: None,
            scope: ImConversationScope::Topic,
            message_id: Some("om_msg_1".to_string()),
            raw_thread_id: Some("oc_team".to_string()),
            raw_root_id: None,
        };

        assert_eq!(
            build_conversation_id(&surface),
            "feishu:default:group:oc_team"
        );
        assert!(build_parent_conversation_candidates(&surface).is_empty());
    }
}
