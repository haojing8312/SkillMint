use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImPeerKind {
    Direct,
    Group,
}

impl ImPeerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Group => "group",
        }
    }

    pub fn is_direct(self) -> bool {
        matches!(self, Self::Direct)
    }

    pub fn is_group(self) -> bool {
        matches!(self, Self::Group)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImConversationScope {
    Peer,
    PeerSender,
    Topic,
    TopicSender,
}

impl ImConversationScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Peer => "peer",
            Self::PeerSender => "peer_sender",
            Self::Topic => "topic",
            Self::TopicSender => "topic_sender",
        }
    }

    pub fn is_peer_scope(self) -> bool {
        matches!(self, Self::Peer | Self::PeerSender)
    }

    pub fn is_topic_scope(self) -> bool {
        matches!(self, Self::Topic | Self::TopicSender)
    }

    pub fn includes_sender(self) -> bool {
        matches!(self, Self::PeerSender | Self::TopicSender)
    }

    pub fn parent_scope(self) -> Option<Self> {
        match self {
            Self::Peer => None,
            Self::PeerSender => Some(Self::Peer),
            Self::Topic => Some(Self::Peer),
            Self::TopicSender => Some(Self::Topic),
        }
    }

    pub fn has_parent_scope(self) -> bool {
        self.parent_scope().is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImConversationSurface {
    pub channel: String,
    pub account_id: String,
    #[serde(default)]
    pub tenant_id: Option<String>,
    pub peer_kind: ImPeerKind,
    pub peer_id: String,
    #[serde(default)]
    pub topic_id: Option<String>,
    #[serde(default)]
    pub sender_id: Option<String>,
    pub scope: ImConversationScope,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub raw_thread_id: Option<String>,
    #[serde(default)]
    pub raw_root_id: Option<String>,
}

impl ImConversationSurface {
    pub fn with_scope(&self, scope: ImConversationScope) -> Self {
        Self {
            channel: self.channel.clone(),
            account_id: self.account_id.clone(),
            tenant_id: self.tenant_id.clone(),
            peer_kind: self.peer_kind,
            peer_id: self.peer_id.clone(),
            topic_id: if scope.is_topic_scope() {
                self.topic_id.clone()
            } else {
                None
            },
            sender_id: if scope.includes_sender() {
                self.sender_id.clone()
            } else {
                None
            },
            scope,
            message_id: self.message_id.clone(),
            raw_thread_id: self.raw_thread_id.clone(),
            raw_root_id: self.raw_root_id.clone(),
        }
    }

    pub fn peer_surface(&self) -> Self {
        self.with_scope(ImConversationScope::Peer)
    }
}

#[cfg(test)]
mod tests {
    use super::{ImConversationScope, ImPeerKind};

    #[test]
    fn peer_kind_labels_match_storage_format() {
        assert_eq!(ImPeerKind::Direct.as_str(), "direct");
        assert_eq!(ImPeerKind::Group.as_str(), "group");
    }

    #[test]
    fn conversation_scope_labels_match_storage_format() {
        assert_eq!(ImConversationScope::Peer.as_str(), "peer");
        assert_eq!(ImConversationScope::PeerSender.as_str(), "peer_sender");
        assert_eq!(ImConversationScope::Topic.as_str(), "topic");
        assert_eq!(ImConversationScope::TopicSender.as_str(), "topic_sender");
    }
}
