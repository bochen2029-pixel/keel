//! # keel-kernel — L1, the spine
//!
//! The kernel runs the genome. It imports **only** L0 (`keel-contracts`) — never adapters,
//! middleware, or services (the layer rule, canon §6). Built one slice at a time:
//!
//! - **landed:** `manifest` (config → behavior) · `context` (the object that flows every call) ·
//!   `registry` (tier → adapter) · `chain` (the middleware executor — where I1/I3/I4 become
//!   structurally unbypassable).
//! - **next:** `lifecycle` (+ the substrate resolver) · `engine` (the closed loop) · `lock`
//!   (reproducibility / substrate pin).

pub mod chain;
pub mod context;
pub mod manifest;
pub mod registry;

pub use chain::Chain;
pub use context::{new_context, new_trace_id, now_millis};
pub use manifest::{CostCfg, Manifest, PriceCfg, RouterCfg, TierCfg};
pub use registry::Registry;
