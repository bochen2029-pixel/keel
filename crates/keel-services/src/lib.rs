//! # keel-services — L4, the services
//!
//! Default (swappable) services composed from the contracts. A service MAY import middleware;
//! middleware may never import a service (the layer rule, canon §6).
//!
//! - **landed:** `router` (the difficulty router — the fusion point, canon §9) · `verifier` (the
//!   externality layer — pluggable oracle registry + joint-wrong, canon §10, the I5 keystone).
//! - **next:** `amplify` (best-of-N, ships OFF) · `memory` · `perception` · `driver`.

pub mod router;
pub mod verifier;

pub use router::DifficultyRouter;
pub use verifier::{GoldenOracle, PropertyOracle, SourceOracle, Verifier};
