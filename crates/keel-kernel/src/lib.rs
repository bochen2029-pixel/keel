//! # keel-kernel — L1, the spine
//!
//! The kernel runs the genome. It imports **only** L0 (`keel-contracts`) — never adapters,
//! middleware, or services (the layer rule, canon §6). Built one slice at a time:
//!
//! - **landed:** `manifest` (config → behavior) · `context` (the object that flows every call) ·
//!   `registry` (tier → adapter) · `chain` (the middleware executor) · `lifecycle` (the substrate
//!   resolver — probe/resolve **and** launch/supervise).
//! - **next:** `engine` (the closed loop) · `lock` (reproducibility / substrate pin).

pub mod chain;
pub mod context;
pub mod lifecycle;
pub mod manifest;
pub mod registry;

pub use chain::Chain;
pub use context::{new_context, new_trace_id, now_millis};
pub use lifecycle::{
    default_local_candidates, launch, probe, resolve_endpoint, LlamaServer, LlamaServerConfig,
};
pub use manifest::{CostCfg, Manifest, PriceCfg, RouterCfg, TierCfg};
pub use registry::Registry;
