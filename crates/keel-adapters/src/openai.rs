//! The shared OpenAI Chat Completions mapping (canon §3). Every tier speaks this protocol, so
//! the request-shaping and response-parsing live here once; each provider module adds only its
//! compat specifics. Pure functions + plain structs — unit-tested without a live endpoint.

use keel_contracts::{
    compute_cost, Content, GenerateRequest, GenerateResult, KeelError, Message, Price, Result, Role,
    ToolCall, ToolDef, Usage,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};

// ── request shaping ──────────────────────────────────────────────────────────

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

/// Multi-part `Content` → OpenAI message content: a bare string for a single text part, else an
/// array of typed parts. Audio/video must be pre-processed (whisper transcript / sampled frames)
/// before the cognition tier — they are an error here, never silently dropped.
fn map_content(parts: &[Content]) -> Result<Value> {
    if parts.is_empty() {
        return Ok(json!(""));
    }
    if let [Content::Text { text }] = parts {
        return Ok(json!(text));
    }
    let mut arr = Vec::with_capacity(parts.len());
    for p in parts {
        match p {
            Content::Text { text } => arr.push(json!({ "type": "text", "text": text })),
            Content::Image { source } => {
                arr.push(json!({ "type": "image_url", "image_url": { "url": source } }))
            }
            Content::Clip { .. } | Content::Audio { .. } => {
                return Err(KeelError::Other(
                    "clip/audio must be pre-processed to frames/text before the cognition tier (canon §12)".into(),
                ))
            }
        }
    }
    Ok(Value::Array(arr))
}

fn map_messages(messages: &[Message]) -> Result<Vec<Value>> {
    let mut out = Vec::with_capacity(messages.len());
    for m in messages {
        let mut obj = Map::new();
        obj.insert("role".into(), json!(role_str(m.role)));
        obj.insert("content".into(), map_content(&m.content)?);
        if let Some(n) = &m.name {
            obj.insert("name".into(), json!(n));
        }
        if let Some(t) = &m.tool_call_id {
            obj.insert("tool_call_id".into(), json!(t));
        }
        // assistant reasoning must be replayed across tool turns or providers 400 (golden case)
        if let Some(r) = &m.reasoning_content {
            obj.insert("reasoning_content".into(), json!(r));
        }
        out.push(Value::Object(obj));
    }
    Ok(out)
}

fn tools_json(tools: &[ToolDef]) -> Value {
    Value::Array(
        tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": { "name": t.name, "description": t.description, "parameters": t.schema }
                })
            })
            .collect(),
    )
}

/// The protocol-common request body (model · messages · tools · max_tokens). Providers extend it.
pub fn base_body(req: &GenerateRequest, model: &str, max_tokens: u32) -> Result<Map<String, Value>> {
    let mut body = Map::new();
    body.insert("model".into(), json!(model));
    body.insert("messages".into(), Value::Array(map_messages(&req.messages)?));
    if !req.tools.is_empty() {
        body.insert("tools".into(), tools_json(&req.tools));
    }
    body.insert("max_tokens".into(), json!(max_tokens));
    Ok(body)
}

// ── response parsing ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
pub struct OaiResponse {
    #[serde(default)]
    pub choices: Vec<OaiChoice>,
    #[serde(default)]
    pub usage: OaiUsage,
}

#[derive(Debug, Default, Deserialize)]
pub struct OaiChoice {
    #[serde(default)]
    pub message: OaiMessage,
}

#[derive(Debug, Default, Deserialize)]
pub struct OaiMessage {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<OaiToolCall>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OaiToolCall {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub function: OaiFunction,
}

#[derive(Debug, Default, Deserialize)]
pub struct OaiFunction {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct OaiUsage {
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
    /// DeepSeek prompt_cache_hit_tokens — the 100× cache lever (absent on local).
    #[serde(default)]
    pub prompt_cache_hit_tokens: u64,
}

/// OpenAI response → uniform `GenerateResult`, with cost computed from usage × tier price.
pub fn parse_response(oai: OaiResponse, tier: &str, model: &str, price: &Price) -> GenerateResult {
    let msg = oai.choices.into_iter().next().map(|c| c.message).unwrap_or_default();
    let usage = Usage {
        input_tokens: oai.usage.prompt_tokens,
        output_tokens: oai.usage.completion_tokens,
        cache_hit_tokens: oai.usage.prompt_cache_hit_tokens,
    };
    let tool_calls = msg
        .tool_calls
        .into_iter()
        .map(|tc| ToolCall {
            id: tc.id,
            kind: if tc.kind.is_empty() { "function".into() } else { tc.kind },
            name: tc.function.name,
            arguments: tc.function.arguments,
        })
        .collect();
    GenerateResult {
        content: msg.content.unwrap_or_default(),
        tool_calls,
        reasoning_content: msg.reasoning_content,
        cost: compute_cost(&usage, price),
        usage,
        tier: tier.to_string(),
        model: model.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::Effort;

    fn user_text(s: &str) -> Message {
        Message {
            role: Role::User,
            content: vec![Content::Text { text: s.into() }],
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        }
    }

    fn req(messages: Vec<Message>) -> GenerateRequest {
        GenerateRequest { messages, model: "m".into(), tools: vec![], grammar: None, effort: Effort::default(), cache_prefix_len: None }
    }

    #[test]
    fn single_text_maps_to_a_string() {
        let body = base_body(&req(vec![user_text("hi")]), "qwen", 256).unwrap();
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "hi"); // bare string, not an array
        assert_eq!(body["model"], "qwen");
        assert_eq!(body["max_tokens"], 256);
    }

    #[test]
    fn image_maps_to_image_url_part() {
        let m = Message {
            role: Role::User,
            content: vec![
                Content::Text { text: "what is this".into() },
                Content::Image { source: "data:image/png;base64,AAAA".into() },
            ],
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        };
        let body = base_body(&req(vec![m]), "qwen", 256).unwrap();
        let parts = body["messages"][0]["content"].as_array().unwrap();
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[1]["type"], "image_url");
        assert_eq!(parts[1]["image_url"]["url"], "data:image/png;base64,AAAA");
    }

    #[test]
    fn audio_content_is_rejected() {
        let m = Message {
            role: Role::User,
            content: vec![Content::Audio { source: "mic.wav".into() }],
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        };
        assert!(base_body(&req(vec![m]), "qwen", 256).is_err());
    }

    #[test]
    fn parses_response_and_computes_cost() {
        // GOLDEN_MODEL_TIER: usage 1000/500/800 × deepseek price → cost 0.0005249
        let json_str = r#"{
            "choices": [{"message": {"content": "ok", "reasoning_content": "hmm",
                "tool_calls": [{"id":"c1","type":"function","function":{"name":"read","arguments":"{}"}}]}}],
            "usage": {"prompt_tokens": 1000, "completion_tokens": 500, "prompt_cache_hit_tokens": 800}
        }"#;
        let oai: OaiResponse = serde_json::from_str(json_str).unwrap();
        let price = Price { input_miss: 0.435, input_hit: 0.003625, output: 0.87 };
        let r = parse_response(oai, "cheap-API", "deepseek-v4-pro", &price);
        assert_eq!(r.content, "ok");
        assert_eq!(r.reasoning_content.as_deref(), Some("hmm"));
        assert_eq!(r.tool_calls.len(), 1);
        assert_eq!(r.tool_calls[0].name, "read");
        assert_eq!(r.usage.cache_hit_tokens, 800);
        assert!((r.cost - 0.0005249).abs() < 1e-6);
        assert_eq!(r.tier, "cheap-API");
    }
}
