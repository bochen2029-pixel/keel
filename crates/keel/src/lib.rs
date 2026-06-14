//! keel (L5) — the shared wiring both apps use: assemble the spine from `keel.lock` into a tier
//! adapter (resolving/cold-starting the local substrate) wrapped in the invariant chain
//! (audit → ledger · privacy · cost). The CLI (`keel`) and the OpenAI server (`keel-serve`) both
//! build on `assemble`, so there is one assembly path, not two.

use keel_adapters::{DeepSeek, LocalLlama};
use keel_contracts::{KeelError, ModelTier, Result};
use keel_kernel::{Chain, Manifest, Registry, TierCfg};
use keel_middleware::{AuditMiddleware, AuditSink, CostMiddleware, FileAuditSink, PrivacyMiddleware, Redactor};
use std::sync::Arc;

pub const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com";
pub const AUDIT_LEDGER: &str = ".keelstate/audit.jsonl";
// local substrate launch (paths match keel.lock; keel.lock-driven config is a refinement)
pub const LLAMA_EXE: &str = r"C:\llama.cpp\llama-server.exe";
pub const LLAMA_MODEL: &str = r"C:\models\Qwen3.5-9B-Q5_K_M.gguf";
pub const LLAMA_MMPROJ: &str = r"C:\models\mmproj-F16.gguf";
pub const LLAMA_LOG: &str = ".keelstate/llama-server.log";

/// A ready-to-run spine: the resolved tier behind the invariant chain.
pub struct Assembled {
    pub tier_name: String,
    pub model: String,
    pub tier: Arc<dyn ModelTier>,
    pub chain: Chain,
    /// The registry (the tier is registered under `tier_name`); kept so callers can extend it.
    pub registry: Registry,
}

/// Build the spine for `tier_override` (else the manifest's default tier): construct the adapter
/// (resolving/launching the local substrate as needed) and wrap it in `audit · privacy · cost`.
pub fn assemble(manifest: &Manifest, tier_override: Option<&str>) -> Result<Assembled> {
    let tier_name = tier_override
        .map(str::to_string)
        .unwrap_or_else(|| manifest.router.default_tier.clone());
    let tcfg = manifest
        .tier(&tier_name)
        .ok_or_else(|| KeelError::Other(format!("no tier '{tier_name}' in manifest")))?;

    let tier: Arc<dyn ModelTier> = match tcfg.adapter.as_str() {
        "local_llama" => {
            let endpoint = resolve_local_endpoint(tcfg)?;
            Arc::new(
                LocalLlama::new(endpoint, tcfg.model.clone(), tier_name.clone(), tcfg.price.to_price(), tcfg.vision)
                    .with_max_tokens(2048),
            )
        }
        "deepseek" => {
            let key = tcfg.api_key().ok_or_else(|| {
                KeelError::Other(format!("tier '{tier_name}' needs env var {}", tcfg.api_key_env.as_deref().unwrap_or("?")))
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
    registry.register(tier_name.clone(), tier.clone());

    // local is sovereign-safe (pass-through); cloud tiers get the I3 egress mask
    let egress = tcfg.adapter != "local_llama";
    let sink: Arc<dyn AuditSink> = Arc::new(
        FileAuditSink::new(AUDIT_LEDGER).map_err(|e| KeelError::Other(format!("ledger {AUDIT_LEDGER}: {e}")))?,
    );
    let chain = Chain::new(vec![
        Arc::new(AuditMiddleware::new(sink)),
        Arc::new(PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), egress)),
        Arc::new(CostMiddleware::new(manifest.cost.hard_stop_at)),
    ]);

    Ok(Assembled { tier_name, model: tcfg.model.clone(), tier, chain, registry })
}

/// Resolve the local endpoint: explicit override → probe a running server (c1) → cold-start one (c2).
fn resolve_local_endpoint(tcfg: &TierCfg) -> Result<String> {
    if let Some(e) = tcfg.endpoint.clone() {
        return Ok(e);
    }
    match keel_kernel::resolve_endpoint(&keel_kernel::default_local_candidates()) {
        Ok(e) => {
            eprintln!("[keel] local substrate → {e}");
            Ok(e)
        }
        Err(_) => {
            eprintln!("[keel] no server up — cold-starting llama-server (first call loads the model)…");
            let mut cfg = keel_kernel::LlamaServerConfig::new(LLAMA_EXE, LLAMA_MODEL);
            cfg.mmproj = Some(LLAMA_MMPROJ.to_string());
            cfg.log_path = Some(LLAMA_LOG.to_string());
            let server = keel_kernel::launch(&cfg)?;
            let ep = server.endpoint().to_string();
            eprintln!("[keel] llama-server ready → {ep} (pid {})", server.pid());
            Ok(ep) // handle drops here; the process keeps running (detached for reuse)
        }
    }
}
