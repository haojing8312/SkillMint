use runtime_chat_app::infer_capability_from_user_message;

#[test]
fn infers_capability_from_user_message_keywords() {
    assert_eq!(
        infer_capability_from_user_message("帮我看图识别内容"),
        "vision"
    );
    assert_eq!(
        infer_capability_from_user_message("请生图，生成图片"),
        "image_gen"
    );
    assert_eq!(
        infer_capability_from_user_message("做个语音转文字"),
        "audio_stt"
    );
    assert_eq!(
        infer_capability_from_user_message("帮我做文字转语音"),
        "audio_tts"
    );
    assert_eq!(infer_capability_from_user_message("普通聊天"), "chat");
}
