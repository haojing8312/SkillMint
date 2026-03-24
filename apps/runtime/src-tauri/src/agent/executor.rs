use super::permissions::PermissionMode;
use super::registry::ToolRegistry;
use super::run_guard::{RunBudgetPolicy, RunBudgetScope};
use super::system_prompts::SystemPromptBuilder;
use super::types::StreamDelta;
use anyhow::Result;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;

const TOOL_CONFIRM_TIMEOUT_SECS: u64 = 15;

pub struct AgentExecutor {
    pub(super) registry: Arc<ToolRegistry>,
    pub(super) max_iterations: usize,
    pub(super) system_prompt_builder: SystemPromptBuilder,
}

impl AgentExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            max_iterations: RunBudgetPolicy::for_scope(RunBudgetScope::GeneralChat).max_turns,
            system_prompt_builder: SystemPromptBuilder::default(),
        }
    }

    pub fn with_max_iterations(registry: Arc<ToolRegistry>, max_iterations: usize) -> Self {
        Self {
            registry,
            max_iterations,
            system_prompt_builder: SystemPromptBuilder::default(),
        }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn registry_arc(&self) -> Arc<ToolRegistry> {
        Arc::clone(&self.registry)
    }

    /// 轮询 cancel_flag，直到收到取消信号
    pub(super) async fn wait_for_cancel(cancel_flag: &Option<Arc<AtomicBool>>) {
        loop {
            if let Some(ref flag) = cancel_flag {
                if flag.load(Ordering::SeqCst) {
                    return;
                }
            } else {
                // 没有 cancel_flag，永远不会取消
                std::future::pending::<()>().await;
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn execute_turn(
        &self,
        api_format: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
        skill_system_prompt: &str,
        messages: Vec<Value>,
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
        self.execute_turn_impl(
            api_format,
            base_url,
            api_key,
            model,
            skill_system_prompt,
            messages,
            on_token,
            app_handle,
            session_id,
            allowed_tools,
            permission_mode,
            tool_confirm_tx,
            work_dir,
            max_iterations_override,
            cancel_flag,
            route_node_timeout_secs,
            route_retry_count,
        )
        .await
    }
}
