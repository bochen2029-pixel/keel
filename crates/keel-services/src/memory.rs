//! keel-services::memory — the persistent self (canon §11). The first `Memory` impl.
//!
//! A file-backed ringed-context assembler over the **Tape** — the lossless, append-only,
//! human-readable JSONL ledger of every turn (canon §11: "the ledger is the Tape, and the Tape is
//! the Spine"). The Tape is the **factual register** (lossless, externalized) — the safe register for
//! critical facts, because a model may never author its own ground truth (I5). It persists on disk,
//! so working memory survives across `keel` invocations: a second call sees the first call's context.
//!
//! ## Minimal first cut (decided 2026-06-14 — decide-and-document under the autonomy grant; Memory is
//! the canon's flagged **highest-risk seam**, §23, so the cut is deliberately small + clearly-correct)
//! - **Tape (factual register):** [`record`](FileMemory::record) appends each `Trace` as one JSONL
//!   line. Lossless, crash-safe per line, human-readable.
//! - **assemble (Ring-0 + Ring-2):** returns a `system` preamble = an optional configured **soul**
//!   (Ring-0; **empty for the bare genome** — persona is a CELL concern, never the core) + the recent
//!   **working turns** (Ring-2) read back from the Tape, compacted to `user → assistant` lines.
//! - **consolidate:** returns a well-formed maintenance `Step` (the seam is real); the model-authored
//!   compression itself is deferred (below).
//!
//! ## Deferred — designed, not built (for a reviewed pass, against `docs/proposals/perpetual-memory.md`)
//! - The model-authored **narrative register** + consolidation *generation* (Ring-2 → Ring-3
//!   compression); Ring-1 calibration exemplars; Ring-4 retrieval (the embedder organ + `GOLDEN_RECALL`).
//! - Injecting Ring-2 as real `conversation` messages rather than a `system` preamble (needs the
//!   engine to consume `AssembledContext.conversation`).
//! - The **Tape ↔ SQLite-Spine unification** (§11): today the Tape is the lossless record and the
//!   SQLite index is the derived checkpoint; "the Tape is the Spine" folds them in a later slice.
//! - **Capture-sanctity early-record** (a two-phase Tape write: the step pre-cognition, the verdict
//!   after); today the full `Trace` is recorded post-verify alongside the checkpoint.

use async_trait::async_trait;
use keel_contracts::{
    AssembledContext, Content, Context, DataClass, KeelError, Kind, Memory, Result, Step, Trace, Trust,
};
use std::path::PathBuf;
use std::sync::Mutex;

/// A file-backed [`Memory`] over the append-only **Tape** (canon §11). The Tape is the lossless
/// system of record; the SQLite index (the `Spine`) is the derived, rebuildable checkpoint.
pub struct FileMemory {
    /// Ring-0 soul (identity / system message). **Empty for the bare genome**; a cell sets its persona.
    soul: String,
    /// The Tape file (append-only JSONL of `Trace`s). Created (with parent dirs) on first record.
    tape: PathBuf,
    /// How many recent turns (Ring-2 working memory) `assemble` reads back from the Tape.
    working_turns: usize,
    /// Serializes appends within a process (the Tape is also crash-safe per JSONL line).
    lock: Mutex<()>,
}

impl FileMemory {
    /// Build a Tape-backed memory. `soul` is Ring-0 (empty = no persona, the genome default).
    pub fn new(soul: impl Into<String>, tape: impl Into<PathBuf>, working_turns: usize) -> Self {
        Self { soul: soul.into(), tape: tape.into(), working_turns, lock: Mutex::new(()) }
    }

    /// Read the last `n` traces from the Tape in chronological order (best-effort: a missing or
    /// short Tape yields fewer; an unparseable line is skipped, never fatal — the Tape outlives schema).
    fn recent_traces(&self, n: usize) -> Vec<Trace> {
        let Ok(raw) = std::fs::read_to_string(&self.tape) else { return Vec::new() };
        let mut recent: Vec<Trace> =
            raw.lines().rev().filter_map(|l| serde_json::from_str::<Trace>(l).ok()).take(n).collect();
        recent.reverse(); // newest-first → chronological
        recent
    }

    /// Compact one trace into a `user → assistant` working-memory entry (text only; multimodal parts
    /// are summarized by their text). The lossless original stays on the Tape.
    fn summarize(trace: &Trace) -> String {
        let user = trace
            .step
            .content
            .iter()
            .find_map(|c| match c {
                Content::Text { text } => Some(text.trim()),
                _ => None,
            })
            .unwrap_or("");
        format!("- user: {user}\n  assistant: {}", trace.result.content.trim())
    }
}

#[async_trait]
impl Memory for FileMemory {
    /// Ring-0 (soul) + Ring-2 (recent working turns from the Tape), folded into the `system` preamble
    /// the engine prepends. Empty when there is no soul and no history (a fresh first turn) — the
    /// engine then prepends nothing.
    async fn assemble(&self, _step: &Step, _ctx: &Context) -> Result<AssembledContext> {
        let mut system = self.soul.clone();
        let recent = self.recent_traces(self.working_turns);
        if !recent.is_empty() {
            if !system.is_empty() {
                system.push_str("\n\n");
            }
            system.push_str("Recent context (oldest first):");
            for t in &recent {
                system.push('\n');
                system.push_str(&Self::summarize(t));
            }
        }
        Ok(AssembledContext { system, ..Default::default() })
    }

    /// Append the full `Trace` to the Tape as one JSONL line — the lossless factual register (§11).
    async fn record(&self, trace: &Trace) -> Result<()> {
        use std::io::Write;
        let line = serde_json::to_string(trace).map_err(|e| KeelError::Other(format!("tape encode: {e}")))?;
        // poison-tolerant: a panicked holder must not wedge the Tape (recover the guard).
        let _g = self.lock.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(dir) = self.tape.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).map_err(|e| KeelError::Other(format!("tape dir: {e}")))?;
            }
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.tape)
            .map_err(|e| KeelError::Other(format!("tape open: {e}")))?;
        writeln!(f, "{line}").map_err(|e| KeelError::Other(format!("tape write: {e}")))?;
        Ok(())
    }

    /// A well-formed maintenance `Step` the engine would route/generate to compress Ring-2 → Ring-3
    /// (the model-authored narrative register). The generation itself is a later slice; this returns
    /// the seam's `Step` so a Driver/threshold can drive consolidation as just another routed turn.
    async fn consolidate(&self) -> Result<Step> {
        Ok(Step {
            kind: Kind::CoreWire,
            ty: "memory:consolidate".into(),
            trust_required: Trust::Normal,
            data_class: DataClass::Normal,
            tier_history: vec![],
            oracle_failures: 0,
            projected_cost: None,
            critical: false,
            source: Some("memory".into()),
            content: vec![],
            golden_refs: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::{Decision, GenerateResult, Verdict};

    fn base_step() -> Step {
        Step {
            kind: Kind::Scaffolding,
            ty: "user_turn".into(),
            trust_required: Trust::Normal,
            data_class: DataClass::Normal,
            tier_history: vec![],
            oracle_failures: 0,
            projected_cost: None,
            critical: false,
            source: None,
            content: vec![],
            golden_refs: vec![],
        }
    }

    fn trace(user: &str, answer: &str) -> Trace {
        Trace {
            step: Step { content: vec![Content::Text { text: user.into() }], ..base_step() },
            decision: Decision { tier: "local".into(), effort: Default::default(), reason: "t".into() },
            result: GenerateResult { content: answer.into(), ..Default::default() },
            verdict: Verdict { passed: true, ..Default::default() },
        }
    }

    fn temp_tape(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("keel-mem-{}-{}.jsonl", tag, std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[tokio::test]
    async fn record_then_assemble_recalls_recent_turns_across_calls() {
        let tape = temp_tape("recall");
        let mem = FileMemory::new("", &tape, 5);

        // a fresh Tape + empty soul → assemble injects nothing (no false context).
        let a0 = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a0.system.is_empty(), "fresh Tape + no soul → empty system");

        // record a turn (the persistent Tape) …
        mem.record(&trace("capital of France?", "Paris")).await.unwrap();
        // … a NEW FileMemory over the same Tape (mimics a second `keel` process) recalls it.
        let mem2 = FileMemory::new("", &tape, 5);
        let a1 = mem2.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a1.system.contains("Paris"), "working memory persists across calls via the Tape");
        assert!(a1.system.contains("capital of France"));

        let _ = std::fs::remove_file(&tape);
    }

    #[tokio::test]
    async fn soul_is_ring0_and_working_window_is_bounded() {
        let tape = temp_tape("bound");
        let mem = FileMemory::new("You are KEEL.", &tape, 2); // window of 2
        for i in 1..=4 {
            mem.record(&trace(&format!("q{i}"), &format!("a{i}"))).await.unwrap();
        }
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.starts_with("You are KEEL."), "Ring-0 soul leads");
        assert!(a.system.contains("a3") && a.system.contains("a4"), "the 2 most recent are kept");
        assert!(!a.system.contains("a1") && !a.system.contains("a2"), "older turns fall outside the window");
        let _ = std::fs::remove_file(&tape);
    }

    #[tokio::test]
    async fn consolidate_returns_a_maintenance_step() {
        let mem = FileMemory::new("", temp_tape("consol"), 5);
        let s = mem.consolidate().await.unwrap();
        assert_eq!(s.ty, "memory:consolidate");
        assert_eq!(s.source.as_deref(), Some("memory"));
    }
}
