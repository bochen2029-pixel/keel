//! keel-services::recall — Ring-4 retrieval (canon §11). The embedder is a Memory **organ** (not a
//! tier; the router never routes here). Recall = **cosine similarity over stored embeddings**,
//! brute-force: an L1 personal memory is small, so brute force is correct and dependency-free
//! (`sqlite-vec` is a scale optimization deferred until it matters — ISSUE-1 decided 2026-06-15).
//!
//! The index is **format-committing** (canon §11): it carries an embedder [`Fingerprint`] (model id +
//! vector dim). On a mismatch the index must **not serve** and must **rebuild from the ledger** (the
//! lossless Tape), so a stale index never returns garbage — `GOLDEN_RECALL`'s deterministic case. (The
//! recall@k / ndcg *uplift* cases are the §23 C1/C2 falsifiers, measured with real embeddings later.)

use async_trait::async_trait;
use keel_contracts::Result;

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

/// Brute-force top-`k` recall: rank `(id, vector)` entries by cosine to `query`, return the top-`k` ids
/// (most similar first). The index is small (an L1 memory), so brute force is correct; `sqlite-vec` is
/// the deferred scale optimization. Stable for equal scores (preserves input order via a stable sort).
pub fn recall_top_k(query: &[f32], entries: &[(String, Vec<f32>)], k: usize) -> Vec<String> {
    let mut scored: Vec<(f32, &String)> = entries.iter().map(|(id, v)| (cosine(query, v), id)).collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(k).map(|(_, id)| id.clone()).collect()
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
    }
}
