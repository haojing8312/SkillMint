use serde_json::Value;

use super::{transcript_hygiene, transcript_repair};

pub(crate) fn sanitize_outbound_messages(messages: Vec<Value>, api_format: &str) -> Vec<Value> {
    let sanitized = transcript_hygiene::sanitize_reconstructed_messages(messages, api_format);
    transcript_repair::repair_outbound_messages(sanitized, api_format)
}

#[cfg(test)]
mod tests {
    use super::sanitize_outbound_messages;
    use serde_json::{json, Value};

    #[test]
    fn openai_policy_runs_hygiene_then_repair() {
        let messages = vec![
            json!({
                "role": "assistant",
                "content": Value::Null,
                "tool_calls": [
                    {
                        "id": "call-1",
                        "type": "function",
                        "function": {
                            "name": "read_file",
                            "arguments": {"path": "README.md"}
                        }
                    }
                ]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call-1",
                "content": "ok"
            }),
        ];

        let sanitized = sanitize_outbound_messages(messages, "openai");

        assert_eq!(sanitized.len(), 2);
        assert_eq!(
            sanitized[0]["tool_calls"][0]["function"]["arguments"].as_str(),
            Some("{\"path\":\"README.md\"}")
        );
        assert_eq!(sanitized[1]["role"].as_str(), Some("tool"));
    }
}
