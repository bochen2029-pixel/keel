//! keel — the daily-driver CLI (L5). The Stage-0 capstone: it assembles the spine from
//! `keel.lock` (kernel `manifest` → `registry` with the tier's adapter → the invariant `chain`
//! of audit · privacy · cost), mints a `Context`, runs one prompt through to the tier, logs every
//! call to the file ledger, and prints the answer.
//!
//!   keel "read me the config"
//!   keel --tier cheap-API "weigh the tradeoffs"   # DeepSeek: real cost, I3 egress mask on
//!   keel --think "..."        keel --manifest C:\KEEL\keel.lock "hello"

use keel_adapters::{DeepSeek, LocalLlama};
use keel_contracts::{Content, Effort, GenerateRequest, KeelError, Message, ModelTier, Role};
use keel_kernel::{new_context, Chain, Manifest, Registry};
use keel_middleware::{AuditMiddleware, AuditSink, CostMiddleware, FileAuditSink, PrivacyMiddleware, Redactor};
use std::sync::Arc;

const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com";
const AUDIT_LEDGER: &str = ".keelstate/audit.jsonl";

// local substrate launch (paths match keel.lock; keel.lock-driven config is a refinement)
const LLAMA_EXE: &str = r"C:\llama.cpp\llama-server.exe";
const LLAMA_MODEL: &str = r"C:\models\Qwen3.5-9B-Q5_K_M.gguf";
const LLAMA_MMPROJ: &str = r"C:\models\mmproj-F16.gguf";
const LLAMA_LOG: &str = ".keelstate/llama-server.log";

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("keel: {e}");
        std::process::exit(1);
    }
}

async fn run() -> keel_contracts::Result<()> {
    // ── args: keel [--manifest PATH] [--tier NAME] [--think] <prompt...> ──
    let mut manifest_path = "keel.lock".to_string();
    let mut tier_override: Option<String> = None;
    let mut think = false;
    let mut prompt = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().unwrap_or_default(),
            "--tier" => tier_override = args.next(),
            "--think" => think = true,
            _ => prompt.push(a),
        }
    }
    let prompt = prompt.join(" ");
    if prompt.trim().is_empty() {
        eprintln!("usage: keel [--manifest PATH] [--tier NAME] [--think] <prompt>");
        std::process::exit(2);
    }

    // ── assemble the spine from the manifest ──
    let manifest = Manifest::load(&manifest_path)?;
    let tier_name = tier_override.unwrap_or_else(|| manifest.router.default_tier.clone());
    let tcfg = manifest
        .tier(&tier_name)
        .ok_or_else(|| KeelError::Other(format!("no tier '{tier_name}' in {manifest_path}")))?;

    // wiring reads the manifest's adapter name → builds the L2 adapter (the kernel never imports L2)
    let adapter: Arc<dyn ModelTier> = match tcfg.adapter.as_str() {
        "local_llama" => {
            // resolve the substrate: explicit endpoint → probe running servers (c1) → launch (c2)
            let endpoint = match tcfg.endpoint.clone() {
                Some(e) => e,
                None => match keel_kernel::resolve_endpoint(&keel_kernel::default_local_candidates()) {
                    Ok(e) => {
                        eprintln!("[keel] local substrate → {e}");
                        e
                    }
                    Err(_) => {
                        eprintln!("[keel] no server up — cold-starting llama-server (first call loads the model)…");
                        let mut cfg = keel_kernel::LlamaServerConfig::new(LLAMA_EXE, LLAMA_MODEL);
                        cfg.mmproj = Some(LLAMA_MMPROJ.to_string());
                        cfg.log_path = Some(LLAMA_LOG.to_string());
                        let server = keel_kernel::launch(&cfg)?;
                        let ep = server.endpoint().to_string();
                        eprintln!("[keel] llama-server ready → {ep} (pid {})", server.pid());
                        ep // handle drops here; the process keeps running (detached for reuse)
                    }
                },
            };
            Arc::new(
                LocalLlama::new(endpoint, tcfg.model.clone(), tier_name.clone(), tcfg.price.to_price(), tcfg.vision)
                    .with_max_tokens(2048),
            )
        }
        "deepseek" => {
            let key = tcfg.api_key().ok_or_else(|| {
                KeelError::Other(format!(
                    "tier '{tier_name}' needs env var {}",
                    tcfg.api_key_env.as_deref().unwrap_or("?")
                ))
            })?;
            let endpoint = tcfg.endpoint.clone().unwrap_or_else(|| DEEPSEEK_ENDPOINT.to_string());
            Arc::new(
                DeepSeek::new(endpoint, tcfg.model.clone(), tier_name.clone(), tcfg.price.to_price(), key)
                    .with_max_tokens(2048),
            )
        }
        other => return Err(KeelError::Other(format!("unknown adapter '{other}' for tier '{tier_name}'"))),
    };
    let mut registry = Registry::new();
    registry.register(tier_name.clone(), adapter);

    // the invariant chain: audit (→ ledger) · privacy (cloud tiers ⇒ egress mask) · cost (hard-stop)
    let egress = tcfg.adapter != "local_llama"; // local is sovereign-safe; cloud tiers get the I3 mask
    let sink: Arc<dyn AuditSink> = Arc::new(
        FileAuditSink::new(AUDIT_LEDGER).map_err(|e| KeelError::Other(format!("ledger {AUDIT_LEDGER}: {e}")))?,
    );
    let chain = Chain::new(vec![
        Arc::new(AuditMiddleware::new(sink)),
        Arc::new(PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), egress)),
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
