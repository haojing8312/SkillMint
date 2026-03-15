use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    AllowOnce,
    AllowAlways,
    Deny,
}

impl ApprovalDecision {
    fn as_db_value(&self) -> &'static str {
        match self {
            ApprovalDecision::AllowOnce => "allow_once",
            ApprovalDecision::AllowAlways => "allow_always",
            ApprovalDecision::Deny => "deny",
        }
    }

    fn resolved_status(&self) -> &'static str {
        match self {
            ApprovalDecision::Deny => "denied",
            ApprovalDecision::AllowOnce | ApprovalDecision::AllowAlways => "approved",
        }
    }

    fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "allow_once" => Some(Self::AllowOnce),
            "allow_always" => Some(Self::AllowAlways),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalResolution {
    pub approval_id: String,
    pub status: String,
    pub decision: ApprovalDecision,
    pub resolved_by_surface: String,
    pub resolved_by_user: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalResolveResult {
    Applied {
        approval_id: String,
        status: String,
        decision: ApprovalDecision,
    },
    AlreadyResolved {
        approval_id: String,
        status: String,
        decision: Option<ApprovalDecision>,
    },
    NotFound {
        approval_id: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ApprovalManager {
    waiters: Arc<Mutex<HashMap<String, oneshot::Sender<ApprovalResolution>>>>,
}

impl ApprovalManager {
    pub fn register_waiter(&self, approval_id: impl Into<String>) -> oneshot::Receiver<ApprovalResolution> {
        let approval_id = approval_id.into();
        let (tx, rx) = oneshot::channel();
        if let Ok(mut guard) = self.waiters.lock() {
            guard.insert(approval_id, tx);
        }
        rx
    }

    pub async fn resolve_with_pool(
        &self,
        pool: &SqlitePool,
        approval_id: &str,
        decision: ApprovalDecision,
        resolved_by_surface: &str,
        resolved_by_user: &str,
    ) -> Result<ApprovalResolveResult, String> {
        let now = Utc::now().to_rfc3339();
        let status = decision.resolved_status().to_string();
        let decision_value = decision.as_db_value().to_string();

        let result = sqlx::query(
            "UPDATE approvals
             SET status = ?, decision = ?, resolved_by_surface = ?, resolved_by_user = ?, resolved_at = ?, updated_at = ?
             WHERE id = ? AND status = 'pending'",
        )
        .bind(&status)
        .bind(&decision_value)
        .bind(resolved_by_surface.trim())
        .bind(resolved_by_user.trim())
        .bind(&now)
        .bind(&now)
        .bind(approval_id.trim())
        .execute(pool)
        .await
        .map_err(|e| format!("更新 approval 状态失败: {e}"))?;

        if result.rows_affected() > 0 {
            self.notify_waiter(ApprovalResolution {
                approval_id: approval_id.to_string(),
                status: status.clone(),
                decision: decision.clone(),
                resolved_by_surface: resolved_by_surface.trim().to_string(),
                resolved_by_user: resolved_by_user.trim().to_string(),
            });

            return Ok(ApprovalResolveResult::Applied {
                approval_id: approval_id.to_string(),
                status,
                decision,
            });
        }

        let current: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT status, NULLIF(decision, '')
             FROM approvals
             WHERE id = ?",
        )
        .bind(approval_id.trim())
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("读取 approval 当前状态失败: {e}"))?;

        match current {
            Some((status, decision_value)) => Ok(ApprovalResolveResult::AlreadyResolved {
                approval_id: approval_id.to_string(),
                status,
                decision: decision_value
                    .as_deref()
                    .and_then(ApprovalDecision::from_db_value),
            }),
            None => Ok(ApprovalResolveResult::NotFound {
                approval_id: approval_id.to_string(),
            }),
        }
    }

    fn notify_waiter(&self, resolution: ApprovalResolution) {
        let sender = self
            .waiters
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(&resolution.approval_id));
        if let Some(sender) = sender {
            let _ = sender.send(resolution);
        }
    }
}
