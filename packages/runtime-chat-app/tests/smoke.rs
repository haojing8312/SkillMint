use runtime_chat_app::{ChatPreparationService, PreparedChatExecution};

#[test]
fn crate_exports_service_and_prepared_execution() {
    let _service = ChatPreparationService::new();
    let prepared = PreparedChatExecution::default();

    assert_eq!(prepared.capability, "chat");
}
