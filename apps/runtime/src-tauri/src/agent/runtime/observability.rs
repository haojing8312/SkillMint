use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex};

const DEFAULT_MAX_EVENTS: usize = 400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeObservedEvent {
    SessionRun(RuntimeObservedRunEvent),
    AdmissionConflict(RuntimeObservedAdmissionConflict),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeObservedRunEvent {
    pub session_id: String,
    pub run_id: String,
    pub event_type: String,
    pub created_at: String,
    pub status: Option<String>,
    pub tool_name: Option<String>,
    pub approval_id: Option<String>,
    pub warning_kind: Option<String>,
    pub error_kind: Option<String>,
    pub child_session_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeObservedAdmissionConflict {
    pub session_id: String,
    pub created_at: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRecentEventsSnapshot {
    pub buffered: usize,
    pub max_buffered: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeTurnsSnapshot {
    pub active: usize,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub average_latency_ms: u64,
    pub max_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeAdmissionsSnapshot {
    pub conflicts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeGuardSnapshot {
    pub warnings_by_kind: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeApprovalsSnapshot {
    pub requested_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeChildSessionsSnapshot {
    pub linked_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeCompactionSnapshot {
    pub runs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeFailoverSnapshot {
    pub errors_by_kind: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeObservabilitySnapshot {
    pub recent_events: RuntimeRecentEventsSnapshot,
    pub turns: RuntimeTurnsSnapshot,
    pub admissions: RuntimeAdmissionsSnapshot,
    pub guard: RuntimeGuardSnapshot,
    pub approvals: RuntimeApprovalsSnapshot,
    pub child_sessions: RuntimeChildSessionsSnapshot,
    pub compaction: RuntimeCompactionSnapshot,
    pub failover: RuntimeFailoverSnapshot,
    pub errors_by_kind: BTreeMap<String, u64>,
}

#[derive(Debug)]
pub struct RuntimeObservability {
    max_events: usize,
    inner: Mutex<RuntimeObservabilityInner>,
}

#[derive(Debug, Clone)]
pub struct RuntimeObservabilityState(pub Arc<RuntimeObservability>);

#[derive(Debug, Default)]
struct RuntimeObservabilityInner {
    recent_events: VecDeque<RuntimeObservedEvent>,
    active_runs: usize,
    completed_runs: u64,
    failed_runs: u64,
    cancelled_runs: u64,
    total_latency_ms: u64,
    max_latency_ms: u64,
    admission_conflicts: u64,
    guard_warnings_by_kind: BTreeMap<String, u64>,
    error_counts_by_kind: BTreeMap<String, u64>,
    approval_requests: u64,
    child_session_links: u64,
    compaction_runs: u64,
    failover_errors_by_kind: BTreeMap<String, u64>,
    started_at_by_run: HashMap<String, i64>,
}

impl Default for RuntimeObservability {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_EVENTS)
    }
}

impl RuntimeObservability {
    pub fn new(max_events: usize) -> Self {
        Self {
            max_events: max_events.max(1),
            inner: Mutex::new(RuntimeObservabilityInner::default()),
        }
    }

    pub fn record_recent_event(&self, event: RuntimeObservedEvent) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::apply_event(&mut inner, &event);
        inner.recent_events.push_back(event);
        while inner.recent_events.len() > self.max_events {
            inner.recent_events.pop_front();
        }
    }

    pub fn record_admission_conflict(&self, session_id: &str) {
        let event = RuntimeObservedEvent::AdmissionConflict(RuntimeObservedAdmissionConflict {
            session_id: session_id.trim().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            code: "SESSION_RUN_CONFLICT".to_string(),
            message: "session still has an active run".to_string(),
        });
        self.record_recent_event(event);
    }

    pub fn record_compaction_run(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.compaction_runs += 1;
    }

    pub fn record_child_session_link(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.child_session_links += 1;
    }

    pub fn record_failover_error_kind(&self, error_kind: &str) {
        let key = normalize_key(error_kind);
        if key.is_empty() {
            return;
        }
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *inner.failover_errors_by_kind.entry(key).or_insert(0) += 1;
    }

    pub fn recent_events(&self) -> Vec<RuntimeObservedEvent> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .recent_events
            .iter()
            .cloned()
            .collect()
    }

    pub fn snapshot(&self) -> RuntimeObservabilitySnapshot {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let terminal_runs = inner.completed_runs + inner.failed_runs + inner.cancelled_runs;
        let average_latency_ms = if terminal_runs == 0 {
            0
        } else {
            inner.total_latency_ms / terminal_runs
        };

        RuntimeObservabilitySnapshot {
            recent_events: RuntimeRecentEventsSnapshot {
                buffered: inner.recent_events.len(),
                max_buffered: self.max_events,
            },
            turns: RuntimeTurnsSnapshot {
                active: inner.active_runs,
                completed: inner.completed_runs,
                failed: inner.failed_runs,
                cancelled: inner.cancelled_runs,
                average_latency_ms,
                max_latency_ms: inner.max_latency_ms,
            },
            admissions: RuntimeAdmissionsSnapshot {
                conflicts: inner.admission_conflicts,
            },
            guard: RuntimeGuardSnapshot {
                warnings_by_kind: inner.guard_warnings_by_kind.clone(),
            },
            approvals: RuntimeApprovalsSnapshot {
                requested_total: inner.approval_requests,
            },
            child_sessions: RuntimeChildSessionsSnapshot {
                linked_total: inner.child_session_links,
            },
            compaction: RuntimeCompactionSnapshot {
                runs: inner.compaction_runs,
            },
            failover: RuntimeFailoverSnapshot {
                errors_by_kind: inner.failover_errors_by_kind.clone(),
            },
            errors_by_kind: inner.error_counts_by_kind.clone(),
        }
    }

    fn apply_event(inner: &mut RuntimeObservabilityInner, event: &RuntimeObservedEvent) {
        match event {
            RuntimeObservedEvent::AdmissionConflict(_) => {
                inner.admission_conflicts += 1;
            }
            RuntimeObservedEvent::SessionRun(event) => {
                let run_key = run_key(&event.session_id, &event.run_id);
                let event_at = parse_timestamp_millis(&event.created_at);
                match event.event_type.as_str() {
                    "run_started" => {
                        inner.active_runs += 1;
                        if let Some(event_at) = event_at {
                            inner.started_at_by_run.insert(run_key, event_at);
                        }
                    }
                    "run_completed" => {
                        inner.completed_runs += 1;
                        settle_run(inner, &run_key, event_at);
                    }
                    "run_failed" | "run_stopped" => {
                        inner.failed_runs += 1;
                        if let Some(error_kind) = event.error_kind.as_deref() {
                            let key = normalize_key(error_kind);
                            if !key.is_empty() {
                                *inner.error_counts_by_kind.entry(key).or_insert(0) += 1;
                            }
                        }
                        settle_run(inner, &run_key, event_at);
                    }
                    "run_cancelled" => {
                        inner.cancelled_runs += 1;
                        settle_run(inner, &run_key, event_at);
                    }
                    "run_guard_warning" => {
                        if let Some(warning_kind) = event.warning_kind.as_deref() {
                            let key = normalize_key(warning_kind);
                            if !key.is_empty() {
                                *inner.guard_warnings_by_kind.entry(key).or_insert(0) += 1;
                            }
                        }
                    }
                    "approval_requested" => {
                        inner.approval_requests += 1;
                    }
                    _ => {}
                }

                if event.child_session_id.as_deref().is_some()
                    && matches!(event.event_type.as_str(), "tool_started" | "approval_requested")
                {
                    inner.child_session_links += 1;
                }
            }
        }
    }
}

fn settle_run(inner: &mut RuntimeObservabilityInner, run_key: &str, terminal_at: Option<i64>) {
    if inner.active_runs > 0 {
        inner.active_runs -= 1;
    }
    if let (Some(started_at), Some(terminal_at)) = (
        inner.started_at_by_run.remove(run_key),
        terminal_at,
    ) {
        if terminal_at >= started_at {
            let latency_ms = (terminal_at - started_at) as u64;
            inner.total_latency_ms += latency_ms;
            inner.max_latency_ms = inner.max_latency_ms.max(latency_ms);
        }
    }
}

fn run_key(session_id: &str, run_id: &str) -> String {
    format!("{}:{}", session_id.trim(), run_id.trim())
}

fn parse_timestamp_millis(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.timestamp_millis())
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{
        RuntimeObservedEvent, RuntimeObservedRunEvent, RuntimeObservability,
        RuntimeObservabilitySnapshot,
    };

    fn run_event(
        session_id: &str,
        run_id: &str,
        event_type: &str,
        created_at: &str,
    ) -> RuntimeObservedRunEvent {
        RuntimeObservedRunEvent {
            session_id: session_id.to_string(),
            run_id: run_id.to_string(),
            event_type: event_type.to_string(),
            created_at: created_at.to_string(),
            status: None,
            tool_name: None,
            approval_id: None,
            warning_kind: None,
            error_kind: None,
            child_session_id: None,
            message: None,
        }
    }

    fn snapshot(subject: &RuntimeObservability) -> RuntimeObservabilitySnapshot {
        subject.snapshot()
    }

    #[test]
    fn recent_runtime_events_trim_to_max_size() {
        let subject = RuntimeObservability::new(2);
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(run_event(
            "session-1",
            "run-1",
            "run_started",
            "2026-03-27T10:00:00Z",
        )));
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(run_event(
            "session-1",
            "run-1",
            "tool_started",
            "2026-03-27T10:00:01Z",
        )));
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(run_event(
            "session-1",
            "run-1",
            "run_completed",
            "2026-03-27T10:00:02Z",
        )));

        let recent = subject.recent_events();
        assert_eq!(recent.len(), 2);
        match &recent[0] {
            RuntimeObservedEvent::SessionRun(event) => {
                assert_eq!(event.event_type, "tool_started");
            }
            other => panic!("unexpected event: {other:?}"),
        }
        match &recent[1] {
            RuntimeObservedEvent::SessionRun(event) => {
                assert_eq!(event.event_type, "run_completed");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn snapshot_starts_at_zero() {
        let subject = RuntimeObservability::new(16);
        let snapshot = snapshot(&subject);

        assert_eq!(snapshot.recent_events.buffered, 0);
        assert_eq!(snapshot.turns.active, 0);
        assert_eq!(snapshot.turns.completed, 0);
        assert_eq!(snapshot.turns.failed, 0);
        assert_eq!(snapshot.turns.cancelled, 0);
        assert_eq!(snapshot.turns.average_latency_ms, 0);
        assert_eq!(snapshot.turns.max_latency_ms, 0);
        assert_eq!(snapshot.admissions.conflicts, 0);
        assert!(snapshot.guard.warnings_by_kind.is_empty());
        assert!(snapshot.errors_by_kind.is_empty());
    }

    #[test]
    fn run_lifecycle_updates_latency_stats() {
        let subject = RuntimeObservability::new(16);
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(run_event(
            "session-1",
            "run-1",
            "run_started",
            "2026-03-27T10:00:00Z",
        )));
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(run_event(
            "session-1",
            "run-1",
            "run_completed",
            "2026-03-27T10:00:03Z",
        )));
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(run_event(
            "session-2",
            "run-2",
            "run_started",
            "2026-03-27T10:01:00Z",
        )));
        let mut failed = run_event("session-2", "run-2", "run_failed", "2026-03-27T10:01:05Z");
        failed.error_kind = Some("network".to_string());
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(failed));

        let snapshot = snapshot(&subject);
        assert_eq!(snapshot.turns.active, 0);
        assert_eq!(snapshot.turns.completed, 1);
        assert_eq!(snapshot.turns.failed, 1);
        assert_eq!(snapshot.turns.average_latency_ms, 4_000);
        assert_eq!(snapshot.turns.max_latency_ms, 5_000);
        assert_eq!(snapshot.errors_by_kind.get("network"), Some(&1));
    }

    #[test]
    fn admission_conflicts_and_guard_warnings_accumulate() {
        let subject = RuntimeObservability::new(16);
        subject.record_admission_conflict("session-1");
        subject.record_admission_conflict("session-1");
        let mut warning = run_event(
            "session-1",
            "run-1",
            "run_guard_warning",
            "2026-03-27T10:02:00Z",
        );
        warning.warning_kind = Some("loop_detected".to_string());
        subject.record_recent_event(RuntimeObservedEvent::SessionRun(warning));

        let snapshot = snapshot(&subject);
        assert_eq!(snapshot.admissions.conflicts, 2);
        assert_eq!(snapshot.guard.warnings_by_kind.get("loop_detected"), Some(&1));

        let recent = subject.recent_events();
        assert_eq!(recent.len(), 3);
    }
}

