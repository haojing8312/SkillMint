use crate::agent::browser_progress::BrowserProgressSnapshot;
use crate::agent::progress::json_progress_signature;
use crate::agent::run_guard::{
    ProgressEvaluation, ProgressFingerprint, ProgressGuard, RunBudgetPolicy,
};
use serde_json::Value;

pub(crate) fn build_tool_call_fingerprint(
    tool_name: &str,
    tool_input: &Value,
) -> ProgressFingerprint {
    ProgressFingerprint::tool(tool_name.to_string(), json_progress_signature(tool_input))
}

pub(crate) fn evaluate_before_tool_call(
    policy: &RunBudgetPolicy,
    history: &[ProgressFingerprint],
    latest_browser_progress: Option<&BrowserProgressSnapshot>,
    tool_name: &str,
    tool_input: &Value,
) -> (ProgressFingerprint, ProgressEvaluation) {
    let fingerprint = build_tool_call_fingerprint(tool_name, tool_input);
    let mut prospective_history = history.to_vec();
    prospective_history.push(fingerprint.clone());
    let evaluation = ProgressGuard::evaluate(policy, &prospective_history)
        .with_last_completed_step(
            latest_browser_progress.and_then(BrowserProgressSnapshot::last_completed_step),
        );
    (fingerprint, evaluation)
}

#[cfg(test)]
mod tests {
    use super::{build_tool_call_fingerprint, evaluate_before_tool_call};
    use crate::agent::browser_progress::{BrowserProgressSnapshot, BrowserStageHints};
    use crate::agent::run_guard::{RunBudgetPolicy, RunBudgetScope, RunStopReasonKind};
    use serde_json::json;

    #[test]
    fn before_tool_call_warns_on_fifth_repeated_call() {
        let policy = RunBudgetPolicy::for_scope(RunBudgetScope::GeneralChat);
        let tool_input = json!({ "selector": "#publish" });
        let history = vec![
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
        ];

        let (_, evaluation) =
            evaluate_before_tool_call(&policy, &history, None, "browser_click", &tool_input);

        assert!(evaluation.stop_reason.is_none());
        assert_eq!(
            evaluation.warning.expect("warning").kind,
            RunStopReasonKind::LoopDetected
        );
    }

    #[test]
    fn before_tool_call_stops_on_sixth_repeated_call() {
        let policy = RunBudgetPolicy::for_scope(RunBudgetScope::GeneralChat);
        let tool_input = json!({ "selector": "#publish" });
        let history = vec![
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
        ];

        let (_, evaluation) =
            evaluate_before_tool_call(&policy, &history, None, "browser_click", &tool_input);

        assert!(evaluation.warning.is_none());
        assert_eq!(
            evaluation.stop_reason.expect("stop reason").kind,
            RunStopReasonKind::LoopDetected
        );
    }

    #[test]
    fn before_tool_call_attaches_last_completed_step_when_available() {
        let policy = RunBudgetPolicy::for_scope(RunBudgetScope::GeneralChat);
        let tool_input = json!({ "selector": "#publish" });
        let history = vec![
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
            build_tool_call_fingerprint("browser_click", &tool_input),
        ];
        let latest_browser_progress = BrowserProgressSnapshot {
            url: "https://example.com".to_string(),
            title: "draft".to_string(),
            page_signature: "page-1".to_string(),
            facts_signature: "facts-1".to_string(),
            stage_hints: BrowserStageHints {
                cover_filled: true,
                title_filled: false,
                body_segment_count: 1,
            },
        };

        let (_, evaluation) = evaluate_before_tool_call(
            &policy,
            &history,
            Some(&latest_browser_progress),
            "browser_click",
            &tool_input,
        );

        assert_eq!(
            evaluation
                .warning
                .expect("warning")
                .last_completed_step
                .as_deref(),
            Some("已填写正文")
        );
    }
}
