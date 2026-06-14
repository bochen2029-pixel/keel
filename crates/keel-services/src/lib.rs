//! # keel-services — L4, the services
//!
//! Default (swappable) services composed from the contracts. A service MAY import middleware;
//! middleware may never import a service (the layer rule, canon §6).
//!
//! - **landed:** `router` (the difficulty router — the fusion point, canon §9) · `verifier` (the
//!   externality layer — pluggable oracle registry + joint-wrong, canon §10, the I5 keystone) ·
//!   `perception` (the afferent change-gate — dHash/VAD, the cost control that ships with the senses,
//!   canon §12, `GOLDEN_PERCEPTION` green).
//! - **next:** `amplify` (best-of-N, ships OFF) · `memory` · `driver` · the perception capture organs.

pub mod perception;
pub mod router;
pub mod verifier;

pub use perception::ChangeGate;
pub use router::DifficultyRouter;
pub use verifier::{GoldenDispatchOracle, GoldenOracle, PropertyOracle, SchemaOracle, SourceOracle, Verifier};
