use runtime_chat_app::capabilities::{
    capability_definition, recommended_tools_for_capability, CapabilityInputKind,
    CapabilityRouteKind,
};

#[test]
fn vision_capability_declares_workspace_resource_and_tool_route() {
    let vision = capability_definition("vision").expect("vision capability");

    assert!(vision
        .input_kinds
        .contains(&CapabilityInputKind::WorkspaceResource));
    assert!(vision
        .preferred_routes
        .contains(&CapabilityRouteKind::RuntimeTool));
    assert_eq!(
        recommended_tools_for_capability("vision"),
        &["vision_analyze"]
    );
}

#[test]
fn unknown_capability_has_no_recommended_tools() {
    assert_eq!(recommended_tools_for_capability("missing"), &[] as &[&str]);
}
