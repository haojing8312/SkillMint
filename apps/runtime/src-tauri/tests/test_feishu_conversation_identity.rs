use runtime_lib::commands::feishu_gateway::{parse_feishu_payload, ParsedFeishuPayload};
use runtime_lib::im::{ImConversationScope, ImPeerKind};

#[test]
fn parse_feishu_payload_assigns_peer_conversation_metadata() {
    let payload = serde_json::json!({
        "header": {
            "event_id": "evt_peer_1",
            "event_type": "im.message.receive_v1",
            "tenant_key": "tenant_peer"
        },
        "event": {
            "message": {
                "message_id": "om_peer_1",
                "chat_id": "oc_peer_1",
                "chat_type": "group",
                "content": "{\"text\":\"请继续推进\"}"
            },
            "sender": {
                "sender_id": {
                    "open_id": "ou_sender_1"
                }
            }
        }
    });

    let parsed = parse_feishu_payload(&payload.to_string()).expect("payload should parse");
    match parsed {
        ParsedFeishuPayload::Event(event) => {
            assert_eq!(
                event.conversation_id.as_deref(),
                Some("feishu:tenant_peer:group:oc_peer_1")
            );
            assert_eq!(
                event.base_conversation_id.as_deref(),
                Some("feishu:tenant_peer:group:oc_peer_1")
            );
            assert!(event.parent_conversation_candidates.is_empty());
            assert_eq!(event.conversation_scope.as_deref(), Some("peer"));
        }
        ParsedFeishuPayload::Challenge(_) => panic!("should parse event"),
    }
}

#[test]
fn parse_feishu_payload_assigns_topic_conversation_metadata() {
    let payload = serde_json::json!({
        "header": {
            "event_id": "evt_topic_1",
            "event_type": "im.message.receive_v1",
            "tenant_key": "tenant_topic"
        },
        "event": {
            "message": {
                "message_id": "om_topic_reply_1",
                "chat_id": "oc_topic_chat_1",
                "chat_type": "group",
                "root_id": "om_topic_root_1",
                "thread_id": "omt_topic_1",
                "content": "{\"text\":\"继续这个主题\"}"
            },
            "sender": {
                "sender_id": {
                    "open_id": "ou_sender_2"
                }
            }
        }
    });

    let parsed = parse_feishu_payload(&payload.to_string()).expect("payload should parse");
    match parsed {
        ParsedFeishuPayload::Event(event) => {
            assert_eq!(
                event.conversation_id.as_deref(),
                Some("feishu:tenant_topic:group:oc_topic_chat_1:topic:om_topic_root_1")
            );
            assert_eq!(
                event.base_conversation_id.as_deref(),
                Some("feishu:tenant_topic:group:oc_topic_chat_1")
            );
            assert_eq!(
                event.parent_conversation_candidates,
                vec!["feishu:tenant_topic:group:oc_topic_chat_1".to_string()]
            );
            assert_eq!(event.conversation_scope.as_deref(), Some("topic"));
        }
        ParsedFeishuPayload::Challenge(_) => panic!("should parse event"),
    }
}

#[test]
fn feishu_identity_labels_remain_stable_for_group_peer_and_topic_scopes() {
    assert_eq!(ImPeerKind::Group.as_str(), "group");
    assert_eq!(ImPeerKind::Direct.as_str(), "direct");
    assert_eq!(ImConversationScope::Peer.as_str(), "peer");
    assert_eq!(ImConversationScope::Topic.as_str(), "topic");
    assert!(ImConversationScope::Peer.is_peer_scope());
    assert!(ImConversationScope::Topic.is_topic_scope());
    assert!(ImConversationScope::Topic.has_parent_scope());
    assert_eq!(
        ImConversationScope::Topic.parent_scope(),
        Some(ImConversationScope::Peer)
    );
}
