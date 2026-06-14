//! keel-serve (L5) — expose KEEL as an **OpenAI-compatible HTTP server** (canon §15), so any app
//! (DAVE, TERMINAL, the C#/Python fleet) consumes KEEL as a brain over protocol and gets the whole
//! economy for free: **router-driven tier selection**, the invariant chain (audit/privacy/cost),
//! the ledger, and the self-resolving substrate. One [`Engine`](keel::Engine) is assembled at
//! startup and shared across requests; every request is routed (it is not pinned to one tier).
//!
//!   keel-serve [keel.lock]      →  POST http://127.0.0.1:7070/v1/chat/completions

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use keel::{Engine, Outcome};
use keel_contracts::{Content, DataClass, Effort, GenerateRequest, Kind, Message, Role, Step, Trust};
use keel_kernel::{new_context, Manifest};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

const ADDR: &str = "127.0.0.1:7070";

#[derive(Clone)]
struct AppState {
    engine: Arc<Engine>,
    manifest: Arc<Manifest>,
}

#[tokio::main]
async fn main() {
    let manifest_path = std::env::args().nth(1).unwrap_or_else(|| "keel.lock".to_string());
    let manifest = Manifest::load(&manifest_path).unwrap_or_else(|e| fail(format!("manifest: {e}")));
    eprintln!("[keel-serve] assembling engine (resolves/launches the substrate, wires available tiers)…");
    let engine = Engine::assemble(&manifest).unwrap_or_else(|e| fail(format!("assemble: {e}")));
    eprintln!("[keel-serve] tiers={:?}", engine.available());

    let state = AppState { engine: Arc::new(engine), manifest: Arc::new(manifest) };
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

/// Advertise the wired tiers as selectable "models" (the router still has final say per turn).
async fn models(State(st): State<AppState>) -> impl IntoResponse {
    let data: Vec<Value> = st
        .engine
        .available()
        .into_iter()
        .map(|t| json!({ "id": t, "object": "model", "owned_by": "keel" }))
        .collect();
    Json(json!({ "object": "list", "data": data }))
}

#[derive(Deserialize)]
struct ChatRequest {
    #[serde(default)]
    messages: Vec<ChatMessage>,
    /// KEEL extension: this turn is core-wire reasoning (else scaffolding) — a routing hint.
    #[serde(default)]
    kind: Option<String>,
    /// KEEL extension: sovereign data — forces the local tier (I3).
    #[serde(default)]
    sovereign: Option<bool>,
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
    let messages: Vec<Message> = body
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

    // The routing Step. Its `content` mirrors the request's modalities, so a request carrying an
    // image routes local by itself (I3, perception is sovereign-by-default) — no caller opt-in.
    let step = Step {
        kind: if matches!(body.kind.as_deref(), Some("core-wire") | Some("core_wire")) { Kind::CoreWire } else { Kind::Scaffolding },
        ty: "api_turn".into(),
        trust_required: Trust::Normal,
        data_class: if body.sovereign.unwrap_or(false) { DataClass::Sovereign } else { DataClass::Normal },
        tier_history: vec![],
        oracle_failures: 0,
        projected_cost: None,
        critical: false,
        source: Some("serve".into()),
        content: messages.iter().flat_map(|m| m.content.clone()).collect(),
        golden_refs: vec![],
    };
    let req = GenerateRequest {
        messages,
        model: String::new(), // the engine sets the routed tier's model
        tools: vec![],
        grammar: None,
        // default to lean; `think:true` opts into reasoning; otherwise the router decides per tier.
        effort: Effort { n: 1, thinking: if body.think.unwrap_or(false) { Some("high".into()) } else { None } },
        cache_prefix_len: None,
    };

    let ctx = new_context(&st.manifest);
    match st.engine.run(&step, &ctx, req).await {
        Ok(outcome) => Json(openai_response(&outcome, &ctx.trace_id)).into_response(),
        Err(e) => {
            let code = if e.code() == "BUDGET_EXCEEDED" { StatusCode::PAYMENT_REQUIRED } else { StatusCode::BAD_GATEWAY };
            (code, Json(json!({ "error": { "message": e.to_string(), "code": e.code() } }))).into_response()
        }
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

/// Map the routed `Outcome` to an OpenAI chat-completion, exposing the routing story under `keel`.
fn openai_response(outcome: &Outcome, id: &str) -> Value {
    let res = &outcome.result;
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
        "keel": {
            "tier": res.tier,
            "cost": res.cost,
            "route": outcome.decision.reason,
            "requested_tier": outcome.decision.tier,
            "substituted": outcome.substituted
        }
    })
}
