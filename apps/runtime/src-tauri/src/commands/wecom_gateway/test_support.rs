use std::sync::{Arc, Mutex};

pub use crate::commands::im_host::channel_runtime_state::ImChannelHostRuntimeState as WecomHostRuntimeStateForTests;

pub fn new_wecom_host_runtime_state_for_tests() -> WecomHostRuntimeStateForTests {
    WecomHostRuntimeStateForTests::default()
}

pub fn wecom_runtime_status_for_tests(
    state: &WecomHostRuntimeStateForTests,
) -> Result<Option<serde_json::Value>, String> {
    super::get_im_channel_runtime_status_in_state(state, "wecom")
}

pub fn install_recording_wecom_send_hook() -> Arc<Mutex<Vec<String>>> {
    let sent_texts = Arc::new(Mutex::new(Vec::<String>::new()));
    let sent_texts_for_hook = sent_texts.clone();
    if let Ok(mut guard) = super::wecom_outbound_send_hook_slot().lock() {
        *guard = Some(Arc::new(move |_thread_id, text| {
            sent_texts_for_hook
                .lock()
                .expect("lock wecom sent texts")
                .push(text.to_string());
            Ok(serde_json::json!({
                "message_id": "wm_test_1",
                "conversation_id": "wecom_test_conversation",
            }))
        }));
    }
    sent_texts
}

pub fn install_recording_wecom_lifecycle_hook() -> Arc<Mutex<Vec<String>>> {
    let recorded = Arc::new(Mutex::new(Vec::<String>::new()));
    let recorded_for_hook = recorded.clone();
    if let Ok(mut guard) = super::wecom_lifecycle_event_hook_slot().lock() {
        *guard = Some(Arc::new(move |request| {
            recorded_for_hook
                .lock()
                .expect("lock wecom lifecycle records")
                .push(format!(
                    "lifecycle:{}:{}",
                    request.message_id.as_deref().unwrap_or(""),
                    serde_json::to_string(&request.phase)
                        .unwrap_or_else(|_| "\"unknown\"".to_string())
                ));
            Ok(())
        }));
    }
    recorded
}

pub fn install_recording_wecom_interactive_lifecycle_hooks() -> Arc<Mutex<Vec<String>>> {
    let recorded = Arc::new(Mutex::new(Vec::<String>::new()));

    let processing_events = recorded.clone();
    if let Ok(mut guard) = super::wecom_processing_stop_hook_slot().lock() {
        *guard = Some(Arc::new(move |request| {
            processing_events
                .lock()
                .expect("lock wecom lifecycle records")
                .push(format!(
                    "processing_stop:{}:{}",
                    request.message_id,
                    request.final_state.as_deref().unwrap_or("")
                ));
            Ok(())
        }));
    }

    let lifecycle_events = recorded.clone();
    if let Ok(mut guard) = super::wecom_lifecycle_event_hook_slot().lock() {
        *guard = Some(Arc::new(move |request| {
            lifecycle_events
                .lock()
                .expect("lock wecom lifecycle records")
                .push(format!(
                    "lifecycle:{}:{}",
                    request.message_id.as_deref().unwrap_or(""),
                    serde_json::to_string(&request.phase)
                        .unwrap_or_else(|_| "\"unknown\"".to_string())
                ));
            Ok(())
        }));
    }

    recorded
}

pub fn clear_wecom_test_hooks() {
    if let Ok(mut guard) = super::wecom_outbound_send_hook_slot().lock() {
        *guard = None;
    }
    if let Ok(mut guard) = super::wecom_processing_stop_hook_slot().lock() {
        *guard = None;
    }
    if let Ok(mut guard) = super::wecom_lifecycle_event_hook_slot().lock() {
        *guard = None;
    }
}
