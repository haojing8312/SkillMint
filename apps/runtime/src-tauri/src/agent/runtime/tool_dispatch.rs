use crate::agent::browser_progress::BrowserProgressSnapshot;
use crate::agent::event_bridge::{append_tool_run_event, build_skill_route_event};
use crate::agent::permissions::PermissionMode;
use crate::agent::progress::{json_progress_signature, text_progress_signature};
use crate::agent::registry::ToolRegistry;
use crate::agent::run_guard::{encode_run_stop_reason, ProgressFingerprint};
use crate::agent::runtime::approval_gate::gate_tool_approval;
use crate::agent::safety::classify_policy_blocked_tool_error;
use crate::agent::types::{AgentStateEvent, Tool, ToolCall, ToolCallEvent, ToolContext, ToolResult};
use crate::session_journal::SessionRunEvent;
use anyhow::{anyhow, Result};
use runtime_executor_core::{
    extract_tool_call_parse_error, split_error_code_and_message, truncate_tool_output,
    update_tool_failure_streak, ToolFailureStreak, MAX_TOOL_OUTPUT_CHARS,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub(crate) struct ToolDispatchContext<'a> {
    pub registry: &'a ToolRegistry,
    pub app_handle: Option<&'a AppHandle>,
    pub session_id: Option<&'a str>,
    pub persisted_run_id: Option<&'a str>,
    pub allowed_tools: Option<&'a [String]>,
    pub permission_mode: PermissionMode,
    pub tool_ctx: &'a ToolContext,
    pub tool_confirm_tx: Option<&'a Arc<Mutex<Option<std::sync::mpsc::Sender<bool>>>>>,
    pub cancel_flag: Option<Arc<AtomicBool>>,
    pub route_run_id: &'a str,
    pub route_node_timeout_secs: u64,
    pub route_retry_count: usize,
    pub iteration: usize,
}

pub(crate) struct ToolDispatchState<'a> {
    pub tool_results: &'a mut Vec<ToolResult>,
    pub repeated_failure_summary: &'a mut Option<String>,
    pub tool_failure_streak: &'a mut Option<ToolFailureStreak>,
    pub progress_history: &'a mut Vec<ProgressFingerprint>,
    pub latest_browser_progress: &'a mut Option<BrowserProgressSnapshot>,
}

pub(crate) enum ToolDispatchOutcome {
    Continue,
    Cancelled,
}

enum ApprovalOutcome {
    Allowed(crate::approval_bus::ApprovalDecision),
    TimedOut,
    Failed(String),
}

enum BlockingToolOutcome {
    Completed((String, bool)),
    Cancelled,
    TimedOut,
}

async fn resolve_approval_outcome(
    ctx: &ToolDispatchContext<'_>,
    call: &ToolCall,
) -> Result<ApprovalOutcome> {
    match gate_tool_approval(
        ctx.app_handle,
        ctx.session_id,
        ctx.persisted_run_id,
        call,
        ctx.tool_ctx.work_dir.as_deref(),
        ctx.tool_confirm_tx,
        ctx.cancel_flag.clone(),
    )
    .await
    {
        Ok(Some(decision)) => Ok(ApprovalOutcome::Allowed(decision)),
        Ok(None) => Ok(ApprovalOutcome::TimedOut),
        Err(err) => Ok(ApprovalOutcome::Failed(err.to_string())),
    }
}

async fn emit_failed_completion(
    ctx: &ToolDispatchContext<'_>,
    call: &ToolCall,
    state: &mut ToolDispatchState<'_>,
    message: String,
) {
    if let (Some(app), Some(sid)) = (ctx.app_handle, ctx.session_id) {
        let _ = app.emit(
            "tool-call-event",
            ToolCallEvent {
                session_id: sid.to_string(),
                tool_name: call.name.clone(),
                tool_input: call.input.clone(),
                tool_output: Some(message.clone()),
                status: "error".to_string(),
            },
        );
        if let Some(run_id) = ctx.persisted_run_id {
            let _ = append_tool_run_event(
                app,
                sid,
                SessionRunEvent::ToolCompleted {
                    run_id: run_id.to_string(),
                    tool_name: call.name.clone(),
                    call_id: call.id.clone(),
                    input: call.input.clone(),
                    output: message.clone(),
                    is_error: true,
                },
            )
            .await;
        }
    }

    state.tool_results.push(ToolResult {
        tool_use_id: call.id.clone(),
        content: message,
    });
}

async fn emit_tool_completion(
    ctx: &ToolDispatchContext<'_>,
    call: &ToolCall,
    node_id: &str,
    skill_name: &str,
    result: &str,
    is_error: bool,
    is_skill_call: bool,
    started_at: std::time::Instant,
) {
    if let (Some(app), Some(sid)) = (ctx.app_handle, ctx.session_id) {
        let _ = app.emit(
            "tool-call-event",
            ToolCallEvent {
                session_id: sid.to_string(),
                tool_name: call.name.clone(),
                tool_input: call.input.clone(),
                tool_output: Some(result.to_string()),
                status: if is_error {
                    "error".to_string()
                } else {
                    "completed".to_string()
                },
            },
        );
        if let Some(run_id) = ctx.persisted_run_id {
            let _ = append_tool_run_event(
                app,
                sid,
                SessionRunEvent::ToolCompleted {
                    run_id: run_id.to_string(),
                    tool_name: call.name.clone(),
                    call_id: call.id.clone(),
                    input: call.input.clone(),
                    output: result.to_string(),
                    is_error,
                },
            )
            .await;
        }

        if is_skill_call {
            let duration_ms = started_at.elapsed().as_millis() as u64;
            let parsed_error = if is_error {
                Some(split_error_code_and_message(result))
            } else {
                None
            };
            let _ = app.emit(
                "skill-route-node-updated",
                build_skill_route_event(
                    sid,
                    ctx.route_run_id,
                    node_id,
                    None,
                    skill_name,
                    1,
                    if is_error { "failed" } else { "completed" },
                    Some(duration_ms),
                    parsed_error.as_ref().map(|(code, _)| code.as_str()),
                    parsed_error.as_ref().map(|(_, msg)| msg.as_str()),
                ),
            );
        }
    }
}

fn record_tool_progress(
    state: &mut ToolDispatchState<'_>,
    call: &ToolCall,
    result: &str,
    is_error: bool,
) {
    if is_error {
        if let Some(summary) = update_tool_failure_streak(
            state.tool_failure_streak,
            &call.name,
            &call.input,
            result,
        ) {
            *state.repeated_failure_summary = Some(summary);
        }
    } else {
        *state.tool_failure_streak = None;
    }

    let input_signature = json_progress_signature(&call.input);
    let browser_progress_snapshot = if is_error {
        None
    } else {
        BrowserProgressSnapshot::from_tool_output(&call.name, result)
    };
    let output_signature = if let Some(snapshot) = browser_progress_snapshot.as_ref() {
        snapshot.progress_signature()
    } else {
        let progress_text = if is_error {
            format!("error:{result}")
        } else {
            result.to_string()
        };
        text_progress_signature(&progress_text)
    };
    if let Some(snapshot) = browser_progress_snapshot {
        *state.latest_browser_progress = Some(snapshot);
    }
    state.progress_history.push(ProgressFingerprint::tool_result(
        call.name.clone(),
        input_signature,
        output_signature,
    ));

    state.tool_results.push(ToolResult {
        tool_use_id: call.id.clone(),
        content: result.to_string(),
    });
}

pub(crate) async fn dispatch_tool_call(
    ctx: &ToolDispatchContext<'_>,
    state: &mut ToolDispatchState<'_>,
    call_index: usize,
    call: &ToolCall,
) -> Result<ToolDispatchOutcome> {
    let skill_name = call
        .input
        .get("skill_name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let is_skill_call = call.name == "skill";
    let node_id = format!("{}-{}-{}", ctx.iteration, call_index, call.id);
    let started_at = std::time::Instant::now();

    if is_skill_call {
        if let (Some(app), Some(sid)) = (ctx.app_handle, ctx.session_id) {
            let _ = app.emit(
                "skill-route-node-updated",
                build_skill_route_event(
                    sid,
                    ctx.route_run_id,
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

    if let Some(ref flag) = ctx.cancel_flag {
        if flag.load(Ordering::SeqCst) {
            eprintln!("[agent] 工具执行中被用户取消");
            return Ok(ToolDispatchOutcome::Cancelled);
        }
    }

    eprintln!("[agent] Calling tool: {}", call.name);

    if is_skill_call {
        if let (Some(app), Some(sid)) = (ctx.app_handle, ctx.session_id) {
            let _ = app.emit(
                "skill-route-node-updated",
                build_skill_route_event(
                    sid,
                    ctx.route_run_id,
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

    if let (Some(app), Some(sid)) = (ctx.app_handle, ctx.session_id) {
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
        if let Some(run_id) = ctx.persisted_run_id {
            let _ = append_tool_run_event(
                app,
                sid,
                SessionRunEvent::ToolStarted {
                    run_id: run_id.to_string(),
                    tool_name: call.name.clone(),
                    call_id: call.id.clone(),
                    input: call.input.clone(),
                },
            )
            .await;
        }
    }

    if ctx
        .permission_mode
        .needs_confirmation(&call.name, &call.input, ctx.tool_ctx.work_dir.as_deref())
    {
        match resolve_approval_outcome(ctx, call).await? {
            ApprovalOutcome::TimedOut => {
                state.tool_results.push(ToolResult {
                    tool_use_id: call.id.clone(),
                    content: "工具确认超时，已取消此操作".to_string(),
                });
                return Ok(ToolDispatchOutcome::Continue);
            }
            ApprovalOutcome::Failed(message) => {
                emit_failed_completion(ctx, call, state, message).await;
                return Ok(ToolDispatchOutcome::Continue);
            }
            ApprovalOutcome::Allowed(decision) => {
                if decision == crate::approval_bus::ApprovalDecision::Deny {
                    emit_failed_completion(ctx, call, state, "用户拒绝了此操作".to_string()).await;
                    return Ok(ToolDispatchOutcome::Continue);
                }
            }
        }
    }

    let max_attempts = if is_skill_call {
        ctx.route_retry_count + 1
    } else {
        1
    };
    let mut attempt = 0usize;
    let (result, is_error) = loop {
        attempt += 1;
        let (result, is_error) = if let Some(parse_error) = extract_tool_call_parse_error(&call.input)
        {
            (
                format!(
                    "工具参数错误: {}。请提供完整且合法的 JSON 参数后再重试。",
                    parse_error
                ),
                true,
            )
        } else {
            match ctx.registry.get(&call.name) {
                Some(tool) => {
                    if let Some(whitelist) = ctx.allowed_tools {
                        if !whitelist.iter().any(|w| w == &call.name) {
                            (format!("此 Skill 不允许使用工具: {}", call.name), true)
                        } else {
                            run_tool(
                                tool,
                                call,
                                ctx.tool_ctx,
                                ctx.cancel_flag.clone(),
                                ctx.route_node_timeout_secs,
                                is_skill_call,
                            )
                            .await
                        }
                    } else {
                        run_tool(
                            tool,
                            call,
                            ctx.tool_ctx,
                            ctx.cancel_flag.clone(),
                            ctx.route_node_timeout_secs,
                            is_skill_call,
                        )
                        .await
                    }
                }
                None => {
                    let available: Vec<String> = ctx
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

    let result = truncate_tool_output(&result, MAX_TOOL_OUTPUT_CHARS);

    emit_tool_completion(
        ctx,
        call,
        &node_id,
        &skill_name,
        &result,
        is_error,
        is_skill_call,
        started_at,
    )
    .await;

    if is_error {
        if let Some(mut stop_reason) = classify_policy_blocked_tool_error(&call.name, &result) {
            if let Some(last_completed_step) = state
                .latest_browser_progress
                .as_ref()
                .and_then(BrowserProgressSnapshot::last_completed_step)
            {
                stop_reason = stop_reason.with_last_completed_step(last_completed_step);
            }
            if let (Some(app), Some(sid)) = (ctx.app_handle, ctx.session_id) {
                let _ = app.emit(
                    "agent-state-event",
                    AgentStateEvent::stopped(sid, ctx.iteration, &stop_reason),
                );
            }
            return Err(anyhow!(encode_run_stop_reason(&stop_reason)));
        }
    }

    record_tool_progress(state, call, &result, is_error);

    Ok(ToolDispatchOutcome::Continue)
}

async fn run_tool(
    tool: Arc<dyn Tool>,
    call: &ToolCall,
    tool_ctx: &ToolContext,
    cancel_flag: Option<Arc<AtomicBool>>,
    route_node_timeout_secs: u64,
    is_skill_call: bool,
) -> (String, bool) {
    let tool_clone = Arc::clone(&tool);
    let input_clone = call.input.clone();
    let ctx_clone = tool_ctx.clone();
    let mut handle =
        tokio::task::spawn_blocking(move || tool_clone.execute(input_clone, &ctx_clone));

    let outcome = if is_skill_call {
        tokio::select! {
            res = &mut handle => BlockingToolOutcome::Completed(classify_blocking_tool_join_result(res)),
            _ = wait_for_cancel(&cancel_flag) => BlockingToolOutcome::Cancelled,
            _ = tokio::time::sleep(std::time::Duration::from_secs(route_node_timeout_secs)) => {
                BlockingToolOutcome::TimedOut
            }
        }
    } else {
        tokio::select! {
            res = &mut handle => BlockingToolOutcome::Completed(classify_blocking_tool_join_result(res)),
            _ = wait_for_cancel(&cancel_flag) => BlockingToolOutcome::Cancelled,
        }
    };

    match outcome {
        BlockingToolOutcome::Completed(result) => result,
        BlockingToolOutcome::Cancelled => {
            let _ = handle.await;
            ("工具执行被用户取消".to_string(), true)
        }
        BlockingToolOutcome::TimedOut => {
            let _ = handle.await;
            ("TIMEOUT: 子 Skill 执行超时".to_string(), true)
        }
    }
}

fn classify_blocking_tool_join_result(
    result: std::result::Result<Result<String>, tokio::task::JoinError>,
) -> (String, bool) {
    match result {
        Ok(Ok(output)) => (output, false),
        Ok(Err(e)) => (format!("工具执行错误: {}", e), true),
        Err(e) => (format!("工具执行线程异常: {}", e), true),
    }
}

async fn wait_for_cancel(cancel_flag: &Option<Arc<AtomicBool>>) {
    loop {
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                return;
            }
        } else {
            std::future::pending::<()>().await;
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::run_tool;
    use crate::agent::types::{Tool, ToolContext};
    use anyhow::Result;
    use serde_json::{json, Value};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    struct BlockingTool {
        started: Arc<AtomicBool>,
        release: Arc<AtomicBool>,
    }

    impl Tool for BlockingTool {
        fn name(&self) -> &str {
            "blocking_tool"
        }

        fn description(&self) -> &str {
            "blocks until released"
        }

        fn input_schema(&self) -> Value {
            json!({})
        }

        fn execute(&self, _input: Value, _ctx: &ToolContext) -> Result<String> {
            self.started.store(true, Ordering::SeqCst);
            while !self.release.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok("done".to_string())
        }
    }

    async fn wait_for_started_flag(started: &AtomicBool) {
        for _ in 0..50 {
            if started.load(Ordering::SeqCst) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("blocking tool never started");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_tool_cancellation_waits_for_blocking_task_to_finish() {
        let started = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let tool = Arc::new(BlockingTool {
            started: Arc::clone(&started),
            release: Arc::clone(&release),
        });
        let call = crate::agent::types::ToolCall {
            id: "call-1".to_string(),
            name: "blocking_tool".to_string(),
            input: json!({}),
        };
        let tool_ctx = ToolContext::default();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let mut run = Box::pin(run_tool(
            tool,
            &call,
            &tool_ctx,
            Some(Arc::clone(&cancel_flag)),
            0,
            false,
        ));

        tokio::select! {
            _ = &mut run => panic!("run_tool completed before the blocking task could be observed"),
            _ = tokio::time::sleep(Duration::from_millis(10)) => {}
        }
        wait_for_started_flag(&started).await;
        cancel_flag.store(true, Ordering::SeqCst);

        assert!(
            tokio::time::timeout(Duration::from_millis(300), &mut run)
                .await
                .is_err(),
            "run_tool returned before draining the blocking task"
        );

        release.store(true, Ordering::SeqCst);
        let (output, is_error) = tokio::time::timeout(Duration::from_secs(1), &mut run)
            .await
            .expect("drained blocking task");
        assert!(is_error);
        assert_eq!(output, "工具执行被用户取消");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_tool_timeout_waits_for_blocking_task_to_finish() {
        let started = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let tool = Arc::new(BlockingTool {
            started: Arc::clone(&started),
            release: Arc::clone(&release),
        });
        let call = crate::agent::types::ToolCall {
            id: "call-2".to_string(),
            name: "blocking_tool".to_string(),
            input: json!({}),
        };
        let tool_ctx = ToolContext::default();
        let mut run = Box::pin(run_tool(tool, &call, &tool_ctx, None, 0, true));

        tokio::select! {
            _ = &mut run => panic!("run_tool completed before the blocking task could be observed"),
            _ = tokio::time::sleep(Duration::from_millis(10)) => {}
        }
        wait_for_started_flag(&started).await;

        assert!(
            tokio::time::timeout(Duration::from_millis(300), &mut run)
                .await
                .is_err(),
            "run_tool returned before draining the timed-out blocking task"
        );

        release.store(true, Ordering::SeqCst);
        let (output, is_error) = tokio::time::timeout(Duration::from_secs(1), &mut run)
            .await
            .expect("drained blocking task");
        assert!(is_error);
        assert_eq!(output, "TIMEOUT: 子 Skill 执行超时");
    }
}
