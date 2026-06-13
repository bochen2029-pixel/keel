//! keel-kernel::context — minting and threading the universal `Context` (canon §14).
//!
//! `Context` is L0 *data* (every contract method receives it). The kernel owns its
//! *construction*: it stamps the clock and the trace id (L0 stays clock-free by design — see
//! `types.rs`), and seeds the per-task budget from the manifest (I4).

use crate::manifest::Manifest;
use keel_contracts::{Context, Time};

/// Unix epoch milliseconds. The kernel is where time is read; contracts never read a clock.
pub fn now_millis() -> Time {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// A fresh 12-hex-char run/trace id (matches the bench's `uuid4().hex[:12]`).
pub fn new_trace_id() -> String {
    let s = uuid::Uuid::new_v4().simple().to_string();
    s[..12].to_string()
}

/// Mint a fresh `Context` for a run: a new trace id, a clean redaction state (I3), and the
/// per-task budget (I4) seeded from the manifest.
pub fn new_context(manifest: &Manifest) -> Context {
    Context {
        trace_id: new_trace_id(),
        task_budget: Some(manifest.cost.budget_per_task),
        redaction_state: "clean".to_string(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_id_is_twelve_hex_and_unique() {
        let a = new_trace_id();
        let b = new_trace_id();
        assert_eq!(a.len(), 12);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn now_millis_is_after_2024() {
        // 2024-01-01T00:00:00Z, in ms — a sanity floor, not a precise clock test.
        assert!(now_millis() > 1_704_067_200_000);
    }

    #[test]
    fn new_context_seeds_budget_and_clean_state() {
        let m = Manifest::default();
        let c = new_context(&m);
        assert_eq!(c.redaction_state, "clean");
        assert_eq!(c.task_budget, Some(m.cost.budget_per_task));
        assert_eq!(c.budget_remaining(), Some(m.cost.budget_per_task));
        assert_eq!(c.trace_id.len(), 12);
    }
}
