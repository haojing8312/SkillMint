use super::conversation_mapper::{build_feishu_conversation_metadata, FeishuConversationInput};
use super::types::ParsedFeishuPayload;
use crate::commands::im_host::project_im_event_conversation_metadata;
use crate::im::types::{ImEvent, ImEventType};

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuEnvelope {
    challenge: Option<String>,
    header: Option<FeishuHeader>,
    event: Option<FeishuEvent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuHeader {
    event_id: Option<String>,
    event_type: Option<String>,
    tenant_key: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuEvent {
    message: Option<FeishuMessage>,
    sender: Option<FeishuSender>,
    mentions: Option<Vec<FeishuMention>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuMessage {
    message_id: Option<String>,
    chat_id: Option<String>,
    chat_type: Option<String>,
    root_id: Option<String>,
    thread_id: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuSender {
    sender_id: Option<FeishuSenderId>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuSenderId {
    open_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuMention {
    key: Option<String>,
    #[serde(rename = "id")]
    mention_id: Option<FeishuMentionId>,
    #[serde(default)]
    open_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FeishuMentionId {
    open_id: Option<String>,
}

fn mention_open_id(mention: &FeishuMention) -> Option<String> {
    mention
        .mention_id
        .as_ref()
        .and_then(|id| id.open_id.clone())
        .or_else(|| mention.open_id.clone())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub(crate) fn strip_placeholder_mentions(mut text: String) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut cleaned = String::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '@' && i + 1 < chars.len() && chars[i + 1] == '_' {
            i += 2;
            while i < chars.len() {
                let c = chars[i];
                if c.is_ascii_alphanumeric() || c == '_' {
                    i += 1;
                    continue;
                }
                break;
            }
            continue;
        }
        cleaned.push(chars[i]);
        i += 1;
    }
    text.clear();
    text.push_str(
        cleaned
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .as_str(),
    );
    text
}

fn parse_message_text(raw: &str, mention_keys: &[String]) -> Option<String> {
    if raw.trim().is_empty() {
        return None;
    }
    let base = if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        v.get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(raw)
            .to_string()
    } else {
        raw.to_string()
    };
    let mut stripped = base;
    for key in mention_keys {
        stripped = stripped.replace(key, " ");
    }
    let stripped = strip_placeholder_mentions(stripped);
    if stripped.trim().is_empty() {
        None
    } else {
        Some(stripped)
    }
}

pub fn parse_feishu_payload(payload: &str) -> Result<ParsedFeishuPayload, String> {
    if let Ok(event) = serde_json::from_str::<ImEvent>(payload) {
        return Ok(ParsedFeishuPayload::Event(
            project_im_event_conversation_metadata(&event),
        ));
    }

    let env: FeishuEnvelope =
        serde_json::from_str(payload).map_err(|e| format!("invalid feishu payload: {}", e))?;
    if let Some(challenge) = env.challenge {
        return Ok(ParsedFeishuPayload::Challenge(challenge));
    }

    let header = env
        .header
        .ok_or_else(|| "feishu payload missing header".to_string())?;
    let event = env
        .event
        .ok_or_else(|| "feishu payload missing event".to_string())?;
    let message = event
        .message
        .ok_or_else(|| "feishu payload missing message".to_string())?;

    let mentions = event.mentions.unwrap_or_default();
    let mention_keys = mentions
        .iter()
        .filter_map(|m| m.key.as_ref().map(|v| v.trim().to_string()))
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    let content_text = parse_message_text(message.content.as_deref().unwrap_or(""), &mention_keys);
    let role_id = mentions.iter().find_map(mention_open_id);

    let event_type = match header
        .event_type
        .as_deref()
        .unwrap_or("im.message.receive_v1")
    {
        "im.message.receive_v1" => ImEventType::MessageCreated,
        "im.message.reaction.created_v1" => ImEventType::MessageCreated,
        other => {
            if other.contains("mention") {
                ImEventType::MentionRole
            } else {
                return Err(format!("unsupported feishu event_type: {}", other));
            }
        }
    };

    let chat_id = message
        .chat_id
        .clone()
        .ok_or_else(|| "feishu payload missing chat_id".to_string())?;
    let tenant_key = header.tenant_key.clone();
    let sender_id = event
        .sender
        .as_ref()
        .and_then(|sender| sender.sender_id.as_ref())
        .and_then(|id| id.open_id.clone());
    let metadata = build_feishu_conversation_metadata(&FeishuConversationInput {
        account_id: tenant_key.as_deref(),
        tenant_id: tenant_key.as_deref(),
        chat_id: &chat_id,
        chat_type: message.chat_type.as_deref(),
        sender_id: sender_id.as_deref(),
        message_id: message.message_id.as_deref(),
        root_id: message.root_id.as_deref(),
        thread_id: message.thread_id.as_deref(),
    });

    Ok(ParsedFeishuPayload::Event(ImEvent {
        channel: "feishu".to_string(),
        event_type,
        thread_id: chat_id,
        event_id: header.event_id,
        message_id: message.message_id,
        text: content_text,
        role_id,
        account_id: tenant_key.clone(),
        sender_id,
        chat_type: message.chat_type,
        tenant_id: tenant_key.or_else(|| {
            event
                .sender
                .and_then(|s| s.sender_id.and_then(|id| id.open_id))
        }),
        conversation_id: Some(metadata.conversation_id),
        base_conversation_id: Some(metadata.base_conversation_id),
        parent_conversation_candidates: metadata.parent_conversation_candidates,
        conversation_scope: Some(metadata.conversation_scope),
    }))
}
