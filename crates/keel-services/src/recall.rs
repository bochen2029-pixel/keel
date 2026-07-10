//! keel-services::recall — Ring-4 retrieval (canon §11). The embedder is a Memory **organ** (not a
//! tier; the router never routes here). Recall = **cosine similarity over stored embeddings**,
//! brute-force: an L1 personal memory is small, so brute force is correct and dependency-free
//! (`sqlite-vec` is a scale optimization deferred until it matters — ISSUE-1 decided 2026-06-15).
//!
//! The index is **format-committing** (canon §11): it carries an embedder [`Fingerprint`] (model id +
//! vector dim). On a mismatch the index must **not serve** and must **rebuild from the ledger** (the
//! lossless Tape), so a stale index never returns garbage — `GOLDEN_RECALL`'s deterministic case.
//!
//! The recall@k / ndcg **uplift** cases are the §23 **C1/C2 falsifiers**: this module also carries
//! their benchmark harness — the [`Rerank`] seam (+ the shipped [`IdentityRerank`] default), the IR
//! metrics, the operator-ratified labeled-set loader ([`RecallSet`] — the `"golden-recall"` set the
//! frozen golden names), and the stub-testable [`run_recall_bench`] pipeline (`keel recall-bench`
//! drives it live). Design: `docs/proposals/golden-recall-set.md`.

use async_trait::async_trait;
use keel_contracts::{KeelError, Result};
use std::collections::BTreeMap;

/// The embed seam (a Memory **organ**, not a tier). A keel-services-local trait so `FileMemory` can hold
/// a `dyn Embed` and tests can stub it without a live model; the real impl is `keel_adapters::Embedder`
/// (HTTP `/v1/embeddings`). Text → a sovereign dense vector (vectors are invertible, never egress, I3).
#[async_trait]
pub trait Embed: Send + Sync {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
impl Embed for keel_adapters::Embedder {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.embed(text).await // inherent method (takes resolution priority — no recursion)
    }
}

/// The embedder fingerprint that commits the index format (canon §11): the model id + the vector dim.
/// The vectors are derived from a specific embedder; reading them with another is meaningless.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    pub embedder: String,
    pub dim: usize,
}

impl Fingerprint {
    pub fn new(embedder: impl Into<String>, dim: usize) -> Self {
        Self { embedder: embedder.into(), dim }
    }
}

/// `GOLDEN_RECALL` (the deterministic case): an index built by one embedder must **not** be served by
/// another — any fingerprint difference (model id OR dim) forces a rebuild from the ledger. Same
/// fingerprint serves; otherwise rebuild. This is a non-model assertion (I5) that a stale/incompatible
/// index can never silently return wrong neighbors.
pub fn should_rebuild(index: &Fingerprint, resolved: &Fingerprint) -> bool {
    index != resolved
}

/// Cosine similarity in `[-1, 1]`; `0.0` for a length-mismatch or a zero/degenerate vector (never a
/// panic). The model-free ranking primitive.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0f32, 0f32, 0f32);
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Brute-force **scored** ranking: every `(id, vector)` entry by cosine to `query`, most similar
/// first. Stable for equal scores (preserves input order). The bench needs the scores (top-1
/// calibration for the relevance floor); [`recall_top_k`] is the id-only view over this.
pub fn rank_all(query: &[f32], entries: &[(String, Vec<f32>)]) -> Vec<(String, f32)> {
    let mut scored: Vec<(f32, &String)> = entries.iter().map(|(id, v)| (cosine(query, v), id)).collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(s, id)| (id.clone(), s)).collect()
}

/// Brute-force top-`k` recall: rank `(id, vector)` entries by cosine to `query`, return the top-`k` ids
/// (most similar first). The index is small (an L1 memory), so brute force is correct; `sqlite-vec` is
/// the deferred scale optimization. Stable for equal scores (preserves input order via a stable sort).
pub fn recall_top_k(query: &[f32], entries: &[(String, Vec<f32>)], k: usize) -> Vec<String> {
    rank_all(query, entries).into_iter().take(k).map(|(id, _)| id).collect()
}

// ---------- the rerank seam (canon §11 `rerank()`; keel.lock `rerank.default: identity`) ----------

/// The rerank seam (a Memory **organ**, like [`Embed`]): score `docs` against `query` jointly
/// (cross-encoder), higher = more relevant, indexed like `docs`. Ships **OFF** — the genome default
/// is [`IdentityRerank`] — until `GOLDEN_RECALL`'s C1 uplift case earns the model ON.
#[async_trait]
pub trait Rerank: Send + Sync {
    async fn rerank(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>>;
}

/// The shipped default (`keel.lock rerank.default: identity`): preserves the incoming (cosine)
/// order by scoring each doc by its position. C1 measures the model reranker **against this**.
pub struct IdentityRerank;

#[async_trait]
impl Rerank for IdentityRerank {
    async fn rerank(&self, _query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        Ok((0..docs.len()).map(|i| (docs.len() - i) as f32).collect())
    }
}

#[async_trait]
impl Rerank for keel_adapters::Reranker {
    async fn rerank(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        keel_adapters::Reranker::rerank(self, query, docs).await // inherent method — no recursion
    }
}

// ---------- IR metrics (the GOLDEN_RECALL uplift measures) ----------

/// recall@k: the fraction of the relevant ids found in the top-`k` of `ranked`. `0.0` when nothing
/// is relevant (callers exclude negative-control queries from aggregation instead).
pub fn recall_at_k(ranked: &[String], relevant: &BTreeMap<String, u32>, k: usize) -> f32 {
    if relevant.is_empty() {
        return 0.0;
    }
    let hits = ranked.iter().take(k).filter(|id| relevant.contains_key(*id)).count();
    hits as f32 / relevant.len() as f32
}

/// nDCG@k with graded relevance (gain `2^grade − 1`, log2 position discount) — the C2 measure
/// (ordering quality, not just presence). `0.0` when no graded docs exist.
pub fn ndcg_at_k(ranked: &[String], grades: &BTreeMap<String, u32>, k: usize) -> f32 {
    let gain = |g: u32| (2f32.powi(g as i32)) - 1.0;
    let dcg: f32 = ranked
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, id)| grades.get(id).map(|g| gain(*g) / ((i as f32) + 2.0).log2()).unwrap_or(0.0))
        .sum();
    let mut ideal: Vec<u32> = grades.values().copied().collect();
    ideal.sort_unstable_by(|a, b| b.cmp(a));
    let idcg: f32 = ideal.iter().take(k).enumerate().map(|(i, g)| gain(*g) / ((i as f32) + 2.0).log2()).sum();
    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

/// Mean-reciprocal-rank contribution of one query: `1/rank` of the first relevant id (`0.0` if none).
pub fn mrr(ranked: &[String], relevant: &BTreeMap<String, u32>) -> f32 {
    ranked
        .iter()
        .position(|id| relevant.contains_key(id))
        .map(|p| 1.0 / (p as f32 + 1.0))
        .unwrap_or(0.0)
}

// ---------- the operator-ratified labeled set (`"set": "golden-recall"` in the frozen golden) ----------

/// One retrieval target. `kind` is `"turn"` (the `FileMemory::summarize` shape) or `"episode"`
/// (the `Episode::text()` shape) — the set mirrors the live Ring-4 text forms exactly.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecallDoc {
    pub id: String,
    pub kind: String,
    pub text: String,
}

/// One labeled query. `relevant` maps doc id → grade (2 = directly answers, 1 = partial; irrelevant
/// docs are omitted). An empty map = a negative control (no relevant doc exists in the corpus).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecallQuery {
    pub id: String,
    pub family: String,
    pub query: String,
    #[serde(default)]
    pub relevant: BTreeMap<String, u32>,
}

/// The labeled golden-recall set (`tests/recall/golden-recall.json`). **Operator-ratified ground
/// truth** — a sibling of (never an edit to) the sealed `golden.json`, which names it by reference.
/// While `ratified` is false every benchmark output is DRAFT and decision lines are withheld.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecallSet {
    pub name: String,
    pub version: u32,
    pub ratified: bool,
    #[serde(default)]
    pub thresholds: serde_json::Value,
    pub docs: Vec<RecallDoc>,
    pub queries: Vec<RecallQuery>,
}

/// The query families the set uses (each isolates a retrieval failure mode — proposal §2).
pub const RECALL_FAMILIES: [&str; 6] = ["paraphrase", "keyword_trap", "entity", "episodic", "multi", "negative"];

impl RecallSet {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| KeelError::Other(format!("recall set read {}: {e}", path.display())))?;
        serde_json::from_str(&raw).map_err(|e| KeelError::Other(format!("recall set parse {}: {e}", path.display())))
    }

    /// Structural coherence lint — **never content** (the operator edits docs/labels freely during
    /// ratification without breaking the gate): ids unique + resolvable, kinds/families known,
    /// grades in 1..=2, negatives unlabeled, non-negatives labeled. Empty = coherent.
    pub fn lint(&self) -> Vec<String> {
        let mut errs = Vec::new();
        let mut doc_ids = std::collections::BTreeSet::new();
        for d in &self.docs {
            if !doc_ids.insert(d.id.as_str()) {
                errs.push(format!("duplicate doc id {}", d.id));
            }
            if d.kind != "turn" && d.kind != "episode" {
                errs.push(format!("doc {}: unknown kind '{}' (turn|episode)", d.id, d.kind));
            }
            if d.text.trim().is_empty() {
                errs.push(format!("doc {}: empty text", d.id));
            }
        }
        let mut q_ids = std::collections::BTreeSet::new();
        for q in &self.queries {
            if !q_ids.insert(q.id.as_str()) {
                errs.push(format!("duplicate query id {}", q.id));
            }
            if !RECALL_FAMILIES.contains(&q.family.as_str()) {
                errs.push(format!("query {}: unknown family '{}'", q.id, q.family));
            }
            for (doc, grade) in &q.relevant {
                if !doc_ids.contains(doc.as_str()) {
                    errs.push(format!("query {}: relevant doc '{doc}' not in docs", q.id));
                }
                if !(1..=2).contains(grade) {
                    errs.push(format!("query {}: grade {grade} out of range (1..=2; omit irrelevant docs)", q.id));
                }
            }
            match (q.family.as_str(), q.relevant.is_empty()) {
                ("negative", false) => errs.push(format!("query {}: a negative control must have no relevant docs", q.id)),
                (f, true) if f != "negative" => errs.push(format!("query {}: non-negative family with no relevant docs", q.id)),
                _ => {}
            }
        }
        errs
    }
}

// ---------- the C1/C2 bench pipeline (model-free-testable; `keel recall-bench` runs it live) ----------

/// Bench knobs. `candidates` = how many cosine top hits the reranker re-scores (the standard
/// retrieve-then-rerank funnel); identity over the same funnel is the C1 baseline by construction.
#[derive(Clone, Debug)]
pub struct BenchConfig {
    pub embedder_id: String,
    pub rerank_id: Option<String>,
    pub k: usize,
    pub ndcg_k: usize,
    pub candidates: usize,
}

impl BenchConfig {
    pub fn new(embedder_id: impl Into<String>, rerank_id: Option<String>) -> Self {
        Self { embedder_id: embedder_id.into(), rerank_id, k: 5, ndcg_k: 10, candidates: 20 }
    }
}

/// One query's outcome. Metrics are `None` for negative controls (recall/nDCG/MRR are undefined
/// with no relevant doc); `top1_*` is the **cosine** top hit either way — the relevance-floor
/// calibration datum (the floor gates on cosine, before any reranker).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct QueryOutcome {
    pub id: String,
    pub family: String,
    pub recall_at_k: Option<f32>,
    pub ndcg_at_k: Option<f32>,
    pub mrr: Option<f32>,
    pub top1_id: Option<String>,
    pub top1_cosine: Option<f32>,
    /// The final ranking's top `ndcg_k` ids (post-rerank when a reranker rode the funnel) — the
    /// set-authoring/ratification feedback loop: which docs outrank the labeled ones, per query.
    #[serde(default)]
    pub top_ids: Vec<String>,
}

/// Mean metrics over the scored (non-negative) queries of one family (or overall).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BenchAgg {
    pub n: usize,
    pub recall_at_k: f32,
    pub ndcg_at_k: f32,
    pub mrr: f32,
}

/// The artifact `keel recall-bench` writes to `.keelstate/bench/` (verify-by-artifact) and reloads
/// for `--baseline` uplift comparison.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BenchReport {
    pub set: String,
    pub set_version: u32,
    pub ratified: bool,
    pub embedder: String,
    pub dim: usize,
    pub rerank: Option<String>,
    pub k: usize,
    pub ndcg_k: usize,
    pub candidates: usize,
    pub overall: BenchAgg,
    pub per_family: BTreeMap<String, BenchAgg>,
    /// Cosine top-1 scores of the negative controls — where a real relevance floor should sit.
    pub negative_top1_cosine: Vec<f32>,
    pub embed_p50_ms: u64,
    pub embed_p95_ms: u64,
    pub rerank_p50_ms: Option<u64>,
    pub rerank_p95_ms: Option<u64>,
    pub queries: Vec<QueryOutcome>,
}

/// Nearest-rank percentile over raw millisecond samples (`0` when empty — honest for "no calls").
fn percentile_ms(samples: &[u64], p: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut s = samples.to_vec();
    s.sort_unstable();
    let rank = ((p / 100.0) * s.len() as f64).ceil().max(1.0) as usize;
    s[rank.min(s.len()) - 1]
}

/// Run the C1/C2 benchmark: embed the corpus + queries via the seam, cosine-rank, optionally
/// rerank the top `candidates`, and score against the labeled set. Pure over its seams — a stub
/// [`Embed`]/[`Rerank`] exercises the whole pipeline model-free; live runs pass the real organs.
pub async fn run_recall_bench(
    embed: &dyn Embed,
    rerank: Option<&dyn Rerank>,
    set: &RecallSet,
    cfg: &BenchConfig,
) -> Result<BenchReport> {
    let errs = set.lint();
    if !errs.is_empty() {
        return Err(KeelError::Other(format!("recall set incoherent: {}", errs.join(" | "))));
    }
    // Embed the corpus (the index the queries run against), timing each call.
    let mut entries: Vec<(String, Vec<f32>)> = Vec::with_capacity(set.docs.len());
    let mut embed_ms: Vec<u64> = Vec::new();
    for d in &set.docs {
        let t0 = std::time::Instant::now();
        let v = embed.embed_text(&d.text).await?;
        embed_ms.push(t0.elapsed().as_millis() as u64);
        entries.push((d.id.clone(), v));
    }
    let dim = entries.first().map(|(_, v)| v.len()).unwrap_or(0);
    let text_of: BTreeMap<&str, &str> = set.docs.iter().map(|d| (d.id.as_str(), d.text.as_str())).collect();

    let mut outcomes: Vec<QueryOutcome> = Vec::with_capacity(set.queries.len());
    let mut negative_top1: Vec<f32> = Vec::new();
    let mut rerank_ms: Vec<u64> = Vec::new();
    for q in &set.queries {
        let t0 = std::time::Instant::now();
        let qv = embed.embed_text(&q.query).await?;
        embed_ms.push(t0.elapsed().as_millis() as u64);
        let scored = rank_all(&qv, &entries);
        // Final ranking: rerank reorders the cosine top-`candidates` funnel; the tail keeps cosine order.
        let ranked: Vec<String> = match rerank {
            Some(rr) => {
                let n = cfg.candidates.min(scored.len());
                let cand_ids: Vec<&String> = scored.iter().take(n).map(|(id, _)| id).collect();
                let cand_texts: Vec<&str> = cand_ids.iter().map(|id| text_of[id.as_str()]).collect();
                let t1 = std::time::Instant::now();
                let scores = rr.rerank(&q.query, &cand_texts).await?;
                rerank_ms.push(t1.elapsed().as_millis() as u64);
                let mut order: Vec<usize> = (0..n).collect();
                order.sort_by(|a, b| scores[*b].partial_cmp(&scores[*a]).unwrap_or(std::cmp::Ordering::Equal));
                order
                    .into_iter()
                    .map(|i| cand_ids[i].clone())
                    .chain(scored.iter().skip(n).map(|(id, _)| id.clone()))
                    .collect()
            }
            None => scored.iter().map(|(id, _)| id.clone()).collect(),
        };
        let top1 = scored.first().cloned();
        let top_ids: Vec<String> = ranked.iter().take(cfg.ndcg_k).cloned().collect();
        if q.relevant.is_empty() {
            if let Some((_, s)) = &top1 {
                negative_top1.push(*s);
            }
            outcomes.push(QueryOutcome {
                id: q.id.clone(),
                family: q.family.clone(),
                recall_at_k: None,
                ndcg_at_k: None,
                mrr: None,
                top1_id: top1.as_ref().map(|(id, _)| id.clone()),
                top1_cosine: top1.as_ref().map(|(_, s)| *s),
                top_ids,
            });
        } else {
            outcomes.push(QueryOutcome {
                id: q.id.clone(),
                family: q.family.clone(),
                recall_at_k: Some(recall_at_k(&ranked, &q.relevant, cfg.k)),
                ndcg_at_k: Some(ndcg_at_k(&ranked, &q.relevant, cfg.ndcg_k)),
                mrr: Some(mrr(&ranked, &q.relevant)),
                top1_id: top1.as_ref().map(|(id, _)| id.clone()),
                top1_cosine: top1.as_ref().map(|(_, s)| *s),
                top_ids,
            });
        }
    }

    // Aggregate the scored queries per family + overall (negatives are reported, never averaged).
    let agg = |of: &[&QueryOutcome]| -> BenchAgg {
        let scored: Vec<&&QueryOutcome> = of.iter().filter(|o| o.recall_at_k.is_some()).collect();
        let n = scored.len();
        if n == 0 {
            return BenchAgg::default();
        }
        let mean = |f: &dyn Fn(&QueryOutcome) -> f32| scored.iter().map(|o| f(o)).sum::<f32>() / n as f32;
        BenchAgg {
            n,
            recall_at_k: mean(&|o| o.recall_at_k.unwrap_or(0.0)),
            ndcg_at_k: mean(&|o| o.ndcg_at_k.unwrap_or(0.0)),
            mrr: mean(&|o| o.mrr.unwrap_or(0.0)),
        }
    };
    let all: Vec<&QueryOutcome> = outcomes.iter().collect();
    let mut per_family: BTreeMap<String, BenchAgg> = BTreeMap::new();
    for fam in RECALL_FAMILIES {
        let of: Vec<&QueryOutcome> = outcomes.iter().filter(|o| o.family == fam).collect();
        if !of.is_empty() && fam != "negative" {
            per_family.insert(fam.to_string(), agg(&of));
        }
    }
    Ok(BenchReport {
        set: set.name.clone(),
        set_version: set.version,
        ratified: set.ratified,
        embedder: cfg.embedder_id.clone(),
        dim,
        rerank: cfg.rerank_id.clone(),
        k: cfg.k,
        ndcg_k: cfg.ndcg_k,
        candidates: cfg.candidates,
        overall: agg(&all),
        per_family,
        negative_top1_cosine: negative_top1,
        embed_p50_ms: percentile_ms(&embed_ms, 50.0),
        embed_p95_ms: percentile_ms(&embed_ms, 95.0),
        rerank_p50_ms: if rerank_ms.is_empty() { None } else { Some(percentile_ms(&rerank_ms, 50.0)) },
        rerank_p95_ms: if rerank_ms.is_empty() { None } else { Some(percentile_ms(&rerank_ms, 95.0)) },
        queries: outcomes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `GOLDEN_RECALL` deterministic conformance: a fingerprint mismatch must not serve and must rebuild.
    #[test]
    fn passes_golden_recall_fingerprint() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/golden.json");
        let golden: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let cases = golden["recall"].as_array().expect("recall cases");
        let fp = cases
            .iter()
            .find(|c| c["input"].get("index_embedder").is_some())
            .expect("the fingerprint-mismatch case");
        // dims are encoded in the golden's embedder names (minilm-384, qwen3-embedding-1024).
        let index = Fingerprint::new(fp["input"]["index_embedder"].as_str().unwrap(), 384);
        let resolved = Fingerprint::new(fp["input"]["resolved_embedder"].as_str().unwrap(), 1024);
        assert!(should_rebuild(&index, &resolved), "a mismatch must rebuild");
        assert_eq!(fp["expect"]["serve"].as_bool(), Some(false), "golden: do not serve a stale index");
        assert_eq!(fp["expect"]["rebuild_from_ledger"].as_bool(), Some(true), "golden: rebuild from the ledger");
        // the same fingerprint serves (no needless rebuild).
        assert!(!should_rebuild(&index, &index.clone()), "matching fingerprint serves");
    }

    #[test]
    fn cosine_handles_degenerate_and_recall_ranks_by_similarity() {
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0); // zero vector -> 0, no panic
        assert_eq!(cosine(&[1.0], &[1.0, 2.0]), 0.0); // length mismatch -> 0
        let entries = vec![
            ("m1".to_string(), vec![1.0, 0.0, 0.0]),
            ("m7".to_string(), vec![0.0, 1.0, 0.0]),
            ("m3".to_string(), vec![0.9, 0.1, 0.0]),
        ];
        assert_eq!(recall_top_k(&[1.0, 0.0, 0.0], &entries, 2), vec!["m1".to_string(), "m3".to_string()]);
        // rank_all carries the scores recall_top_k drops (top-1 floor calibration needs them).
        let scored = rank_all(&[1.0, 0.0, 0.0], &entries);
        assert_eq!(scored[0].0, "m1");
        assert!((scored[0].1 - 1.0).abs() < 1e-6);
    }

    fn rel(pairs: &[(&str, u32)]) -> BTreeMap<String, u32> {
        pairs.iter().map(|(id, g)| (id.to_string(), *g)).collect()
    }

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ir_metrics_match_hand_computed_values() {
        // recall@2: one of the two relevant ids is in the top-2.
        let ranked = ids(&["a", "b", "c", "x"]);
        assert!((recall_at_k(&ranked, &rel(&[("a", 2), ("x", 1)]), 2) - 0.5).abs() < 1e-6);
        assert_eq!(recall_at_k(&ranked, &BTreeMap::new(), 2), 0.0); // no relevant -> 0, never a panic
        // nDCG@2: perfect order = 1.0; inverted order = DCG(1/1 + 3/log2(3)) / IDCG(3/1 + 1/log2(3)).
        let grades = rel(&[("a", 2), ("b", 1)]);
        assert!((ndcg_at_k(&ids(&["a", "b"]), &grades, 2) - 1.0).abs() < 1e-6);
        let inverted = ndcg_at_k(&ids(&["b", "a"]), &grades, 2);
        let expect = (1.0 + 3.0 / 3f32.log2()) / (3.0 + 1.0 / 3f32.log2());
        assert!((inverted - expect).abs() < 1e-6);
        assert!(inverted < 1.0);
        // MRR: first relevant at rank 3 -> 1/3; none -> 0.
        assert!((mrr(&ids(&["x", "y", "a"]), &rel(&[("a", 2)])) - (1.0 / 3.0)).abs() < 1e-6);
        assert_eq!(mrr(&ids(&["x", "y"]), &rel(&[("a", 2)])), 0.0);
        // percentile: nearest-rank on a small sample.
        assert_eq!(percentile_ms(&[10, 20, 30, 40], 50.0), 20);
        assert_eq!(percentile_ms(&[10, 20, 30, 40], 95.0), 40);
        assert_eq!(percentile_ms(&[], 50.0), 0);
    }

    fn doc(id: &str, kind: &str, text: &str) -> RecallDoc {
        RecallDoc { id: id.into(), kind: kind.into(), text: text.into() }
    }

    fn query(id: &str, family: &str, q: &str, relevant: BTreeMap<String, u32>) -> RecallQuery {
        RecallQuery { id: id.into(), family: family.into(), query: q.into(), relevant }
    }

    #[test]
    fn set_lint_catches_incoherence_never_content() {
        let set = RecallSet {
            name: "t".into(),
            version: 1,
            ratified: false,
            thresholds: serde_json::Value::Null,
            docs: vec![doc("d1", "turn", "x"), doc("d1", "scroll", " ")],
            queries: vec![
                query("q1", "paraphrase", "?", rel(&[("ghost", 3)])),
                query("q2", "negative", "?", rel(&[("d1", 1)])),
                query("q3", "entity", "?", BTreeMap::new()),
                query("q3", "made-up", "?", rel(&[("d1", 1)])),
            ],
        };
        let errs = set.lint();
        let has = |s: &str| errs.iter().any(|e| e.contains(s));
        assert!(has("duplicate doc id d1"));
        assert!(has("unknown kind 'scroll'"));
        assert!(has("empty text"));
        assert!(has("'ghost' not in docs"));
        assert!(has("grade 3 out of range"));
        assert!(has("negative control must have no relevant"));
        assert!(has("non-negative family with no relevant"));
        assert!(has("duplicate query id q3"));
        assert!(has("unknown family 'made-up'"));
    }

    /// The shipped DRAFT set stays structurally coherent (ids resolve, families known, negatives
    /// unlabeled). Content is the operator's to edit — this asserts structure ONLY, so ratification
    /// edits can never break the gate.
    #[test]
    fn the_golden_recall_set_file_loads_and_lints_clean() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/recall/golden-recall.json");
        let set = RecallSet::load(std::path::Path::new(path)).expect("set loads");
        assert_eq!(set.name, "golden-recall", "the frozen golden names this set");
        assert!(!set.docs.is_empty() && !set.queries.is_empty());
        let errs = set.lint();
        assert!(errs.is_empty(), "structural lint must be clean: {errs:?}");
    }

    /// Keyword-hash stub: "alpha"->x, "beta"->y, "gamma"->z (mixtures for queries). The pipeline is
    /// exercised end-to-end with zero live models (the live legs are `keel recall-bench`'s).
    struct BenchStubEmbed;

    #[async_trait]
    impl Embed for BenchStubEmbed {
        async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
            let mut v = vec![0.0f32; 3];
            if text.contains("alpha") {
                v[0] = 1.0;
            }
            if text.contains("beta") {
                v[1] = 1.0;
            }
            if text.contains("gamma") {
                v[2] = 1.0;
            }
            Ok(v)
        }
    }

    /// An adversarial stub reranker: inverts whatever order it is given.
    struct InvertRerank;

    #[async_trait]
    impl Rerank for InvertRerank {
        async fn rerank(&self, _query: &str, docs: &[&str]) -> Result<Vec<f32>> {
            Ok((0..docs.len()).map(|i| i as f32).collect())
        }
    }

    fn bench_set() -> RecallSet {
        RecallSet {
            name: "stub".into(),
            version: 1,
            ratified: false,
            thresholds: serde_json::Value::Null,
            docs: vec![
                doc("d_a", "turn", "- user: alpha?\n  assistant: alpha."),
                doc("d_b", "turn", "- user: beta?\n  assistant: beta."),
                doc("d_g", "episode", "[episode] gamma | changed: - | matters: - | unresolved: - | anchors: gamma"),
            ],
            queries: vec![
                query("q_a", "paraphrase", "tell me about alpha", rel(&[("d_a", 2)])),
                query("q_bg", "multi", "beta and gamma", rel(&[("d_b", 2), ("d_g", 1)])),
                query("q_n", "negative", "delta epsilon", BTreeMap::new()),
            ],
        }
    }

    #[tokio::test]
    async fn bench_pipeline_scores_identity_and_rerank_changes_the_final_ranking() {
        let set = bench_set();
        let mut cfg = BenchConfig::new("stub-embed", None);
        cfg.k = 1;
        cfg.candidates = 3;
        // identity leg: cosine order stands — q_a's top-1 is its relevant doc.
        let base = run_recall_bench(&BenchStubEmbed, Some(&IdentityRerank), &set, &cfg).await.unwrap();
        assert_eq!(base.dim, 3);
        assert!(!base.ratified, "the draft flag propagates into the artifact");
        assert_eq!(base.overall.n, 2, "negatives are reported, never averaged");
        let q_a = base.queries.iter().find(|o| o.id == "q_a").unwrap();
        assert_eq!(q_a.recall_at_k, Some(1.0));
        assert_eq!(q_a.top1_id.as_deref(), Some("d_a"));
        assert_eq!(q_a.top_ids.first().map(String::as_str), Some("d_a"), "the final ranking is recorded");
        let q_n = base.queries.iter().find(|o| o.id == "q_n").unwrap();
        assert_eq!(q_n.recall_at_k, None, "a negative control has no recall");
        assert_eq!(base.negative_top1_cosine.len(), 1, "its cosine top-1 is floor-calibration data");
        assert!(base.per_family.contains_key("paraphrase") && !base.per_family.contains_key("negative"));
        assert!(base.rerank_p50_ms.is_some(), "a reranker was in the loop");
        // adversarial leg: an order-inverting reranker demotes the relevant doc out of the top-1 —
        // proof the reranker owns the final ranking (what C1 measures, in miniature).
        let mut rcfg = cfg.clone();
        rcfg.rerank_id = Some("invert".into());
        let flipped = run_recall_bench(&BenchStubEmbed, Some(&InvertRerank), &set, &rcfg).await.unwrap();
        let q_a_f = flipped.queries.iter().find(|o| o.id == "q_a").unwrap();
        assert_eq!(q_a_f.recall_at_k, Some(0.0), "the inverted funnel buried the relevant doc");
        assert!(flipped.overall.recall_at_k < base.overall.recall_at_k);
        // no reranker at all: pure cosine, no rerank latency recorded.
        let none = run_recall_bench(&BenchStubEmbed, None, &set, &BenchConfig::new("stub-embed", None)).await.unwrap();
        assert!(none.rerank_p50_ms.is_none());
    }

    #[tokio::test]
    async fn bench_refuses_an_incoherent_set() {
        let mut set = bench_set();
        set.queries.push(query("q_bad", "entity", "?", rel(&[("nope", 2)])));
        let err = run_recall_bench(&BenchStubEmbed, None, &set, &BenchConfig::new("e", None)).await;
        assert!(err.is_err(), "an unresolvable label must abort, never skew metrics");
    }
}
