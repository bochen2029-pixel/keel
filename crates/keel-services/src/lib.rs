//! # keel-services — L4, the services
//!
//! Default (swappable) services composed from the contracts. A service MAY import middleware;
//! middleware may never import a service (the layer rule, canon §6).
//!
//! - **landed:** `router` (the difficulty router — the fusion point, canon §9) · `verifier` (the
//!   externality layer — pluggable oracle registry + joint-wrong, canon §10, the I5 keystone) ·
//!   `perception` (the afferent change-gate — dHash/VAD, the cost control that ships with the senses,
//!   canon §12, `GOLDEN_PERCEPTION` green) · `memory` (the persistent self — the Tape-backed
//!   factual register + Ring-0/Ring-2 assembly, canon §11; minimal first cut, narrative register
//!   deferred) · `driver` (initiative — user-turn + heartbeat + watch behind the one `poll()` joint,
//!   canon §7/§8; the §23 seam falsifier; the daemon select-loop is a follow-on).
//! - **next:** `amplify` (best-of-N, ships OFF) · the perception capture organs · memory's
//!   narrative register + retrieval.

pub mod driver;
pub mod memory;
pub mod perception;
pub mod router;
pub mod verifier;

pub use driver::{HeartbeatDriver, UserTurnDriver, WatchDriver};
pub use memory::FileMemory;
pub use perception::{ChangeGate, FrameGate};
pub use router::DifficultyRouter;
pub use verifier::{GoldenDispatchOracle, GoldenOracle, PropertyOracle, SchemaOracle, SourceOracle, Verifier};
