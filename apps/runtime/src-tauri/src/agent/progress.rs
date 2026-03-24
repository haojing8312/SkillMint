use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) fn text_progress_signature(raw: &str) -> String {
    let mut hasher = DefaultHasher::new();
    raw.trim().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub(super) fn json_progress_signature(value: &Value) -> String {
    let serialized = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
    text_progress_signature(&serialized)
}

#[cfg(test)]
mod tests {
    use super::{json_progress_signature, text_progress_signature};
    use serde_json::json;

    #[test]
    fn text_signature_is_stable_for_trimmed_input() {
        assert_eq!(
            text_progress_signature("  hello  "),
            text_progress_signature("hello")
        );
    }

    #[test]
    fn json_signature_is_stable_for_json_value() {
        let value = json!({"a": 1, "b": ["x", "y"]});
        assert_eq!(
            json_progress_signature(&value),
            json_progress_signature(&value)
        );
    }
}
