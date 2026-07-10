//! keel-adapters::rerank — the reranker (a Memory **organ**, NOT a tier; the router never routes
//! here, canon §11). A cross-encoder scores (query, doc) pairs jointly via llama-server's
//! `/v1/rerank` (Qwen3-Reranker-0.6B by default, served with `--reranking`). **Local + sovereign:**
//! queries/docs/scores never egress (I3). Ships OFF (`keel.lock rerank.default: identity`) until
//! `GOLDEN_RECALL`'s C1 uplift case earns it ON — this adapter exists so that benchmark can run.

use keel_contracts::{KeelError, Result};
use serde_json::{json, Value};

/// A llama-server-backed reranker. **Not a tier** — a sovereign Memory organ (like the embedder).
pub struct Reranker {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl Reranker {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            model: model.into(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Pull the per-document scores out of a `/v1/rerank` response
    /// (`results[].{index, relevance_score}`; some builds emit `score`) into a dense vector indexed
    /// like the request's `documents`. Factored out for a model-free unit test.
    fn parse_scores(v: &Value, n: usize) -> Result<Vec<f32>> {
        let results = v["results"]
            .as_array()
            .ok_or_else(|| KeelError::Other("rerank: no results[] in response".into()))?;
        let mut scores = vec![f32::NEG_INFINITY; n];
        for r in results {
            let i = r["index"]
                .as_u64()
                .ok_or_else(|| KeelError::Other("rerank: result missing index".into()))? as usize;
            let s = r["relevance_score"]
                .as_f64()
                .or_else(|| r["score"].as_f64())
                .ok_or_else(|| KeelError::Other("rerank: result missing relevance_score".into()))?;
            if i >= n {
                return Err(KeelError::Other(format!("rerank: index {i} out of range (n={n})")));
            }
            scores[i] = s as f32;
        }
        if scores.iter().any(|s| *s == f32::NEG_INFINITY) {
            return Err(KeelError::Other("rerank: response did not score every document".into()));
        }
        Ok(scores)
    }

    /// Score each doc against the query (higher = more relevant), indexed like `docs`.
    /// **Sovereign + local** — nothing egresses (I3).
    pub async fn rerank(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        let url = format!("{}/v1/rerank", self.endpoint);
        let body = json!({ "model": self.model, "query": query, "documents": docs, "top_n": docs.len() });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| KeelError::TierUnavailable(format!("rerank post: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status().as_u16();
            return Err(KeelError::Other(format!("rerank HTTP {s}: {}", resp.text().await.unwrap_or_default())));
        }
        let v: Value = resp.json().await.map_err(|e| KeelError::Other(format!("rerank decode: {e}")))?;
        Self::parse_scores(&v, docs.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scores_indexes_by_document_and_rejects_partial_responses() {
        // out-of-order results land at their request index; `score` is accepted as a fallback key.
        let v = json!({ "results": [
            { "index": 1, "relevance_score": 0.9 },
            { "index": 0, "score": 0.2 }
        ]});
        assert_eq!(Reranker::parse_scores(&v, 2).unwrap(), vec![0.2f32, 0.9]);
        // a response that skips a document is an honest error, never a silent hole.
        let partial = json!({ "results": [ { "index": 0, "relevance_score": 0.5 } ] });
        assert!(Reranker::parse_scores(&partial, 2).is_err());
        // an out-of-range index errors (never a panic).
        let oob = json!({ "results": [ { "index": 7, "relevance_score": 0.5 } ] });
        assert!(Reranker::parse_scores(&oob, 2).is_err());
        let r = Reranker::new("http://127.0.0.1:8091/", "qwen3-reranker-0.6b-q8");
        assert_eq!(r.endpoint, "http://127.0.0.1:8091");
        assert_eq!(r.model(), "qwen3-reranker-0.6b-q8");
    }

    /// Live: needs a llama-server with `--reranking` serving the reranker model. Run with `-- --ignored`.
    #[tokio::test]
    #[ignore = "requires a live llama-server /v1/rerank endpoint"]
    async fn live_rerank() {
        let r = Reranker::new("http://127.0.0.1:8091", "qwen3-reranker-0.6b-q8");
        let scores = r
            .rerank("what is the capital of France", &["Paris is the capital of France.", "The mitochondria is the powerhouse of the cell."])
            .await
            .expect("live rerank");
        assert_eq!(scores.len(), 2);
        assert!(scores[0] > scores[1], "the relevant doc must outscore the distractor");
    }
}
