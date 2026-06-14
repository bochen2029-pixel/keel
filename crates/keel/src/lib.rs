//! keel (L5) — the shared wiring both apps use. Two ways to assemble the spine from `keel.lock`:
//!
//! - [`Engine`] — the **self-driving** path: a multi-tier registry of every *available* tier (local
//!   always; a cloud tier only when its API key is in the env), each behind its **own** invariant
//!   chain (audit → privacy → cost), fronted by a swappable [`Router`] policy. `keel` (default) and
//!   `keel-serve` route **every** turn through it — the router picks the tier, not the operator.
//! - [`assemble`] — the **single-tier** path: build exactly one tier behind the chain. The CLI's
//!   `--tier` override uses it so an explicit cloud call never needlessly cold-starts the local
//!   substrate.
//!
//! Both share `build_tier` / `build_chain`, so there is one wiring truth, not two. The privacy
//! egress mask (I3) is **per tier**: local stays on the box (pass-through), every cloud tier is
//! masked before egress — which is why the `Engine` gives each tier its own chain rather than one
//! shared chain with a single static flag.

use keel_adapters::{Anthropic, DeepSeek, LocalLlama};
use keel_contracts::{
    Context, Decision, Effort, GenerateRequest, GenerateResult, KeelError, ModelTier, Result, Router, Step,
};
use keel_kernel::{Chain, Manifest, Registry, TierCfg};
use keel_middleware::{AuditMiddleware, AuditSink, CostMiddleware, FileAuditSink, PrivacyMiddleware, Redactor};
use keel_services::DifficultyRouter;
use std::collections::BTreeMap;
use std::sync::Arc;

pub const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com";
pub const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com";
pub const AUDIT_LEDGER: &str = ".keelstate/audit.jsonl";
// local substrate launch (paths match keel.lock; keel.lock-driven config is a refinement)
pub const LLAMA_EXE: &str = r"C:\llama.cpp\llama-server.exe";
pub const LLAMA_MODEL: &str = r"C:\models\Qwen3.5-9B-Q5_K_M.gguf";
pub const LLAMA_MMPROJ: &str = r"C:\models\mmproj-F16.gguf";
pub const LLAMA_LOG: &str = ".keelstate/llama-server.log";

/// The tier ladder, cheapest first — the engine walks DOWN it to fall back to an available tier.
const LADDER: [&str; 3] = ["local", "cheap-API", "frontier"];

// ── the self-driving engine ──────────────────────────────────────────────────

/// One wired tier: its brain adapter, the model id to send it, and the **egress-correct** invariant
/// chain for that tier (local pass-through vs cloud-masked, I3).
struct TierSlot {
    tier: Arc<dyn ModelTier>,
    chain: Chain,
    model: String,
}

/// The self-driving engine (canon §9, the fusion point wired up): hold every *available* tier, let
/// the [`Router`] pick one per turn, run it through that tier's chain. `keel`/`keel-serve` build on
/// this so a turn auto-lands on the cheapest brain that clears the bar instead of a pinned `--tier`.
pub struct Engine {
    slots: BTreeMap<String, TierSlot>,
    router: Box<dyn Router>,
    default_tier: String,
}

/// The result of a routed turn: the model output plus the routing story (for the operator/ledger).
pub struct Outcome {
    pub result: GenerateResult,
    /// What the router chose (its tier + reason).
    pub decision: Decision,
    /// The tier actually run — equals `decision.tier` unless that tier was unplugged.
    pub tier_used: String,
    /// True when the chosen tier was unavailable and the engine fell back down the ladder.
    pub substituted: bool,
}

impl Engine {
    /// Assemble from `keel.lock`: wire every tier whose substrate is reachable — **local always**
    /// (resolving/launching llama-server), a **cloud tier only when its key is in the env** (absent
    /// key ⇒ skipped, not fatal) — each behind an `audit · privacy · cost` chain whose egress mask
    /// matches whether that tier leaves the box (I3). The policy is the §9 `DifficultyRouter`, kept
    /// behind the `Router` seam so a learned policy can slot in later.
    pub fn assemble(manifest: &Manifest) -> Result<Engine> {
        let sink: Arc<dyn AuditSink> = Arc::new(
            FileAuditSink::new(AUDIT_LEDGER).map_err(|e| KeelError::Other(format!("ledger {AUDIT_LEDGER}: {e}")))?,
        );
        // operator sovereign markers (rung 1) are a future manifest field; rung 2 runs regardless.
        let redactor = Arc::new(Redactor::new(vec![]));

        let mut slots = BTreeMap::new();
        for (name, tcfg) in &manifest.tiers {
            let is_local = tcfg.adapter == "local_llama";
            // a cloud tier is available only when its key is present — skip (don't fail) otherwise.
            if !is_local && tcfg.api_key().is_none() {
                let env = tcfg.api_key_env.as_deref().unwrap_or("?");
                eprintln!("[keel] tier '{name}' unavailable ({env} unset) — skipped");
                continue;
            }
            let tier = match build_tier(name, tcfg) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[keel] tier '{name}' could not be wired ({e}) — skipped");
                    continue;
                }
            };
            let chain = build_chain(sink.clone(), redactor.clone(), manifest.cost.hard_stop_at, !is_local);
            slots.insert(name.clone(), TierSlot { tier, chain, model: tcfg.model.clone() });
        }

        if slots.is_empty() {
            return Err(KeelError::SubstrateUnresolved("no tier could be wired (local substrate down and no cloud keys)".into()));
        }
        eprintln!("[keel] engine wired: {:?}", slots.keys().collect::<Vec<_>>());

        let router = Box::new(DifficultyRouter::new(manifest.router.escalate_after_oracle_failures));
        Ok(Engine { slots, router, default_tier: manifest.router.default_tier.clone() })
    }

    /// The tiers actually wired (sorted).
    pub fn available(&self) -> Vec<String> {
        self.slots.keys().cloned().collect()
    }

    /// Route `step`, then run `req` on the chosen *available* tier through that tier's chain.
    /// Honors `BLOCK` (I4 → `BudgetExceeded`). If the routed tier is unplugged, falls back DOWN the
    /// ladder to the nearest available tier (local is always present) and flags the substitution.
    pub async fn run(&self, step: &Step, ctx: &Context, mut req: GenerateRequest) -> Result<Outcome> {
        let decision = self.router.route(step, ctx);
        if decision.tier == "BLOCK" {
            return Err(KeelError::BudgetExceeded(decision.reason));
        }
        let tier_used = self.resolve_down(&decision.tier);
        let slot = self
            .slots
            .get(&tier_used)
            .ok_or_else(|| KeelError::TierUnavailable(format!("no slot for routed tier '{tier_used}'")))?;
        req.model = slot.model.clone();
        // Effort: the router picks thinking depth per tier; a caller's explicit thinking overrides
        // it. best-of-N (n>1) ships OFF until `amplify` earns it (§23 falsifier) — single-shot now.
        let thinking = req.effort.thinking.take().or_else(|| decision.effort.thinking.clone());
        req.effort = Effort { n: 1, thinking };
        let result = slot.chain.run(req, ctx, slot.tier.clone()).await?;
        let substituted = tier_used != decision.tier;
        Ok(Outcome { result, decision, tier_used, substituted })
    }

    /// Manual override: run `req` directly on `tier`, skipping the router but keeping the full
    /// invariant chain. Errors if that tier isn't wired. (The CLI's `--tier` uses the lighter
    /// single-tier [`assemble`] instead, to avoid waking the local substrate for a cloud-only call.)
    pub async fn run_on(&self, tier: &str, ctx: &Context, mut req: GenerateRequest) -> Result<GenerateResult> {
        let slot = self
            .slots
            .get(tier)
            .ok_or_else(|| KeelError::TierUnavailable(format!("tier '{tier}' not wired (have {:?})", self.available())))?;
        req.model = slot.model.clone();
        req.effort.n = 1; // amplify OFF
        slot.chain.run(req, ctx, slot.tier.clone()).await
    }

    /// Walk DOWN the ladder from `want` to the nearest wired tier. Local (idx 0) is always present,
    /// so this terminates; off-ladder names fall back to the manifest default, else any wired tier.
    fn resolve_down(&self, want: &str) -> String {
        if self.slots.contains_key(want) {
            return want.to_string();
        }
        if let Some(idx) = LADDER.iter().position(|&t| t == want) {
            for i in (0..=idx).rev() {
                if self.slots.contains_key(LADDER[i]) {
                    return LADDER[i].to_string();
                }
            }
        }
        if self.slots.contains_key(&self.default_tier) {
            return self.default_tier.clone();
        }
        self.slots.keys().next().cloned().unwrap_or_default()
    }
}

// ── single-tier assembly (the `--tier` override path) ─────────────────────────

/// A ready-to-run single tier: the resolved adapter behind the invariant chain.
pub struct Assembled {
    pub tier_name: String,
    pub model: String,
    pub tier: Arc<dyn ModelTier>,
    pub chain: Chain,
    /// The registry (the tier is registered under `tier_name`); kept so callers can extend it.
    pub registry: Registry,
}

/// Build the spine for exactly `tier_override` (else the manifest's default tier): construct the one
/// adapter (resolving/launching the local substrate as needed) and wrap it in `audit · privacy ·
/// cost`. Used for the explicit `--tier` override; the self-driving path is [`Engine`].
pub fn assemble(manifest: &Manifest, tier_override: Option<&str>) -> Result<Assembled> {
    let tier_name = tier_override
        .map(str::to_string)
        .unwrap_or_else(|| manifest.router.default_tier.clone());
    let tcfg = manifest
        .tier(&tier_name)
        .ok_or_else(|| KeelError::Other(format!("no tier '{tier_name}' in manifest")))?;

    let tier = build_tier(&tier_name, tcfg)?;
    let mut registry = Registry::new();
    registry.register(tier_name.clone(), tier.clone());

    let sink: Arc<dyn AuditSink> = Arc::new(
        FileAuditSink::new(AUDIT_LEDGER).map_err(|e| KeelError::Other(format!("ledger {AUDIT_LEDGER}: {e}")))?,
    );
    let egress = tcfg.adapter != "local_llama";
    let chain = build_chain(sink, Arc::new(Redactor::new(vec![])), manifest.cost.hard_stop_at, egress);

    Ok(Assembled { tier_name, model: tcfg.model.clone(), tier, chain, registry })
}

// ── shared construction ───────────────────────────────────────────────────────

/// Construct the brain adapter for a tier from its config, dispatching on `adapter`. Local resolves
/// (probe) or cold-starts llama-server; cloud tiers read their key from the env (never inlined).
fn build_tier(tier_name: &str, tcfg: &TierCfg) -> Result<Arc<dyn ModelTier>> {
    let tier: Arc<dyn ModelTier> = match tcfg.adapter.as_str() {
        "local_llama" => {
            let endpoint = resolve_local_endpoint(tcfg)?;
            Arc::new(
                LocalLlama::new(endpoint, tcfg.model.clone(), tier_name.to_string(), tcfg.price.to_price(), tcfg.vision)
                    .with_max_tokens(2048),
            )
        }
        "deepseek" => {
            let key = api_key(tier_name, tcfg)?;
            let endpoint = tcfg.endpoint.clone().unwrap_or_else(|| DEEPSEEK_ENDPOINT.to_string());
            Arc::new(
                DeepSeek::new(endpoint, tcfg.model.clone(), tier_name.to_string(), tcfg.price.to_price(), key)
                    .with_max_tokens(2048),
            )
        }
        "anthropic" => {
            let key = api_key(tier_name, tcfg)?;
            let endpoint = tcfg.endpoint.clone().unwrap_or_else(|| ANTHROPIC_ENDPOINT.to_string());
            Arc::new(
                Anthropic::new(endpoint, tcfg.model.clone(), tier_name.to_string(), tcfg.price.to_price(), key)
                    .with_max_tokens(2048),
            )
        }
        other => return Err(KeelError::Other(format!("unknown adapter '{other}' for tier '{tier_name}'"))),
    };
    Ok(tier)
}

/// Read a cloud tier's API key from its env var (never inlined; the operator rotates keys).
fn api_key(tier_name: &str, tcfg: &TierCfg) -> Result<String> {
    tcfg.api_key().ok_or_else(|| {
        KeelError::Other(format!("tier '{tier_name}' needs env var {}", tcfg.api_key_env.as_deref().unwrap_or("?")))
    })
}

/// Build the invariant chain (canon §6): audit (I1) → privacy (I3, masks on `egress`) → cost (I4).
/// One chain per tier so the egress mask is correct for that tier's destination.
fn build_chain(sink: Arc<dyn AuditSink>, redactor: Arc<Redactor>, hard_stop: f64, egress: bool) -> Chain {
    Chain::new(vec![
        Arc::new(AuditMiddleware::new(sink)),
        Arc::new(PrivacyMiddleware::new(redactor, egress)),
        Arc::new(CostMiddleware::new(hard_stop)),
    ])
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
