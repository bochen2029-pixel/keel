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
    Context, Driver, GenerateRequest, GenerateResult, GoldenCase, KeelError, ModelTier, Oracle, Result, Router, Spine,
    Step,
};
use keel_kernel::engine::{Engine as KernelEngine, EngineConfig, TierSlot};
use keel_kernel::{Chain, Manifest, Registry, TierCfg};
use keel_middleware::{AuditMiddleware, AuditSink, CostMiddleware, FileAuditSink, PrivacyMiddleware, Redactor};
use keel_services::{DifficultyRouter, FileMemory, FileTraceSink, GoldenDispatchOracle, Verifier};
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
/// The append-only Memory **Tape** — the lossless factual register (canon §11); persistent working
/// memory across runs. The SQLite index above is the derived checkpoint.
pub const TAPE_PATH: &str = ".keelstate/tape/tape.jsonl";
/// The append-only **distill corpus** (canon §8 step 9): passed verdicts, **secrets scrubbed** before
/// write (the reversibility gate, §5 — never train on a secret). Feedstock for out-of-band distillation.
pub const TRACES_PATH: &str = ".keelstate/traces/corpus.jsonl";
/// Operator-frozen ground truth the engine resolves `step.golden_refs` against (read-only). KEEL's
/// own conformance set; a cell points the registry at its own goldens instead.
pub const GOLDEN_PATH: &str = "tests/golden/golden.json";
// llama-server launch paths are keel.lock-driven (servers.llama_cpp + substrate.llm_vision, via the
// Manifest helpers); only the KEEL-owned log path stays a const here.
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
            let tier = match build_tier(name, tcfg, manifest) {
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
        // A4: the no-SSN output STOPGAP is retired — `mw::privacy` now masks output PII as a proper I3
        // rung on every tier. The engine's optional `baseline` slot remains a generic always-on
        // extra-oracle seam (excluded from the critical-step #3), but the genome wires none.
        let baseline: Option<Arc<dyn Oracle>> = None;
        // I2: the SQLite index is the first Spine. The append-only file ledger stays the system of
        // record; this index is derived and rebuildable from it. (.keelstate is created by the sink.)
        let spine: Arc<dyn Spine> =
            Arc::new(SqliteStore::open(INDEX_DB).map_err(|e| KeelError::Other(format!("index {INDEX_DB}: {e}")))?);
        // I5 golden registry: resolve `step.golden_refs` against operator-frozen ground truth
        // (read-only, best-effort from golden.json; a cell injects its own). Empty if the file is
        // absent — a plain chat turn carries no refs, so the fail-closed guard only bites a ref'd step.
        let goldens = load_goldens(GOLDEN_PATH);
        // Memory: the Tape-backed `FileMemory` (canon §11) — the lossless factual register. `assemble`
        // injects Ring-0 (empty genome soul; persona is a cell concern) + Ring-2 recent turns read
        // back from the Tape, so working memory persists ACROSS `keel` invocations; the engine appends
        // each Trace to the Tape post-checkpoint. `TraceSink` (the flywheel feed) stays `None` until Stage 3.
        let memory: Option<Arc<dyn keel_contracts::Memory>> = Some(Arc::new(FileMemory::new("", TAPE_PATH, 6)));
        // The flywheel feed (canon §8 step 9): a passed verdict → an append-only distill corpus, with
        // secrets SCRUBBED first (reversibility gate §5) via the SAME `redactor` as the I3 egress mask —
        // one definition of "secret", so the corpus never carries what egress would have masked.
        let trace_sink: Option<Arc<dyn keel_contracts::TraceSink>> =
            Some(Arc::new(FileTraceSink::new(TRACES_PATH, redactor.clone())));
        let engine = KernelEngine::new(EngineConfig {
            slots,
            router,
            oracle,
            baseline,
            spine,
            memory,
            trace_sink,
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

    /// One driver tick (canon §8 `select(drivers).poll()`): select a Step from the drivers in priority
    /// order and run it through the full loop; `Ok(None)` = every driver idle. The L5 `keel daemon`
    /// loop calls this with a distinct `trace_id` per attempt so each turn checkpoints as its own run.
    pub async fn tick(&self, drivers: &[Arc<dyn Driver>], ctx: &mut Context) -> Result<Option<Outcome>> {
        self.0.tick(drivers, ctx).await
    }

    /// The bounded driver loop (canon §8): [`tick`](Engine::tick) up to `max_ticks` times, stopping
    /// early when every driver is idle. Each turn gets a distinct `trace_id` (`{base}-{n}`) so it
    /// checkpoints + Tapes as its own run. The continuously-running form (idle = sleep) is the
    /// `keel daemon` wrapper in `main.rs`; this is the bounded burst.
    pub async fn run_until_idle(&self, drivers: &[Arc<dyn Driver>], ctx: &mut Context, max_ticks: usize) -> Result<Vec<Outcome>> {
        self.0.run_until_idle(drivers, ctx, max_ticks).await
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

    let tier = build_tier(&tier_name, tcfg, manifest)?;
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
fn build_tier(tier_name: &str, tcfg: &TierCfg, manifest: &Manifest) -> Result<Arc<dyn ModelTier>> {
    let tier: Arc<dyn ModelTier> = match tcfg.adapter.as_str() {
        "local_llama" => {
            let endpoint = resolve_local_endpoint(tcfg, manifest)?;
            Arc::new(
                LocalLlama::new(endpoint, tcfg.model.clone(), tier_name.to_string(), tcfg.price.to_price(), tcfg.vision)
                    .with_max_tokens(tcfg.max_tokens),
            )
        }
        "deepseek" => {
            let key = api_key(tier_name, tcfg)?;
            let endpoint = tcfg.endpoint.clone().unwrap_or_else(|| DEEPSEEK_ENDPOINT.to_string());
            Arc::new(
                DeepSeek::new(endpoint, tcfg.model.clone(), tier_name.to_string(), tcfg.price.to_price(), key)
                    .with_max_tokens(tcfg.max_tokens),
            )
        }
        "anthropic" => {
            let key = api_key(tier_name, tcfg)?;
            let endpoint = tcfg.endpoint.clone().unwrap_or_else(|| ANTHROPIC_ENDPOINT.to_string());
            Arc::new(
                Anthropic::new(endpoint, tcfg.model.clone(), tier_name.to_string(), tcfg.price.to_price(), key)
                    .with_max_tokens(tcfg.max_tokens),
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
        Arc::new(AuditMiddleware::new(sink.clone())),
        // privacy shares the same sink so every egress redaction is an I1 event (canon §5.1).
        Arc::new(PrivacyMiddleware::new(redactor, egress, Some(sink))),
        Arc::new(CostMiddleware::new(hard_stop)),
    ])
}

/// Resolve the local endpoint: explicit override → probe a running server (c1) → cold-start one (c2).
fn resolve_local_endpoint(tcfg: &TierCfg, manifest: &Manifest) -> Result<String> {
    if let Some(e) = tcfg.endpoint.clone() {
        return Ok(e);
    }
    match keel_kernel::resolve_endpoint(&keel_kernel::default_local_candidates()) {
        Ok(e) => {
            eprintln!("[keel] local substrate → {e}");
            Ok(e)
        }
        Err(_) => {
            // cold-start needs the launch paths — keel.lock-driven (servers.llama_cpp + substrate.llm_vision).
            // Missing config => fail honestly (SUBSTRATE_UNRESOLVED), never launch a guessed binary.
            let exe = manifest.llama_exe().ok_or_else(|| {
                KeelError::SubstrateUnresolved("keel.lock servers.llama_cpp {path, exe} required to cold-start llama-server".into())
            })?;
            let model = manifest.llm_model_path().ok_or_else(|| {
                KeelError::SubstrateUnresolved("keel.lock servers.models_dir + substrate.llm_vision.file required to cold-start".into())
            })?;
            eprintln!("[keel] no server up — cold-starting llama-server (first call loads the model)…");
            let mut cfg = keel_kernel::LlamaServerConfig::new(exe, model);
            cfg.mmproj = manifest.llm_mmproj_path();
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

// ── the Driver daemon helpers (L5; the loop logic lives in kernel::engine) ──────

/// A file-change token for a [`WatchDriver`](keel_services::WatchDriver) probe: combines the path's
/// mtime + length into one comparable `u64`, or `None` if the path is absent/unreadable. **Model-free**
/// — a plain token, never a model judgement (the same discipline as perception's dHash gate). A change
/// to the file's content (length) or mtime changes the token, so the watch fires on the next poll.
pub fn watch_token(path: &std::path::Path) -> Option<u64> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    Some(mtime.wrapping_shl(1) ^ meta.len())
}

/// The daemon's bound decision: **perpetual** (run until interrupted) when `--max-ticks 0`, or when a
/// `--watch` is set without an explicit `--max-ticks` (a watch is a perpetual mode); else **bounded**
/// by `--max-ticks` (default 1 — a single self-tick that terminates). Keeps the autonomous default safe
/// (a plain `keel daemon` runs one tick and exits, never an unbounded loop).
pub fn daemon_perpetual(max_ticks: usize, max_ticks_set: bool, has_watch: bool) -> bool {
    max_ticks == 0 || (has_watch && !max_ticks_set)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_token_changes_on_edit_and_is_none_when_absent() {
        let path = std::env::temp_dir().join(format!("keel_watch_test_{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert!(watch_token(&path).is_none(), "absent path -> None (nothing to observe)");
        std::fs::write(&path, b"one").unwrap();
        let t1 = watch_token(&path).expect("present file -> a token");
        std::fs::write(&path, b"a longer body").unwrap(); // changes length -> token differs even if mtime equal
        let t2 = watch_token(&path).expect("still present");
        assert_ne!(t1, t2, "a content change changes the token -> the watch fires");
        std::fs::remove_file(&path).unwrap();
        assert!(watch_token(&path).is_none(), "removed -> None");
    }

    #[test]
    fn daemon_perpetual_rules() {
        assert!(!daemon_perpetual(1, true, false), "default bounded");
        assert!(!daemon_perpetual(5, true, false), "explicit bound");
        assert!(daemon_perpetual(0, true, false), "--max-ticks 0 -> perpetual");
        assert!(daemon_perpetual(1, false, true), "--watch w/o explicit bound -> perpetual");
        assert!(!daemon_perpetual(3, true, true), "--watch + explicit bound -> bounded");
    }
}
