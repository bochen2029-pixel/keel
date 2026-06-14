//! # keel-services — L4, the services
//!
//! Default (swappable) services composed from the contracts. A service MAY import middleware;
//! middleware may never import a service (the layer rule, canon §6).
//!
//! - **landed:** `router` (the difficulty router — the fusion point, canon §9).
//! - **next:** `verifier` (oracle registry + joint-wrong) · `amplify` (best-of-N, ships OFF) ·
//!   `memory` · `perception` · `driver`.

pub mod router;

pub use router::DifficultyRouter;
