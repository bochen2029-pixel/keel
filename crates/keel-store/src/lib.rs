//! keel-store — L2, the index (canon §13). **SQLite baked into the binary** (the `bundled`
//! feature; ~1 MB, the one primitive small enough to truly embed), behind a `Store` seam. This
//! first cut implements the `Spine` contract (I2): durable, resumable run-state. The append-only
//! file ledger remains the system of record; this index is **derived and rebuildable** from it.

use async_trait::async_trait;
use keel_contracts::{KeelError, Result, RunId, Spine, State};
use rusqlite::{params, Connection, OptionalExtension};
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
}
