//! deepseek — the cheap-API reasoning tier (canon §4). OpenAI Chat Completions over HTTPS with
//! DeepSeek's compat specifics: the chat path is `/chat/completions` (no `/v1`); thinking is
//! `thinking:{type:enabled}` + `reasoning_effort` (vs Qwen's `chat_template_kwargs`); cost is
//! **real money**, computed from usage × price with the prompt-cache-hit lever. It reuses the
//! shared `openai` mapping wholesale — the payoff of factoring it out.

use crate::openai::{base_body, parse_response, OaiResponse};
use async_trait::async_trait;
use keel_contracts::{
    Capabilities, Context, Effort, GenerateRequest, GenerateResult, KeelError, ModelTier, Price,
    Result,
};
use serde_json::{json, Value};

/// A DeepSeek-backed cheap-API tier.
pub struct DeepSeek {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    tier: String,
    price: Price,
    api_key: String,
    max_tokens: u32,
}

impl DeepSeek {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        tier: impl Into<String>,
        price: Price,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            model: model.into(),
            tier: tier.into(),
            price,
            api_key: api_key.into(),
            max_tokens: 4096,
        }
    }

    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.endpoint) // NOT /v1 (DeepSeek)
    }

    fn build_body(&self, req: &GenerateRequest) -> Result<Value> {
        let mut body = base_body(req, &self.model, self.max_tokens)?;
        if let Some(effort) = reasoning_effort(&req.effort) {
            body.insert("thinking".into(), json!({ "type": "enabled" }));
            body.insert("reasoning_effort".into(), json!(effort));
        }
        Ok(Value::Object(body))
    }
}

/// `Effort.thinking` → DeepSeek `reasoning_effort`. Lean signals (or `None`) ⇒ non-thinking.
fn reasoning_effort(effort: &Effort) -> Option<&str> {
    match effort.thinking.as_deref() {
        Some("low") | Some("off") | Some("none") | Some("no") | None => None,
        Some(v) => Some(v), // "high" | "max" | …
    }
}

#[async_trait]
impl ModelTier for DeepSeek {
    fn caps(&self) -> Capabilities {
        Capabilities { vision: false, video: false, thinking: true }
    }

    async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
        let body = self.build_body(&req)?;
        let resp = self
            .client
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| KeelError::TierUnavailable(format!("deepseek post: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            let msg = format!("deepseek HTTP {}: {}", status.as_u16(), txt);
            return Err(if status.is_server_error() {
                KeelError::TierUnavailable(msg)
            } else {
                KeelError::Other(msg)
            });
        }
        let oai: OaiResponse = resp
            .json()
            .await
            .map_err(|e| KeelError::Other(format!("deepseek decode: {e}")))?;
        Ok(parse_response(oai, &self.tier, &req.model, &self.price))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::{Content, Message, Role};

    fn req_text(s: &str, effort: Effort) -> GenerateRequest {
        GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: vec![Content::Text { text: s.into() }],
                name: None,
                reasoning_content: None,
                tool_call_id: None,
            }],
            model: "deepseek-v4-pro".into(),
            tools: vec![],
            grammar: None,
            effort,
            cache_prefix_len: None,
        }
    }

    fn tier() -> DeepSeek {
        DeepSeek::new(
            "https://api.deepseek.com/",
            "deepseek-v4-pro",
            "cheap-API",
            Price { input_miss: 0.435, input_hit: 0.003625, output: 0.87 },
            "test-key",
        )
    }

    #[test]
    fn chat_url_has_no_v1() {
        assert_eq!(tier().chat_url(), "https://api.deepseek.com/chat/completions");
    }

    #[test]
    fn reasoning_effort_mapping() {
        assert_eq!(reasoning_effort(&Effort { n: 1, thinking: Some("high".into()) }), Some("high"));
        assert_eq!(reasoning_effort(&Effort { n: 1, thinking: Some("low".into()) }), None);
        assert_eq!(reasoning_effort(&Effort { n: 1, thinking: None }), None);
    }

    #[test]
    fn thinking_block_only_when_enabled() {
        let t = tier();
        let on = t.build_body(&req_text("x", Effort { n: 1, thinking: Some("high".into()) })).unwrap();
        assert_eq!(on["thinking"]["type"], "enabled");
        assert_eq!(on["reasoning_effort"], "high");
        let off = t.build_body(&req_text("x", Effort::default())).unwrap();
        assert!(off.get("thinking").is_none());
        assert!(off.get("reasoning_effort").is_none());
    }

    #[test]
    fn caps_have_no_vision() {
        assert!(!tier().caps().vision);
    }

    /// Live against the real DeepSeek API (spends a few tokens of real money). Run with the key in
    /// env: `cargo test -p keel-adapters deepseek -- --ignored`
    #[tokio::test]
    #[ignore = "hits the real DeepSeek API; needs DEEPSEEK_API_KEY"]
    async fn live_generate() {
        let key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
        assert!(!key.is_empty(), "DEEPSEEK_API_KEY not set");
        let t = DeepSeek::new(
            "https://api.deepseek.com",
            "deepseek-v4-pro",
            "cheap-API",
            Price { input_miss: 0.435, input_hit: 0.003625, output: 0.87 },
            key,
        );
        let res = t
            .generate(req_text("Reply with exactly the word: pong", Effort { n: 1, thinking: None }), &Context::default())
            .await
            .expect("live deepseek");
        eprintln!(
            "content={:?} reasoning?={} usage={:?} cost=${:.6}",
            res.content,
            res.reasoning_content.is_some(),
            res.usage,
            res.cost
        );
        assert!(!res.content.is_empty());
        assert_eq!(res.tier, "cheap-API");
        assert!(res.cost > 0.0); // real money — I4 finally bites
    }
}
