//! Neutral IM ingress and routing facade for Rust callers.
//!
//! The current implementations still live behind the legacy OpenClaw command
//! module so public Tauri command compatibility can stay unchanged. New native
//! IM code should import these neutral APIs from this module.

pub use super::openclaw_gateway::{
    plan_im_role_dispatch_requests, plan_im_role_events, resolve_im_route_with_pool,
};
