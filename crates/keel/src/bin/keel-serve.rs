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
    /// A7.4: at most one background maintenance pass at a time (a compare-exchange gate).
    maintenance_busy: Arc<std::sync::atomic::AtomicBool>,
}

#[tokio::main]
async fn main() {
    let manifest_path = std::env::args().nth(1).unwrap_or_else(|| "keel.lock".to_string());
    let manifest = Manifest::load(&manifest_path).unwrap_or_else(|e| fail(format!("manifest: {e}")));
    eprintln!("[keel-serve] assembling engine (resolves/launches the substrate, wires available tiers)…");
    let engine = Engine::assemble(&manifest).unwrap_or_else(|e| fail(format!("assemble: {e}")));
    eprintln!("[keel-serve] tiers={:?}", engine.available());

    let state = AppState {
        engine: Arc::new(engine),
        manifest: Arc::new(manifest),
        maintenance_busy: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat))
        .route("/v1/audio/transcriptions", post(transcribe))
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
    /// KEEL extension (I5): this turn's wrong output would corrupt the operator's substrate, so it
    /// must carry a correctness assertion — a fired `golden_ref` or a domain oracle — or it fails closed.
    #[serde(default)]
    critical: Option<bool>,
    /// KEEL extension (I5): operator-frozen golden-case names this turn asserts against. An unresolved
    /// name fails closed; a resolved schema/property ref gates by family (the "schema-valid or reject" gate).
    #[serde(default)]
    golden_refs: Vec<String>,
    /// KEEL extension: constrained decode — a GBNF string or a JSON-schema object the local tier
    /// enforces at decode time (the path that lets a turn emit schema-valid output, then pass via a ref).
    #[serde(default)]
    grammar: Option<Value>,
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

    // I5 + constrained-decode extensions — parity with the CLI's --critical/--golden-ref, plus a
    // `grammar` path serve gets first. Moved out of `body` before the literals; the remaining fields
    // (kind/sovereign/think) stay readable (disjoint partial move).
    let critical = body.critical.unwrap_or(false);
    let golden_refs = body.golden_refs;
    let grammar = body.grammar;

    // The routing Step. Its `content` mirrors the request's modalities, so a request carrying an
    // image routes local by itself (I3, perception is sovereign-by-default) — no caller opt-in.
    let mut step = Step {
        kind: if matches!(body.kind.as_deref(), Some("core-wire") | Some("core_wire")) { Kind::CoreWire } else { Kind::Scaffolding },
        ty: "api_turn".into(),
        trust_required: Trust::Normal,
        data_class: if body.sovereign.unwrap_or(false) { DataClass::Sovereign } else { DataClass::Normal },
        tier_history: vec![],
        oracle_failures: 0,
        projected_cost: None,
        critical,
        source: Some("serve".into()),
        content: messages.iter().flat_map(|m| m.content.clone()).collect(),
        golden_refs,
    };
    let req = GenerateRequest {
        messages,
        model: String::new(), // the engine sets the routed tier's model
        tools: vec![],
        grammar, // constrained decode (GBNF / JSON-schema), enforced by the local tier at decode time
        // default to lean; `think:true` opts into reasoning; otherwise the router decides per tier.
        effort: Effort { n: 1, thinking: if body.think.unwrap_or(false) { Some("high".into()) } else { None } },
        cache_prefix_len: None,
    };

    let mut ctx = new_context(&st.manifest);
    match st.engine.run(&mut step, &mut ctx, req).await {
        Ok(outcome) => {
            // A7.4: fire any due memory maintenance in the BACKGROUND (response latency stays
            // clean); the compare-exchange gate keeps passes serialized, and a pass is bounded
            // (at most one consolidation + one cold-eyes).
            use std::sync::atomic::Ordering;
            if st.maintenance_busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                let st2 = st.clone();
                tokio::spawn(async move {
                    let mem = keel::build_memory(&st2.manifest, "", Some(12));
                    let mut mctx = new_context(&st2.manifest);
                    keel::run_maintenance(&st2.engine, &mem, &st2.manifest, &mut mctx, false).await;
                    st2.maintenance_busy.store(false, Ordering::SeqCst);
                });
            }
            Json(openai_response(&outcome, &ctx.trace_id)).into_response()
        }
        Err(e) => {
            let code = if e.code() == "BUDGET_EXCEEDED" { StatusCode::PAYMENT_REQUIRED } else { StatusCode::BAD_GATEWAY };
            (code, Json(json!({ "error": { "message": e.to_string(), "code": e.code() } }))).into_response()
        }
    }
}

/// `/v1/audio/transcriptions` — the ears over protocol (D1, canon §12/§15). **Sidecar-local by
/// design:** the request carries a *path* on the shared filesystem, not an upload — a localhost
/// consumer and KEEL see the same disk, so a two-hour WAV never crosses the wire and raw audio
/// never leaves the box (I3; the server binds 127.0.0.1 only). The reply is OpenAI-verbose_json-
/// shaped (`text` + `segments[]` with float seconds) plus millisecond offsets as a keel extension.
#[derive(Deserialize)]
struct TranscribeRequest {
    /// Absolute path to a 16 kHz mono PCM WAV on the shared filesystem.
    path: String,
}

async fn transcribe(State(st): State<AppState>, Json(body): Json<TranscribeRequest>) -> impl IntoResponse {
    let (Some(cli), Some(model)) = (st.manifest.whisper_cli(), st.manifest.whisper_model_path()) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": { "message": "keel.lock servers.whisper {path, exe} + substrate.audio.file required", "code": "SUBSTRATE_UNRESOLVED" } })),
        )
            .into_response();
    };
    let wav = std::path::PathBuf::from(&body.path);
    if !wav.is_file() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": { "message": format!("no such audio file: {}", body.path), "code": "BAD_REQUEST" } })),
        )
            .into_response();
    }
    let w = keel_adapters::Whisper::new(cli, model);
    match w.transcribe_segments(&wav).await {
        Ok(segments) => {
            let text = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
            let segs: Vec<Value> = segments
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    json!({
                        "id": i,
                        "start": s.start_ms as f64 / 1000.0,
                        "end": s.end_ms as f64 / 1000.0,
                        "start_ms": s.start_ms,
                        "end_ms": s.end_ms,
                        "text": s.text
                    })
                })
                .collect();
            Json(json!({ "text": text, "segments": segs })).into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": { "message": e.to_string(), "code": e.code() } })),
        )
            .into_response(),
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
            "substituted": outcome.substituted,
            "verdict_passed": outcome.verdict.passed,
            "joint_wrong": outcome.verdict.joint_wrong
        }
    })
}

#[cfg(test)]
mod tests {
    use super::ChatRequest;

    // serve↔CLI parity: the over-protocol request carries the I5 + constrained-decode extensions.
    #[test]
    fn parses_i5_and_grammar_extensions() {
        let body: ChatRequest = serde_json::from_str(
            r#"{ "messages": [{"role":"user","content":"hi"}],
                 "kind": "core-wire", "sovereign": true,
                 "critical": true, "golden_refs": ["dir-schema"],
                 "grammar": {"type":"object","required":["path"]} }"#,
        )
        .unwrap();
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.critical, Some(true));
        assert_eq!(body.golden_refs, vec!["dir-schema".to_string()]);
        assert!(body.grammar.is_some());
    }

    // absent extensions default cleanly — a plain OpenAI request still works (no critical, no refs, no grammar).
    #[test]
    fn extensions_default_when_absent() {
        let body: ChatRequest = serde_json::from_str(r#"{ "messages": [] }"#).unwrap();
        assert_eq!(body.critical, None);
        assert!(body.golden_refs.is_empty());
        assert!(body.grammar.is_none());
    }
}
