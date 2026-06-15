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
//! - **narrative register (Ring-3, A6.1):** a model-authored, lossy compressed arc in a sibling file of
//!   the Tape ([`narrative`](FileMemory::narrative) / [`set_narrative`](FileMemory::set_narrative)) —
//!   **separate from the factual Tape** (I5: a model may not author its own ground truth). `assemble`
//!   layers it: Ring-0 soul → Ring-3 narrative → Ring-2 recent.
//! - **consolidate (A6.1):** returns a `Step` carrying a real **self-interview / forward-narrative**
//!   prompt (the perpetual-memory proposal) over the prior narrative + recent turns; the model authors
//!   the *result* when it is routed.
//!
//! ## Deferred — designed, not built (for a reviewed pass, against `docs/proposals/perpetual-memory.md`)
//! - **Wiring the consolidation loop (A6.2):** a Driver/threshold emits `consolidate()` → the engine
//!   routes + generates the narrative → L5 stores it via `set_narrative` (the generation needs the model
//!   — a bounded / operator-verified step). Plus the **cold-eyes validation** Step (periodically diff the
//!   narrative against the Tape, I5) and a swappable consolidation policy.
//! - Ring-1 calibration exemplars; Ring-4 retrieval (the embedder organ + `GOLDEN_RECALL`).
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
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// A file-backed [`Memory`] over the append-only **Tape** (canon §11). The Tape is the lossless
/// system of record; the SQLite index (the `Spine`) is the derived, rebuildable checkpoint.
pub struct FileMemory {
    /// Ring-0 soul (identity / system message). **Empty for the bare genome**; a cell sets its persona.
    soul: String,
    /// The Tape file (append-only JSONL of `Trace`s). Created (with parent dirs) on first record.
    tape: PathBuf,
    /// The Ring-3 **narrative register** (a sibling of the Tape): the model-authored, lossy, compressed
    /// arc — the self-curated memory. **Separate from the lossless Tape** because a model may not author
    /// its own ground truth (I5): the narrative gives a successor the *arc + intent*, the Tape the *facts*.
    narrative: PathBuf,
    /// How many recent turns (Ring-2 working memory) `assemble` reads back from the Tape.
    working_turns: usize,
    /// Serializes appends within a process (the Tape is also crash-safe per JSONL line).
    lock: Mutex<()>,
}

impl FileMemory {
    /// Build a Tape-backed memory. `soul` is Ring-0 (empty = no persona, the genome default). The Ring-3
    /// narrative register is a sibling of the Tape (`<tape_stem>.narrative.md`), so distinct Tapes get
    /// distinct narratives (no cross-talk).
    pub fn new(soul: impl Into<String>, tape: impl Into<PathBuf>, working_turns: usize) -> Self {
        let tape = tape.into();
        let narrative = narrative_path_for(&tape);
        Self { soul: soul.into(), tape, narrative, working_turns, lock: Mutex::new(()) }
    }

    /// The Ring-3 narrative register (the model-authored compressed arc), trimmed; `None` when absent or
    /// empty. **Lossy / model-authored** — never the source for a critical fact (I5; the Tape is).
    pub fn narrative(&self) -> Option<String> {
        std::fs::read_to_string(&self.narrative).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// Persist a (re)generated Ring-3 narrative (overwrites). The model authors this during a
    /// consolidation turn; L5 wiring stores the turn's output here. **Not a factual source** (I5).
    pub fn set_narrative(&self, narrative: &str) -> Result<()> {
        let _g = self.lock.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(dir) = self.narrative.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).map_err(|e| KeelError::Other(format!("narrative dir: {e}")))?;
            }
        }
        std::fs::write(&self.narrative, narrative.trim()).map_err(|e| KeelError::Other(format!("narrative write: {e}")))?;
        Ok(())
    }

    /// Build the consolidation prompt (canon §11 / the perpetual-memory proposal): a **self-interview /
    /// handshake + forward-narrative** instruction over the prior narrative + the recent turns. Pure
    /// string assembly (model-free, unit-testable); the model authors the *result* when this Step is
    /// routed. The I5 boundary is stated in the prompt — facts of record live in the Tape, not here.
    fn consolidation_prompt(&self, recent: &[Trace]) -> String {
        let mut p = String::from(
            "You are compressing your working memory before older context is lost, for your successor \
             instance (it will wake with only this narrative + the lossless Tape). Write an updated, \
             concise narrative capturing:\n\
             1. the durable arc - how we got here, cause -> effect (not a flat list);\n\
             2. the current state and what is in flight;\n\
             3. the questions your successor will ask, with their answers.\n\
             Exact facts of record live in the Tape - capture intent + arc, do not restate facts as if \
             you are their source (you may be wrong; the Tape is not).\n",
        );
        if let Some(prior) = self.narrative() {
            p.push_str("\nPrior narrative:\n");
            p.push_str(&prior);
            p.push('\n');
        }
        if recent.is_empty() {
            p.push_str("\n(no new turns since the prior narrative)");
        } else {
            p.push_str("\nNew turns since (oldest first):\n");
            for t in recent {
                p.push_str(&Self::summarize(t));
                p.push('\n');
            }
        }
        p
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

/// The Ring-3 narrative register path for a Tape: `<tape_dir>/<tape_stem>.narrative.md`. Keyed to the
/// Tape's stem so distinct Tapes (incl. per-test temp Tapes) get distinct, co-located narratives.
fn narrative_path_for(tape: &Path) -> PathBuf {
    let stem = tape.file_stem().and_then(|s| s.to_str()).unwrap_or("tape");
    let mut p = tape.to_path_buf();
    p.set_file_name(format!("{stem}.narrative.md"));
    p
}

#[async_trait]
impl Memory for FileMemory {
    /// Ring-0 (soul) + Ring-2 (recent working turns from the Tape), folded into the `system` preamble
    /// the engine prepends. Empty when there is no soul and no history (a fresh first turn) — the
    /// engine then prepends nothing.
    async fn assemble(&self, _step: &Step, _ctx: &Context) -> Result<AssembledContext> {
        let mut system = self.soul.clone(); // Ring-0 (soul / persona — empty for the bare genome)
        // Ring-3: the model-authored narrative arc (lossy; facts of record stay in the Tape, I5).
        if let Some(narrative) = self.narrative() {
            if !system.is_empty() {
                system.push_str("\n\n");
            }
            system.push_str("Narrative so far (your compressed memory; facts of record live in the Tape):\n");
            system.push_str(&narrative);
        }
        // Ring-2: the recent working turns, read back from the Tape (chronological).
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
        // a real self-interview / forward-narrative prompt over the prior narrative + recent turns; the
        // model authors the result when this Step is routed, and L5 stores it via `set_narrative`.
        let recent = self.recent_traces(self.working_turns);
        let prompt = self.consolidation_prompt(&recent);
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
            content: vec![Content::Text { text: prompt }],
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

    // ── A6.1: the Ring-3 narrative register (model-authored arc, separate from the factual Tape) ──

    #[tokio::test]
    async fn narrative_register_injects_into_assemble_separate_from_the_tape() {
        let tape = temp_tape("narr");
        let narr = narrative_path_for(&tape);
        let _ = std::fs::remove_file(&narr);
        let mem = FileMemory::new("", &tape, 5);
        assert!(mem.narrative().is_none(), "no narrative yet -> None");

        // a Tape turn (factual register) + a model-authored narrative (Ring-3, a separate file).
        mem.record(&trace("what's the plan?", "ship A6")).await.unwrap();
        mem.set_narrative("Arc: building KEEL's perpetual memory; A6 in flight.").unwrap();

        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.contains("Narrative so far"), "Ring-3 label present");
        assert!(a.system.contains("A6 in flight"), "the narrative (Ring-3) is injected");
        assert!(a.system.contains("ship A6"), "Ring-2 recent turn still injected from the Tape");
        // the narrative is a SEPARATE file: removing it leaves the factual Tape recall intact.
        std::fs::remove_file(&narr).unwrap();
        let a2 = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(!a2.system.contains("A6 in flight"), "narrative gone");
        assert!(a2.system.contains("ship A6"), "Tape (facts) untouched by losing the narrative");
        let _ = std::fs::remove_file(&tape);
    }

    #[tokio::test]
    async fn consolidate_prompt_carries_prior_narrative_and_recent_turns() {
        let tape = temp_tape("consol-prompt");
        let mem = FileMemory::new("", &tape, 5);
        mem.set_narrative("PRIOR-ARC-TOKEN").unwrap();
        mem.record(&trace("question seventeen", "answer seventeen")).await.unwrap();

        let s = mem.consolidate().await.unwrap();
        let Content::Text { text } = &s.content[0] else { panic!("consolidate Step carries a text prompt") };
        assert!(text.contains("successor"), "self-interview / handoff framing");
        assert!(text.contains("PRIOR-ARC-TOKEN"), "prior narrative folded in");
        assert!(text.contains("question seventeen") && text.contains("answer seventeen"), "recent turns folded in");
        assert!(text.contains("Tape"), "the I5 boundary is stated (facts of record live in the Tape)");
        let _ = std::fs::remove_file(&tape);
        let _ = std::fs::remove_file(narrative_path_for(&tape));
    }

    #[test]
    fn narrative_path_is_a_distinct_sibling_per_tape() {
        let a = narrative_path_for(Path::new("/x/tape.jsonl"));
        assert_eq!(a.file_name().unwrap().to_str().unwrap(), "tape.narrative.md", "stem-keyed sibling");
        assert_ne!(a, narrative_path_for(Path::new("/x/other.jsonl")), "distinct tapes -> distinct narratives");
    }
}
