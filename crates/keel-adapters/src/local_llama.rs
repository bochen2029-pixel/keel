//! local_llama — the on-box workhorse tier (canon §4, §13): HTTP → llama-server, OpenAI
//! Chat Completions. Vision rides the cognition protocol (`image_url`); **constrained decoding**
//! is the small-model superpower (a GBNF grammar or JSON schema enforced at decode time so the
//! model cannot emit a malformed tool call); thinking is toggled per-step via Qwen's
//! `chat_template_kwargs.enable_thinking` so scaffolding runs lean. Local cognition is electricity
//! — cost is $0 (price defaults to zero), tracked uniformly through `compute_cost`.

use crate::openai::{base_body, parse_response, OaiResponse};
use async_trait::async_trait;
use keel_contracts::{
    Capabilities, Context, Effort, GenerateRequest, GenerateResult, KeelError, ModelTier, Price,
    Result,
};
use serde_json::{json, Value};

/// A llama-server-backed tier.
pub struct LocalLlama {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    tier: String,
    price: Price,
    vision: bool,
    max_tokens: u32,
}

impl LocalLlama {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        tier: impl Into<String>,
        price: Price,
        vision: bool,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            model: model.into(),
            tier: tier.into(),
            price,
            vision,
            max_tokens: 4096,
        }
    }

    /// Override the generation cap (the contract carries no `max_tokens`; the adapter sets it).
    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }

    fn build_body(&self, req: &GenerateRequest) -> Result<Value> {
        let mut body = base_body(req, &self.model, self.max_tokens)?;
        // constrained decode: GBNF string → `grammar`, JSON schema object → `json_schema`
        if let Some(grammar) = &req.grammar {
            match grammar {
                Value::String(s) => {
                    body.insert("grammar".into(), Value::String(s.clone()));
                }
                other => {
                    body.insert("json_schema".into(), other.clone());
                }
            }
        }
        // thinking toggle (Qwen/llama-server): None = server default
        if let Some(enable) = thinking_enabled(&req.effort) {
            body.insert("chat_template_kwargs".into(), json!({ "enable_thinking": enable }));
        }
        Ok(Value::Object(body))
    }
}

/// `Effort.thinking` → `enable_thinking`. Lean signals turn it off (scaffolding); `None` defers
/// to the server default.
fn thinking_enabled(effort: &Effort) -> Option<bool> {
    match effort.thinking.as_deref() {
        Some("low") | Some("off") | Some("none") | Some("no") => Some(false),
        Some(_) => Some(true),
        None => None,
    }
}

#[async_trait]
impl ModelTier for LocalLlama {
    fn caps(&self) -> Capabilities {
        Capabilities { vision: self.vision, video: false, thinking: true }
    }

    async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
        let body = self.build_body(&req)?;
        let url = format!("{}/v1/chat/completions", self.endpoint);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| KeelError::TierUnavailable(format!("local_llama post: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            let msg = format!("local_llama HTTP {}: {}", status.as_u16(), txt);
            // 5xx/OOM → unavailable (failover); 4xx → a request fault
            return Err(if status.is_server_error() {
                KeelError::TierUnavailable(msg)
            } else {
                KeelError::Other(msg)
            });
        }
        let oai: OaiResponse = resp
            .json()
            .await
            .map_err(|e| KeelError::Other(format!("local_llama decode: {e}")))?;
        Ok(parse_response(oai, &self.tier, &req.model, &self.price))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::{Content, Message, Role};

    fn req_text(s: &str, effort: Effort, grammar: Option<Value>) -> GenerateRequest {
        GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: vec![Content::Text { text: s.into() }],
                name: None,
                reasoning_content: None,
                tool_call_id: None,
            }],
            model: "qwen3.5-9b".into(),
            tools: vec![],
            grammar,
            effort,
            cache_prefix_len: None,
        }
    }

    fn tier() -> LocalLlama {
        LocalLlama::new("http://127.0.0.1:8080/", "qwen3.5-9b", "local", Price::default(), true)
    }

    #[test]
    fn thinking_toggle_mapping() {
        assert_eq!(thinking_enabled(&Effort { n: 1, thinking: Some("high".into()) }), Some(true));
        assert_eq!(thinking_enabled(&Effort { n: 1, thinking: Some("low".into()) }), Some(false));
        assert_eq!(thinking_enabled(&Effort { n: 1, thinking: None }), None);
    }

    #[test]
    fn body_carries_thinking_and_endpoint_is_trimmed() {
        let t = tier();
        assert_eq!(t.endpoint, "http://127.0.0.1:8080"); // trailing slash trimmed
        let body = t.build_body(&req_text("hi", Effort { n: 1, thinking: Some("low".into()) }, None)).unwrap();
        assert_eq!(body["chat_template_kwargs"]["enable_thinking"], false);
        // None → no kwargs at all
        let plain = t.build_body(&req_text("hi", Effort::default(), None)).unwrap();
        assert!(plain.get("chat_template_kwargs").is_none());
    }

    #[test]
    fn grammar_string_and_object_route_to_the_right_field() {
        let t = tier();
        let gbnf = t.build_body(&req_text("x", Effort::default(), Some(json!("root ::= \"ok\"")))).unwrap();
        assert_eq!(gbnf["grammar"], "root ::= \"ok\"");
        assert!(gbnf.get("json_schema").is_none());

        let schema = json!({ "type": "object", "required": ["path"] });
        let js = t.build_body(&req_text("x", Effort::default(), Some(schema.clone()))).unwrap();
        assert_eq!(js["json_schema"], schema);
        assert!(js.get("grammar").is_none());
    }

    #[test]
    fn caps_report_vision() {
        assert!(tier().caps().vision);
    }

    /// Live end-to-end against a running llama-server. Run with:
    ///   cargo test -p keel-adapters -- --ignored
    #[tokio::test]
    #[ignore = "requires a live llama-server on 127.0.0.1:8080"]
    async fn live_generate() {
        let t = tier();
        let req = req_text("Reply with the single word: ready", Effort { n: 1, thinking: Some("low".into()) }, None);
        let res = t.generate(req, &Context::default()).await.expect("live generate");
        assert!(!res.content.is_empty() || res.reasoning_content.is_some());
        assert_eq!(res.tier, "local");
        assert_eq!(res.cost, 0.0); // local is electricity
    }
}
