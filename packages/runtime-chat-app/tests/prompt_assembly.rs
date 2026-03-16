use runtime_chat_app::{compose_system_prompt, ChatExecutionGuidance};

#[test]
fn compose_system_prompt_includes_execution_guidance_and_optional_sections() {
    let prompt = compose_system_prompt(
        "Base skill prompt",
        "bash, read, write, browser",
        "gpt-4.1",
        8,
        &ChatExecutionGuidance {
            effective_work_dir: "E:/workspace/demo".to_string(),
        },
        Some(
            "<available_skills>\n<skill><name>xhs</name><invoke_name>xhs</invoke_name><location>E:/workspace/demo/skills/xhs/SKILL.md</location></skill>\n</available_skills>",
        ),
        Some("Collaborate with employee-1 when domain knowledge is required."),
        Some("Remember previous delivery constraints."),
    );

    assert!(prompt.contains("Base skill prompt"));
    assert!(prompt.contains("工作目录: E:/workspace/demo"));
    assert!(prompt.contains("可用工具: bash, read, write, browser"));
    assert!(prompt.contains("模型: gpt-4.1"));
    assert!(prompt.contains("最大迭代次数: 8"));
    assert!(prompt.contains("Skills (mandatory):"));
    assert!(prompt.contains("<available_skills>"));
    assert!(prompt.contains("use its <invoke_name> or <location> as skill_name"));
    assert!(prompt.contains("E:/workspace/demo/skills/xhs/SKILL.md"));
    assert!(prompt.contains("WorkClaw 内置本地 browser sidecar"));
    assert!(prompt.contains("http://localhost:8765"));
    assert!(prompt.contains("不要要求用户手动启动 OpenClaw 浏览器服务"));
    assert!(prompt.contains("不要检查 openclaw-desktop.exe"));
    assert!(prompt.contains("不要要求固定安装目录"));
    assert!(prompt.contains("Collaborate with employee-1"));
    assert!(prompt.contains("持久内存:\nRemember previous delivery constraints."));
}
