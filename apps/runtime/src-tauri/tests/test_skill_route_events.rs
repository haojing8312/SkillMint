use runtime_lib::commands::chat::SkillRouteEvent;
use serde_json::Value;

#[test]
fn skill_route_event_payload_has_required_fields() {
    let evt = SkillRouteEvent {
        session_id: "s1".to_string(),
        route_run_id: "r1".to_string(),
        node_id: "n1".to_string(),
        parent_node_id: Some("root".to_string()),
        skill_name: "using-superpowers".to_string(),
        depth: 1,
        status: "routing".to_string(),
        duration_ms: None,
        error_code: None,
        error_message: None,
    };

    let v: Value = serde_json::to_value(evt).expect("event should serialize");
    assert_eq!(v["session_id"], "s1");
    assert_eq!(v["route_run_id"], "r1");
    assert_eq!(v["node_id"], "n1");
    assert_eq!(v["parent_node_id"], "root");
    assert_eq!(v["skill_name"], "using-superpowers");
    assert_eq!(v["depth"], 1);
    assert_eq!(v["status"], "routing");
}
