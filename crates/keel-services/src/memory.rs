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

use crate::recall::{cosine, Embed, Fingerprint};
use async_trait::async_trait;
use keel_contracts::{
    AssembledContext, Content, Context, DataClass, KeelError, Kind, Memory, Message, Result, Role, Step, TokenBudget,
    Trace, Trust,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Chars-per-token proxy for the ring budgets (A7.1): budgets are declared in **tokens** (the frozen
/// `TokenBudget`), enforced in **chars** at ~4 chars/token — no tokenizer dep (the proposal's
/// falsifier: if lived runs overflow the window despite the caps, add a real tokenizer).
const CHARS_PER_TOKEN: usize = 4;

/// Cap on chars sent to the embed organ (~375 tokens at the 4-chars/token proxy) — safely under a
/// MiniLM-class 512-token window (the C2-decided floor embedder), and bounded work on any model.
const EMBED_INPUT_MAX_CHARS: usize = 1500;

/// A ring's char cap from its token budget; `0` tokens = **uncapped** (the explicit opt-out).
fn chars_cap(tokens: u32) -> Option<usize> {
    (tokens > 0).then_some(tokens as usize * CHARS_PER_TOKEN)
}

/// Truncate to at most `max` chars on a char boundary (UTF-8-safe; the budgets count chars, not bytes).
fn take_chars(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}

/// A7.2 — one **episode digest**: the durable mid-resolution register between the rolling Ring-3
/// narrative (topology, overwritten) and the raw Tape (everything, lossless). The REEL §6.2
/// five-field consolidation schema. Episodes are **append-only and never re-compressed** — written
/// once from near-in-time material, so compression-of-compression loss (REEL §10.2) cannot touch
/// them. They cost zero context (never loaded wholesale): they are Ring-4 retrieval targets (A7.3)
/// and the cold-eyes diff substrate (A7.5).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Episode {
    /// Unix seconds when the consolidation ran (L4 reads a clock; the L0 contracts stay clock-free).
    pub at_epoch_s: u64,
    /// How many recent Tape turns this consolidation covered.
    pub span_turns: usize,
    pub what_happened: String,
    pub what_changed: String,
    pub what_matters: String,
    pub unresolved: String,
    /// Search keys for future retrieval (REEL "retrieval anchors").
    pub anchors: Vec<String>,
    /// `false` = a deterministic fallback stub (the model output missed the layout); flagged, never silent.
    pub parsed: bool,
}

impl Episode {
    /// The flat text used as this episode's Ring-4 embedding target (A7.3) and for prompt injection.
    pub fn text(&self) -> String {
        format!(
            "[episode] {} | changed: {} | matters: {} | unresolved: {} | anchors: {}",
            self.what_happened,
            self.what_changed,
            self.what_matters,
            self.unresolved,
            self.anchors.join("; ")
        )
    }
}

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
    /// Ring-4 retrieval (optional, canon §11): the embed organ. `None` = no semantic recall (the bare
    /// genome default — recall is opt-in, like a cell wiring the embedder).
    embedder: Option<Arc<dyn Embed>>,
    /// The Ring-4 vector sidecar (`<tape_stem>.vec.jsonl`): `{text, vec}` per recorded turn. Derived +
    /// rebuildable from the Tape; cleared on an embedder fingerprint mismatch (never serve stale, §11).
    vecs: PathBuf,
    /// The A7.2 episodes register (`<tape_stem>.episodes.jsonl`): append-only five-field digests, one
    /// per consolidation — the durable mid-resolution layer (never overwritten, never re-compressed).
    episodes: PathBuf,
    /// The episode vector sidecar (`<tape_stem>.epvec.jsonl`, A7.3): the coarse tier of the two-tier
    /// Ring-4 index. Episodes stay few, so this tier always scans whole.
    epvecs: PathBuf,
    /// The bounded recent-turn vector-scan window (A7.3): only the newest N turn vectors are scanned
    /// per recall, so brute-force stays O(window + episodes) regardless of Tape size (ISSUE-1 kept;
    /// the latency falsifier re-opens `sqlite-vec`). Older detail stays reachable via episodes.
    recall_window: usize,
    /// Cold-start backfill (A7.3): when the turn sidecar is absent but the Tape has turns, embed the
    /// last N summaries once, lazily, on the next assemble (bounded — never O(Tape)).
    backfill: usize,
    /// Ring-1 calibration exemplars (A7.6): an operator/cell-authored markdown pool
    /// (`<tape_stem>.exemplars.md`, `## `-sectioned). Absent = zero cost (the genome default).
    exemplars_file: PathBuf,
    /// A7.6 (opt-in): inject Ring-2 as REAL `user`/`assistant` conversation messages instead of a
    /// system-preamble block (needs the engine's `AssembledContext.conversation` splice — landed).
    ring2_conversation: bool,
    /// How many semantically-relevant earlier turns Ring-4 injects (0 = off).
    recall_k: usize,
    /// The embedder fingerprint's vector dim (0 = no embedder) — the write-side guard: a vector
    /// from a mismatched embed server (server model ≠ keel.lock, e.g. a stale process still on the
    /// configured port after a C2 default flip) is never stored. Reads are safe by construction
    /// (`cosine` returns 0.0 on a length mismatch and the relevance floor blocks zero-scores).
    fp_dim: usize,
    /// A7.1 — the per-ring context budget (canon §7 "ringed + budgeted"; REEL §4.7). Declared in
    /// tokens (the frozen `TokenBudget`), enforced in chars (~4/token). Ring-0 is **never** trimmed
    /// (identity protection, REEL §3.4) — its field is informational. `0` on a ring = uncapped.
    budget: TokenBudget,
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
        let vecs = vec_path_for(&tape);
        let episodes = episodes_path_for(&tape);
        let epvecs = epvec_path_for(&tape);
        let exemplars_file = exemplars_path_for(&tape);
        Self {
            soul: soul.into(),
            tape,
            narrative,
            working_turns,
            embedder: None,
            vecs,
            episodes,
            epvecs,
            recall_window: 4096,
            backfill: 32,
            recall_k: 0,
            fp_dim: 0,
            budget: Self::default_budget(),
            exemplars_file,
            ring2_conversation: false,
            lock: Mutex::new(()),
        }
    }

    /// A7.6 (opt-in): Ring-2 as real conversation messages. The genome default keeps the system
    /// preamble (proven live); a persona-shaped cell flips this for natural dialogue continuity.
    pub fn with_ring2_as_conversation(mut self) -> Self {
        self.ring2_conversation = true;
        self
    }

    /// Where a cell/operator authors the Ring-1 exemplar pool (`## `-sectioned markdown; the FIRST
    /// section is the calibration anchor and always loads; sections are never auto-deleted — REEL
    /// §3.4 identity protection is editorial, not automated).
    pub fn exemplars_path(&self) -> PathBuf {
        self.exemplars_file.clone()
    }

    /// Read the exemplar pool as sections (split on `## ` headers; text before the first header is
    /// a comment area and ignored). Absent/empty file → no sections (zero context cost).
    fn exemplar_sections(&self) -> Vec<String> {
        let Ok(raw) = std::fs::read_to_string(&self.exemplars_file) else { return Vec::new() };
        let mut sections: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut started = false;
        for line in raw.lines() {
            if line.starts_with("## ") {
                if started && !cur.trim().is_empty() {
                    sections.push(cur.trim().to_string());
                }
                cur = String::from(line);
                cur.push('\n');
                started = true;
            } else if started {
                cur.push_str(line);
                cur.push('\n');
            }
        }
        if started && !cur.trim().is_empty() {
            sections.push(cur.trim().to_string());
        }
        sections
    }

    /// Tune the Ring-4 scan bound + cold-start backfill (A7.3; keel.lock `memory.recall_window` /
    /// `memory.backfill`). `recall_window` bounds the turn-vector scan; episodes always scan whole.
    pub fn with_recall_tuning(mut self, recall_window: usize, backfill: usize) -> Self {
        self.recall_window = recall_window;
        self.backfill = backfill;
        self
    }

    /// The genome-default ring budget (A7.1): conservative caps that keep `assemble` O(1) regardless
    /// of Tape size. Ring-0 = 0 (never trimmed — identity protection); ring-1 exemplars 1000 tok ·
    /// ring-2 working 2000 · ring-3 narrative 1000 · ring-4 recall 1000. A cell (or keel.lock
    /// `memory.budget`) overrides via [`with_budget`](FileMemory::with_budget).
    pub fn default_budget() -> TokenBudget {
        TokenBudget { ring1: 1000, ring2: 2000, ring3: 1000, ring4: 1000, ..Default::default() }
    }

    /// Override the per-ring context budget (tokens; `0` = uncapped). Ring-0 is never trimmed
    /// regardless — the soul loads verbatim (REEL §3.4: identity is constitutional).
    pub fn with_budget(mut self, budget: TokenBudget) -> Self {
        self.budget = budget;
        self
    }

    /// Wire **Ring-4 semantic recall** (canon §11): an embed organ + how many relevant earlier turns to
    /// inject. The `fp` commits the vector sidecar's format; on a fingerprint mismatch the stale sidecar
    /// is cleared (never serve a mismatched index — `GOLDEN_RECALL`), to be rebuilt from the Tape as new
    /// turns are recorded. Recall is opt-in: without this, memory is Ring-0/2/3 only (no embed dependency).
    pub fn with_embedder(mut self, embedder: Arc<dyn Embed>, fp: Fingerprint, recall_k: usize) -> Self {
        // fingerprint guard: a sidecar from a different embedder is meaningless → clear BOTH tiers
        // (turn + episode vectors — don't serve stale; rebuild-from-ledger, GOLDEN_RECALL).
        let fp_file = self.vecs.with_extension("fp");
        let prev = std::fs::read_to_string(&fp_file).ok().map(|s| s.trim().to_string());
        let now = format!("{}:{}", fp.embedder, fp.dim);
        if prev.as_deref() != Some(now.as_str()) {
            let _ = std::fs::remove_file(&self.vecs); // rebuild-from-ledger as turns re-accumulate
            let _ = std::fs::remove_file(&self.epvecs); // episodes re-embed on backfill
            if let Some(d) = fp_file.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            let _ = std::fs::write(&fp_file, &now);
        }
        self.embedder = Some(embedder);
        self.recall_k = recall_k;
        self.fp_dim = fp.dim;
        self
    }

    /// The slice of `text` actually sent to the embed organ — head-capped so a long turn still
    /// embeds under a MiniLM-class 512-token window (the stored/injected text stays full).
    fn embed_input(text: &str) -> &str {
        take_chars(text, EMBED_INPUT_MAX_CHARS)
    }

    /// Append a `{text, vec}` line to a vector sidecar (best-effort; recall is non-critical, so a
    /// sidecar write never fails a turn — the lossless Tape is the system of record). A vector
    /// whose dim doesn't match the fingerprint is **dropped loudly** — it came from a mismatched
    /// embed server (server ≠ keel.lock), and storing it would fossilize wrong-model neighbors.
    fn append_vec_line(&self, path: &Path, text: &str, vec: &[f32]) {
        if self.fp_dim != 0 && vec.len() != self.fp_dim {
            eprintln!(
                "[keel] ring-4: dropped a {}-dim vector (fingerprint expects {}) - a stale embed server may be running on the configured port",
                vec.len(),
                self.fp_dim
            );
            return;
        }
        let line = serde_json::json!({ "text": text, "vec": vec }).to_string();
        let _g = self.lock.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            use std::io::Write;
            let _ = writeln!(f, "{line}");
        }
    }

    /// Append a turn embedding to the fine tier.
    fn append_vec(&self, text: &str, vec: &[f32]) {
        self.append_vec_line(&self.vecs.clone(), text, vec);
    }

    /// Read the newest `tail` `(text, vec)` pairs from a vector sidecar (`usize::MAX` = whole file;
    /// best-effort; a missing/garbled line is skipped). Order is irrelevant to cosine ranking.
    fn read_vec_lines(&self, path: &Path, tail: usize) -> Vec<(String, Vec<f32>)> {
        let Ok(raw) = std::fs::read_to_string(path) else { return Vec::new() };
        raw.lines()
            .rev()
            .take(tail)
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .filter_map(|v| {
                let text = v["text"].as_str()?.to_string();
                let vec: Vec<f32> = v["vec"].as_array()?.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                Some((text, vec))
            })
            .collect()
    }

    /// The two-tier Ring-4 scan set (A7.3): ALL episode vectors (coarse, few) + the newest
    /// `recall_window` turn vectors (fine, bounded) — O(window + episodes) regardless of Tape size.
    fn recall_candidates(&self) -> Vec<(String, Vec<f32>)> {
        let mut all = self.read_vec_lines(&self.epvecs.clone(), usize::MAX);
        all.extend(self.read_vec_lines(&self.vecs.clone(), self.recall_window));
        all
    }

    /// Cold-start backfill (A7.3): if a tier's sidecar is absent but its source register has content,
    /// embed a bounded slice once (turns: the newest `backfill`; episodes: all — they stay few).
    /// Lazy + idempotent-by-existence; an embed failure just retries on a later assemble.
    async fn backfill_vectors(&self) {
        let Some(embedder) = &self.embedder else { return };
        if !self.vecs.exists() {
            let seed = self.recent_traces(self.backfill);
            for t in &seed {
                let s = Self::summarize(t);
                if let Ok(v) = embedder.embed_text(Self::embed_input(&s)).await {
                    self.append_vec(&s, &v);
                }
            }
        }
        if !self.epvecs.exists() {
            for ep in self.read_episodes(usize::MAX) {
                let s = ep.text();
                if let Ok(v) = embedder.embed_text(Self::embed_input(&s)).await {
                    self.append_vec_line(&self.epvecs.clone(), &s, &v);
                }
            }
        }
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

    // ── A7.2: the episodes register (append-only mid-resolution digests) ──

    /// Append one episode to the register (append-only JSONL — never overwritten, unlike the
    /// narrative). Best-effort dir creation; a write failure is an honest error (the register is a
    /// durable layer, not a cache).
    pub fn append_episode(&self, ep: &Episode) -> Result<()> {
        use std::io::Write;
        let line = serde_json::to_string(ep).map_err(|e| KeelError::Other(format!("episode encode: {e}")))?;
        let _g = self.lock.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(dir) = self.episodes.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).map_err(|e| KeelError::Other(format!("episodes dir: {e}")))?;
            }
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.episodes)
            .map_err(|e| KeelError::Other(format!("episodes open: {e}")))?;
        writeln!(f, "{line}").map_err(|e| KeelError::Other(format!("episodes write: {e}")))?;
        Ok(())
    }

    /// Read the last `n` episodes in chronological order (best-effort; a garbled line is skipped —
    /// the register outlives schema, like the Tape).
    pub fn read_episodes(&self, n: usize) -> Vec<Episode> {
        let Ok(raw) = std::fs::read_to_string(&self.episodes) else { return Vec::new() };
        let mut eps: Vec<Episode> = raw.lines().rev().filter_map(|l| serde_json::from_str::<Episode>(l).ok()).take(n).collect();
        eps.reverse();
        eps
    }

    /// How many episodes the register holds (A7.4 policy input; a garbled line still counts a line).
    pub fn episodes_count(&self) -> usize {
        std::fs::read_to_string(&self.episodes).map(|s| s.lines().filter(|l| !l.trim().is_empty()).count()).unwrap_or(0)
    }

    // ── A7.4: the observable stats the maintenance policy decides over ──

    /// Total turns on the Tape (the policy's progress axis; a garbled line still counts a line).
    pub fn tape_turns(&self) -> usize {
        std::fs::read_to_string(&self.tape).map(|s| s.lines().filter(|l| !l.trim().is_empty()).count()).unwrap_or(0)
    }

    /// Ring-2 context pressure: does the working window currently exceed its char budget? (One of
    /// the A7.4 consolidation triggers — REEL §6.2 "context pressure".)
    pub fn ring2_over_budget(&self) -> bool {
        let Some(cap) = chars_cap(self.budget.ring2) else { return false };
        let total: usize =
            self.recent_traces(self.working_turns).iter().map(|t| Self::summarize(t).chars().count() + 1).sum();
        total > cap
    }

    /// A narrative exists to validate (cold-eyes is meaningless before the first consolidation).
    pub fn has_narrative(&self) -> bool {
        self.narrative().is_some()
    }

    /// The autopilot's durable cursor sidecar (`<tape_stem>.maint.json`, A7.4).
    pub fn maint_state_path(&self) -> PathBuf {
        let stem = self.tape.file_stem().and_then(|s| s.to_str()).unwrap_or("tape");
        let mut p = self.tape.clone();
        p.set_file_name(format!("{stem}.maint.json"));
        p
    }

    /// Parse a consolidation turn's model output into `(narrative, episode-fields)` per the layout the
    /// prompt requests (`=== NARRATIVE === / === EPISODE ===` with `key: value` lines). Tolerant:
    /// missing markers → the whole output is the narrative and the episode is `None` (the caller
    /// falls back to a deterministic stub — flagged `parsed: false`, never silent).
    pub fn parse_consolidation(output: &str) -> (String, Option<Episode>) {
        const N_MARK: &str = "=== NARRATIVE ===";
        const E_MARK: &str = "=== EPISODE ===";
        let output = crate::maintenance::strip_think(output); // never store a reasoning envelope
        let (Some(ni), Some(ei)) = (output.find(N_MARK), output.find(E_MARK)) else {
            return (output.trim().to_string(), None);
        };
        if ei < ni {
            return (output.trim().to_string(), None);
        }
        let narrative = output[ni + N_MARK.len()..ei].trim().to_string();
        let (mut happened, mut changed, mut matters, mut unresolved) =
            (String::new(), String::new(), String::new(), String::new());
        let mut anchors: Vec<String> = Vec::new();
        for line in output[ei + E_MARK.len()..].lines() {
            let line = line.trim();
            let Some((key, val)) = line.split_once(':') else { continue };
            let val = val.trim();
            match key.trim().to_lowercase().as_str() {
                "happened" => happened = val.to_string(),
                "changed" => changed = val.to_string(),
                "matters" => matters = val.to_string(),
                "unresolved" => unresolved = val.to_string(),
                "anchors" => anchors = val.split(';').map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect(),
                _ => {}
            }
        }
        if happened.is_empty() && changed.is_empty() && matters.is_empty() {
            return (if narrative.is_empty() { output.trim().to_string() } else { narrative }, None);
        }
        let ep = Episode {
            at_epoch_s: now_epoch_s(),
            span_turns: 0, // the caller stamps the span
            what_happened: happened,
            what_changed: changed,
            what_matters: matters,
            unresolved,
            anchors,
            parsed: true,
        };
        (narrative, Some(ep))
    }

    /// The deterministic fallback episode when the model output missed the layout: built model-free
    /// from the consolidated turns' user texts (flagged `parsed: false` — visible, never silent).
    fn fallback_episode(recent: &[Trace]) -> Episode {
        let users: Vec<String> = recent
            .iter()
            .filter_map(|t| {
                t.step.content.iter().find_map(|c| match c {
                    Content::Text { text } => Some(text.trim().to_string()),
                    _ => None,
                })
            })
            .filter(|s| !s.is_empty())
            .collect();
        let anchors: Vec<String> =
            users.iter().rev().take(4).map(|u| u.split_whitespace().take(5).collect::<Vec<_>>().join(" ")).collect();
        Episode {
            at_epoch_s: now_epoch_s(),
            span_turns: recent.len(),
            what_happened: format!("(unparsed consolidation) covered {} turn(s)", recent.len()),
            what_changed: String::new(),
            what_matters: String::new(),
            unresolved: String::new(),
            anchors,
            parsed: false,
        }
    }

    /// Store a consolidation turn's output into BOTH registers (A7.2): the rolling Ring-3 narrative
    /// (overwritten — the topology) and one appended episode (durable — the mid-resolution layer).
    /// Returns `(narrative_chars, episode_parsed)`; an empty output stores nothing (`(0, false)`).
    pub async fn store_consolidation(&self, output: &str) -> Result<(usize, bool)> {
        let out = output.trim();
        if out.is_empty() {
            return Ok((0, false));
        }
        let recent = self.recent_traces(self.working_turns);
        let (narrative, parsed_ep) = Self::parse_consolidation(out);
        // placeholder guard (defense in depth behind the prompt fix): a model that parroted the
        // layout hint produced no memory — store nothing rather than fossilize junk in Ring-3.
        let n = narrative.trim();
        if n.is_empty() || ((n.starts_with('<') || n.starts_with('(')) && (n.ends_with('>') || n.ends_with(')')) && !n.contains('\n') && n.chars().count() < 64) {
            return Ok((0, false));
        }
        let parsed = parsed_ep.is_some();
        let mut ep = parsed_ep.unwrap_or_else(|| Self::fallback_episode(&recent));
        ep.span_turns = recent.len();
        self.set_narrative(&narrative)?;
        self.append_episode(&ep)?;
        // A7.3: the episode is a Ring-4 retrieval target — embed it into the episode vector sidecar.
        self.embed_episode(&ep).await;
        Ok((narrative.chars().count(), parsed))
    }

    /// Embed an episode into the coarse Ring-4 tier (best-effort — a no-op without an embedder, and
    /// an embed failure never fails the consolidation; the episode register itself is already durable).
    async fn embed_episode(&self, ep: &Episode) {
        if let Some(embedder) = &self.embedder {
            let s = ep.text();
            if let Ok(v) = embedder.embed_text(Self::embed_input(&s)).await {
                self.append_vec_line(&self.epvecs.clone(), &s, &v);
            }
        }
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
        // A7.1: budget-aware generation — the narrative is loaded under a fixed ring budget, so ask
        // the model to write within it (assembly hard-trims regardless; this keeps the trim rare).
        if let Some(c) = chars_cap(self.budget.ring3) {
            p.push_str(&format!("\nKeep the narrative under about {c} characters - it is loaded within a fixed budget.\n"));
        }
        // A7.2: one generation, two artifacts — the rolling narrative (topology, overwritten) AND an
        // append-only episode digest (the REEL five-field schema; the durable mid-resolution layer).
        // The format contract comes LAST (recency anchoring — a small model finishes reading here
        // and then writes) with parenthesized never-copy hints; both anti-parrot measures are lived
        // lessons (2026-07-09).
        p.push_str(
            "\nReply with BOTH sections in exactly this layout, keeping the two marker lines. Base \
             every line on the turns above; replace every parenthesized hint with real content and \
             never copy a hint verbatim:\n\
             === NARRATIVE ===\n\
             (the updated narrative)\n\
             === EPISODE ===\n\
             happened: (one line - the factual core of the new turns)\n\
             changed: (one line - what shifted)\n\
             matters: (one line - why this deserves remembering)\n\
             unresolved: (one line - open loops carried forward)\n\
             anchors: (3-6 short search phrases, semicolon-separated)\n",
        );
        p
    }

    /// The **cold-eyes validation** prompt (canon §10.2 / the perpetual-memory proposal): a FRESH,
    /// uninvested pass that diffs the model-authored narrative against the lossless Tape (the ground
    /// truth) — catching the self-curation blind spot (the invested model over-keeps what it found
    /// salient) and satisfying I5 (the Tape validates the narrative, never the reverse). `None` when
    /// there is no narrative to validate. The reviewer replies `CONSISTENT` or lists unsupported claims.
    pub fn cold_eyes_prompt(&self) -> Option<String> {
        let narrative = self.narrative()?;
        let recent = self.recent_traces(self.working_turns);
        // A7.5 lesson (lived): the narrative legitimately carries history OLDER than the recent Tape
        // window — the episodes register is the durable digest of exactly that history, so it joins
        // the reviewer's ground truth (single-hop, near-in-time digests — the safest model-authored
        // layer). Without it, every legitimate old-arc claim reads as drift and the damper alarms forever.
        // Precision over recall, deliberately: cold-eyes is a drift DAMPER, not a factual oracle —
        // critical facts never come from the narrative (I5; the Tape is the register for those), so
        // a false alarm costs trust while a missed nitpick costs nothing. Flag only material fabrication.
        let mut p = String::from(
            "You are a fresh, uninvested reviewer. Below is a NARRATIVE (model-authored, possibly wrong), \
             the EPISODES (durable digests of older history), and the TAPE (the lossless record of the \
             most recent turns - the ground truth). Flag ONLY materially false or invented claims - \
             facts, names, numbers, or events that appear in the narrative but in NEITHER the Tape nor \
             the episodes, or that CONTRADICT them. Differences of wording, chronology, ordering, or \
             emphasis are NOT drift. List each materially false claim on its own line; if none, reply \
             with exactly the word CONSISTENT.\n\nNARRATIVE:\n",
        );
        p.push_str(&narrative);
        let eps = self.read_episodes(8);
        if !eps.is_empty() {
            p.push_str("\n\nEPISODES (older history, oldest first):\n");
            for e in &eps {
                p.push_str(&e.text());
                p.push('\n');
            }
        }
        p.push_str("\nTAPE (the most recent turns, oldest first):\n");
        for t in &recent {
            p.push_str(&Self::summarize(t));
            p.push('\n');
        }
        Some(p)
    }

    /// A7.5 — the **regenerate-from-ground-truth** prompt: a corrective consolidation after cold-eyes
    /// flagged drift. The REEL §10.2 fix — the narrative is rebuilt from the SOURCE registers (the
    /// Tape + the episodes), and the drifted copy is deliberately NOT included (never re-anchor on
    /// drifted text). Pure string assembly (model-free, unit-testable).
    pub fn corrective_consolidation_prompt(&self, findings: &[String]) -> String {
        let mut p = String::from(
            "A fresh reviewer diffed your memory narrative against the lossless Tape (the ground \
             truth) and found claims the Tape does NOT support:\n",
        );
        for f in findings {
            p.push_str("- ");
            p.push_str(f);
            p.push('\n');
        }
        p.push_str(
            "\nRegenerate the narrative FROM THE GROUND TRUTH below. Requirements:\n\
             - Do NOT mention, quote, refute, or allude to the unsupported claims - write the \
             narrative as if they never existed.\n\
             - Keep only what the Tape and episodes support; do not restate facts as if you are \
             their source (the Tape is).\n\
             - Reply with ONLY the narrative text - no headings, no analysis, no commentary.\n",
        );
        if let Some(c) = chars_cap(self.budget.ring3) {
            p.push_str(&format!("- Keep it under about {c} characters.\n"));
        }
        let eps = self.read_episodes(5);
        if !eps.is_empty() {
            p.push_str("\nEpisodes (durable digests, oldest first):\n");
            for e in &eps {
                p.push_str(&e.text());
                p.push('\n');
            }
        }
        p.push_str("\nTAPE (the facts, oldest first):\n");
        for t in &self.recent_traces(self.working_turns) {
            p.push_str(&Self::summarize(t));
            p.push('\n');
        }
        p
    }

    /// Store a corrective turn's output as the Ring-3 narrative (A7.5): narrative ONLY — a correction
    /// appends **no episode** (nothing new happened; the mid-resolution history stays honest). A model
    /// that still emits the two-section layout is tolerated (only the narrative part is kept).
    pub fn store_corrected_narrative(&self, output: &str) -> Result<usize> {
        let (narrative, _) = Self::parse_consolidation(output.trim());
        let n = narrative.chars().count();
        // degenerate guard (same rationale as store_consolidation): a stub/placeholder correction
        // must never CLOBBER the narrative — leaving the drifted copy (still alarmed on the next
        // cadence) beats replacing memory with nothing.
        if n < 40 {
            return Ok(0);
        }
        self.set_narrative(&narrative)?;
        Ok(n)
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

/// The Ring-4 vector sidecar path for a Tape: `<tape_dir>/<tape_stem>.vec.jsonl` (the fingerprint lives
/// alongside as `.vec.fp`). Keyed to the Tape stem so distinct Tapes get distinct, co-located sidecars.
fn vec_path_for(tape: &Path) -> PathBuf {
    let stem = tape.file_stem().and_then(|s| s.to_str()).unwrap_or("tape");
    let mut p = tape.to_path_buf();
    p.set_file_name(format!("{stem}.vec.jsonl"));
    p
}

/// Unix seconds now (L4 reads a clock — the L0 contracts stay clock-free, the same discipline as
/// `svc::driver`). `0` if the system clock is before the epoch (never a panic).
fn now_epoch_s() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// The A7.2 episodes-register path for a Tape: `<tape_dir>/<tape_stem>.episodes.jsonl` — append-only,
/// never overwritten (unlike the rolling narrative). Derived-from-consolidation, but durable: the
/// mid-resolution history layer.
fn episodes_path_for(tape: &Path) -> PathBuf {
    let stem = tape.file_stem().and_then(|s| s.to_str()).unwrap_or("tape");
    let mut p = tape.to_path_buf();
    p.set_file_name(format!("{stem}.episodes.jsonl"));
    p
}

/// The coarse Ring-4 tier path (A7.3): `<tape_dir>/<tape_stem>.epvec.jsonl` — episode embeddings.
/// Derived + rebuildable (backfilled from the episodes register); cleared on fingerprint mismatch.
fn epvec_path_for(tape: &Path) -> PathBuf {
    let stem = tape.file_stem().and_then(|s| s.to_str()).unwrap_or("tape");
    let mut p = tape.to_path_buf();
    p.set_file_name(format!("{stem}.epvec.jsonl"));
    p
}

/// The Ring-1 exemplar-pool path (A7.6): `<tape_dir>/<tape_stem>.exemplars.md` — operator/cell-authored.
fn exemplars_path_for(tape: &Path) -> PathBuf {
    let stem = tape.file_stem().and_then(|s| s.to_str()).unwrap_or("tape");
    let mut p = tape.to_path_buf();
    p.set_file_name(format!("{stem}.exemplars.md"));
    p
}

#[async_trait]
impl Memory for FileMemory {
    /// Ring-0 (soul) + Ring-2 (recent working turns from the Tape), folded into the `system` preamble
    /// the engine prepends. Empty when there is no soul and no history (a fresh first turn) — the
    /// engine then prepends nothing.
    async fn assemble(&self, step: &Step, _ctx: &Context) -> Result<AssembledContext> {
        let mut system = self.soul.clone(); // Ring-0 (soul / persona) — NEVER trimmed (REEL §3.4)
        // Ring-1 (A7.6): calibration exemplars — the anchor (first section) always loads, then the
        // most recently authored others, the whole block under the ring-1 budget (REEL §4.2; the
        // pool itself is operator-authored and never auto-deleted — identity protection is editorial).
        let sections = self.exemplar_sections();
        if !sections.is_empty() {
            let mut picked: Vec<&String> = vec![&sections[0]];
            let mut rest: Vec<&String> = sections.iter().skip(1).rev().take(2).collect();
            rest.reverse();
            picked.extend(rest);
            let mut block = picked.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n\n");
            if let Some(c) = chars_cap(self.budget.ring1) {
                if block.chars().count() > c {
                    block = format!("{}\n...[exemplars trimmed to budget]", take_chars(&block, c));
                }
            }
            if !system.is_empty() {
                system.push_str("\n\n");
            }
            system.push_str("Calibration exemplars (how you engage):\n");
            system.push_str(&block);
        }
        // Ring-3: the model-authored narrative arc (lossy; facts of record stay in the Tape, I5),
        // hard-trimmed to its budget (A7.1) — head kept (the durable arc leads), marker on a cut.
        if let Some(narrative) = self.narrative() {
            let shown = match chars_cap(self.budget.ring3) {
                Some(c) if narrative.chars().count() > c => {
                    format!("{}\n...[narrative trimmed to budget]", take_chars(&narrative, c))
                }
                _ => narrative,
            };
            if !system.is_empty() {
                system.push_str("\n\n");
            }
            system.push_str("Narrative so far (your compressed memory; facts of record live in the Tape):\n");
            system.push_str(&shown);
        }
        // Ring-2 selection (A7.1): newest-first accumulation under the ring-2 char budget (then back
        // to chronological order). The newest turn always survives — trimmed to the cap if it alone
        // exceeds it — so working memory never goes fully blind under pressure.
        let recent = self.recent_traces(self.working_turns);
        let (ring2_traces, ring2): (Vec<usize>, Vec<String>) = {
            let cap = chars_cap(self.budget.ring2);
            let mut idxs: Vec<usize> = Vec::new();
            let mut texts: Vec<String> = Vec::new();
            let mut used = 0usize;
            for (i, t) in recent.iter().enumerate().rev() {
                let entry = Self::summarize(t);
                let len = entry.chars().count() + 1;
                match cap {
                    Some(c) if texts.is_empty() && len > c => {
                        idxs.push(i);
                        texts.push(format!("{}...", take_chars(&entry, c.saturating_sub(3))));
                        break;
                    }
                    Some(c) if used + len > c => break,
                    _ => {}
                }
                used += len;
                idxs.push(i);
                texts.push(entry);
            }
            idxs.reverse();
            texts.reverse();
            (idxs, texts)
        };
        // Ring-4: semantically-relevant EARLIER turns (canon §11) — embed the current query, cosine-rank
        // the vector sidecar, inject the top-k under the ring-4 budget (excluding what Ring-2 already
        // carries). Opt-in (needs an embedder).
        if self.recall_k > 0 {
            if let Some(embedder) = &self.embedder {
                // A7.3: lazy cold-start backfill — an existing Tape/episode register gains its
                // vector sidecars on first budgeted assemble (bounded, idempotent-by-existence).
                self.backfill_vectors().await;
                if let Some(query) = step.content.iter().find_map(|c| match c {
                    Content::Text { text } if !text.trim().is_empty() => Some(text.trim()),
                    _ => None,
                }) {
                    if let Ok(qv) = embedder.embed_text(Self::embed_input(query)).await {
                        let recent_set: std::collections::HashSet<&String> = ring2.iter().collect();
                        let mut scored: Vec<(f32, String)> = self
                            .recall_candidates()
                            .into_iter()
                            .filter(|(t, _)| !recent_set.contains(t))
                            .map(|(t, v)| (cosine(&qv, &v), t))
                            .collect();
                        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                        let cap = chars_cap(self.budget.ring4);
                        let mut used = 0usize;
                        let mut picks: Vec<String> = Vec::new();
                        for (s, t) in scored.into_iter() {
                            // relevance floor: an orthogonal/degenerate match (cos <= 0) never
                            // injects — k bounds how many, it never forces junk in.
                            if picks.len() >= self.recall_k || s <= 0.0 {
                                break;
                            }
                            let len = t.chars().count() + 1;
                            if let Some(c) = cap {
                                if used + len > c {
                                    break;
                                }
                            }
                            used += len;
                            picks.push(t);
                        }
                        if !picks.is_empty() {
                            if !system.is_empty() {
                                system.push_str("\n\n");
                            }
                            system.push_str("Relevant earlier (semantic recall):");
                            for p in &picks {
                                system.push('\n');
                                system.push_str(p);
                            }
                        }
                    }
                }
            }
        }
        // Ring-2: the recent working turns (chronological, budget-selected) — a system-preamble
        // block by default, or REAL conversation messages when the cell opts in (A7.6; the engine
        // splices `AssembledContext.conversation` before the live turn).
        let mut conversation: Vec<Message> = Vec::new();
        if self.ring2_conversation {
            let cap = chars_cap(self.budget.ring2).unwrap_or(usize::MAX);
            for &i in &ring2_traces {
                let t = &recent[i];
                let user = t
                    .step
                    .content
                    .iter()
                    .find_map(|c| match c {
                        Content::Text { text } => Some(text.trim()),
                        _ => None,
                    })
                    .unwrap_or("");
                let msg = |role: Role, text: &str| Message {
                    role,
                    content: vec![Content::Text { text: take_chars(text, cap).to_string() }],
                    name: None,
                    reasoning_content: None,
                    tool_call_id: None,
                };
                conversation.push(msg(Role::User, user));
                conversation.push(msg(Role::Assistant, t.result.content.trim()));
            }
        } else if !ring2.is_empty() {
            if !system.is_empty() {
                system.push_str("\n\n");
            }
            system.push_str("Recent context (oldest first):");
            for e in &ring2 {
                system.push('\n');
                system.push_str(e);
            }
        }
        Ok(AssembledContext { system, conversation, budget: self.budget.clone(), ..Default::default() })
    }

    /// Append the full `Trace` to the Tape as one JSONL line — the lossless factual register (§11).
    async fn record(&self, trace: &Trace) -> Result<()> {
        use std::io::Write;
        // M2 (audit): memory-MAINTENANCE turns (consolidate, cold-eyes — source "memory" / ty "memory:*")
        // are model-authored and must NOT land on the lossless FACTUAL Tape: recording them would confuse
        // the registers (a model-authored summary in the factual record — canon §22.8 / I5) and create a
        // self-ingest loop (the next consolidation re-reads the prior one). Their output lives only in the
        // Ring-3 narrative register (via `set_narrative`), never the Tape.
        if trace.step.source.as_deref() == Some("memory") || trace.step.ty.starts_with("memory:") {
            return Ok(());
        }
        let line = serde_json::to_string(trace).map_err(|e| KeelError::Other(format!("tape encode: {e}")))?;
        {
            // poison-tolerant; the guard is scoped to this block so no lock is held across an await.
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
        }
        // Ring-4: embed the turn's summary into the vector sidecar (best-effort — recall is non-critical,
        // so an embed failure never fails the turn; the lossless Tape already holds the record).
        if let Some(embedder) = &self.embedder {
            let summary = Self::summarize(trace);
            if let Ok(vec) = embedder.embed_text(Self::embed_input(&summary)).await {
                self.append_vec(&summary, &vec);
            }
        }
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
            // sovereign: consolidating personal turn-history stays on-box (forces local, I3) — a
            // memory narrative may carry private context; privacy beats a stronger cloud summarizer.
            data_class: DataClass::Sovereign,
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

    #[tokio::test]
    async fn memory_maintenance_turns_are_not_recorded_to_the_tape() {
        // M2 (audit): a consolidate/cold-eyes turn (source "memory") must NOT land on the factual Tape
        // (no register confusion, no self-ingest loop) — only normal turns do.
        let tape = temp_tape("maint");
        let mem = FileMemory::new("", &tape, 5);
        let mut maint = trace("consolidation prompt", "MODEL-AUTHORED-NARRATIVE");
        maint.step.source = Some("memory".into());
        maint.step.ty = "memory:consolidate".into();
        mem.record(&maint).await.unwrap();
        mem.record(&trace("a real question", "a real answer")).await.unwrap();
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.contains("a real answer"), "the normal turn is on the Tape");
        assert!(!a.system.contains("MODEL-AUTHORED-NARRATIVE"), "the maintenance turn is NOT on the factual Tape");
        let _ = std::fs::remove_file(&tape);
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

    #[tokio::test]
    async fn cold_eyes_prompt_diffs_narrative_against_the_tape() {
        let tape = temp_tape("coldeyes");
        let _ = std::fs::remove_file(narrative_path_for(&tape));
        let mem = FileMemory::new("", &tape, 5);
        assert!(mem.cold_eyes_prompt().is_none(), "no narrative -> nothing to validate");
        mem.record(&trace("what is my number", "42")).await.unwrap();
        mem.set_narrative("The user's number is 42 and the sky is green.").unwrap();
        let p = mem.cold_eyes_prompt().expect("a narrative exists -> a validation prompt");
        assert!(p.contains("ground truth"), "the Tape-is-ground-truth framing");
        assert!(p.contains("CONSISTENT"), "the consistent-reply token");
        assert!(p.contains("sky is green"), "the narrative is included for review");
        assert!(p.contains("42"), "the Tape facts are included");
        // A7.5 lesson: episodes join the reviewer's ground truth (older history the window can't show).
        mem.append_episode(&Episode {
            at_epoch_s: 1,
            span_turns: 1,
            what_happened: "older-history digest".into(),
            what_changed: String::new(),
            what_matters: String::new(),
            unresolved: String::new(),
            anchors: vec![],
            parsed: true,
        })
        .unwrap();
        let p2 = mem.cold_eyes_prompt().unwrap();
        assert!(p2.contains("older-history digest"), "episodes ride the cold-eyes ground truth");
        assert!(p2.contains("NEITHER"), "the neither-register-supports framing");
        let _ = std::fs::remove_file(&tape);
        let _ = std::fs::remove_file(narrative_path_for(&tape));
    }

    #[test]
    fn narrative_path_is_a_distinct_sibling_per_tape() {
        let a = narrative_path_for(Path::new("/x/tape.jsonl"));
        assert_eq!(a.file_name().unwrap().to_str().unwrap(), "tape.narrative.md", "stem-keyed sibling");
        assert_ne!(a, narrative_path_for(Path::new("/x/other.jsonl")), "distinct tapes -> distinct narratives");
    }

    // ── A3 Ring-4: semantic recall wired into assemble (stub embedder; model-free) ──

    struct StubEmbed;
    #[async_trait]
    impl Embed for StubEmbed {
        async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
            // a toy 3-d embedding keyed on a topic word → deterministic recall in tests.
            let t = text.to_lowercase();
            let v = if t.contains("france") || t.contains("paris") {
                vec![1.0, 0.0, 0.0]
            } else if t.contains("2+2") || t.contains("math") {
                vec![0.0, 1.0, 0.0]
            } else {
                vec![0.0, 0.0, 1.0]
            };
            Ok(v)
        }
    }

    /// A stub that emulates a STALE embed server: it returns vectors of the wrong dimension
    /// (server model ≠ the keel.lock fingerprint — the C2-flip sharp edge).
    struct WrongDimEmbed;
    #[async_trait]
    impl Embed for WrongDimEmbed {
        async fn embed_text(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.5; 8])
        }
    }

    #[tokio::test]
    async fn mismatched_dim_vectors_are_never_stored() {
        let tape = temp_tape("dimguard");
        clean_ring4(&tape);
        // fingerprint says 3-dim; the "server" returns 8-dim → the write guard drops every vector.
        let mem = FileMemory::new("", &tape, 2).with_embedder(Arc::new(WrongDimEmbed), Fingerprint::new("stub", 3), 1);
        mem.record(&trace("the capital of France?", "Paris")).await.unwrap();
        let sidecar = vec_path_for(&tape);
        assert!(
            !sidecar.exists() || std::fs::read_to_string(&sidecar).unwrap().trim().is_empty(),
            "a wrong-dim vector must never land in the sidecar"
        );
        // the same record through a matching-dim embedder stores fine (the guard is dim-keyed, not off).
        let mem_ok = FileMemory::new("", &tape, 2).with_embedder(Arc::new(StubEmbed), Fingerprint::new("stub", 3), 1);
        mem_ok.record(&trace("the capital of France?", "Paris")).await.unwrap();
        assert!(!std::fs::read_to_string(vec_path_for(&tape)).unwrap().trim().is_empty());
        let _ = std::fs::remove_file(&tape);
        clean_ring4(&tape);
    }

    #[test]
    fn embed_input_is_head_capped_for_small_window_models() {
        let short = "a normal turn";
        assert_eq!(FileMemory::embed_input(short), short, "short text passes through whole");
        let long = "x".repeat(EMBED_INPUT_MAX_CHARS + 500);
        assert_eq!(FileMemory::embed_input(&long).chars().count(), EMBED_INPUT_MAX_CHARS);
    }

    fn clean_ring4(tape: &Path) {
        let _ = std::fs::remove_file(tape);
        let _ = std::fs::remove_file(narrative_path_for(tape));
        let _ = std::fs::remove_file(vec_path_for(tape));
        let _ = std::fs::remove_file(vec_path_for(tape).with_extension("fp"));
        let _ = std::fs::remove_file(episodes_path_for(tape));
        let _ = std::fs::remove_file(epvec_path_for(tape));
    }

    #[tokio::test]
    async fn ring4_recall_injects_the_semantically_relevant_turn() {
        let tape = temp_tape("ring4");
        clean_ring4(&tape);
        // working_turns = 0 isolates Ring-4 (no Ring-2); k = 1.
        let mem = FileMemory::new("", &tape, 0).with_embedder(Arc::new(StubEmbed), Fingerprint::new("stub", 3), 1);
        mem.record(&trace("tell me about France", "Paris is the capital")).await.unwrap();
        mem.record(&trace("what is 2+2", "4")).await.unwrap();

        let mut q = base_step();
        q.content = vec![Content::Text { text: "a France question".into() }];
        let a = mem.assemble(&q, &Context::default()).await.unwrap();
        assert!(a.system.contains("Relevant earlier"), "Ring-4 header present");
        assert!(a.system.contains("Paris is the capital"), "the France turn is recalled (most similar)");
        assert!(!a.system.contains("2+2"), "the less-similar math turn is not recalled (k=1)");
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn fingerprint_mismatch_clears_the_stale_sidecar() {
        let tape = temp_tape("ring4-fp");
        clean_ring4(&tape);
        let m1 = FileMemory::new("", &tape, 0).with_embedder(Arc::new(StubEmbed), Fingerprint::new("embA", 3), 1);
        m1.record(&trace("France", "Paris")).await.unwrap();
        assert!(vec_path_for(&tape).exists(), "sidecar written");
        // re-open with a DIFFERENT embedder fingerprint → the stale sidecar is cleared (never serve stale).
        let _m2 = FileMemory::new("", &tape, 0).with_embedder(Arc::new(StubEmbed), Fingerprint::new("embB", 3), 1);
        assert!(!vec_path_for(&tape).exists(), "mismatched fingerprint clears the stale sidecar (GOLDEN_RECALL)");
        clean_ring4(&tape);
    }

    // ── A7.1: budgeted assembly (canon §7 "ringed + budgeted"; REEL §4.7) ──

    /// A budget in tokens whose ring-2 cap admits roughly `chars` characters.
    fn budget(ring2_chars: usize, ring3_chars: usize, ring4_chars: usize) -> TokenBudget {
        TokenBudget {
            ring2: (ring2_chars / CHARS_PER_TOKEN) as u32,
            ring3: (ring3_chars / CHARS_PER_TOKEN) as u32,
            ring4: (ring4_chars / CHARS_PER_TOKEN) as u32,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn budget_caps_ring2_and_the_newest_turn_survives() {
        let tape = temp_tape("budget-r2");
        clean_ring4(&tape);
        // 8 turns of ~60 chars each under a ~160-char ring-2 cap → only the newest ~2 fit.
        let mem = FileMemory::new("", &tape, 20).with_budget(budget(160, 4000, 4000));
        for i in 1..=8 {
            mem.record(&trace(&format!("question number {i} padded {}", "x".repeat(20)), &format!("answer-{i}"))).await.unwrap();
        }
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.contains("answer-8"), "the newest turn always survives the budget");
        assert!(!a.system.contains("answer-1"), "the oldest turn is dropped under budget pressure");
        assert!(a.system.chars().count() < 400, "the assembled context is bounded (O(1) per turn)");
        // budget is reported on the frozen AssembledContext for the engine/caller.
        assert_eq!(a.budget.ring2, (160 / CHARS_PER_TOKEN) as u32);
        // an oversized single newest turn is itself trimmed, never dropped (memory never goes blind).
        mem.record(&trace(&"y".repeat(2000), "needle-answer")).await.unwrap();
        let a2 = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a2.system.contains("..."), "the lone oversized newest turn is trimmed to the cap");
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn narrative_is_trimmed_to_its_ring3_budget_head_first() {
        let tape = temp_tape("budget-r3");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5).with_budget(budget(4000, 100, 4000));
        let long = format!("HEAD-OF-ARC {} TAIL-END", "z".repeat(500));
        mem.set_narrative(&long).unwrap();
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.contains("HEAD-OF-ARC"), "the head (durable arc) is kept");
        assert!(!a.system.contains("TAIL-END"), "the tail beyond the budget is cut");
        assert!(a.system.contains("[narrative trimmed to budget]"), "the cut is marked, never silent");
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn ring0_soul_is_never_trimmed_regardless_of_budget() {
        let tape = temp_tape("budget-r0");
        clean_ring4(&tape);
        let soul = format!("SOUL-START {} SOUL-END", "s".repeat(1200));
        // every capped ring squeezed to ~40 chars — the soul must still load verbatim (REEL §3.4).
        let mem = FileMemory::new(&soul, &tape, 5).with_budget(budget(40, 40, 40));
        mem.record(&trace("q", "a")).await.unwrap();
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.starts_with("SOUL-START"), "Ring-0 leads");
        assert!(a.system.contains("SOUL-END"), "Ring-0 is loaded verbatim - identity is constitutional");
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn ring4_recall_respects_its_budget() {
        let tape = temp_tape("budget-r4");
        clean_ring4(&tape);
        // k = 5 but a tiny ring-4 char budget → at most one (short) recall entry is injected.
        let mem = FileMemory::new("", &tape, 0)
            .with_embedder(Arc::new(StubEmbed), Fingerprint::new("stub", 3), 5)
            .with_budget(budget(4000, 4000, 80));
        mem.record(&trace("France facts one", "Paris one")).await.unwrap();
        mem.record(&trace("France facts two", "Paris two")).await.unwrap();
        mem.record(&trace("France facts three", "Paris three")).await.unwrap();
        let mut q = base_step();
        q.content = vec![Content::Text { text: "a France question".into() }];
        let a = mem.assemble(&q, &Context::default()).await.unwrap();
        let injected = a.system.matches("Paris").count();
        assert!(injected >= 1, "recall still surfaces the best match");
        assert!(injected < 3, "the ring-4 budget bounds how many recalls inject (k alone would allow 3)");
        clean_ring4(&tape);
    }

    // ── F-M1 (the "functionally forever" acceptance): a 5k-turn Tape assembles bounded ──

    #[tokio::test]
    async fn f_m1_a_5k_turn_tape_assembles_bounded_and_fast() {
        let tape = temp_tape("fm1");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 6)
            .with_embedder(Arc::new(StubEmbed), Fingerprint::new("stub", 3), 3)
            .with_recall_tuning(4096, 32);
        // 5k turns straight onto the Tape + the vector sidecar (the stub embedder keeps this model-free).
        for i in 0..5000 {
            mem.record(&trace(&format!("filler question {i} about topic {}", i % 7), &format!("answer {i}"))).await.unwrap();
        }
        let mut q = base_step();
        q.content = vec![Content::Text { text: "a France question".into() }];
        let t0 = std::time::Instant::now();
        let a = mem.assemble(&q, &Context::default()).await.unwrap();
        let elapsed = t0.elapsed();
        // O(1) context: the assembled preamble is bounded by the ring budgets, not the Tape size.
        let total_cap_chars = (FileMemory::default_budget().ring2 as usize
            + FileMemory::default_budget().ring3 as usize
            + FileMemory::default_budget().ring4 as usize
            + FileMemory::default_budget().ring1 as usize)
            * CHARS_PER_TOKEN
            + 512; // labels/joiners
        assert!(a.system.chars().count() < total_cap_chars, "assemble is O(1) in Tape size (got {} chars)", a.system.chars().count());
        // bounded scan: generous wall-clock bound (brute force over the window, never the whole Tape).
        assert!(elapsed < std::time::Duration::from_secs(2), "recall scan stays bounded (took {elapsed:?})");
        clean_ring4(&tape);
    }

    #[test]
    fn take_chars_is_utf8_safe_and_zero_budget_means_uncapped() {
        assert_eq!(take_chars("héllo wörld", 5), "héllo"); // char boundaries, not bytes
        assert_eq!(take_chars("ab", 10), "ab");
        assert_eq!(chars_cap(0), None, "0 tokens = uncapped (the explicit opt-out)");
        assert_eq!(chars_cap(10), Some(40), "~4 chars per token");
    }

    // ── A7.2: the episodes register (append-only mid-resolution digests) ──

    #[test]
    fn parse_consolidation_extracts_both_registers() {
        let out = "=== NARRATIVE ===\nThe arc so far: we built the thing.\n=== EPISODE ===\n\
                   happened: built A7.2\nchanged: episodes now durable\nmatters: memory autopilot depends on it\n\
                   unresolved: A7.3 wiring\nanchors: episodes register; autopilot; A7.2\n";
        let (narr, ep) = FileMemory::parse_consolidation(out);
        assert_eq!(narr, "The arc so far: we built the thing.");
        let ep = ep.expect("episode parsed");
        assert!(ep.parsed);
        assert_eq!(ep.what_happened, "built A7.2");
        assert_eq!(ep.what_changed, "episodes now durable");
        assert_eq!(ep.unresolved, "A7.3 wiring");
        assert_eq!(ep.anchors, vec!["episodes register", "autopilot", "A7.2"]);
        assert!(ep.text().contains("[episode] built A7.2"), "the flat retrieval text carries the digest");
    }

    #[tokio::test]
    async fn store_consolidation_rejects_a_parroted_placeholder() {
        let tape = temp_tape("placeholder");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5);
        mem.record(&trace("q", "a")).await.unwrap();
        // a lazy model copying the layout hint stores NOTHING (never fossilize junk into Ring-3).
        let out = "=== NARRATIVE ===\n(the updated narrative)\n=== EPISODE ===\nhappened: real thing\nchanged: -\nmatters: -\nunresolved: -\nanchors: a\n";
        let (n, _) = mem.store_consolidation(out).await.unwrap();
        assert_eq!(n, 0, "placeholder narrative rejected");
        assert!(mem.narrative().is_none(), "nothing stored");
        let out2 = "=== NARRATIVE ===\n<the narrative>\n=== EPISODE ===\nhappened: x\nchanged: -\nmatters: -\nunresolved: -\nanchors: a\n";
        assert_eq!(mem.store_consolidation(out2).await.unwrap().0, 0, "angle-bracket variant rejected too");
        clean_ring4(&tape);
    }

    #[test]
    fn parse_consolidation_missing_layout_falls_back_to_narrative_only() {
        let (narr, ep) = FileMemory::parse_consolidation("just a plain narrative, no markers");
        assert_eq!(narr, "just a plain narrative, no markers");
        assert!(ep.is_none(), "no layout -> the caller stores a flagged deterministic stub");
        // markers in the wrong order are treated as unparsed too (never a garbled slice).
        let (n2, e2) = FileMemory::parse_consolidation("=== EPISODE ===\nx\n=== NARRATIVE ===\ny");
        assert!(e2.is_none());
        assert!(n2.contains("EPISODE"), "the whole output survives as the narrative");
    }

    #[tokio::test]
    async fn store_consolidation_overwrites_narrative_but_appends_episodes() {
        let tape = temp_tape("episodes");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5);
        mem.record(&trace("first question about keels", "keels answered")).await.unwrap();

        let out1 = "=== NARRATIVE ===\nArc v1.\n=== EPISODE ===\nhappened: one\nchanged: c1\nmatters: m1\nunresolved: u1\nanchors: a1; b1\n";
        let (n1, p1) = mem.store_consolidation(out1).await.unwrap();
        assert!(n1 > 0 && p1);
        let out2 = "=== NARRATIVE ===\nArc v2 (rewritten).\n=== EPISODE ===\nhappened: two\nchanged: c2\nmatters: m2\nunresolved: u2\nanchors: a2\n";
        let (_, p2) = mem.store_consolidation(out2).await.unwrap();
        assert!(p2);

        // the narrative is ROLLING (overwritten) …
        assert_eq!(mem.narrative().unwrap(), "Arc v2 (rewritten).");
        // … the episodes are DURABLE (append-only, both survive, chronological, span stamped).
        let eps = mem.read_episodes(10);
        assert_eq!(eps.len(), 2, "append-only: nothing is overwritten");
        assert_eq!(eps[0].what_happened, "one");
        assert_eq!(eps[1].what_happened, "two");
        assert_eq!(eps[1].span_turns, 1, "the consolidated span is stamped");
        assert_eq!(mem.episodes_count(), 2);
        // an unparsed output still stores BOTH registers - the episode as a flagged stub.
        let (n3, p3) = mem.store_consolidation("a layoutless blob narrative").await.unwrap();
        assert!(n3 > 0 && !p3);
        let eps = mem.read_episodes(10);
        assert_eq!(eps.len(), 3);
        assert!(!eps[2].parsed, "the fallback stub is flagged, never silent");
        assert!(eps[2].what_happened.contains("unparsed"), "the stub says what it is");
        assert!(!eps[2].anchors.is_empty(), "anchors derived model-free from the turns");
        clean_ring4(&tape);
    }

    // ── A7.3: the two-tier Ring-4 index (episodes + bounded turn window) + cold-start backfill ──

    #[tokio::test]
    async fn ring4_two_tier_recalls_episodes_beyond_the_turn_window() {
        let tape = temp_tape("two-tier");
        clean_ring4(&tape);
        // window = 1: only the newest turn vector is scanned; episodes ALWAYS scan whole.
        let mem = FileMemory::new("", &tape, 0)
            .with_embedder(Arc::new(StubEmbed), Fingerprint::new("stub", 3), 3)
            .with_recall_tuning(1, 32);
        // an old France turn (falls outside the 1-turn window once a newer turn lands) …
        mem.record(&trace("tell me about France", "Paris is the capital")).await.unwrap();
        // … an episode digest about France (the coarse tier remembers what the window forgets) …
        let out = "=== NARRATIVE ===\nArc.\n=== EPISODE ===\nhappened: discussed France geography\nchanged: -\nmatters: -\nunresolved: -\nanchors: France; Paris\n";
        mem.store_consolidation(out).await.unwrap();
        // … then a newer unrelated turn takes the only window slot.
        mem.record(&trace("what is 2+2 math", "4")).await.unwrap();

        let mut q = base_step();
        q.content = vec![Content::Text { text: "a France question".into() }];
        let a = mem.assemble(&q, &Context::default()).await.unwrap();
        assert!(a.system.contains("[episode] discussed France geography"), "the episode tier recalls beyond the window");
        assert!(!a.system.contains("Paris is the capital"), "the old raw turn is outside the bounded scan (O(window+episodes))");
        assert!(!a.system.contains("2+2"), "an orthogonal match (cos<=0) never injects - the relevance floor");
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn cold_start_backfill_indexes_an_existing_tape_lazily() {
        let tape = temp_tape("backfill");
        clean_ring4(&tape);
        // an existing Tape recorded BEFORE any embedder was wired (no vector sidecar) …
        let plain = FileMemory::new("", &tape, 0);
        plain.record(&trace("the France fact", "Paris")).await.unwrap();
        plain.record(&trace("a math fact 2+2", "4")).await.unwrap();
        assert!(!vec_path_for(&tape).exists());
        // … then recall turns on (the A7.3 default): the first assemble backfills the bounded tail.
        let mem = FileMemory::new("", &tape, 0)
            .with_embedder(Arc::new(StubEmbed), Fingerprint::new("stub", 3), 1)
            .with_recall_tuning(64, 32);
        let mut q = base_step();
        q.content = vec![Content::Text { text: "about France".into() }];
        let a = mem.assemble(&q, &Context::default()).await.unwrap();
        assert!(vec_path_for(&tape).exists(), "backfill created the turn sidecar from the Tape");
        assert!(a.system.contains("Paris"), "the pre-embedder turn is immediately recallable");
        clean_ring4(&tape);
    }

    // ── A7.5: the self-correcting narrative (regenerate from ground truth) ──

    #[tokio::test]
    async fn corrective_prompt_carries_findings_and_ground_truth_never_the_drifted_narrative() {
        let tape = temp_tape("correct");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5);
        mem.record(&trace("what is my number", "42")).await.unwrap();
        mem.store_consolidation(
            "=== NARRATIVE ===\nDRIFTED-SKY-GREEN claim.\n=== EPISODE ===\nhappened: number recorded\nchanged: -\nmatters: -\nunresolved: -\nanchors: number\n",
        )
        .await
        .unwrap();
        let p = mem.corrective_consolidation_prompt(&["claims the sky is green".to_string()]);
        assert!(p.contains("claims the sky is green"), "the findings ride the prompt");
        assert!(p.contains("42"), "the Tape ground truth is included");
        assert!(p.contains("[episode] number recorded"), "the episode digests are included");
        assert!(!p.contains("DRIFTED-SKY-GREEN"), "the drifted narrative is EXCLUDED (REEL 10.2 - never re-anchor on drift)");
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn store_corrected_narrative_overwrites_without_appending_an_episode() {
        let tape = temp_tape("correct-store");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5);
        mem.store_consolidation(
            "=== NARRATIVE ===\nold arc\n=== EPISODE ===\nhappened: h\nchanged: c\nmatters: m\nunresolved: u\nanchors: a\n",
        )
        .await
        .unwrap();
        assert_eq!(mem.episodes_count(), 1);
        let corrected = "corrected arc: the unsupported claim about the sky being green is removed entirely";
        let n = mem.store_corrected_narrative(corrected).unwrap();
        assert!(n > 0);
        assert_eq!(mem.narrative().unwrap(), corrected);
        assert_eq!(mem.episodes_count(), 1, "a correction appends NO episode (nothing new happened)");
        // a degenerate correction (a stub like a lone CONSISTENT) must never clobber the narrative.
        assert_eq!(mem.store_corrected_narrative("CONSISTENT").unwrap(), 0);
        assert_eq!(mem.narrative().unwrap(), corrected, "the stub did not clobber");
        // a model that still emits the two-section layout is tolerated - only the narrative is kept.
        let n2 = mem
            .store_corrected_narrative("=== NARRATIVE ===\na layouted correction long enough to clear the degenerate floor\n=== EPISODE ===\nhappened: x\nchanged: y\nmatters: z\nunresolved: -\nanchors: q\n")
            .unwrap();
        assert!(n2 > 0);
        assert_eq!(mem.narrative().unwrap(), "a layouted correction long enough to clear the degenerate floor");
        assert_eq!(mem.episodes_count(), 1, "still no episode on a correction");
        clean_ring4(&tape);
    }

    // ── A7.6: Ring-1 exemplars + Ring-2 as conversation ──

    #[tokio::test]
    async fn ring1_exemplars_load_anchor_plus_recent_under_budget() {
        let tape = temp_tape("exemplars");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5);
        std::fs::write(
            mem.exemplars_path(),
            "pool notes (ignored preamble)\n## Anchor\nthe calibration anchor\n## Old\nold exemplar\n## Mid\nmid exemplar\n## New\nnewest exemplar\n",
        )
        .unwrap();
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a.system.contains("Calibration exemplars"), "Ring-1 label");
        assert!(a.system.contains("the calibration anchor"), "the anchor ALWAYS loads");
        assert!(a.system.contains("newest exemplar") && a.system.contains("mid exemplar"), "the most recent others rotate in");
        assert!(!a.system.contains("old exemplar"), "older pool sections stay in the pool (rotation, never deletion)");
        assert!(!a.system.contains("pool notes"), "text before the first header is a comment area");
        // a tiny ring-1 budget trims the block; the pool file itself is untouched.
        let tight =
            FileMemory::new("", &tape, 5).with_budget(TokenBudget { ring1: 10, ..FileMemory::default_budget() });
        let a2 = tight.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(a2.system.contains("[exemplars trimmed to budget]"));
        assert!(std::fs::read_to_string(mem.exemplars_path()).unwrap().contains("old exemplar"), "the pool is never pruned");
        let _ = std::fs::remove_file(mem.exemplars_path());
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn ring2_as_conversation_injects_real_messages() {
        let tape = temp_tape("conv");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5).with_ring2_as_conversation();
        mem.record(&trace("first question", "first answer")).await.unwrap();
        mem.record(&trace("second question", "second answer")).await.unwrap();
        let a = mem.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(!a.system.contains("Recent context"), "no preamble block in conversation mode");
        assert_eq!(a.conversation.len(), 4, "two turns -> two user/assistant message pairs");
        assert!(matches!(a.conversation[0].role, Role::User));
        assert!(matches!(a.conversation[1].role, Role::Assistant));
        let Content::Text { text } = &a.conversation[3].content[0] else { panic!("text content") };
        assert_eq!(text, "second answer");
        // the default (no opt-in) still folds Ring-2 into the preamble - nothing regresses.
        let plain = FileMemory::new("", &tape, 5);
        let ap = plain.assemble(&base_step(), &Context::default()).await.unwrap();
        assert!(ap.system.contains("Recent context") && ap.conversation.is_empty());
        clean_ring4(&tape);
    }

    #[tokio::test]
    async fn consolidation_prompt_requests_the_two_register_layout() {
        let tape = temp_tape("layout");
        clean_ring4(&tape);
        let mem = FileMemory::new("", &tape, 5);
        let s = mem.consolidate().await.unwrap();
        let Content::Text { text } = &s.content[0] else { panic!("text prompt") };
        assert!(text.contains("=== NARRATIVE ==="), "asks for the narrative section");
        assert!(text.contains("=== EPISODE ==="), "asks for the episode section");
        assert!(text.contains("anchors:"), "asks for retrieval anchors");
        assert!(text.contains("characters"), "the ring-3 budget hint rides the prompt (A7.1)");
        clean_ring4(&tape);
    }
}
