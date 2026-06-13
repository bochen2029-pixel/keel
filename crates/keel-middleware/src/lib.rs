//! # keel-middleware — L3, the invariant chain
//!
//! Cross-cutting concerns applied to **every** call via the kernel's `Chain` (canon §6, §8):
//! I1 audit, I3 privacy, I4 cost. Each is a `Middleware`; composed in a chain they become
//! structurally unbypassable. Ships depending only on L0 contracts (it *may* import the kernel —
//! it sits beside adapters, above it — but doesn't need to).
//!
//! - **landed:** `cost` (I4 — the budget hard-stop gate).
//! - **next:** `audit` (I1) · `privacy` (I3, rungs 1–2: operator markers + regex/checksums).

pub mod cost;

pub use cost::CostMiddleware;
