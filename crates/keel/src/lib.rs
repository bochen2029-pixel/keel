//! keel (L5) — the shared wiring both apps use. The loop itself lives in [`kernel::engine`](keel_kernel::engine)
//! (L1, over injected joints); this crate is the **injection layer** that reads `keel.lock`, builds
//! the concrete adapters/services, and hands them down as L0 trait objects. Two ways to assemble:
//!
//! - [`Engine`] — the **self-driving** path: a multi-tier registry of every *available* tier (local
//!   always; a cloud tier only when its API key is in the env), each behind its **own** invariant
//!   chain (audit → privacy → cost), fronted by a swappable [`Router`], an I5 [`Oracle`] (the
//!   verifier-as-composite), and an I2 [`Spine`] (the SQLite index). `keel` (default) and
//!   `keel-serve` route **every** turn through it — route → chain → verify → checkpoint → emit.
//! - [`assemble`] — the **single-tier** path: build exactly one tier behind the chain. The CLI's
//!   `--tier` override uses it so an explicit cloud call never needlessly cold-starts the local
//!   substrate (and it skips the router + the I5 loop by design).
//!
//! The privacy egress mask (I3) is **per tier**: local stays on the box (pass-through), every cloud
//! tier is masked before egress — which is why each tier gets its own chain (canon §8 footnote).

use keel_adapters::{Anthropic, DeepSeek, LocalLlama};
use keel_contracts::{
    Context, GenerateRequest, GenerateResult, GoldenCase, KeelError, ModelTier, Oracle, Result, Router, Spine, Step,
};
use keel_kernel::engine::{Engine as KernelEngine, EngineConfig, TierSlot};
use keel_kernel::{Chain, Manifest, Registry, TierCfg};
use keel_middleware::{AuditMiddleware, AuditSink, CostMiddleware, FileAuditSink, PrivacyMiddleware, Redactor};
use keel_services::{DifficultyRouter, GoldenDispatchOracle, PropertyOracle, Verifier};
use keel_store::SqliteStore;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// The outcome of a routed, verified turn (defined in the kernel; re-exported for app callers).
pub use keel_kernel::engine::Outcome;

pub const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com";
pub const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com";
pub const AUDIT_LEDGER: &str = ".keelstate/audit.jsonl";
/// The SQLite index backing the I2 `Spine` (derived/rebuildable; the file ledger is the record).
pub const INDEX_DB: &str = ".keelstate/index.db";
/// Operator-frozen ground truth the engine resolves `step.golden_refs` against (read-only). KEEL's
/// own conformance set; a cell points the registry at its own goldens instead.
pub const GOLDEN_PATH: &str = "tests/golden/golden.json";
// local substrate launch (paths match keel.lock; keel.lock-driven config is a refinement)
pub const LLAMA_EXE: &str = r"C:\llama.cpp\llama-server.exe";
pub const LLAMA_MODEL: &str = r"C:\models\Qwen3.5-9B-Q5_K_M.gguf";
pub const LLAMA_MMPROJ: &str = r"C:\models\mmproj-F16.gguf";
pub const LLAMA_LOG: &str = ".keelstate/llama-server.log";

// ── the self-driving engine (L5 injection → L1 loop) ──────────────────────────

/// The self-driving engine: a thin **injection wrapper** over [`kernel::engine::Engine`](keel_kernel::engine::Engine).
/// It wires every available tier behind its egress-correct chain and constructs the swappable joints
/// (the §9 `DifficultyRouter`, the I5 `Verifier` as a composite `Oracle`, the SQLite `Spine`), then
/// delegates the canonical loop to the kernel. The policy stays behind the seams so a learned router
/// or a cell's oracles slot in later without touching this crate.
pub struct Engine(KernelEngine);

impl Engine {
    /// Assemble from `keel.lock`: wire every tier whose substrate is reachable — **local always**
    /// (resolving/launching llama-server), a **cloud tier only when its key is in the env** (absent
    /// key ⇒ skipped, not fatal) — each behind an `audit · privacy · cost` chain whose egress mask
    /// matches whether that tier leaves the box (I3). Then inject the joints into the kernel engine.
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
        eprintln!("[keel] engine wired: {:?}", slots.keys().collect::<Vec<_>>());

        let router: Box<dyn Router> = Box::new(DifficultyRouter::new(manifest.router.escalate_after_oracle_failures));
        // I5 CORRECTNESS oracle: the GoldenDispatchOracle makes a resolved `golden_ref` actually
        // assert (schema/property family -> the matching oracle), with no cell pre-registration. A cell
        // registers its domain oracles here too. This is what satisfies the critical-step guard (#3).
        let mut correctness = Verifier::new();
        correctness.register(Box::new(GoldenDispatchOracle));
        let oracle: Arc<dyn Oracle> = Arc::new(correctness);
        // I3 sovereignty BASELINE (no-SSN-on-output): always-on, folded into the verdict, but it NEVER
        // counts for #3 (a privacy baseline is not a correctness oracle). STOPGAP until mw::privacy
        // gains an output-side rung (Stage 2) — see EngineConfig.baseline.
        let mut baseline_v = Verifier::new();
        baseline_v.register(Box::new(PropertyOracle::new(vec!["no_ssn_pattern".into()])));
        let baseline: Option<Arc<dyn Oracle>> = Some(Arc::new(baseline_v));
        // I2: the SQLite index is the first Spine. The append-only file ledger stays the system of
        // record; this index is derived and rebuildable from it. (.keelstate is created by the sink.)
        let spine: Arc<dyn Spine> =
            Arc::new(SqliteStore::open(INDEX_DB).map_err(|e| KeelError::Other(format!("index {INDEX_DB}: {e}")))?);
        // I5 golden registry: resolve `step.golden_refs` against operator-frozen ground truth
        // (read-only, best-effort from golden.json; a cell injects its own). Empty if the file is
        // absent — a plain chat turn carries no refs, so the fail-closed guard only bites a ref'd step.
        let goldens = load_goldens(GOLDEN_PATH);
        // Memory + TraceSink: the seams exist; their impls (the ringed Tape, the flywheel feed) land
        // in Stage 2 (svc::memory). `None` today ⇒ no Ring-0 assembly and no distill emit — no-ops.
        let engine = KernelEngine::new(EngineConfig {
            slots,
            router,
            oracle,
            baseline,
            spine,
            memory: None,
            trace_sink: None,
            default_tier: manifest.router.default_tier.clone(),
            goldens,
        })?;
        Ok(Engine(engine))
    }

    /// The tiers actually wired (sorted).
    pub fn available(&self) -> Vec<String> {
        self.0.available()
    }

    /// Route + run + verify + checkpoint one turn (canon §8). Mutates `step` (history/failure
    /// feedback) and `ctx` (cost accumulation) — the engine owns both.
    pub async fn run(&self, step: &mut Step, ctx: &mut Context, req: GenerateRequest) -> Result<Outcome> {
        self.0.run(step, ctx, req).await
    }

    /// Manual override: run on a named wired tier, skipping route + verify (the full chain still runs).
    pub async fn run_on(&self, tier: &str, ctx: &Context, req: GenerateRequest) -> Result<GenerateResult> {
        self.0.run_on(tier, ctx, req).await
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

/// Load operator-frozen ground truth into a name→case map for the engine's `golden_refs` resolver.
/// **Read-only**, best-effort: a missing/unparseable file yields an empty map (a plain chat turn
/// carries no refs, so the engine's fail-closed guard only bites a step that *names* a golden). The
/// flat sections of `golden.json` (every key but `_meta`) are merged by case `name`.
fn load_goldens(path: &str) -> HashMap<String, GoldenCase> {
    let mut map = HashMap::new();
    let Ok(raw) = std::fs::read_to_string(path) else { return map };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else { return map };
    let Some(obj) = json.as_object() else { return map };
    for (section, cases) in obj {
        if section == "_meta" {
            continue;
        }
        let Some(arr) = cases.as_array() else { continue };
        for case in arr {
            if let Ok(gc) = serde_json::from_value::<GoldenCase>(case.clone()) {
                map.insert(gc.name.clone(), gc);
            }
        }
    }
    map
}
