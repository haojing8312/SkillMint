use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvalReportStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EvalReportDecision {
    pub capability_id: String,
    #[serde(default)]
    pub selected_skill: Option<String>,
    #[serde(default)]
    pub selected_runner: Option<String>,
    #[serde(default)]
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EvalAssertionResults {
    pub route: String,
    pub execution: String,
    pub structured: String,
    pub output: String,
    pub thresholds: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvalReport {
    pub run_id: String,
    pub scenario_id: String,
    pub status: EvalReportStatus,
    pub decision: EvalReportDecision,
    pub assertions: EvalAssertionResults,
    pub final_output_excerpt: String,
}

impl EvalReport {
    pub fn passing(scenario_id: impl Into<String>, capability_id: impl Into<String>) -> Self {
        Self {
            run_id: String::new(),
            scenario_id: scenario_id.into(),
            status: EvalReportStatus::Pass,
            decision: EvalReportDecision {
                capability_id: capability_id.into(),
                ..EvalReportDecision::default()
            },
            assertions: EvalAssertionResults {
                route: "pass".to_string(),
                execution: "pass".to_string(),
                structured: "pass".to_string(),
                output: "pass".to_string(),
                thresholds: "pass".to_string(),
            },
            final_output_excerpt: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EvalReport, EvalReportStatus};

    #[test]
    fn passing_helper_sets_pass_defaults() {
        let report = EvalReport::passing("scenario-1", "pm_weekly_summary");

        assert_eq!(report.scenario_id, "scenario-1");
        assert_eq!(report.status, EvalReportStatus::Pass);
        assert_eq!(report.decision.capability_id, "pm_weekly_summary");
        assert_eq!(report.assertions.route, "pass");
        assert_eq!(report.assertions.thresholds, "pass");
    }

    #[test]
    fn report_serializes_stable_pass_status() {
        let report = EvalReport::passing("scenario-1", "pm_weekly_summary");
        let value = serde_json::to_value(&report).expect("serialize report");

        assert_eq!(value["status"], "pass");
        assert_eq!(value["scenario_id"], "scenario-1");
        assert_eq!(value["decision"]["capability_id"], "pm_weekly_summary");
        assert_eq!(report.status, EvalReportStatus::Pass);
    }
}
