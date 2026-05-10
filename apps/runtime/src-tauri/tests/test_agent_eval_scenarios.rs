#![cfg(feature = "headless-evals")]

use runtime_lib::agent::evals::{EvalScenario, LocalEvalConfig};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

#[test]
fn tracked_agent_eval_scenarios_parse_and_match_file_names() {
    let scenarios_dir = repo_root().join("agent-evals").join("scenarios");
    let mut count = 0usize;
    for entry in fs::read_dir(&scenarios_dir).expect("read scenarios dir") {
        let path = entry.expect("scenario entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path).expect("read scenario yaml");
        let scenario: EvalScenario = serde_yaml::from_str(&raw)
            .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
        assert_eq!(
            path.file_stem().and_then(|value| value.to_str()),
            Some(scenario.id.as_str())
        );
        assert!(!scenario.capability_id.trim().is_empty());
        count += 1;
    }
    assert!(count >= 2);
}

#[test]
fn skill_curator_lifecycle_parity_scenario_is_wired_to_example_config() {
    let root = repo_root();
    let scenario_raw = fs::read_to_string(
        root.join("agent-evals")
            .join("scenarios")
            .join("skill_curator_lifecycle_parity_2026_05_09.yaml"),
    )
    .expect("read skill curator lifecycle scenario");
    let scenario: EvalScenario =
        serde_yaml::from_str(&scenario_raw).expect("parse skill curator lifecycle scenario");

    let config_raw = fs::read_to_string(
        root.join("agent-evals")
            .join("config")
            .join("config.example.yaml"),
    )
    .expect("read config example");
    let config: LocalEvalConfig = serde_yaml::from_str(&config_raw).expect("parse config example");

    assert_eq!(scenario.capability_id, "skill_curator_lifecycle_parity");
    assert_eq!(scenario.input.user_turns.len(), 2);
    assert_eq!(
        scenario.input.profile_id.as_deref(),
        Some("eval-profile-skill-curator")
    );
    assert!(scenario
        .expect
        .tools
        .called_all
        .contains(&"skills".to_string()));
    assert!(scenario
        .expect
        .tools
        .called_all
        .contains(&"curator".to_string()));
    assert!(config.capabilities.contains_key(&scenario.capability_id));
}
