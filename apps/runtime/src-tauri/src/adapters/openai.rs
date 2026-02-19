use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{json, Value};
use futures_util::StreamExt;

pub async fn chat_stream(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<Value>,
    mut on_token: impl FnMut(String) + Send,
) -> Result<()> {
    let client = Client::new();
    let mut all_messages = vec![json!({"role": "system", "content": system_prompt})];
    all_messages.extend(messages);

    let body = json!({
        "model": model,
        "messages": all_messages,
        "stream": true
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
        return Err(anyhow!("API error: {text}"));
    }

    let mut stream = resp.bytes_stream();
    let mut in_think = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let delta = &v["choices"][0]["delta"];
                    // Skip DeepSeek reasoning_content tokens entirely
                    if delta["reasoning_content"].as_str().map(|s| !s.is_empty()).unwrap_or(false) {
                        continue;
                    }
                    if let Some(token) = delta["content"].as_str() {
                        let filtered = filter_thinking(token, &mut in_think);
                        if !filtered.is_empty() {
                            on_token(filtered);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Strip <think>…</think> spans from a streaming token chunk.
/// `in_think` carries state across chunk boundaries.
fn filter_thinking(input: &str, in_think: &mut bool) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut buf = String::new();

    while let Some(c) = chars.next() {
        buf.push(c);
        if *in_think {
            // Look for </think>
            if buf.ends_with("</think>") {
                *in_think = false;
                buf.clear();
            }
            // Keep buf bounded so it doesn't grow unbounded on large thinking blocks
            if buf.len() > 16 { buf = buf[buf.len()-16..].to_string(); }
        } else {
            // Look for <think>
            if buf.ends_with("<think>") {
                *in_think = true;
                // Remove the <think> prefix we may have already added to out
                let clean_len = out.len().saturating_sub(6); // len("<think>") - 1
                out.truncate(clean_len);
                buf.clear();
            } else {
                // Safe to emit everything except the last 6 chars (potential partial tag)
                if buf.len() > 7 {
                    let safe = buf.len() - 7;
                    out.push_str(&buf[..safe]);
                    buf = buf[safe..].to_string();
                }
            }
        }
    }
    // Flush remaining buffer if not in a thinking block
    if !*in_think {
        out.push_str(&buf);
    }
    out
}

pub async fn test_connection(base_url: &str, api_key: &str, model: &str) -> Result<bool> {
    let client = Client::new();
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 10
    });
    let resp = client.post(&url).bearer_auth(api_key).json(&body).send().await?;
    Ok(resp.status().is_success())
}
