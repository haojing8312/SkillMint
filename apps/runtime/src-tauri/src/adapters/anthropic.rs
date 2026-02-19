use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{json, Value};
use futures_util::StreamExt;

pub async fn chat_stream(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<Value>,
    mut on_token: impl FnMut(String) + Send,
) -> Result<()> {
    let client = Client::new();
    let body = json!({
        "model": model,
        "max_tokens": 4096,
        "system": system_prompt,
        "messages": messages,
        "stream": true
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(anyhow!("Anthropic API error: {text}"));
    }

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    if let Some(token) = v["delta"]["text"].as_str() {
                        on_token(token.to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn test_connection(api_key: &str, model: &str) -> Result<bool> {
    let client = Client::new();
    let body = json!({
        "model": model,
        "max_tokens": 10,
        "messages": [{"role": "user", "content": "hi"}]
    });
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;
    Ok(resp.status().is_success())
}
