use super::approval_flow::{
    request_tool_approval_and_wait, resolve_approval_wait_runtime, wait_for_tool_confirmation,
    ApprovalWaitRuntime, ToolConfirmationDecision,
};
use super::browser_progress::BrowserProgressSnapshot;
use super::context::build_tool_context;
use super::event_bridge::{
    append_run_guard_warning_event, append_tool_run_event, build_skill_route_event,
    resolve_current_session_run_id,
};
#[cfg(test)]
use super::execution_caps::detect_execution_caps;
use super::executor::AgentExecutor;
use super::permissions::PermissionMode;
use super::progress::{json_progress_signature, text_progress_signature};
use super::run_guard::{
    encode_run_stop_reason, ProgressFingerprint, ProgressGuard, RunBudgetPolicy, RunBudgetScope,
    RunStopReason,
};
use super::safety::classify_policy_blocked_tool_error;
use super::types::{AgentStateEvent, LLMResponse, StreamDelta, ToolCallEvent, ToolResult};
use crate::adapters;
use crate::approval_bus::{approval_bus_rollout_enabled_with_pool, ApprovalDecision};
use crate::approval_rules::find_matching_approval_rule_with_pool;
use crate::session_journal::SessionRunEvent;
use anyhow::{anyhow, Result};
use runtime_executor_core::{
    estimate_tokens, extract_tool_call_parse_error, micro_compact, split_error_code_and_message,
    trim_messages, truncate_tool_output, update_tool_failure_streak, ToolFailureStreak,
    DEFAULT_TOKEN_BUDGET, MAX_TOOL_OUTPUT_CHARS,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

const TOOL_CONFIRM_TIMEOUT_SECS: u64 = 15;

impl AgentExecutor {
    pub(super) async fn execute_turn_impl(
        &self,
        api_format: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
        skill_system_prompt: &str,
        mut messages: Vec<Value>,
        on_token: impl Fn(StreamDelta) + Send + Clone,
        app_handle: Option<&AppHandle>,
        session_id: Option<&str>,
        allowed_tools: Option<&[String]>,
        permission_mode: PermissionMode,
        tool_confirm_tx: Option<
            std::sync::Arc<std::sync::Mutex<Option<std::sync::mpsc::Sender<bool>>>>,
        >,
        work_dir: Option<String>,
        max_iterations_override: Option<usize>,
        cancel_flag: Option<Arc<AtomicBool>>,
        route_node_timeout_secs: Option<u64>,
        route_retry_count: Option<usize>,
    ) -> Result<Vec<Value>> {
        // 组合系统级 prompt 和 Skill prompt
        let system_prompt = self.system_prompt_builder.build(skill_system_prompt);

        let tool_ctx = build_tool_context(session_id, work_dir.map(PathBuf::from), allowed_tools)?;
        let max_iterations = max_iterations_override.unwrap_or(self.max_iterations);
        let mut run_budget_policy = RunBudgetPolicy::for_scope(RunBudgetScope::GeneralChat);
        run_budget_policy.max_turns = max_iterations;
        let route_node_timeout_secs = route_node_timeout_secs.unwrap_or(60).clamp(5, 600);
        let route_retry_count = route_retry_count.unwrap_or(0).clamp(0, 2);
        let mut iteration = 0;
        let route_run_id = Uuid::new_v4().to_string();
        let persisted_run_id = if let (Some(app), Some(sid)) = (app_handle, session_id) {
            resolve_current_session_run_id(app, sid).await
        } else {
            None
        };
        let mut tool_failure_streak: Option<ToolFailureStreak> = None;
        let mut progress_history: Vec<ProgressFingerprint> = Vec::new();
        let mut latest_browser_progress: Option<BrowserProgressSnapshot> = None;

        loop {
            // 检查取消标志
            if let Some(ref flag) = cancel_flag {
                if flag.load(Ordering::SeqCst) {
                    eprintln!("[agent] 任务被用户取消");
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let _ = app.emit(
                            "agent-state-event",
                            AgentStateEvent::basic(
                                sid,
                                "finished",
                                Some("用户取消".to_string()),
                                iteration,
                            ),
                        );
                    }
                    messages.push(json!({
                        "role": "assistant",
                        "content": "任务已被取消。"
                    }));
                    return Ok(messages);
                }
            }

            if iteration >= max_iterations {
                let stop_reason = RunStopReason::max_turns(max_iterations);
                if let (Some(app), Some(sid)) = (app_handle, session_id) {
                    let _ = app.emit(
                        "agent-state-event",
                        AgentStateEvent::stopped(sid, iteration, &stop_reason),
                    );
                }
                return Err(anyhow!(encode_run_stop_reason(&stop_reason)));
            }
            iteration += 1;

            eprintln!("[agent] Iteration {}/{}", iteration, max_iterations);

            // 发射 thinking 状态事件
            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                let _ = app.emit(
                    "agent-state-event",
                    AgentStateEvent::basic(sid, "thinking", None, iteration),
                );
            }

            // 自动压缩检查（仅在第二轮及之后，避免首轮触发）
            if iteration > 1 {
                let tokens = estimate_tokens(&messages);
                if super::compactor::needs_auto_compact(tokens) {
                    eprintln!("[agent] Token 数 {} 超过阈值，触发自动压缩", tokens);
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let transcript_dir = app
                            .path()
                            .app_data_dir()
                            .unwrap_or_default()
                            .join("transcripts");
                        if let Ok(path) =
                            super::compactor::save_transcript(&transcript_dir, sid, &messages)
                        {
                            let path_str = path.to_string_lossy().to_string();
                            match super::compactor::auto_compact(
                                api_format, base_url, api_key, model, &messages, &path_str,
                            )
                            .await
                            {
                                Ok(compacted) => {
                                    eprintln!(
                                        "[agent] 自动压缩完成，消息数 {} → {}",
                                        messages.len(),
                                        compacted.len()
                                    );
                                    messages = compacted;
                                }
                                Err(e) => eprintln!("[agent] 自动压缩失败: {}", e),
                            }
                        }
                    }
                }
            }

            // 根据白名单过滤工具定义
            let tools = match allowed_tools {
                Some(whitelist) => self.registry.get_filtered_tool_definitions(whitelist),
                None => self.registry.get_tool_definitions(),
            };

            // 上下文压缩：Layer 1 微压缩 + token 预算裁剪
            let compacted = micro_compact(&messages, 3);
            let trimmed = trim_messages(&compacted, DEFAULT_TOKEN_BUDGET);

            // 调用 LLM（使用组合后的系统 prompt）
            let response_result = if api_format == "anthropic" {
                adapters::anthropic::chat_stream_with_tools(
                    base_url,
                    api_key,
                    model,
                    &system_prompt,
                    trimmed.clone(),
                    tools,
                    on_token.clone(),
                )
                .await
            } else {
                // OpenAI 兼容格式
                adapters::openai::chat_stream_with_tools(
                    base_url,
                    api_key,
                    model,
                    &system_prompt,
                    trimmed.clone(),
                    tools,
                    on_token.clone(),
                )
                .await
            };

            let response = match response_result {
                Ok(response) => response,
                Err(err) => {
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let _ = app.emit(
                            "agent-state-event",
                            AgentStateEvent::basic(sid, "error", Some(err.to_string()), iteration),
                        );
                    }
                    return Err(err);
                }
            };

            // 处理响应
            match response {
                LLMResponse::Text(content) => {
                    // 纯文本响应 - 结束循环
                    messages.push(json!({
                        "role": "assistant",
                        "content": content
                    }));
                    eprintln!("[agent] Finished with text response");

                    // 发射 finished 状态事件
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let _ = app.emit(
                            "agent-state-event",
                            AgentStateEvent::basic(sid, "finished", None, iteration),
                        );
                    }

                    return Ok(messages);
                }
                tc_response
                @ (LLMResponse::ToolCalls(_) | LLMResponse::TextWithToolCalls(_, _)) => {
                    let (companion_text, tool_calls) = match tc_response {
                        LLMResponse::ToolCalls(tc) => (String::new(), tc),
                        LLMResponse::TextWithToolCalls(text, tc) => (text, tc),
                        _ => unreachable!(),
                    };

                    eprintln!(
                        "[agent] Executing {} tool calls (companion_text={})",
                        tool_calls.len(),
                        !companion_text.is_empty()
                    );

                    // 发射 tool_calling 状态事件
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let tool_names: Vec<&str> =
                            tool_calls.iter().map(|tc| tc.name.as_str()).collect();
                        let _ = app.emit(
                            "agent-state-event",
                            AgentStateEvent::basic(
                                sid,
                                "tool_calling",
                                Some(tool_names.join(", ")),
                                iteration,
                            ),
                        );
                    }

                    // 执行所有工具调用
                    let mut tool_results = vec![];
                    let mut repeated_failure_summary: Option<String> = None;
                    for (call_index, call) in tool_calls.iter().enumerate() {
                        let skill_name = call
                            .input
                            .get("skill_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let is_skill_call = call.name == "skill";
                        let node_id = format!("{}-{}-{}", iteration, call_index, call.id);
                        let started_at = std::time::Instant::now();

                        if is_skill_call {
                            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                let _ = app.emit(
                                    "skill-route-node-updated",
                                    build_skill_route_event(
                                        sid,
                                        &route_run_id,
                                        &node_id,
                                        None,
                                        &skill_name,
                                        1,
                                        "routing",
                                        None,
                                        None,
                                        None,
                                    ),
                                );
                            }
                        }
                        // 执行每个工具前检查取消标志
                        if let Some(ref flag) = cancel_flag {
                            if flag.load(Ordering::SeqCst) {
                                eprintln!("[agent] 工具执行中被用户取消");
                                // 发射 finished 事件，确保前端清除状态指示器
                                if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                    let _ = app.emit(
                                        "agent-state-event",
                                        AgentStateEvent::basic(
                                            sid,
                                            "finished",
                                            Some("用户取消".to_string()),
                                            iteration,
                                        ),
                                    );
                                }
                                messages.push(json!({
                                    "role": "assistant",
                                    "content": "任务已被取消。"
                                }));
                                return Ok(messages);
                            }
                        }

                        eprintln!("[agent] Calling tool: {}", call.name);

                        if is_skill_call {
                            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                let _ = app.emit(
                                    "skill-route-node-updated",
                                    build_skill_route_event(
                                        sid,
                                        &route_run_id,
                                        &node_id,
                                        None,
                                        &skill_name,
                                        1,
                                        "executing",
                                        None,
                                        None,
                                        None,
                                    ),
                                );
                            }
                        }

                        // 发送工具开始事件
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit(
                                "tool-call-event",
                                ToolCallEvent {
                                    session_id: sid.to_string(),
                                    tool_name: call.name.clone(),
                                    tool_input: call.input.clone(),
                                    tool_output: None,
                                    status: "started".to_string(),
                                },
                            );
                            if let Some(run_id) = persisted_run_id.as_ref() {
                                let _ = append_tool_run_event(
                                    app,
                                    sid,
                                    SessionRunEvent::ToolStarted {
                                        run_id: run_id.clone(),
                                        tool_name: call.name.clone(),
                                        call_id: call.id.clone(),
                                        input: call.input.clone(),
                                    },
                                )
                                .await;
                            }
                        }

                        // 权限确认检查：在执行工具前判断是否需要用户确认
                        if permission_mode.needs_confirmation(
                            &call.name,
                            &call.input,
                            tool_ctx.work_dir.as_deref(),
                        ) {
                            let approval_decision = if let (Some(app), Some(sid)) =
                                (app_handle, session_id)
                            {
                                let runtime = match resolve_approval_wait_runtime(app) {
                                    Ok(runtime) => runtime,
                                    Err(err) => {
                                        let rejection_message = err.to_string();
                                        let _ = app.emit(
                                            "tool-call-event",
                                            ToolCallEvent {
                                                session_id: sid.to_string(),
                                                tool_name: call.name.clone(),
                                                tool_input: call.input.clone(),
                                                tool_output: Some(rejection_message.clone()),
                                                status: "error".to_string(),
                                            },
                                        );
                                        if let Some(run_id) = persisted_run_id.as_ref() {
                                            let _ = append_tool_run_event(
                                                app,
                                                sid,
                                                SessionRunEvent::ToolCompleted {
                                                    run_id: run_id.clone(),
                                                    tool_name: call.name.clone(),
                                                    call_id: call.id.clone(),
                                                    input: call.input.clone(),
                                                    output: rejection_message.clone(),
                                                    is_error: true,
                                                },
                                            )
                                            .await;
                                        }
                                        tool_results.push(ToolResult {
                                            tool_use_id: call.id.clone(),
                                            content: rejection_message,
                                        });
                                        continue;
                                    }
                                };
                                let approval_bus_enabled =
                                    approval_bus_rollout_enabled_with_pool(&runtime.pool)
                                        .await
                                        .unwrap_or(true);

                                if approval_bus_enabled {
                                    match find_matching_approval_rule_with_pool(
                                        &runtime.pool,
                                        &call.name,
                                        &call.input,
                                    )
                                    .await
                                    {
                                        Ok(Some(_)) => Some(ApprovalDecision::AllowAlways),
                                        Ok(None) | Err(_) => {
                                            match request_tool_approval_and_wait(
                                                &runtime,
                                                Some(app),
                                                sid,
                                                persisted_run_id.as_deref(),
                                                &call.name,
                                                &call.id,
                                                &call.input,
                                                tool_ctx.work_dir.as_deref(),
                                                cancel_flag.clone(),
                                            )
                                            .await
                                            {
                                                Ok(decision) => Some(decision),
                                                Err(err) => {
                                                    let rejection_message = err.to_string();
                                                    let _ = app.emit(
                                                        "tool-call-event",
                                                        ToolCallEvent {
                                                            session_id: sid.to_string(),
                                                            tool_name: call.name.clone(),
                                                            tool_input: call.input.clone(),
                                                            tool_output: Some(
                                                                rejection_message.clone(),
                                                            ),
                                                            status: "error".to_string(),
                                                        },
                                                    );
                                                    if let Some(run_id) = persisted_run_id.as_ref()
                                                    {
                                                        let _ = append_tool_run_event(
                                                            app,
                                                            sid,
                                                            SessionRunEvent::ToolCompleted {
                                                                run_id: run_id.clone(),
                                                                tool_name: call.name.clone(),
                                                                call_id: call.id.clone(),
                                                                input: call.input.clone(),
                                                                output: rejection_message.clone(),
                                                                is_error: true,
                                                            },
                                                        )
                                                        .await;
                                                    }
                                                    tool_results.push(ToolResult {
                                                        tool_use_id: call.id.clone(),
                                                        content: rejection_message,
                                                    });
                                                    None
                                                }
                                            }
                                        }
                                    }
                                } else if let Some(ref confirm_state) = tool_confirm_tx {
                                    let (tx, rx) = std::sync::mpsc::channel::<bool>();
                                    if let Ok(mut guard) = confirm_state.lock() {
                                        *guard = Some(tx);
                                    }

                                    let confirmation = wait_for_tool_confirmation(
                                        &rx,
                                        std::time::Duration::from_secs(TOOL_CONFIRM_TIMEOUT_SECS),
                                    );

                                    if let Ok(mut guard) = confirm_state.lock() {
                                        *guard = None;
                                    }

                                    match confirmation {
                                        ToolConfirmationDecision::Confirmed => {
                                            Some(ApprovalDecision::AllowOnce)
                                        }
                                        ToolConfirmationDecision::Rejected => {
                                            Some(ApprovalDecision::Deny)
                                        }
                                        ToolConfirmationDecision::TimedOut => {
                                            tool_results.push(ToolResult {
                                                tool_use_id: call.id.clone(),
                                                content: "工具确认超时，已取消此操作".to_string(),
                                            });
                                            None
                                        }
                                    }
                                } else {
                                    Some(ApprovalDecision::AllowOnce)
                                }
                            } else if let Some(ref confirm_state) = tool_confirm_tx {
                                let (tx, rx) = std::sync::mpsc::channel::<bool>();
                                if let Ok(mut guard) = confirm_state.lock() {
                                    *guard = Some(tx);
                                }

                                let confirmation = wait_for_tool_confirmation(
                                    &rx,
                                    std::time::Duration::from_secs(TOOL_CONFIRM_TIMEOUT_SECS),
                                );

                                if let Ok(mut guard) = confirm_state.lock() {
                                    *guard = None;
                                }

                                match confirmation {
                                    ToolConfirmationDecision::Confirmed => {
                                        Some(ApprovalDecision::AllowOnce)
                                    }
                                    ToolConfirmationDecision::Rejected => {
                                        Some(ApprovalDecision::Deny)
                                    }
                                    ToolConfirmationDecision::TimedOut => {
                                        tool_results.push(ToolResult {
                                            tool_use_id: call.id.clone(),
                                            content: "工具确认超时，已取消此操作".to_string(),
                                        });
                                        None
                                    }
                                }
                            } else {
                                Some(ApprovalDecision::AllowOnce)
                            };

                            let Some(approval_decision) = approval_decision else {
                                continue;
                            };

                            if approval_decision == ApprovalDecision::Deny {
                                let rejection_message = "用户拒绝了此操作";
                                if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                    let _ = app.emit(
                                        "tool-call-event",
                                        ToolCallEvent {
                                            session_id: sid.to_string(),
                                            tool_name: call.name.clone(),
                                            tool_input: call.input.clone(),
                                            tool_output: Some(rejection_message.to_string()),
                                            status: "error".to_string(),
                                        },
                                    );
                                    if let Some(run_id) = persisted_run_id.as_ref() {
                                        let _ = append_tool_run_event(
                                            app,
                                            sid,
                                            SessionRunEvent::ToolCompleted {
                                                run_id: run_id.clone(),
                                                tool_name: call.name.clone(),
                                                call_id: call.id.clone(),
                                                input: call.input.clone(),
                                                output: rejection_message.to_string(),
                                                is_error: true,
                                            },
                                        )
                                        .await;
                                    }
                                }
                                tool_results.push(ToolResult {
                                    tool_use_id: call.id.clone(),
                                    content: rejection_message.to_string(),
                                });
                                continue;
                            }
                        }

                        let max_attempts = if is_skill_call {
                            route_retry_count + 1
                        } else {
                            1
                        };
                        let mut attempt = 0usize;
                        let (result, is_error) = loop {
                            attempt += 1;
                            let (result, is_error) = if let Some(parse_error) =
                                extract_tool_call_parse_error(&call.input)
                            {
                                (
                                    format!(
                                        "工具参数错误: {}。请提供完整且合法的 JSON 参数后再重试。",
                                        parse_error
                                    ),
                                    true,
                                )
                            } else {
                                match self.registry.get(&call.name) {
                                    Some(tool) => {
                                        // 检查白名单：若设置了白名单但工具不在其中，拒绝执行
                                        if let Some(whitelist) = allowed_tools {
                                            if !whitelist.iter().any(|w| w == &call.name) {
                                                (
                                                    format!(
                                                        "此 Skill 不允许使用工具: {}",
                                                        call.name
                                                    ),
                                                    true,
                                                )
                                            } else {
                                                let tool_clone = Arc::clone(&tool);
                                                let input_clone = call.input.clone();
                                                let ctx_clone = tool_ctx.clone();
                                                let handle =
                                                    tokio::task::spawn_blocking(move || {
                                                        tool_clone.execute(input_clone, &ctx_clone)
                                                    });
                                                let cancel_flag_ref = cancel_flag.clone();
                                                let exec_future = async move {
                                                    tokio::select! {
                                                        res = handle => {
                                                            match res {
                                                                Ok(Ok(output)) => (output, false),
                                                                Ok(Err(e)) => (format!("工具执行错误: {}", e), true),
                                                                Err(e) => (format!("工具执行线程异常: {}", e), true),
                                                            }
                                                        }
                                                        _ = Self::wait_for_cancel(&cancel_flag_ref) => {
                                                            ("工具执行被用户取消".to_string(), true)
                                                        }
                                                    }
                                                };
                                                if is_skill_call {
                                                    match tokio::time::timeout(
                                                        std::time::Duration::from_secs(
                                                            route_node_timeout_secs,
                                                        ),
                                                        exec_future,
                                                    )
                                                    .await
                                                    {
                                                        Ok(v) => v,
                                                        Err(_) => (
                                                            "TIMEOUT: 子 Skill 执行超时"
                                                                .to_string(),
                                                            true,
                                                        ),
                                                    }
                                                } else {
                                                    exec_future.await
                                                }
                                            }
                                        } else {
                                            let tool_clone = Arc::clone(&tool);
                                            let input_clone = call.input.clone();
                                            let ctx_clone = tool_ctx.clone();
                                            let handle = tokio::task::spawn_blocking(move || {
                                                tool_clone.execute(input_clone, &ctx_clone)
                                            });
                                            let cancel_flag_ref = cancel_flag.clone();
                                            let exec_future = async move {
                                                tokio::select! {
                                                    res = handle => {
                                                        match res {
                                                            Ok(Ok(output)) => (output, false),
                                                            Ok(Err(e)) => (format!("工具执行错误: {}", e), true),
                                                            Err(e) => (format!("工具执行线程异常: {}", e), true),
                                                        }
                                                    }
                                                    _ = Self::wait_for_cancel(&cancel_flag_ref) => {
                                                        ("工具执行被用户取消".to_string(), true)
                                                    }
                                                }
                                            };
                                            if is_skill_call {
                                                match tokio::time::timeout(
                                                    std::time::Duration::from_secs(
                                                        route_node_timeout_secs,
                                                    ),
                                                    exec_future,
                                                )
                                                .await
                                                {
                                                    Ok(v) => v,
                                                    Err(_) => (
                                                        "TIMEOUT: 子 Skill 执行超时".to_string(),
                                                        true,
                                                    ),
                                                }
                                            } else {
                                                exec_future.await
                                            }
                                        }
                                    }
                                    None => {
                                        // 列出可用工具，引导 LLM 使用正确的工具
                                        let available: Vec<String> = self
                                            .registry
                                            .get_tool_definitions()
                                            .iter()
                                            .filter_map(|t| t["name"].as_str().map(String::from))
                                            .collect();
                                        (
                                        format!(
                                            "错误: 工具 '{}' 不存在。请勿再次调用此工具。可用工具: {}",
                                            call.name,
                                            available.join(", ")
                                        ),
                                        true,
                                    )
                                    }
                                }
                            };
                            if !is_error || attempt >= max_attempts {
                                break (result, is_error);
                            }
                        };
                        // 截断过长的工具输出，防止超出上下文窗口
                        let result = truncate_tool_output(&result, MAX_TOOL_OUTPUT_CHARS);

                        if is_error {
                            if let Some(summary) = update_tool_failure_streak(
                                &mut tool_failure_streak,
                                &call.name,
                                &call.input,
                                &result,
                            ) {
                                repeated_failure_summary = Some(summary);
                            }
                        } else {
                            tool_failure_streak = None;
                        }

                        // 发送工具完成事件
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit(
                                "tool-call-event",
                                ToolCallEvent {
                                    session_id: sid.to_string(),
                                    tool_name: call.name.clone(),
                                    tool_input: call.input.clone(),
                                    tool_output: Some(result.clone()),
                                    status: if is_error {
                                        "error".to_string()
                                    } else {
                                        "completed".to_string()
                                    },
                                },
                            );
                            if let Some(run_id) = persisted_run_id.as_ref() {
                                let _ = append_tool_run_event(
                                    app,
                                    sid,
                                    SessionRunEvent::ToolCompleted {
                                        run_id: run_id.clone(),
                                        tool_name: call.name.clone(),
                                        call_id: call.id.clone(),
                                        input: call.input.clone(),
                                        output: result.clone(),
                                        is_error,
                                    },
                                )
                                .await;
                            }
                        }

                        if is_skill_call {
                            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                let duration_ms = started_at.elapsed().as_millis() as u64;
                                let parsed_error = if is_error {
                                    Some(split_error_code_and_message(&result))
                                } else {
                                    None
                                };
                                let _ = app.emit(
                                    "skill-route-node-updated",
                                    build_skill_route_event(
                                        sid,
                                        &route_run_id,
                                        &node_id,
                                        None,
                                        &skill_name,
                                        1,
                                        if is_error { "failed" } else { "completed" },
                                        Some(duration_ms),
                                        parsed_error.as_ref().map(|(code, _)| code.as_str()),
                                        parsed_error.as_ref().map(|(_, msg)| msg.as_str()),
                                    ),
                                );
                            }
                        }

                        if is_error {
                            if let Some(mut stop_reason) =
                                classify_policy_blocked_tool_error(&call.name, &result)
                            {
                                if let Some(last_completed_step) = latest_browser_progress
                                    .as_ref()
                                    .and_then(BrowserProgressSnapshot::last_completed_step)
                                {
                                    stop_reason =
                                        stop_reason.with_last_completed_step(last_completed_step);
                                }
                                if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                    let _ = app.emit(
                                        "agent-state-event",
                                        AgentStateEvent::stopped(sid, iteration, &stop_reason),
                                    );
                                }
                                return Err(anyhow!(encode_run_stop_reason(&stop_reason)));
                            }
                        }

                        let input_signature = json_progress_signature(&call.input);
                        let browser_progress_snapshot = if is_error {
                            None
                        } else {
                            BrowserProgressSnapshot::from_tool_output(&call.name, &result)
                        };
                        let output_signature =
                            if let Some(snapshot) = browser_progress_snapshot.as_ref() {
                                snapshot.progress_signature()
                            } else {
                                let progress_text = if is_error {
                                    format!("error:{result}")
                                } else {
                                    result.clone()
                                };
                                text_progress_signature(&progress_text)
                            };
                        if let Some(snapshot) = browser_progress_snapshot {
                            latest_browser_progress = Some(snapshot);
                        }
                        progress_history.push(ProgressFingerprint::tool_result(
                            call.name.clone(),
                            input_signature,
                            output_signature,
                        ));

                        tool_results.push(ToolResult {
                            tool_use_id: call.id.clone(),
                            content: result,
                        });

                        if repeated_failure_summary.is_some() {
                            break;
                        }
                    }

                    // 添加工具调用和结果到消息历史（包含伴随文本）
                    if api_format == "anthropic" {
                        // Anthropic 格式: assistant 消息包含 text block + tool_use blocks
                        let mut content_blocks: Vec<Value> = vec![];
                        if !companion_text.is_empty() {
                            content_blocks.push(json!({"type": "text", "text": companion_text}));
                        }
                        for tc in &tool_calls {
                            content_blocks.push(json!({
                                "type": "tool_use",
                                "id": tc.id,
                                "name": tc.name,
                                "input": tc.input,
                            }));
                        }
                        messages.push(json!({
                            "role": "assistant",
                            "content": content_blocks
                        }));

                        // user 消息包含 tool_result blocks
                        messages.push(json!({
                            "role": "user",
                            "content": tool_results.iter().map(|tr| json!({
                                "type": "tool_result",
                                "tool_use_id": tr.tool_use_id,
                                "content": tr.content,
                            })).collect::<Vec<_>>()
                        }));
                    } else {
                        // OpenAI 格式: companion_text 放 content 字段
                        let content_val = if companion_text.is_empty() {
                            Value::Null
                        } else {
                            Value::String(companion_text.clone())
                        };
                        messages.push(json!({
                            "role": "assistant",
                            "content": content_val,
                            "tool_calls": tool_calls.iter().map(|tc| json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": serde_json::to_string(&tc.input).unwrap_or_default(),
                                }
                            })).collect::<Vec<_>>()
                        }));
                        // OpenAI: 每个工具结果是独立的 "tool" 角色消息
                        for tr in &tool_results {
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": tr.tool_use_id,
                                "content": tr.content,
                            }));
                        }
                    }

                    if let Some(summary) = repeated_failure_summary {
                        messages.push(json!({
                            "role": "assistant",
                            "content": summary
                        }));
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit(
                                "agent-state-event",
                                AgentStateEvent::basic(
                                    sid,
                                    "finished",
                                    Some("重复工具失败已熔断".to_string()),
                                    iteration,
                                ),
                            );
                        }
                        return Ok(messages);
                    }

                    let progress_evaluation =
                        ProgressGuard::evaluate(&run_budget_policy, &progress_history);
                    if let Some(mut warning) = progress_evaluation.warning {
                        if let Some(last_completed_step) = latest_browser_progress
                            .as_ref()
                            .and_then(BrowserProgressSnapshot::last_completed_step)
                        {
                            warning = warning.with_last_completed_step(last_completed_step);
                        }
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = append_run_guard_warning_event(app, sid, &warning).await;
                        }
                    }
                    if let Some(mut stop_reason) = progress_evaluation.stop_reason {
                        if let Some(last_completed_step) = latest_browser_progress
                            .as_ref()
                            .and_then(BrowserProgressSnapshot::last_completed_step)
                        {
                            stop_reason = stop_reason.with_last_completed_step(last_completed_step);
                        }
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit(
                                "agent-state-event",
                                AgentStateEvent::stopped(sid, iteration, &stop_reason),
                            );
                        }
                        return Err(anyhow!(encode_run_stop_reason(&stop_reason)));
                    }

                    // 继续下一轮迭代
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::classify_policy_blocked_tool_error;
    use super::request_tool_approval_and_wait;
    use super::wait_for_tool_confirmation;
    use super::ApprovalWaitRuntime;
    use super::ToolConfirmationDecision;
    use crate::agent::run_guard::RunStopReasonKind;
    use crate::agent::{FileDeleteTool, Tool, ToolContext};
    use crate::approval_bus::{ApprovalDecision, ApprovalManager};
    use crate::session_journal::SessionJournalStore;
    use serde_json::json;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn tool_confirmation_timeout_is_treated_as_rejection() {
        let (_tx, rx) = mpsc::channel::<bool>();
        let decision = wait_for_tool_confirmation(&rx, Duration::from_millis(5));
        assert_eq!(decision, ToolConfirmationDecision::TimedOut);
    }

    #[test]
    fn tool_confirmation_false_is_rejected() {
        let (tx, rx) = mpsc::channel::<bool>();
        tx.send(false).expect("send");
        let decision = wait_for_tool_confirmation(&rx, Duration::from_millis(5));
        assert_eq!(decision, ToolConfirmationDecision::Rejected);
    }

    #[test]
    fn workspace_boundary_error_maps_to_policy_blocked() {
        let reason = classify_policy_blocked_tool_error(
            "list_dir",
            "工具执行错误: 路径 C:\\Users\\Administrator\\Desktop 不在工作目录 C:\\Users\\Administrator\\WorkClaw\\workspace 范围内",
        )
        .expect("should classify");

        assert_eq!(reason.kind, RunStopReasonKind::PolicyBlocked);
        assert!(reason
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("切换当前会话的工作目录"));
    }

    #[test]
    fn skill_allowlist_error_is_not_policy_blocked() {
        let reason = classify_policy_blocked_tool_error("bash", "此 Skill 不允许使用工具: bash");

        assert!(reason.is_none());
    }

    #[test]
    fn ordinary_tool_failure_is_not_policy_blocked() {
        let reason = classify_policy_blocked_tool_error(
            "read_file",
            "工具执行错误: 文件不存在: missing.txt",
        );

        assert!(reason.is_none());
    }

    #[test]
    fn tool_context_construction_includes_p0_metadata_slots() {
        let work_dir = Some(PathBuf::from("workspace"));
        let allowed_tools = Some(vec!["read_file".to_string(), "skill".to_string()]);

        let ctx = super::build_tool_context(
            Some("session-123"),
            work_dir.clone(),
            allowed_tools.as_deref(),
        )
        .expect("build tool context");

        assert_eq!(ctx.session_id.as_deref(), Some("session-123"));
        assert_eq!(ctx.work_dir, work_dir);
        assert_eq!(ctx.allowed_tools, allowed_tools);
        let temp_dir = ctx.task_temp_dir.expect("task temp dir");
        let temp_dir_name = temp_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("temp dir name");
        assert_eq!(temp_dir_name, "workclaw-task-session-123");
        assert!(temp_dir.exists());

        let caps = ctx.execution_caps.expect("execution caps");
        assert_eq!(caps.platform.as_deref(), Some(std::env::consts::OS));
        assert_eq!(
            caps.preferred_shell.as_deref(),
            Some(if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "bash"
            })
        );
        assert!(caps.python_candidates.is_empty());
        assert!(caps.node_candidates.is_empty());
        assert_eq!(caps.notes, vec!["static P0 detection".to_string()]);
        assert!(ctx.file_task_caps.is_none());
    }

    #[test]
    fn tool_context_reuses_task_temp_dir_for_same_session() {
        let first = super::build_tool_context(Some("session-123"), None, None)
            .expect("first tool context")
            .task_temp_dir
            .expect("first temp dir");
        let second = super::build_tool_context(Some("session-123"), None, None)
            .expect("second tool context")
            .task_temp_dir
            .expect("second temp dir");

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn approval_bus_blocks_file_delete_until_resolved() {
        let db_dir = tempdir().expect("create db dir");
        let db_url = format!(
            "sqlite://{}?mode=rwc",
            db_dir.path().join("approval-test.db").to_string_lossy()
        );
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect(&db_url)
            .await
            .expect("connect sqlite");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS session_runs (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                user_message_id TEXT NOT NULL DEFAULT '',
                assistant_message_id TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'queued',
                buffered_text TEXT NOT NULL DEFAULT '',
                error_kind TEXT NOT NULL DEFAULT '',
                error_message TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create session_runs");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS session_run_events (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create session_run_events");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS approvals (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                call_id TEXT NOT NULL DEFAULT '',
                tool_name TEXT NOT NULL,
                input_json TEXT NOT NULL DEFAULT '{}',
                summary TEXT NOT NULL DEFAULT '',
                impact TEXT NOT NULL DEFAULT '',
                irreversible INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                decision TEXT NOT NULL DEFAULT '',
                notify_targets_json TEXT NOT NULL DEFAULT '[]',
                resume_payload_json TEXT NOT NULL DEFAULT '{}',
                resolved_by_surface TEXT NOT NULL DEFAULT '',
                resolved_by_user TEXT NOT NULL DEFAULT '',
                resolved_at TEXT,
                resumed_at TEXT,
                expires_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create approvals");

        let journal_dir = tempdir().expect("create journal dir");
        let runtime = ApprovalWaitRuntime {
            pool: pool.clone(),
            journal: Arc::new(SessionJournalStore::new(journal_dir.path().to_path_buf())),
            approval_manager: Arc::new(ApprovalManager::default()),
            pending_bridge: Arc::new(Mutex::new(None)),
        };

        let work_dir = tempdir().expect("create work dir");
        let target_dir = work_dir.path().join("danger");
        std::fs::create_dir_all(target_dir.join("nested")).expect("create target tree");
        std::fs::write(target_dir.join("nested").join("file.txt"), "danger")
            .expect("write nested file");

        let input = json!({
            "path": target_dir.to_string_lossy().to_string(),
            "recursive": true,
        });
        let tool_ctx = ToolContext {
            work_dir: Some(PathBuf::from(work_dir.path())),
            allowed_tools: None,
            session_id: Some("sess-approval".to_string()),
            task_temp_dir: Some(PathBuf::from(std::env::temp_dir())),
            execution_caps: Some(super::detect_execution_caps()),
            file_task_caps: None,
        };

        let runtime_clone = runtime.clone();
        let manager = runtime.approval_manager.clone();
        let pool_clone = pool.clone();
        let input_clone = input.clone();
        let tool_ctx_clone = tool_ctx.clone();
        let work_dir_path = work_dir.path().to_path_buf();

        let handle = tokio::spawn(async move {
            let decision = request_tool_approval_and_wait(
                &runtime_clone,
                None,
                "sess-approval",
                Some("run-approval"),
                "file_delete",
                "call-approval",
                &input_clone,
                Some(work_dir_path.as_path()),
                None,
            )
            .await
            .expect("approval should resolve");
            assert_eq!(decision, ApprovalDecision::AllowOnce);

            let tool = FileDeleteTool;
            tool.execute(input_clone, &tool_ctx_clone)
        });

        let mut pending_row: Option<(String, String)> = None;
        for _ in 0..20 {
            if let Some(row) = sqlx::query_as::<_, (String, String)>(
                "SELECT id, status FROM approvals WHERE session_id = ?",
            )
            .bind("sess-approval")
            .fetch_optional(&pool)
            .await
            .expect("query pending approval")
            {
                pending_row = Some(row);
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        assert!(target_dir.exists(), "directory must remain before approval");

        let (approval_id, status) = pending_row.expect("load pending approval");
        assert_eq!(status, "pending");

        manager
            .resolve_with_pool(
                &pool_clone,
                &approval_id,
                ApprovalDecision::AllowOnce,
                "desktop",
                "tester",
            )
            .await
            .expect("resolve pending approval");

        let result = handle
            .await
            .expect("join task")
            .expect("file delete success");
        assert!(result.contains("成功删除"));
        assert!(
            !target_dir.exists(),
            "directory should be removed after approval"
        );
    }
}
