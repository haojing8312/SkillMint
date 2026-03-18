use anyhow::Result;
use serde_json::{json, Value};

pub fn success(tool: &str, summary: impl Into<String>, details: Value) -> Result<String> {
    Ok(serde_json::to_string_pretty(&json!({
        "ok": true,
        "tool": tool,
        "summary": summary.into(),
        "details": details,
    }))?)
}

pub fn failure(
    tool: &str,
    summary: impl Into<String>,
    error_code: impl Into<String>,
    error_message: impl Into<String>,
    details: Value,
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&json!({
        "ok": false,
        "tool": tool,
        "summary": summary.into(),
        "error_code": error_code.into(),
        "error_message": error_message.into(),
        "details": details,
    }))?)
}
