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
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    if let Some(token) = v["choices"][0]["delta"]["content"].as_str() {
                        on_token(token.to_string());
                    }
                }
            }
        }
    }
    Ok(())
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
