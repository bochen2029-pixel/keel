//! keel — the daily-driver CLI (L5). The Stage-0 capstone: it assembles the spine from
//! `keel.lock` (kernel `manifest` → `registry` with a `local_llama` adapter → the invariant
//! `chain` of audit · privacy · cost), mints a `Context`, runs one prompt through to the real
//! tier, logs every call to the file ledger, and prints the answer.
//!
//!   keel "read me the config"
//!   keel --think "weigh the tradeoffs"
//!   keel --manifest C:\KEEL\keel.lock "hello"

use keel_adapters::LocalLlama;
use keel_contracts::{Content, Effort, GenerateRequest, KeelError, Message, Role};
use keel_kernel::{new_context, Chain, Manifest, Registry};
use keel_middleware::{AuditMiddleware, AuditSink, CostMiddleware, FileAuditSink, PrivacyMiddleware, Redactor};
use std::sync::Arc;

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8080";
const AUDIT_LEDGER: &str = ".keelstate/audit.jsonl";

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("keel: {e}");
        std::process::exit(1);
    }
}

async fn run() -> keel_contracts::Result<()> {
    // ── args: keel [--manifest PATH] [--think] <prompt...> ──
    let mut manifest_path = "keel.lock".to_string();
    let mut think = false;
    let mut prompt = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().unwrap_or_default(),
            "--think" => think = true,
            _ => prompt.push(a),
        }
    }
    let prompt = prompt.join(" ");
    if prompt.trim().is_empty() {
        eprintln!("usage: keel [--manifest PATH] [--think] <prompt>");
        std::process::exit(2);
    }

    // ── assemble the spine from the manifest ──
    let manifest = Manifest::load(&manifest_path)?;
    let tier_name = manifest.router.default_tier.clone();
    let tcfg = manifest
        .tier(&tier_name)
        .ok_or_else(|| KeelError::Other(format!("no tier '{tier_name}' in {manifest_path}")))?;
    let endpoint = tcfg.endpoint.clone().unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());

    let adapter = Arc::new(
        LocalLlama::new(endpoint, tcfg.model.clone(), tier_name.clone(), tcfg.price.to_price(), tcfg.vision)
            .with_max_tokens(2048),
    );
    let mut registry = Registry::new();
    registry.register(tier_name.clone(), adapter);

    // the invariant chain: audit (→ file ledger) · privacy (local ⇒ pass-through) · cost (hard-stop)
    let sink: Arc<dyn AuditSink> = Arc::new(
        FileAuditSink::new(AUDIT_LEDGER).map_err(|e| KeelError::Other(format!("ledger {AUDIT_LEDGER}: {e}")))?,
    );
    let chain = Chain::new(vec![
        Arc::new(AuditMiddleware::new(sink)),
        Arc::new(PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), false)),
        Arc::new(CostMiddleware::new(manifest.cost.hard_stop_at)),
    ]);

    // ── run one turn ──
    let ctx = new_context(&manifest);
    let req = GenerateRequest {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text { text: prompt }],
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        }],
        model: tcfg.model.clone(),
        tools: vec![],
        grammar: None,
        effort: Effort { n: 1, thinking: Some((if think { "high" } else { "low" }).into()) },
        cache_prefix_len: None,
    };
    let tier = registry.get(&tier_name)?;
    let res = chain.run(req, &ctx, tier).await?;

    // ── output ──
    println!("{}", res.content.trim());
    if think {
        if let Some(rc) = res.reasoning_content.as_deref() {
            if !rc.trim().is_empty() {
                eprintln!("\n[thinking] {}", rc.trim());
            }
        }
    }
    eprintln!(
        "\n— tier={} model={} cost=${:.4} tokens={}+{} · trace {} · audit→{}",
        res.tier,
        res.model,
        res.cost,
        res.usage.input_tokens,
        res.usage.output_tokens,
        ctx.trace_id,
        AUDIT_LEDGER
    );
    Ok(())
}
