//! # keel-middleware — L3, the invariant chain
//!
//! Cross-cutting concerns applied to **every** call via the kernel's `Chain` (canon §6, §8):
//! I1 audit, I3 privacy, I4 cost. Each is a `Middleware`; composed in a chain they become
//! structurally unbypassable.
//!
//! - **landed (the deterministic invariant trio):** `audit` (I1 — a structured event per call,
//!   behind an `AuditSink`) · `privacy` (I3 — the rung-1/2 deterministic mask: operator markers +
//!   regex/checksums) · `cost` (I4 — budget hard-stop gate).
//! - **next:** privacy rung 3 (the OpenAI Privacy Filter, a verification pass) lands in Stage 2
//!   behind `GOLDEN_PRIVACY`.

pub mod audit;
pub mod cost;
pub mod privacy;

pub use audit::{AuditEvent, AuditMiddleware, AuditSink};
pub use cost::CostMiddleware;
pub use privacy::{Finding, PrivacyMiddleware, Redactor};
