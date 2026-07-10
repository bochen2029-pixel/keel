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

pub mod distill;
pub mod driver;
pub mod maintenance;
pub mod memory;
pub mod perception;
pub mod recall;
pub mod router;
pub mod trace_sink;
pub mod verifier;

pub use distill::{export_training_jsonl, training_pair};
pub use driver::{HeartbeatDriver, UserTurnDriver, WatchDriver};
pub use maintenance::{parse_cold_eyes, ColdEyesVerdict, MaintState, Maintenance, MaintenancePolicy, MaintenanceStats};
pub use memory::{Episode, FileMemory};
pub use perception::{ChangeGate, FrameGate};
pub use recall::{
    cosine, rank_all, recall_top_k, run_recall_bench, should_rebuild, BenchConfig, BenchReport, Fingerprint,
    IdentityRerank, RecallSet, Rerank,
};
pub use router::DifficultyRouter;
pub use trace_sink::FileTraceSink;
pub use verifier::{GoldenDispatchOracle, GoldenOracle, PropertyOracle, SchemaOracle, SourceOracle, Verifier};
