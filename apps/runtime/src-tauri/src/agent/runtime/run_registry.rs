use crate::session_journal::{SessionJournalState, SessionRunStatus};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct RunRegistry {
    active_runs: RwLock<HashMap<String, String>>,
}

impl RunRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_active_run<S, R>(&self, session_id: S, run_id: R)
    where
        S: Into<String>,
        R: Into<String>,
    {
        let session_id = normalize_key(session_id.into());
        let run_id = normalize_key(run_id.into());
        if session_id.is_empty() || run_id.is_empty() {
            return;
        }

        self.active_runs
            .write()
            .expect("run registry lock")
            .insert(session_id, run_id);
    }

    pub fn complete_run(&self, session_id: &str, run_id: &str) {
        self.clear_if_matches(session_id, run_id);
    }

    pub fn cancel_run(&self, session_id: &str, run_id: &str) {
        self.clear_if_matches(session_id, run_id);
    }

    pub fn resolve_current_run_id(&self, session_id: &str) -> Option<String> {
        let session_id = normalize_key(session_id.to_string());
        if session_id.is_empty() {
            return None;
        }

        self.active_runs
            .read()
            .expect("run registry lock")
            .get(&session_id)
            .cloned()
    }

    pub fn sync_session_projection(&self, session_id: &str, current_run_id: Option<&str>) {
        match current_run_id.map(normalize_key).filter(|run_id| !run_id.is_empty()) {
            Some(run_id) => self.register_active_run(session_id, run_id),
            None => self.clear_session(session_id),
        }
    }

    pub fn hydrate_from_session_state(&self, state: &SessionJournalState) {
        if let Some(run_id) = current_active_run_id(state) {
            self.register_active_run(&state.session_id, run_id);
        } else {
            self.clear_session(&state.session_id);
        }
    }

    pub fn clear_session(&self, session_id: &str) {
        let session_id = normalize_key(session_id.to_string());
        if session_id.is_empty() {
            return;
        }

        self.active_runs
            .write()
            .expect("run registry lock")
            .remove(&session_id);
    }

    fn clear_if_matches(&self, session_id: &str, run_id: &str) {
        let session_id = normalize_key(session_id.to_string());
        let run_id = normalize_key(run_id.to_string());
        if session_id.is_empty() || run_id.is_empty() {
            return;
        }

        let mut guard = self.active_runs.write().expect("run registry lock");
        if guard.get(&session_id).map(|current| current == &run_id).unwrap_or(false) {
            guard.remove(&session_id);
        }
    }
}

fn normalize_key(value: impl AsRef<str>) -> String {
    value.as_ref().trim().to_string()
}

fn current_active_run_id(state: &SessionJournalState) -> Option<String> {
    if let Some(run_id) = state
        .current_run_id
        .as_deref()
        .map(str::trim)
        .filter(|run_id| !run_id.is_empty())
    {
        return Some(run_id.to_string());
    }

    state
        .runs
        .iter()
        .rev()
        .find(|run| matches!(run.status, SessionRunStatus::Thinking | SessionRunStatus::ToolCalling | SessionRunStatus::WaitingApproval))
        .map(|run| run.run_id.clone())
}
