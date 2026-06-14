//! anthropic — the frontier tier (canon §4): premier intelligence (Claude Opus). Anthropic speaks
//! its **own** Messages API, not OpenAI Chat Completions — so this adapter is the "thin gateway"
//! (canon §3): it maps KEEL's neutral `GenerateRequest` ↔ Anthropic Messages (`x-api-key` auth,
//! `anthropic-version` header, top-level `system`, content-block responses). Cost is real money;
//! extended thinking is opt-in via `Effort.thinking`.

use async_trait::async_trait;
use keel_contracts::{
    compute_cost, Capabilities, Content, Context, Effort, GenerateRequest, GenerateResult,
    KeelError, ModelTier, Price, Result, Role, ToolCall, Usage,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};

const ANTHROPIC_VERSION: &str = "2023-06-01";

/// An Anthropic-backed frontier tier.
pub struct Anthropic {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    tier: String,
    price: Price,
    api_key: String,
    max_tokens: u32,
}

impl Anthropic {
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

    fn build_body(&self, req: &GenerateRequest) -> Result<Value> {
        // Anthropic: system is a top-level string; messages are user/assistant only.
        let mut system = String::new();
        let mut messages = Vec::new();
        for m in &req.messages {
            match m.role {
                Role::System => {
                    let t = text_of(&m.content);
                    if !system.is_empty() {
                        system.push('\n');
                    }
                    system.push_str(&t);
                }
                Role::Assistant => messages.push(anthropic_message("assistant", &m.content)?),
                Role::User | Role::Tool => messages.push(anthropic_message("user", &m.content)?),
            }
        }

        let mut body = Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("messages".into(), Value::Array(messages));
        if !system.is_empty() {
            body.insert("system".into(), json!(system));
        }
        // extended thinking (opt-in): high/max → enabled with a budget; max_tokens must exceed it.
        match thinking_budget(&req.effort) {
            Some(budget) => {
                body.insert("thinking".into(), json!({ "type": "enabled", "budget_tokens": budget }));
                body.insert("max_tokens".into(), json!(self.max_tokens.max(budget + 4096)));
            }
            None => {
                body.insert("max_tokens".into(), json!(self.max_tokens));
            }
        }
        Ok(Value::Object(body))
    }

    fn parse(&self, resp: AnthropicResponse) -> GenerateResult {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = Vec::new();
        for block in &resp.content {
            match block.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(t) = block.get("text").and_then(Value::as_str) {
                        content.push_str(t);
                    }
                }
                Some("thinking") => {
                    if let Some(t) = block.get("thinking").and_then(Value::as_str) {
                        reasoning.push_str(t);
                    }
                }
                Some("tool_use") => tool_calls.push(ToolCall {
                    id: block.get("id").and_then(Value::as_str).unwrap_or_default().to_string(),
                    kind: "function".to_string(),
                    name: block.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
                    arguments: block.get("input").map(ToString::to_string).unwrap_or_default(),
                }),
                _ => {}
            }
        }
        let usage = Usage {
            input_tokens: resp.usage.input_tokens,
            output_tokens: resp.usage.output_tokens,
            cache_hit_tokens: resp.usage.cache_read_input_tokens,
        };
        GenerateResult {
            content,
            tool_calls,
            reasoning_content: if reasoning.is_empty() { None } else { Some(reasoning) },
            cost: compute_cost(&usage, &self.price),
            usage,
            tier: self.tier.clone(),
            model: if resp.model.is_empty() { self.model.clone() } else { resp.model },
        }
    }
}

#[async_trait]
impl ModelTier for Anthropic {
    fn caps(&self) -> Capabilities {
        Capabilities { vision: true, video: false, thinking: true }
    }

    async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
        let body = self.build_body(&req)?;
        let resp = self
            .client
            .post(format!("{}/v1/messages", self.endpoint))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| KeelError::TierUnavailable(format!("anthropic post: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            let msg = format!("anthropic HTTP {}: {}", status.as_u16(), txt);
            return Err(if status.is_server_error() {
                KeelError::TierUnavailable(msg)
            } else {
                KeelError::Other(msg)
            });
        }
        let ar: AnthropicResponse = resp
            .json()
            .await
            .map_err(|e| KeelError::Other(format!("anthropic decode: {e}")))?;
        Ok(self.parse(ar))
    }
}

/// `Effort.thinking` → extended-thinking budget (None ⇒ no thinking).
fn thinking_budget(effort: &Effort) -> Option<u32> {
    match effort.thinking.as_deref() {
        Some("high") | Some("max") => Some(8192),
        _ => None,
    }
}

fn text_of(content: &[Content]) -> String {
    content
        .iter()
        .filter_map(|c| match c {
            Content::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn anthropic_message(role: &str, content: &[Content]) -> Result<Value> {
    if let [Content::Text { text }] = content {
        return Ok(json!({ "role": role, "content": text }));
    }
    let mut blocks = Vec::with_capacity(content.len());
    for c in content {
        match c {
            Content::Text { text } => blocks.push(json!({ "type": "text", "text": text })),
            Content::Image { source } => blocks.push(image_block(source)?),
            Content::Clip { .. } | Content::Audio { .. } => {
                return Err(KeelError::Other("anthropic: clip/audio must be pre-processed before cognition".into()))
            }
        }
    }
    Ok(json!({ "role": role, "content": blocks }))
}

/// KEEL `Image.source` → an Anthropic image block (base64 data-uri or url).
fn image_block(source: &str) -> Result<Value> {
    if let Some(rest) = source.strip_prefix("data:") {
        if let Some((meta, data)) = rest.split_once(',') {
            let media_type = meta.split(';').next().unwrap_or("image/png");
            return Ok(json!({ "type": "image", "source": { "type": "base64", "media_type": media_type, "data": data } }));
        }
    }
    if source.starts_with("http://") || source.starts_with("https://") {
        return Ok(json!({ "type": "image", "source": { "type": "url", "url": source } }));
    }
    Err(KeelError::Other("anthropic: image must be a data-uri or http(s) url".into()))
}

#[derive(Debug, Default, Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<Value>,
    #[serde(default)]
    usage: AnthropicUsage,
    #[serde(default)]
    model: String,
}

#[derive(Debug, Default, Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::Message;

    fn tier() -> Anthropic {
        Anthropic::new(
            "https://api.anthropic.com/",
            "claude-opus-4-8",
            "frontier",
            Price { input_miss: 5.0, input_hit: 0.5, output: 25.0 },
            "test-key",
        )
    }

    fn msg(role: Role, text: &str) -> Message {
        Message { role, content: vec![Content::Text { text: text.into() }], name: None, reasoning_content: None, tool_call_id: None }
    }

    fn req(messages: Vec<Message>, thinking: Option<&str>) -> GenerateRequest {
        GenerateRequest {
            messages,
            model: "claude-opus-4-8".into(),
            tools: vec![],
            grammar: None,
            effort: Effort { n: 1, thinking: thinking.map(str::to_string) },
            cache_prefix_len: None,
        }
    }

    #[test]
    fn system_is_hoisted_top_level_and_user_is_a_string() {
        let body = tier()
            .build_body(&req(vec![msg(Role::System, "be terse"), msg(Role::User, "hi")], None))
            .unwrap();
        assert_eq!(body["system"], "be terse");
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1); // system is NOT a message
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "hi");
        assert_eq!(body["max_tokens"], 4096);
        assert!(body.get("thinking").is_none());
    }

    #[test]
    fn thinking_high_enables_with_budget_and_bumps_max_tokens() {
        let body = tier().build_body(&req(vec![msg(Role::User, "hard")], Some("high"))).unwrap();
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["thinking"]["budget_tokens"], 8192);
        assert_eq!(body["max_tokens"], 8192 + 4096); // max_tokens must exceed the budget
    }

    #[test]
    fn image_data_uri_becomes_a_base64_block() {
        let b = image_block("data:image/png;base64,AAAA").unwrap();
        assert_eq!(b["type"], "image");
        assert_eq!(b["source"]["type"], "base64");
        assert_eq!(b["source"]["media_type"], "image/png");
        assert_eq!(b["source"]["data"], "AAAA");
        assert!(image_block("/local/path.png").is_err());
    }

    #[test]
    fn parses_content_blocks_and_real_cost() {
        let json_str = r#"{
            "content": [
                {"type":"thinking","thinking":"hmm"},
                {"type":"text","text":"Hello."},
                {"type":"tool_use","id":"t1","name":"read","input":{"path":"x"}}
            ],
            "usage": {"input_tokens": 1000, "output_tokens": 500, "cache_read_input_tokens": 0},
            "model": "claude-opus-4-8"
        }"#;
        let ar: AnthropicResponse = serde_json::from_str(json_str).unwrap();
        let r = tier().parse(ar);
        assert_eq!(r.content, "Hello.");
        assert_eq!(r.reasoning_content.as_deref(), Some("hmm"));
        assert_eq!(r.tool_calls.len(), 1);
        assert_eq!(r.tool_calls[0].name, "read");
        // 1000*5 + 500*25 = 5000 + 12500 = 17500 / 1e6 = $0.0175
        assert!((r.cost - 0.0175).abs() < 1e-6);
        assert_eq!(r.tier, "frontier");
    }

    /// Live against the real Anthropic API (spends real money). Run with the key in env:
    ///   cargo test -p keel-adapters anthropic -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "hits the real Anthropic API; needs ANTHROPIC_API_KEY"]
    async fn live_generate() {
        let key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        assert!(!key.is_empty(), "ANTHROPIC_API_KEY not set");
        let t = Anthropic::new(
            "https://api.anthropic.com",
            "claude-opus-4-8",
            "frontier",
            Price { input_miss: 5.0, input_hit: 0.5, output: 25.0 },
            key,
        )
        .with_max_tokens(64);
        let res = t
            .generate(req(vec![msg(Role::User, "Reply with exactly the word: pong")], None), &Context::default())
            .await
            .expect("live anthropic");
        eprintln!("content={:?} usage={:?} cost=${:.6}", res.content, res.usage, res.cost);
        assert!(!res.content.is_empty());
        assert_eq!(res.tier, "frontier");
        assert!(res.cost > 0.0);
    }
}
