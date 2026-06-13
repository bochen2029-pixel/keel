//! # keel-middleware — L3, the invariant chain
//!
//! Cross-cutting concerns applied to **every** call via the kernel's `Chain` (canon §6, §8):
//! I1 audit, I3 privacy, I4 cost. Each is a `Middleware`; composed in a chain they become
//! structurally unbypassable.
//!
//! - **landed:** `cost` (I4 — budget hard-stop gate) · `audit` (I1 — a structured event per call,
//!   behind an `AuditSink`).
//! - **next:** `privacy` (I3, rungs 1–2: operator markers + regex / checksums).

pub mod audit;
pub mod cost;

pub use audit::{AuditEvent, AuditMiddleware, AuditSink};
pub use cost::CostMiddleware;
