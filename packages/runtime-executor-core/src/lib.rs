use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;

pub const MAX_TOOL_OUTPUT_CHARS: usize = 30_000;
pub const REPEATED_TOOL_FAILURE_THRESHOLD: usize = 3;
pub const TOOL_CALL_PARSE_ERROR_KEY: &str = "__tool_call_parse_error";
pub const CHARS_PER_TOKEN: usize = 4;
pub const DEFAULT_TOKEN_BUDGET: usize = 100_000;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolFailureStreak {
    pub signature: String,
    pub error: String,
    pub count: usize,
}

fn parse_structured_tool_output(output: &str) -> Option<Value> {
    let trimmed = output.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    let parsed: Value = serde_json::from_str(trimmed).ok()?;
    if parsed.get("summary").is_some()
        || parsed.get("details").is_some()
        || parsed.get("error_code").is_some()
    {
        Some(parsed)
    } else {
        None
    }
}

fn structured_tool_summary(output: &str) -> Option<String> {
    let parsed = parse_structured_tool_output(output)?;
    parsed
        .get("summary")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn structured_tool_error_parts(output: &str) -> Option<(String, String)> {
    let parsed = parse_structured_tool_output(output)?;
    let code = parsed
        .get("error_code")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("SKILL_EXECUTION_ERROR")
        .to_string();
    let message = parsed
        .get("error_message")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("summary").and_then(Value::as_str))
        .unwrap_or(output)
        .to_string();
    Some((code, message))
}

pub fn truncate_tool_output(output: &str, max_chars: usize) -> String {
    if output.len() <= max_chars {
        return output.to_string();
    }
    if let Some(mut parsed) = parse_structured_tool_output(output) {
        parsed["details"] = json!({
            "truncated": true,
            "note": format!("原始输出共 {} 字符，已为上下文压缩 details", output.len()),
        });
        return serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| {
            format!(
                "{}\n\n[输出已截断，共 {} 字符，已保留结构化摘要]",
                structured_tool_summary(output).unwrap_or_else(|| "[结构化工具结果]".to_string()),
                output.len()
            )
        });
    }
    let truncated: String = output.chars().take(max_chars).collect();
    format!(
        "{}\n\n[输出已截断，共 {} 字符，已显示前 {} 字符]",
        truncated,
        output.len(),
        max_chars
    )
}

pub fn stable_tool_input_signature(input: &Value) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| "<unserializable>".to_string())
}

pub fn extract_tool_call_parse_error(input: &Value) -> Option<String> {
    input
        .get(TOOL_CALL_PARSE_ERROR_KEY)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub fn update_tool_failure_streak(
    streak: &mut Option<ToolFailureStreak>,
    tool_name: &str,
    input: &Value,
    error: &str,
) -> Option<String> {
    let normalized_error = structured_tool_error_parts(error)
        .map(|(_, msg)| msg)
        .unwrap_or_else(|| error.to_string());
    let signature = format!("{}:{}", tool_name, stable_tool_input_signature(input));
    match streak {
        Some(current) if current.signature == signature && current.error == normalized_error => {
            current.count += 1;
            if current.count >= REPEATED_TOOL_FAILURE_THRESHOLD {
                Some(format!(
                    "检测到同一工具重复调用且持续失败，已停止自动重试。工具: {}，错误: {}",
                    tool_name, normalized_error
                ))
            } else {
                None
            }
        }
        _ => {
            *streak = Some(ToolFailureStreak {
                signature,
                error: normalized_error,
                count: 1,
            });
            None
        }
    }
}

pub fn estimate_tokens(messages: &[Value]) -> usize {
    let total_chars: usize = messages
        .iter()
        .map(|m| {
            let text_len = m["content"].as_str().map_or(0, |s| s.len());
            let array_len = m["content"].as_array().map_or(0, |arr| {
                arr.iter()
                    .map(|v| serde_json::to_string(v).map_or(0, |s| s.len()))
                    .sum()
            });
            text_len + array_len
        })
        .sum();
    total_chars / CHARS_PER_TOKEN
}

pub fn micro_compact(messages: &[Value], keep_recent: usize) -> Vec<Value> {
    let tool_result_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m["content"].as_array().is_some_and(|arr| {
                arr.iter()
                    .any(|v| v["type"].as_str() == Some("tool_result"))
            }) || m["role"].as_str() == Some("tool")
        })
        .map(|(i, _)| i)
        .collect();

    if tool_result_indices.len() <= keep_recent {
        return messages.to_vec();
    }

    let cutoff = tool_result_indices.len() - keep_recent;
    let old_indices: HashSet<usize> = tool_result_indices[..cutoff].iter().copied().collect();

    messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if old_indices.contains(&i) {
                if m["role"].as_str() == Some("tool") {
                    json!({
                        "role": "tool",
                        "tool_call_id": m["tool_call_id"],
                        "content": "[已执行]"
                    })
                } else {
                    let replaced = m["content"].as_array().map(|arr| {
                        arr.iter()
                            .map(|v| {
                                if v["type"].as_str() == Some("tool_result") {
                                    let content = v["content"].as_str().unwrap_or_default();
                                    let compact_content = structured_tool_summary(content)
                                        .map(|summary| format!("[已执行] {}", summary))
                                        .unwrap_or_else(|| "[已执行]".to_string());
                                    json!({
                                        "type": "tool_result",
                                        "tool_use_id": v["tool_use_id"],
                                        "content": compact_content
                                    })
                                } else {
                                    v.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                    });
                    match replaced {
                        Some(arr) => json!({"role": "user", "content": arr}),
                        None => m.clone(),
                    }
                }
            } else {
                m.clone()
            }
        })
        .collect()
}

pub fn trim_messages(messages: &[Value], token_budget: usize) -> Vec<Value> {
    if messages.len() <= 2 || estimate_tokens(messages) <= token_budget {
        return messages.to_vec();
    }

    let first = &messages[0];
    let last = &messages[messages.len() - 1];

    let budget_chars = token_budget * CHARS_PER_TOKEN * 70 / 100;
    let first_chars = first["content"].as_str().map_or(0, |s| s.len());
    let last_chars = last["content"].as_str().map_or(0, |s| s.len());
    let mut char_count = first_chars + last_chars;

    let mut keep_from_end: Vec<&Value> = Vec::new();

    for msg in messages[1..messages.len() - 1].iter().rev() {
        let msg_chars = msg["content"].as_str().map_or(0, |s| s.len())
            + msg["content"].as_array().map_or(0, |arr| {
                arr.iter()
                    .map(|v| serde_json::to_string(v).map_or(0, |s| s.len()))
                    .sum()
            });
        if char_count + msg_chars > budget_chars {
            break;
        }
        char_count += msg_chars;
        keep_from_end.push(msg);
    }
    keep_from_end.reverse();

    let trimmed_count = messages.len() - 2 - keep_from_end.len();
    let mut result = vec![first.clone()];

    if trimmed_count > 0 {
        result.push(json!({
            "role": "user",
            "content": format!("[前 {} 条消息已省略]", trimmed_count)
        }));
    }

    for msg in keep_from_end {
        result.push(msg.clone());
    }
    result.push(last.clone());

    result
}

pub fn split_error_code_and_message(text: &str) -> (String, String) {
    if let Some((code, msg)) = structured_tool_error_parts(text) {
        return (code, msg);
    }
    if let Some((code, msg)) = text.split_once(':') {
        let code = code.trim();
        if !code.is_empty()
            && code
                .chars()
                .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
        {
            return (code.to_string(), msg.trim().to_string());
        }
    }
    ("SKILL_EXECUTION_ERROR".to_string(), text.to_string())
}
