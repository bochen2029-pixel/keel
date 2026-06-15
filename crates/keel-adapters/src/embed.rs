//! keel-adapters::embed — the embedder (a Memory **organ**, NOT a tier; the router never routes here,
//! canon §11). Text → a dense vector via llama-server's OpenAI-compatible `/v1/embeddings`
//! (Qwen3-Embedding-0.6B by default). **Local + sovereign:** embedding vectors are *invertible*, so
//! they never egress (I3). The embedder is **format-committing** (canon §11): the index carries the
//! model id + vector dim as a fingerprint; a mismatch rebuilds from the ledger (see `svc::recall`).

use keel_contracts::{KeelError, Result};
use serde_json::{json, Value};

/// A llama-server-backed embedder. **Not a tier** — a sovereign Memory organ (like whisper / the mic).
pub struct Embedder {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl Embedder {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            model: model.into(),
        }
    }

    /// The model id — half of the index fingerprint (canon §11; the other half is the vector dim).
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Pull the dense vector out of an OpenAI `/v1/embeddings` response (`data[0].embedding`). Factored
    /// out for a model-free unit test (the live HTTP is the `#[ignore]` test).
    fn parse_embedding(v: &Value) -> Result<Vec<f32>> {
        let arr = v["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| KeelError::Other("embed: no data[0].embedding in response".into()))?;
        Ok(arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
    }

    /// Embed one text into a dense vector. **Sovereign + local** — the vector never egresses (I3).
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/v1/embeddings", self.endpoint);
        let body = json!({ "model": self.model, "input": text });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| KeelError::TierUnavailable(format!("embed post: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status().as_u16();
            return Err(KeelError::Other(format!("embed HTTP {s}: {}", resp.text().await.unwrap_or_default())));
        }
        let v: Value = resp.json().await.map_err(|e| KeelError::Other(format!("embed decode: {e}")))?;
        Self::parse_embedding(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_embedding_pulls_the_vector_and_endpoint_is_trimmed() {
        let e = Embedder::new("http://127.0.0.1:8080/", "qwen3-embedding-0.6b");
        assert_eq!(e.endpoint, "http://127.0.0.1:8080");
        assert_eq!(e.model(), "qwen3-embedding-0.6b");
        let v = json!({ "data": [{ "embedding": [0.1, 0.2, -0.3] }] });
        assert_eq!(Embedder::parse_embedding(&v).unwrap(), vec![0.1f32, 0.2, -0.3]);
        // a malformed response errors honestly (never a panic).
        assert!(Embedder::parse_embedding(&json!({ "data": [] })).is_err());
    }

    /// Live: needs a llama-server serving the embedding model. Run with `-- --ignored`.
    #[tokio::test]
    #[ignore = "requires a live llama-server embeddings endpoint"]
    async fn live_embed() {
        let e = Embedder::new("http://127.0.0.1:8080", "qwen3-embedding-0.6b");
        let v = e.embed("the capital of France is Paris").await.expect("live embed");
        assert!(!v.is_empty(), "a real embedding is non-empty");
    }
}
