pub mod capability_router;
pub mod openai_compat;
pub mod anthropic_compat;
pub mod deepseek;
pub mod qwen;
pub mod moonshot;
pub mod registry;
pub mod traits;

pub use capability_router::{RouteFailureKind, RouteTarget, RoutingPolicy, route_with_fallback};
pub use registry::ProviderRegistry;
pub use traits::ProviderPlugin;
