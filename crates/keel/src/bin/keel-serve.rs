//! keel-serve (L5) — expose KEEL as an **OpenAI-compatible HTTP server** (canon §15), so any app
//! (DAVE, TERMINAL, the C#/Python fleet) consumes KEEL as a brain over protocol and gets the whole
//! economy for free: tier routing, the invariant chain (audit/privacy/cost), the ledger, and the
//! self-resolving substrate. One spine is assembled at startup and shared across requests.
//!
//!   keel-serve [keel.lock]      →  POST http://127.0.0.1:7070/v1/chat/completions

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use keel::Assembled;
use keel_contracts::{Content, Effort, GenerateRequest, GenerateResult, Message, Role};
use keel_kernel::{new_context, Manifest};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

const ADDR: &str = "127.0.0.1:7070";

#[derive(Clone)]
struct AppState {
    asm: Arc<Assembled>,
    manifest: Arc<Manifest>,
}

#[tokio::main]
async fn main() {
    let manifest_path = std::env::args().nth(1).unwrap_or_else(|| "keel.lock".to_string());
    let manifest = Manifest::load(&manifest_path).unwrap_or_else(|e| fail(format!("manifest: {e}")));
    eprintln!("[keel-serve] assembling spine (resolves/launches the substrate)…");
    let asm = keel::assemble(&manifest, None).unwrap_or_else(|e| fail(format!("assemble: {e}")));
    eprintln!("[keel-serve] tier={} model={}", asm.tier_name, asm.model);

    let state = AppState { asm: Arc::new(asm), manifest: Arc::new(manifest) };
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap_or_else(|e| fail(format!("bind {ADDR}: {e}")));
    eprintln!("[keel-serve] OpenAI-compatible endpoint → http://{ADDR}/v1/chat/completions");
    axum::serve(listener, app).await.unwrap();
}

fn fail(msg: String) -> ! {
    eprintln!("keel-serve: {msg}");
    std::process::exit(1);
}

async fn models(State(st): State<AppState>) -> impl IntoResponse {
    Json(json!({ "object": "list", "data": [ { "id": st.asm.model, "object": "model", "owned_by": "keel" } ] }))
}

#[derive(Deserialize)]
struct ChatRequest {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    messages: Vec<ChatMessage>,
    /// KEEL extension: route this turn through the model's thinking mode.
    #[serde(default)]
    think: Option<bool>,
}

#[derive(Deserialize)]
struct ChatMessage {
    role: String,
    #[serde(default)]
    content: Value, // string or array of parts
}

async fn chat(State(st): State<AppState>, Json(body): Json<ChatRequest>) -> impl IntoResponse {
    if body.messages.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": { "message": "no messages" } }))).into_response();
    }
    let req = to_request(&body, &st.asm.model);
    let ctx = new_context(&st.manifest);
    match st.asm.chain.run(req, &ctx, st.asm.tier.clone()).await {
        Ok(res) => Json(openai_response(&res, &ctx.trace_id)).into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": { "message": e.to_string(), "code": e.code() } })),
        )
            .into_response(),
    }
}

fn to_request(body: &ChatRequest, default_model: &str) -> GenerateRequest {
    let messages = body
        .messages
        .iter()
        .map(|m| Message {
            role: parse_role(&m.role),
            content: parse_content(&m.content),
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        })
        .collect();
    GenerateRequest {
        messages,
        model: body.model.clone().unwrap_or_else(|| default_model.to_string()),
        tools: vec![],
        grammar: None,
        // default to lean (content-forward, like a normal OpenAI endpoint); `think:true` opts into reasoning
        effort: Effort { n: 1, thinking: Some(if body.think.unwrap_or(false) { "high" } else { "low" }.to_string()) },
        cache_prefix_len: None,
    }
}

fn parse_role(r: &str) -> Role {
    match r {
        "system" => Role::System,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User,
    }
}

fn parse_content(v: &Value) -> Vec<Content> {
    match v {
        Value::String(s) => vec![Content::Text { text: s.clone() }],
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| match p.get("type").and_then(Value::as_str) {
                Some("text") => p.get("text").and_then(Value::as_str).map(|s| Content::Text { text: s.to_string() }),
                Some("image_url") => p
                    .get("image_url")
                    .and_then(|i| i.get("url"))
                    .and_then(Value::as_str)
                    .map(|s| Content::Image { source: s.to_string() }),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

fn openai_response(res: &GenerateResult, id: &str) -> Value {
    let mut message = json!({ "role": "assistant", "content": res.content });
    if let Some(rc) = &res.reasoning_content {
        message["reasoning_content"] = json!(rc);
    }
    json!({
        "id": format!("chatcmpl-{id}"),
        "object": "chat.completion",
        "model": res.model,
        "choices": [ { "index": 0, "message": message, "finish_reason": "stop" } ],
        "usage": {
            "prompt_tokens": res.usage.input_tokens,
            "completion_tokens": res.usage.output_tokens,
            "total_tokens": res.usage.input_tokens + res.usage.output_tokens
        },
        "keel": { "tier": res.tier, "cost": res.cost }
    })
}
