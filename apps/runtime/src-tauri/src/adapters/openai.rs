use crate::agent::types::{LLMResponse, ToolCall};
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;

fn mock_response_text(model: &str, messages: &[Value]) -> String {
    let last_user = messages
        .iter()
        .rev()
        .find_map(|message| {
            if message["role"].as_str() == Some("user") {
                message["content"]
                    .as_str()
                    .map(|content| content.trim().to_string())
                    .filter(|content| !content.is_empty())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "未提供任务".to_string());
    format!("MOCK_RESPONSE [{}] {}", model, last_user)
}

fn is_mock_text_base_url(base_url: &str) -> bool {
    base_url.trim().eq_ignore_ascii_case("http://mock")
}

fn is_mock_tool_loop_base_url(base_url: &str) -> bool {
    base_url
        .trim()
        .eq_ignore_ascii_case("http://mock-tool-loop")
}

fn is_mock_repeat_invalid_write_file_base_url(base_url: &str) -> bool {
    base_url
        .trim()
        .eq_ignore_ascii_case("http://mock-repeat-invalid-write-file")
}

fn parse_tool_call_arguments(args_str: &str) -> Result<Value> {
    let trimmed = args_str.trim();
    if trimmed.is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(trimmed)
        .map_err(|e| anyhow!("工具参数 JSON 解析失败: {}; raw={}", e, trimmed))
}

fn build_http_client() -> Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| anyhow!("构建 OpenAI HTTP 客户端失败: {}", e))
}

fn validate_test_connection_response(body: &str) -> Result<bool> {
    let parsed: Value = serde_json::from_str(body).map_err(|_| {
        let preview: String = body.chars().take(160).collect();
        anyhow!("OpenAI 连接测试返回了非 JSON 内容: {}", preview)
    })?;

    if parsed.get("choices").and_then(Value::as_array).is_some() {
        return Ok(true);
    }

    if let Some(error_message) = parsed
        .get("error")
        .and_then(|error| error.get("message").or(Some(error)))
        .and_then(Value::as_str)
    {
        return Err(anyhow!("OpenAI 连接测试返回错误: {}", error_message));
    }

    Err(anyhow!(
        "OpenAI 连接测试返回了非标准响应，缺少 choices 字段"
    ))
}

/// Strip <think>…</think> spans from a streaming token chunk.
/// `in_think` carries state across chunk boundaries.
fn tag_matches(chars: &[char], start: usize, tag: &[char]) -> bool {
    start + tag.len() <= chars.len() && chars[start..start + tag.len()] == *tag
}

fn tag_prefix_at_end(chars: &[char], start: usize, tag: &[char]) -> bool {
    let remaining = chars.len().saturating_sub(start);
    remaining > 0 && remaining < tag.len() && chars[start..] == tag[..remaining]
}

fn filter_thinking(input: &str, in_think: &mut bool, pending_tag: &mut String) -> String {
    const OPEN_TAG: [char; 7] = ['<', 't', 'h', 'i', 'n', 'k', '>'];
    const CLOSE_TAG: [char; 8] = ['<', '/', 't', 'h', 'i', 'n', 'k', '>'];

    let mut combined = String::with_capacity(pending_tag.len() + input.len());
    combined.push_str(pending_tag);
    combined.push_str(input);
    pending_tag.clear();

    let chars: Vec<char> = combined.chars().collect();
    let mut out = String::with_capacity(combined.len());
    let mut index = 0;

    while index < chars.len() {
        if *in_think {
            if tag_matches(&chars, index, &CLOSE_TAG) {
                *in_think = false;
                index += CLOSE_TAG.len();
                continue;
            }

            if tag_prefix_at_end(&chars, index, &CLOSE_TAG) {
                pending_tag.extend(chars[index..].iter());
                break;
            }

            index += 1;
            continue;
        }

        if tag_matches(&chars, index, &OPEN_TAG) {
            *in_think = true;
            index += OPEN_TAG.len();
            continue;
        }

        if tag_prefix_at_end(&chars, index, &OPEN_TAG) {
            pending_tag.extend(chars[index..].iter());
            break;
        }

        out.push(chars[index]);
        index += 1;
    }

    out
}

#[derive(Default)]
struct OpenAiStreamState {
    text_content: String,
    in_think: bool,
    pending_think_tag: String,
    tool_calls_map: HashMap<u64, (String, String, String)>,
    finish_reason: Option<String>,
    stop_stream: bool,
    pending_line: String,
}

fn process_openai_sse_text(
    text: &str,
    state: &mut OpenAiStreamState,
    on_token: &mut impl FnMut(String),
) -> Result<()> {
    state.pending_line.push_str(text);
    let ends_with_newline = state.pending_line.ends_with('\n');
    let owned = std::mem::take(&mut state.pending_line);
    let mut lines: Vec<&str> = owned.lines().collect();

    if !ends_with_newline {
        state.pending_line = lines.pop().unwrap_or_default().to_string();
    }

    for line in lines {
        if let Some(data) = line.strip_prefix("data: ") {
            if data.trim() == "[DONE]" {
                state.stop_stream = true;
                break;
            }

            if let Ok(v) = serde_json::from_str::<Value>(data) {
                let choice = &v["choices"][0];
                let delta = &choice["delta"];

                if let Some(fr) = choice["finish_reason"].as_str() {
                    state.finish_reason = Some(fr.to_string());
                }

                if delta["reasoning_content"]
                    .as_str()
                    .map(|s| !s.is_empty())
                    .unwrap_or(false)
                {
                    continue;
                }

                if let Some(token) = delta["content"].as_str() {
                    let filtered =
                        filter_thinking(token, &mut state.in_think, &mut state.pending_think_tag);
                    if !filtered.is_empty() {
                        state.text_content.push_str(&filtered);
                        on_token(filtered);
                    }
                }

                if let Some(tc_array) = delta["tool_calls"].as_array() {
                    for tc_delta in tc_array {
                        let index = tc_delta["index"].as_u64().unwrap_or(0);

                        let entry = state
                            .tool_calls_map
                            .entry(index)
                            .or_insert_with(|| (String::new(), String::new(), String::new()));

                        if let Some(id) = tc_delta["id"].as_str() {
                            entry.0 = id.to_string();
                        }
                        if let Some(name) = tc_delta["function"]["name"].as_str() {
                            entry.1 = name.to_string();
                        }

                        if let Some(args) = tc_delta["function"]["arguments"].as_str() {
                            entry.2.push_str(args);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn finish_openai_stream(mut state: OpenAiStreamState) -> LLMResponse {
    if !state.in_think && !state.pending_think_tag.is_empty() {
        state.text_content.push_str(&state.pending_think_tag);
    }

    if state.finish_reason.as_deref() == Some("tool_calls") || !state.tool_calls_map.is_empty() {
        let mut indices: Vec<u64> = state.tool_calls_map.keys().cloned().collect();
        indices.sort();

        let tool_calls: Vec<ToolCall> = indices
            .into_iter()
            .map(|idx| {
                let (id, name, args_str) = state.tool_calls_map.remove(&idx).unwrap();
                let input = match parse_tool_call_arguments(&args_str) {
                    Ok(value) => value,
                    Err(err) => json!({
                        "__tool_call_parse_error": err.to_string(),
                        "__raw_arguments": args_str,
                    }),
                };
                ToolCall { id, name, input }
            })
            .collect();

        if !state.text_content.is_empty() {
            LLMResponse::TextWithToolCalls(state.text_content, tool_calls)
        } else {
            LLMResponse::ToolCalls(tool_calls)
        }
    } else {
        LLMResponse::Text(state.text_content)
    }
}

/// OpenAI 兼容的流式 tool calling
///
/// 将 Anthropic 格式的工具定义转换为 OpenAI function calling 格式，
/// 发送带 `tools` 和 `stream: true` 的请求，并解析增量 SSE delta 中的 tool_calls。
///
/// 当 `finish_reason == "tool_calls"` 时返回 `LLMResponse::ToolCalls`，
/// 否则返回 `LLMResponse::Text`。
pub async fn chat_stream_with_tools(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<Value>,
    tools: Vec<Value>,
    mut on_token: impl FnMut(String) + Send,
) -> Result<crate::agent::types::LLMResponse> {
    if is_mock_text_base_url(base_url) {
        let mock_text = mock_response_text(model, &messages);
        on_token(mock_text.clone());
        return Ok(LLMResponse::Text(mock_text));
    }
    if is_mock_tool_loop_base_url(base_url) {
        return Err(anyhow!("达到最大迭代次数 8"));
    }
    if is_mock_repeat_invalid_write_file_base_url(base_url) {
        return Ok(LLMResponse::ToolCalls(vec![ToolCall {
            id: "mock-write-file-empty".to_string(),
            name: "write_file".to_string(),
            input: json!({}),
        }]));
    }

    let client = build_http_client()?;

    // 构建消息数组，前置 system 消息
    let mut all_messages = vec![json!({"role": "system", "content": system_prompt})];
    all_messages.extend(messages);

    // 将 Anthropic 格式工具定义转换为 OpenAI function calling 格式
    let openai_tools: Vec<Value> = tools
        .iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t["name"],
                    "description": t["description"],
                    "parameters": t["input_schema"],
                }
            })
        })
        .collect();

    let body = json!({
        "model": model,
        "messages": all_messages,
        "tools": openai_tools,
        "stream": true,
    });

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(anyhow!("OpenAI API error: {}", text));
    }

    let mut stream = resp.bytes_stream();
    let mut state = OpenAiStreamState::default();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        process_openai_sse_text(&text, &mut state, &mut on_token)?;
        if state.stop_stream {
            break;
        }
    }

    Ok(finish_openai_stream(state))
}

pub async fn test_connection(base_url: &str, api_key: &str, model: &str) -> Result<bool> {
    if is_mock_text_base_url(base_url)
        || is_mock_tool_loop_base_url(base_url)
        || is_mock_repeat_invalid_write_file_base_url(base_url)
    {
        return Ok(true);
    }
    let client = build_http_client()?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 10
    });
    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(anyhow!("OpenAI API error: {}", text));
    }
    validate_test_connection_response(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn invalid_tool_arguments_should_not_silently_become_empty_object() {
        let parsed = parse_tool_call_arguments(r#"{"path":"brief.html""#);
        assert!(parsed.is_err(), "损坏的 tool arguments 应返回错误");
    }

    fn parse_openai_chunks_for_test(chunks: &[&str]) -> Result<LLMResponse> {
        let mut state = OpenAiStreamState::default();
        let mut sink = Vec::new();
        for chunk in chunks {
            process_openai_sse_text(chunk, &mut state, &mut |token| sink.push(token))?;
            if state.stop_stream {
                break;
            }
        }
        Ok(finish_openai_stream(state))
    }

    #[test]
    fn done_marker_stops_processing_later_chunks() {
        let response = parse_openai_chunks_for_test(&[
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n",
            "data: [DONE]\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"ignored\"},\"finish_reason\":null}]}\n",
        ])
        .expect("parse chunks");

        match response {
            LLMResponse::Text(text) => assert_eq!(text, "hello"),
            other => panic!("expected text response, got {other:?}"),
        }
    }

    #[test]
    fn done_marker_split_across_chunks_still_stops_stream() {
        let mut state = OpenAiStreamState::default();
        let mut sink = Vec::new();

        for chunk in [
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n",
            "data: [DO",
            "NE]\n",
        ] {
            process_openai_sse_text(chunk, &mut state, &mut |token| sink.push(token))
                .expect("parse chunk");
            if state.stop_stream {
                break;
            }
        }

        assert!(state.stop_stream, "split [DONE] marker should stop stream");
        match finish_openai_stream(state) {
            LLMResponse::Text(text) => assert_eq!(text, "hello"),
            other => panic!("expected text response, got {other:?}"),
        }
    }

    #[test]
    fn test_connection_rejects_html_success_pages() {
        let result = validate_test_connection_response(
            "<!doctype html><html><head><title>Gateway</title></head><body>ok</body></html>",
        );

        assert!(result.is_err(), "HTML 落地页不能被视为模型接口成功响应");
    }

    #[test]
    fn test_connection_accepts_openai_chat_completion_json() {
        let result = validate_test_connection_response(
            r#"{"id":"chatcmpl-1","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"ok"},"finish_reason":"stop"}]}"#,
        )
        .expect("valid openai response");

        assert!(result);
    }

    #[test]
    fn filter_thinking_keeps_multibyte_text_without_panicking() {
        let mut in_think = false;
        let mut pending_tag = String::new();
        let result = catch_unwind(AssertUnwindSafe(|| {
            filter_thinking("有什么", &mut in_think, &mut pending_tag)
        }));

        assert!(result.is_ok(), "多字节文本不应触发 panic");
        assert_eq!(result.unwrap(), "有什么");
        assert!(!in_think);
        assert!(pending_tag.is_empty());
    }

    #[test]
    fn filter_thinking_hides_cross_chunk_think_blocks() {
        let mut in_think = false;
        let mut pending_tag = String::new();

        let first = filter_thinking("<think>推理中", &mut in_think, &mut pending_tag);
        let second = filter_thinking("</think>你好", &mut in_think, &mut pending_tag);

        assert_eq!(first, "");
        assert_eq!(second, "你好");
        assert!(!in_think);
        assert!(pending_tag.is_empty());
    }

    #[test]
    fn filter_thinking_handles_split_open_tag_across_chunks() {
        let mut in_think = false;
        let mut pending_tag = String::new();

        let first = filter_thinking("<thi", &mut in_think, &mut pending_tag);
        let second = filter_thinking("nk>内部</think>结果", &mut in_think, &mut pending_tag);

        assert_eq!(first, "");
        assert_eq!(second, "结果");
        assert!(!in_think);
        assert!(pending_tag.is_empty());
    }

    #[test]
    fn filter_thinking_preserves_multibyte_text_after_think_block() {
        let response = parse_openai_chunks_for_test(&[
            "data: {\"choices\":[{\"delta\":{\"content\":\"<think>内部推理\"},\"finish_reason\":null}]}\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"</think>有什么文件夹\"},\"finish_reason\":null}]}\n",
            "data: [DONE]\n",
        ])
        .expect("parse chunks");

        match response {
            LLMResponse::Text(text) => assert_eq!(text, "有什么文件夹"),
            other => panic!("expected text response, got {other:?}"),
        }
    }
}
