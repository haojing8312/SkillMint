use serde::{Deserialize, Serialize};

use crate::im::conversation_surface::ImConversationScope;

fn default_im_channel() -> String {
    "feishu".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImEvent {
    #[serde(default = "default_im_channel")]
    pub channel: String,
    pub event_type: ImEventType,
    pub thread_id: String,
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub role_id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub sender_id: Option<String>,
    #[serde(default)]
    pub chat_type: Option<String>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub base_conversation_id: Option<String>,
    #[serde(default)]
    pub parent_conversation_candidates: Vec<String>,
    #[serde(default)]
    pub conversation_scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImEventType {
    #[serde(rename = "message.created")]
    MessageCreated,
    #[serde(rename = "mention.role")]
    MentionRole,
    #[serde(rename = "command.pause")]
    CommandPause,
    #[serde(rename = "command.resume")]
    CommandResume,
    #[serde(rename = "human.override")]
    HumanOverride,
}

impl ImEvent {
    pub fn conversation_id_or_thread_id(&self) -> &str {
        self.conversation_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.thread_id.trim())
    }

    pub fn base_conversation_id_or_current(&self) -> &str {
        self.base_conversation_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.conversation_id_or_thread_id())
    }

    pub fn conversation_scope_label(&self) -> &str {
        self.conversation_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(ImConversationScope::Peer.as_str())
    }
}
