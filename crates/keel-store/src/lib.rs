//! keel-store — L2, the index (canon §13). **SQLite baked into the binary** (the `bundled`
//! feature; ~1 MB, the one primitive small enough to truly embed), behind a `Store` seam. It
//! implements the `Spine` contract (I2): durable, resumable run-state. The append-only file ledger
//! remains the system of record; this index is **derived and rebuildable** from it. It also serves
//! a **derived, off-loop metric rollup** ([`SqliteStore::metrics`]) — a *reader*, never middleware
//! (canon §13: KEEL owns metric rollups in this Store).

use async_trait::async_trait;
use keel_contracts::{Kind, KeelError, Result, RunId, Spine, State, Trace};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;
use std::sync::Mutex;

const SCHEMA: &str =
    "CREATE TABLE IF NOT EXISTS runs (run_id TEXT PRIMARY KEY, state TEXT NOT NULL, updated_at INTEGER NOT NULL);";

/// A SQLite-backed index. The `Spine` impl persists run-state for crash-resume (I2). The
/// connection sits behind a `Mutex` (rusqlite is sync; the guard is never held across an await).
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// Open (creating the file + schema) at `path`.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).map_err(sql_err)?;
        conn.execute_batch(SCHEMA).map_err(sql_err)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// An ephemeral in-memory index (tests, scratch).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(sql_err)?;
        conn.execute_batch(SCHEMA).map_err(sql_err)?;
        Ok(Self { conn: Mutex::new(conn) })
    }
}

fn sql_err(e: rusqlite::Error) -> KeelError {
    KeelError::Other(format!("sqlite: {e}"))
}

fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[async_trait]
impl Spine for SqliteStore {
    async fn checkpoint(&self, run: &RunId, state: &State) -> Result<()> {
        let json = serde_json::to_string(state).map_err(|e| KeelError::Other(format!("state encode: {e}")))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO runs (run_id, state, updated_at) VALUES (?1, ?2, ?3) \
             ON CONFLICT(run_id) DO UPDATE SET state = ?2, updated_at = ?3",
            params![run, json, now_millis()],
        )
        .map_err(sql_err)?;
        Ok(())
    }

    async fn resume(&self, run: &RunId) -> Result<Option<State>> {
        let conn = self.conn.lock().unwrap();
        let row: Option<String> = conn
            .query_row("SELECT state FROM runs WHERE run_id = ?1", params![run], |r| r.get(0))
            .optional()
            .map_err(sql_err)?;
        match row {
            Some(s) => {
                Ok(Some(serde_json::from_str(&s).map_err(|e| KeelError::Other(format!("state decode: {e}")))?))
            }
            None => Ok(None),
        }
    }
}

/// A derived, **off-loop** rollup over the I2 index (canon §13/§19) — a *reader*, never middleware.
/// The flywheel instrument: `escalation_rate` should trend down as the local tier handles more; a
/// rising `rework_rate` means convincing-wrong output is slipping past the oracles (I5 weakening).
#[derive(Debug, Default, Clone)]
pub struct MetricsSummary {
    /// Turns recorded. One row per run today (see the upsert note on [`SqliteStore::metrics`]).
    pub turns: usize,
    /// **Proxy** for the canon §19 `rework_rate`: model/content verify-fails / turns — a turn whose
    /// verdict failed for a *correctness* reason (an oracle rejected the output), **excluding wiring
    /// faults** (config-fault, unresolved golden_ref). The precise §19 metric (convincing-wrong that
    /// *escapes* the oracles) needs a downstream/human signal and is deferred.
    pub rework_rate: f64,
    /// Turns whose final tier climbed **above the kind's base tier** (scaffolding→local,
    /// core-wire→cheap-API) / turns — the flywheel escalation signal (target: downward). Reads ~0
    /// until a multi-turn Driver feeds `oracle_failures` back across turns.
    pub escalation_rate: f64,
    /// Descriptive: distribution of the tier that actually ran (NOT escalation).
    pub by_tier: BTreeMap<String, usize>,
    /// Descriptive: total cost (USD) across recorded turns.
    pub total_cost: f64,
}

impl SqliteStore {
    /// Read-only rollup over the `runs` index — **off-loop; never touches the engine**. **Upsert
    /// note:** the index keeps the *latest* state per `run_id`, which is complete while each turn is
    /// its own run (today). When a multi-turn Driver reuses a `run_id`, the metric source must move
    /// to the append-only traces ledger (the `TraceSink`) so intermediate turns aren't overwritten.
    pub fn metrics(&self) -> Result<MetricsSummary> {
        let states: Vec<String> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT state FROM runs").map_err(sql_err)?;
            let mapped = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql_err)?;
            mapped.collect::<std::result::Result<Vec<String>, rusqlite::Error>>().map_err(sql_err)?
        };
        let mut m = MetricsSummary::default();
        let (mut escalated, mut rework) = (0usize, 0usize);
        for s in &states {
            // skip non-Trace rows (other/older checkpoints) — the rollup reads only verified turns.
            let Ok(trace) = serde_json::from_str::<Trace>(s) else { continue };
            m.turns += 1;
            *m.by_tier.entry(trace.result.tier.clone()).or_default() += 1;
            m.total_cost += trace.result.cost;
            if tier_rank(&trace.result.tier) > base_rank(trace.step.kind) {
                escalated += 1;
            }
            if !trace.verdict.passed && trace.verdict.failures.iter().any(|f| !is_wiring_fault(f)) {
                rework += 1;
            }
        }
        if m.turns > 0 {
            m.escalation_rate = escalated as f64 / m.turns as f64;
            m.rework_rate = rework as f64 / m.turns as f64;
        }
        Ok(m)
    }
}

/// Ladder rank for the escalation comparison (cheapest first). Unknown tiers rank 0 (never "above").
fn tier_rank(tier: &str) -> usize {
    match tier {
        "cheap-API" => 1,
        "frontier" => 2,
        _ => 0, // "local" and unknowns
    }
}

/// The base tier a kind routes to **before** any escalation (canon §9): scaffolding→local (0),
/// core-wire→cheap-API (1). A final tier above this is genuine escalation, not normal difficulty routing.
fn base_rank(kind: Kind) -> usize {
    match kind {
        Kind::Scaffolding => 0,
        Kind::CoreWire => 1,
    }
}

/// Wiring faults (config-fault, unresolved golden_ref) are **not** model rework — they are
/// engine/registry misconfiguration. Matched on the engine's stable ASCII alarm markers so they are
/// excluded from `rework_rate` (lumping them in inflates it — the lesson from the first rollup).
fn is_wiring_fault(failure: &str) -> bool {
    failure.contains("config fault") || failure.contains("unresolved golden_ref")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn checkpoint_then_resume_roundtrips() {
        let store = SqliteStore::in_memory().unwrap();
        let run = "run-1".to_string();
        assert!(store.resume(&run).await.unwrap().is_none()); // nothing yet
        store.checkpoint(&run, &json!({ "step": 3, "plan": "x" })).await.unwrap();
        let got = store.resume(&run).await.unwrap().unwrap();
        assert_eq!(got["step"], 3);
        assert_eq!(got["plan"], "x");
    }

    #[tokio::test]
    async fn checkpoint_upserts_latest_state() {
        let store = SqliteStore::in_memory().unwrap();
        let run = "run-2".to_string();
        store.checkpoint(&run, &json!({ "step": 1 })).await.unwrap();
        store.checkpoint(&run, &json!({ "step": 9 })).await.unwrap();
        assert_eq!(store.resume(&run).await.unwrap().unwrap()["step"], 9);
    }

    #[tokio::test]
    async fn unknown_run_is_none() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(store.resume(&"nope".to_string()).await.unwrap().is_none());
    }

    /// Build a checkpoint in the exact shape the engine persists (a `Trace`).
    fn trace(tier: &str, kind: &str, passed: bool, fail: &str, cost: f64) -> serde_json::Value {
        let failures: Vec<&str> = if fail.is_empty() { vec![] } else { vec![fail] };
        json!({
            "step": { "kind": kind, "ty": "t", "trust_required": "normal", "data_class": "normal",
                      "tier_history": [], "oracle_failures": 0, "critical": false, "content": [], "golden_refs": [] },
            "decision": { "tier": tier, "effort": { "n": 1 }, "reason": "x" },
            "result": { "content": "", "tool_calls": [], "usage": { "input_tokens": 0, "output_tokens": 0, "cache_hit_tokens": 0 },
                        "cost": cost, "tier": tier, "model": "m" },
            "verdict": { "passed": passed, "failures": failures, "joint_wrong": false, "evidence": [] }
        })
    }

    #[tokio::test]
    async fn metrics_rolls_up_turns_tiers_and_rates() {
        let store = SqliteStore::in_memory().unwrap();
        store.checkpoint(&"r1".to_string(), &trace("local", "scaffolding", true, "", 0.0)).await.unwrap();
        // core-wire on cheap-API is its BASE — normal difficulty routing, NOT escalation:
        store.checkpoint(&"r2".to_string(), &trace("cheap-API", "core_wire", true, "", 0.10)).await.unwrap();
        // scaffolding on cheap-API is ABOVE base — genuine escalation:
        store.checkpoint(&"r3".to_string(), &trace("cheap-API", "scaffolding", true, "", 0.05)).await.unwrap();
        // a model/content rejection — counts as rework:
        store.checkpoint(&"r4".to_string(), &trace("local", "scaffolding", false, "schema: output is not JSON - rejected", 0.0)).await.unwrap();
        // a wiring fault — NOT model rework, excluded:
        store.checkpoint(&"r5".to_string(), &trace("local", "scaffolding", false, "critical step with no applicable correctness oracle - config fault", 0.0)).await.unwrap();

        let m = store.metrics().unwrap();
        assert_eq!(m.turns, 5);
        assert!((m.escalation_rate - 0.2).abs() < 1e-9, "only r3 escalated (scaffolding ran above local)");
        assert!((m.rework_rate - 0.2).abs() < 1e-9, "only r4 is model rework; r5 (config fault) is wiring, excluded");
        assert!((m.total_cost - 0.15).abs() < 1e-9);
        assert_eq!(m.by_tier.get("local"), Some(&3));
        assert_eq!(m.by_tier.get("cheap-API"), Some(&2));
    }

    #[tokio::test]
    async fn metrics_empty_store_is_zero() {
        let m = SqliteStore::in_memory().unwrap().metrics().unwrap();
        assert_eq!(m.turns, 0);
        assert_eq!(m.escalation_rate, 0.0);
        assert_eq!(m.rework_rate, 0.0);
        assert!(m.by_tier.is_empty());
    }

    /// Non-`Trace` rows (other checkpoints) are skipped, not counted or errored.
    #[tokio::test]
    async fn metrics_skips_non_trace_rows() {
        let store = SqliteStore::in_memory().unwrap();
        store.checkpoint(&"x".to_string(), &json!({ "step": 3, "plan": "not a trace" })).await.unwrap();
        store.checkpoint(&"r1".to_string(), &trace("local", "scaffolding", true, "", 0.0)).await.unwrap();
        assert_eq!(store.metrics().unwrap().turns, 1);
    }
}
