use crate::commands::openclaw_gateway::{plan_im_role_dispatch_requests, plan_im_role_events};
use crate::im::runtime_bridge::{ImRoleDispatchRequest, ImRoleEventPayload};
use crate::im::types::ImEvent;
use sqlx::SqlitePool;

pub async fn plan_role_events_for_feishu(
    pool: &SqlitePool,
    event: &ImEvent,
) -> Result<Vec<ImRoleEventPayload>, String> {
    plan_im_role_events(pool, event).await
}

pub async fn plan_role_dispatch_requests_for_feishu(
    pool: &SqlitePool,
    event: &ImEvent,
) -> Result<Vec<ImRoleDispatchRequest>, String> {
    plan_im_role_dispatch_requests(pool, event).await
}
