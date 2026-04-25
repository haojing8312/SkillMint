use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelErrorKind {
    Billing,
    Auth,
    RateLimit,
    ContextOverflow,
    InvalidTokenBudget,
    MediaTooLarge,
    Timeout,
    Network,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NormalizedModelError {
    pub kind: ModelErrorKind,
    pub raw_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelConnectionTestResult {
    pub ok: bool,
    pub kind: ModelErrorKind,
    pub title: String,
    pub message: String,
    pub raw_message: Option<String>,
}

pub(crate) fn normalize_model_error(raw_message: &str) -> NormalizedModelError {
    let lower = normalized_error_search_text(raw_message);
    let kind = if lower.contains("insufficient_balance")
        || lower.contains("insufficient balance")
        || lower.contains("balance too low")
        || lower.contains("account balance too low")
        || lower.contains("insufficient_quota")
        || lower.contains("insufficient quota")
        || lower.contains("billing")
        || lower.contains("payment required")
        || lower.contains("credit balance")
        || lower.contains("余额不足")
        || lower.contains("欠费")
    {
        ModelErrorKind::Billing
    } else if lower.contains("api key")
        || lower.contains("unauthorized")
        || lower.contains("invalid_api_key")
        || lower.contains("authentication")
        || lower.contains("permission denied")
        || lower.contains("forbidden")
    {
        ModelErrorKind::Auth
    } else if is_rate_limit_error(&lower) {
        ModelErrorKind::RateLimit
    } else if is_context_overflow_error(&lower) {
        ModelErrorKind::ContextOverflow
    } else if is_invalid_token_budget_error(&lower) {
        ModelErrorKind::InvalidTokenBudget
    } else if is_media_too_large_error(&lower) {
        ModelErrorKind::MediaTooLarge
    } else if lower.contains("timeout") || lower.contains("timed out") || lower.contains("deadline")
    {
        ModelErrorKind::Timeout
    } else if is_retryable_minimax_gateway_error(&lower) {
        ModelErrorKind::Network
    } else if lower.contains("connection")
        || lower.contains("network")
        || lower.contains("dns")
        || lower.contains("connect")
        || lower.contains("socket")
        || lower.contains("decoding response body")
        || lower.contains("decode response body")
        || lower.contains("error decoding response body")
        || lower.contains("error sending request for url")
        || lower.contains("sending request for url")
    {
        ModelErrorKind::Network
    } else {
        ModelErrorKind::Unknown
    };

    NormalizedModelError {
        kind,
        raw_message: raw_message.to_string(),
    }
}

pub(crate) fn model_error_title(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Billing => "模型余额不足",
        ModelErrorKind::Auth => "鉴权失败",
        ModelErrorKind::RateLimit => "请求过于频繁",
        ModelErrorKind::ContextOverflow => "上下文过长",
        ModelErrorKind::InvalidTokenBudget => "模型输出空间不足",
        ModelErrorKind::MediaTooLarge => "附件或图片过大",
        ModelErrorKind::Timeout => "请求超时",
        ModelErrorKind::Network => "网络连接失败",
        ModelErrorKind::Unknown => "连接失败",
    }
}

pub(crate) fn model_error_message(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Billing => {
            "当前模型平台返回余额或额度不足，请到对应服务商控制台充值或检查套餐额度。"
        }
        ModelErrorKind::Auth => "请检查 API Key、组织权限或接口访问范围是否正确。",
        ModelErrorKind::RateLimit => "模型平台当前触发限流，请稍后重试或降低并发频率。",
        ModelErrorKind::ContextOverflow => {
            "当前会话内容超过了模型可处理的上下文。请减少历史内容、开启新会话，或使用更大上下文的模型。"
        }
        ModelErrorKind::InvalidTokenBudget => {
            "模型请求没有剩余空间生成回复。请减少当前会话上下文、压缩图片，或使用更大上下文的模型后重试。"
        }
        ModelErrorKind::MediaTooLarge => {
            "上传的图片或附件超过了当前模型请求限制。请压缩图片、减少附件数量，或移除不必要的附件后重试。"
        }
        ModelErrorKind::Timeout => "模型平台响应超时，请稍后重试，或检查网络和所选模型是否可用。",
        ModelErrorKind::Network => "无法连接到模型接口，请检查 Base URL、网络环境或代理配置。",
        ModelErrorKind::Unknown => "模型平台返回了未识别错误，可查看详细信息进一步排查。",
    }
}

pub(crate) fn build_failed_connection_test_result(raw_message: &str) -> ModelConnectionTestResult {
    let normalized = normalize_model_error(raw_message);
    ModelConnectionTestResult {
        ok: false,
        kind: normalized.kind,
        title: model_error_title(normalized.kind).to_string(),
        message: model_error_message(normalized.kind).to_string(),
        raw_message: Some(normalized.raw_message),
    }
}

pub(crate) fn build_success_connection_test_result() -> ModelConnectionTestResult {
    ModelConnectionTestResult {
        ok: true,
        kind: ModelErrorKind::Unknown,
        title: "连接成功".to_string(),
        message: "模型连接测试成功。".to_string(),
        raw_message: None,
    }
}

fn normalized_error_search_text(raw_message: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<Value>(raw_message) {
        let mut parts = Vec::new();
        collect_error_strings(&parsed, &mut parts);
        if !parts.is_empty() {
            return parts.join(" ").to_ascii_lowercase();
        }
    }
    raw_message.to_ascii_lowercase()
}

fn has_tpm_rate_limit_hint(lower: &str) -> bool {
    lower.contains("tokens per minute") || lower.contains(" tpm") || lower.contains("tpm ")
}

fn is_rate_limit_error(lower: &str) -> bool {
    lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("overloaded_error")
        || lower.contains("high traffic detected")
        || lower.contains("quota")
        || has_tpm_rate_limit_hint(lower)
        || contains_http_status_code(lower, "429")
        || (contains_numeric_code(lower, "529")
            && (lower.contains("overloaded") || lower.contains("high traffic")))
}

fn contains_http_status_code(lower: &str, code: &str) -> bool {
    lower.contains(&format!("http {code}"))
        || lower.contains(&format!("http status {code}"))
        || lower.contains(&format!("status {code}"))
        || lower.contains(&format!("status_code {code}"))
        || lower.contains(&format!("status code {code}"))
}

fn contains_numeric_code(lower: &str, code: &str) -> bool {
    lower.match_indices(code).any(|(index, _)| {
        let before = lower[..index].chars().next_back();
        let after = lower[index + code.len()..].chars().next();

        before.map_or(true, |character| !character.is_ascii_digit())
            && after.map_or(true, |character| !character.is_ascii_digit())
    })
}

fn is_context_overflow_error(lower: &str) -> bool {
    if has_tpm_rate_limit_hint(lower) {
        return false;
    }
    lower.contains("prompt is too long")
        || lower.contains("prompt too long")
        || lower.contains("context length exceeded")
        || lower.contains("maximum context length")
        || lower.contains("context window exceeded")
        || lower.contains("context_window_exceeded")
        || lower.contains("model_context_window_exceeded")
        || lower.contains("exceeds model context window")
        || lower.contains("model token limit")
        || lower.contains("exceed context limit")
        || lower.contains("exceeds the model's maximum context")
        || (lower.contains("input length") && lower.contains("exceed") && lower.contains("context"))
        || (lower.contains("max_tokens") && lower.contains("exceed") && lower.contains("context"))
        || lower.contains("上下文过长")
        || lower.contains("上下文超出")
        || lower.contains("上下文长度超")
        || lower.contains("超出最大上下文")
        || lower.contains("请压缩上下文")
}

fn is_invalid_token_budget_error(lower: &str) -> bool {
    lower.contains("max_tokens must be at least 1")
        || (lower.contains("max_tokens") && lower.contains("got -"))
}

fn is_media_too_large_error(lower: &str) -> bool {
    lower.contains("image exceeds")
        || lower.contains("image dimensions exceed")
        || lower.contains("media too large")
        || lower.contains("payload too large")
        || lower.contains("request too large")
        || lower.contains("request size exceeds")
        || lower.contains("request exceeds the maximum size")
        || lower.contains("附件或图片过大")
        || lower.contains("图片附件总大小")
}

fn is_retryable_minimax_gateway_error(lower: &str) -> bool {
    lower.contains("unknown error, 794") && lower.contains("(1000)")
}

fn collect_error_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_error_strings(item, out);
            }
        }
        Value::Object(map) => {
            for key in ["message", "code", "type", "error", "detail"] {
                if let Some(value) = map.get(key) {
                    collect_error_strings(value, out);
                }
            }
            for value in map.values() {
                collect_error_strings(value, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_model_error_detects_billing_from_balance_text() {
        let result = normalize_model_error("insufficient_balance: account balance too low");
        assert_eq!(result.kind, ModelErrorKind::Billing);
    }

    #[test]
    fn normalize_model_error_detects_auth_from_invalid_key_text() {
        let result = normalize_model_error("Unauthorized: invalid_api_key");
        assert_eq!(result.kind, ModelErrorKind::Auth);
    }

    #[test]
    fn normalize_model_error_extracts_openai_json_error_message() {
        let raw = r#"{"error":{"message":"insufficient_quota","code":"insufficient_quota"}}"#;
        let result = normalize_model_error(raw);
        assert_eq!(result.kind, ModelErrorKind::Billing);
    }

    #[test]
    fn normalize_model_error_handles_plain_text_gateway_errors() {
        let raw = "error sending request for url (https://provider.example/v1/chat/completions)";
        let result = normalize_model_error(raw);
        assert_eq!(result.kind, ModelErrorKind::Network);
    }

    #[test]
    fn normalize_model_error_treats_response_body_decode_failures_as_network_errors() {
        let result = normalize_model_error("error decoding response body");
        assert_eq!(result.kind, ModelErrorKind::Network);
    }

    #[test]
    fn normalize_model_error_treats_minimax_794_gateway_failures_as_network_errors() {
        let raw = r#"{"type":"error","error":{"type":"api_error","message":"unknown error, 794 (1000)"},"request_id":"0619614fa6873d3861ed0c9dfe062551"}"#;
        let result = normalize_model_error(raw);
        assert_eq!(result.kind, ModelErrorKind::Network);
    }

    #[test]
    fn normalize_model_error_treats_minimax_529_overloaded_as_rate_limit() {
        let raw = r#"{"type":"error","error":{"type":"overloaded_error","message":"High traffic detected. For a more stable experience, upgrade to our Plus plan and use the highspeed model. (2064) (529)"},"request_id":"063620c00704398bce2eaa17b0537c68"}"#;
        let result = normalize_model_error(raw);
        assert_eq!(result.kind, ModelErrorKind::RateLimit);
    }

    #[test]
    fn normalize_model_error_context_overflow_wins_over_numeric_429_token_counts() {
        let result = normalize_model_error("prompt is too long: 429000 tokens > 200000 maximum");
        assert_eq!(result.kind, ModelErrorKind::ContextOverflow);
    }

    #[test]
    fn normalize_model_error_still_detects_http_429_rate_limit() {
        let result = normalize_model_error("HTTP 429 Too Many Requests");
        assert_eq!(result.kind, ModelErrorKind::RateLimit);
    }

    #[test]
    fn normalize_model_error_context_overflow_wins_when_http_text_contains_token_count_429() {
        let result =
            normalize_model_error("HTTP 400: prompt is too long: 429 tokens > 200 maximum");
        assert_eq!(result.kind, ModelErrorKind::ContextOverflow);
    }

    #[test]
    fn normalize_model_error_detects_context_overflow_from_prompt_too_long() {
        let result = normalize_model_error("prompt is too long: 277403 tokens > 200000 maximum");
        assert_eq!(
            serde_json::to_string(&result.kind).unwrap(),
            r#""context_overflow""#
        );
    }

    #[test]
    fn normalize_model_error_detects_context_overflow_from_max_tokens_context_limit() {
        let result = normalize_model_error(
            "input length and `max_tokens` exceed context limit: 188059 + 20000 > 200000",
        );
        assert_eq!(
            serde_json::to_string(&result.kind).unwrap(),
            r#""context_overflow""#
        );
    }

    #[test]
    fn normalize_model_error_detects_invalid_token_budget_without_claiming_context() {
        let result = normalize_model_error("max_tokens must be at least 1, got -1024");
        assert_eq!(
            serde_json::to_string(&result.kind).unwrap(),
            r#""invalid_token_budget""#
        );
    }

    #[test]
    fn normalize_model_error_detects_media_size_errors() {
        let result =
            normalize_model_error("image exceeds 5 MB maximum: 5316852 bytes > 5242880 bytes");
        assert_eq!(
            serde_json::to_string(&result.kind).unwrap(),
            r#""media_too_large""#
        );
    }

    #[test]
    fn normalize_model_error_keeps_tpm_413_as_rate_limit_not_context_overflow() {
        let result = normalize_model_error("413 tokens per minute limit exceeded");
        assert_eq!(result.kind, ModelErrorKind::RateLimit);
    }

    #[test]
    fn model_error_copy_includes_invalid_token_budget_action() {
        let result =
            build_failed_connection_test_result("max_tokens must be at least 1, got -1024");

        assert_eq!(result.title, "模型输出空间不足");
        assert_eq!(
            result.message,
            "模型请求没有剩余空间生成回复。请减少当前会话上下文、压缩图片，或使用更大上下文的模型后重试。"
        );
    }

    #[test]
    fn connection_test_failure_maps_billing_to_shared_copy() {
        let result =
            build_failed_connection_test_result("insufficient_balance: account balance too low");

        assert!(!result.ok);
        assert_eq!(result.kind, ModelErrorKind::Billing);
        assert_eq!(result.title, "模型余额不足");
        assert_eq!(
            result.message,
            "当前模型平台返回余额或额度不足，请到对应服务商控制台充值或检查套餐额度。"
        );
    }
}
