//! keel-kernel::manifest — declarative config → behavior (canon §14).
//!
//! Parses the operator's `keel.lock` (YAML) into a typed [`Manifest`]: which tiers exist, their
//! price / effort / caps, and the router policy. It reuses the L0 types (`Price`, `Effort`,
//! `Capabilities`) so the manifest is the single place wiring reads — never a second source of
//! truth for the contracts. Sections this slice doesn't yet model (`substrate`, `servers`,
//! `ledger`, …) are ignored; later kernel slices (`lock`, `lifecycle`) pick them up. Secrets are
//! referenced by env-var **name**, never inlined — the operator rotates keys.

use keel_contracts::{Capabilities, Effort, KeelError, Price, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Per-tier price in `keel.lock` shorthand ($/1M tokens). Converts to the frozen L0 `Price`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct PriceCfg {
    #[serde(default)]
    pub miss: f64,
    #[serde(default)]
    pub hit: f64,
    #[serde(default)]
    pub out: f64,
}
impl PriceCfg {
    /// Map to the L0 `Price` (miss / hit / out → input_miss / input_hit / output).
    pub fn to_price(self) -> Price {
        Price { input_miss: self.miss, input_hit: self.hit, output: self.out }
    }
}

/// One routable cognition source (canon §4: a `tier`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TierCfg {
    /// Which adapter implements it (`local_llama` | `deepseek` | `anthropic` | …).
    pub adapter: String,
    pub model: String,
    #[serde(default)]
    pub vision: bool,
    /// Env-var name holding the key; resolved at runtime, never inlined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    /// Optional explicit endpoint (e.g. a cloud API base). Local resolves to the llama-server URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub price: PriceCfg,
    /// Max output tokens for this tier (keel.lock-driven; default 2048). The adapter caps generation.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub default_effort: Effort,
}
impl TierCfg {
    /// The API key for this tier, read from its env var (`None` if unset/absent). Never logged.
    pub fn api_key(&self) -> Option<String> {
        self.api_key_env.as_ref().and_then(|name| std::env::var(name).ok())
    }
    /// What this tier can do; the router and the egress filter read this (canon §9).
    pub fn capabilities(&self) -> Capabilities {
        Capabilities { vision: self.vision, video: false, thinking: self.default_effort.thinking.is_some() }
    }
}

fn default_max_tokens() -> u32 {
    2048
}

/// Local inference servers + model dir (keel.lock `servers`) — the substrate resolver's launch
/// targets. Only the fields the wiring needs are modeled; the rest (build/cuda/launch/…) are ignored.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ServersCfg {
    #[serde(default)]
    pub llama_cpp: LlamaCppCfg,
    /// Directory holding the model files (joined with `substrate.llm_vision.file`/`mmproj_file`).
    #[serde(default)]
    pub models_dir: String,
}

/// llama.cpp install + server exe (keel.lock `servers.llama_cpp`).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LlamaCppCfg {
    #[serde(default)]
    pub path: String,
    /// Server executable name under `path` (e.g. `llama-server.exe`).
    #[serde(default)]
    pub exe: String,
    /// Endpoint of an already-running server (the resolver probes it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Resolved substrate models (keel.lock `substrate`). `llm_vision` + `embedding` are modeled; the
/// audio/rerank/privacy organs are picked up by their own slices.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SubstrateCfg {
    #[serde(default)]
    pub llm_vision: LlmVisionCfg,
    #[serde(default)]
    pub embedding: EmbeddingCfg,
}

/// The embedding organ (keel.lock `substrate.embedding`, canon §11 — a Memory organ, NOT a tier).
/// `id`+`dim` form the index fingerprint (format-committing, ADR #13); `file` is the on-disk GGUF
/// under `servers.models_dir`; `port` is the embed llama-server's own port (the LLM owns its own).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmbeddingCfg {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
    #[serde(default = "default_embed_dim")]
    pub dim: usize,
    #[serde(default = "default_embed_port")]
    pub port: u16,
}
impl Default for EmbeddingCfg {
    fn default() -> Self {
        Self { id: String::new(), file: String::new(), dim: default_embed_dim(), port: default_embed_port() }
    }
}
fn default_embed_dim() -> usize {
    1024
}
fn default_embed_port() -> u16 {
    8090
}

/// Memory autopilot config (keel.lock `memory`, A7 — config, not pins). Defaults make memory
/// self-managing with zero flags: recall on-when-resolvable, budgets enforced, maintenance on the
/// A7.4 policy cadence. Every field has a genome default, so the section may be absent entirely.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryCfg {
    /// Ring-4 semantic recall: ON by default — wired only when the embed substrate resolves
    /// (graceful degrade to Ring-0/2/3 otherwise; a turn is never blocked on the embedder).
    #[serde(default = "default_true")]
    pub recall: bool,
    /// How many recalled entries Ring-4 injects per turn.
    #[serde(default = "default_recall_k")]
    pub recall_k: usize,
    /// The bounded recent-turn vector-scan window (episodes always scan whole — they stay few).
    /// Keeps ISSUE-1 brute-force O(window + episodes) forever. Latency falsifier re-opens sqlite-vec.
    #[serde(default = "default_recall_window")]
    pub recall_window: usize,
    /// Cold-start backfill: how many recent Tape turns to embed when the vector sidecar is absent.
    #[serde(default = "default_backfill")]
    pub backfill: usize,
    /// Ring-2 working window (turns read back from the Tape).
    #[serde(default = "default_working_turns")]
    pub working_turns: usize,
    /// Per-ring token budgets (A7.1; 0 = uncapped; Ring-0 is never trimmed regardless).
    #[serde(default)]
    pub budget: MemBudgetCfg,
    /// A7.4 policy: consolidate after this many new turns.
    #[serde(default = "default_consolidate_every")]
    pub consolidate_every_turns: usize,
    /// A7.4 policy: consolidate at session end only if at least this many new turns accumulated.
    #[serde(default = "default_session_end_min")]
    pub session_end_min_turns: usize,
    /// A7.4 policy: run cold-eyes every N consolidations (0 = never).
    #[serde(default = "default_cold_eyes_every")]
    pub cold_eyes_every: usize,
}
impl Default for MemoryCfg {
    fn default() -> Self {
        Self {
            recall: true,
            recall_k: default_recall_k(),
            recall_window: default_recall_window(),
            backfill: default_backfill(),
            working_turns: default_working_turns(),
            budget: MemBudgetCfg::default(),
            consolidate_every_turns: default_consolidate_every(),
            session_end_min_turns: default_session_end_min(),
            cold_eyes_every: default_cold_eyes_every(),
        }
    }
}
fn default_true() -> bool {
    true
}
fn default_recall_k() -> usize {
    3
}
fn default_recall_window() -> usize {
    4096
}
fn default_backfill() -> usize {
    32
}
fn default_working_turns() -> usize {
    6
}
fn default_consolidate_every() -> usize {
    24
}
fn default_session_end_min() -> usize {
    4
}
fn default_cold_eyes_every() -> usize {
    4
}

/// keel.lock `memory.budget` shorthand → the frozen L0 `TokenBudget` (tokens; 0 = uncapped).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MemBudgetCfg {
    #[serde(default = "default_ring1")]
    pub ring1: u32,
    #[serde(default = "default_ring2")]
    pub ring2: u32,
    #[serde(default = "default_ring3")]
    pub ring3: u32,
    #[serde(default = "default_ring4")]
    pub ring4: u32,
}
impl Default for MemBudgetCfg {
    fn default() -> Self {
        Self { ring1: default_ring1(), ring2: default_ring2(), ring3: default_ring3(), ring4: default_ring4() }
    }
}
impl MemBudgetCfg {
    pub fn to_budget(self) -> keel_contracts::TokenBudget {
        keel_contracts::TokenBudget {
            ring1: self.ring1,
            ring2: self.ring2,
            ring3: self.ring3,
            ring4: self.ring4,
            ..Default::default()
        }
    }
}
fn default_ring1() -> u32 {
    1000
}
fn default_ring2() -> u32 {
    2000
}
fn default_ring3() -> u32 {
    1000
}
fn default_ring4() -> u32 {
    1000
}

/// The local vision model (keel.lock `substrate.llm_vision`). `id` is the logical (normalized) id;
/// `file`/`mmproj_file` are the on-disk names under `servers.models_dir` — the resolver needs the
/// path, not the id, to cold-start llama-server.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LlmVisionCfg {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub mmproj_file: String,
}

/// Join a directory + filename with the OS separator (Windows `\`, Unix `/`).
fn join_path(dir: &str, name: &str) -> String {
    Path::new(dir).join(name).to_string_lossy().into_owned()
}

fn default_tier() -> String {
    "local".to_string()
}
fn default_escalate() -> u32 {
    2
}

/// Router policy (canon §9). Every fallback defaults to the sovereign-first local tier.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouterCfg {
    #[serde(default = "default_tier")]
    pub default_tier: String,
    /// Sovereign / PHI data forces this tier (I3).
    #[serde(default = "default_tier")]
    pub sovereign_forces: String,
    /// Raw frames / audio force this tier — perception never egresses (I3).
    #[serde(default = "default_tier")]
    pub perception_forces: String,
    /// Escalate one tier after this many oracle failures (canon §9).
    #[serde(default = "default_escalate")]
    pub escalate_after_oracle_failures: u32,
}
impl Default for RouterCfg {
    fn default() -> Self {
        Self {
            default_tier: default_tier(),
            sovereign_forces: default_tier(),
            perception_forces: default_tier(),
            escalate_after_oracle_failures: default_escalate(),
        }
    }
}

fn default_budget() -> f64 {
    5.0
}
fn default_hard_stop() -> f64 {
    1.0
}

/// Cost governance (I4). Seeds the per-task budget on the `Context`.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct CostCfg {
    #[serde(default = "default_budget")]
    pub budget_per_task: f64,
    /// Block to the operator when projected remaining drops below this.
    #[serde(default = "default_hard_stop")]
    pub hard_stop_at: f64,
}
impl Default for CostCfg {
    fn default() -> Self {
        Self { budget_per_task: default_budget(), hard_stop_at: default_hard_stop() }
    }
}

/// The whole manifest. *Same code, different manifest, different behavior.*
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub tiers: BTreeMap<String, TierCfg>,
    #[serde(default)]
    pub router: RouterCfg,
    #[serde(default)]
    pub cost: CostCfg,
    #[serde(default)]
    pub servers: ServersCfg,
    #[serde(default)]
    pub substrate: SubstrateCfg,
    #[serde(default)]
    pub memory: MemoryCfg,
}
impl Manifest {
    /// Parse from a YAML string (the `keel.lock` dialect).
    pub fn from_yaml(s: &str) -> Result<Self> {
        serde_yaml_ng::from_str(s).map_err(|e| KeelError::Other(format!("manifest parse: {e}")))
    }

    /// Load + parse a manifest file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .map_err(|e| KeelError::Other(format!("manifest read: {e}")))?;
        Self::from_yaml(&s)
    }

    /// A tier config by name.
    pub fn tier(&self, name: &str) -> Option<&TierCfg> {
        self.tiers.get(name)
    }

    /// The default effort for a tier (its configured `Effort`, else the neutral default).
    pub fn tier_effort(&self, name: &str) -> Effort {
        self.tiers.get(name).map(|t| t.default_effort.clone()).unwrap_or_default()
    }

    /// The llama-server executable path (`servers.llama_cpp.path` + `exe`), or `None` if unconfigured
    /// (then the resolver fails honestly rather than launching a guessed binary).
    pub fn llama_exe(&self) -> Option<String> {
        let s = &self.servers.llama_cpp;
        (!s.path.is_empty() && !s.exe.is_empty()).then(|| join_path(&s.path, &s.exe))
    }

    /// The local vision model file path (`servers.models_dir` + `substrate.llm_vision.file`).
    pub fn llm_model_path(&self) -> Option<String> {
        let (dir, file) = (&self.servers.models_dir, &self.substrate.llm_vision.file);
        (!dir.is_empty() && !file.is_empty()).then(|| join_path(dir, file))
    }

    /// The vision projector (mmproj) file path (`servers.models_dir` + `substrate.llm_vision.mmproj_file`).
    pub fn llm_mmproj_path(&self) -> Option<String> {
        let (dir, mmproj) = (&self.servers.models_dir, &self.substrate.llm_vision.mmproj_file);
        (!dir.is_empty() && !mmproj.is_empty()).then(|| join_path(dir, mmproj))
    }

    /// The embedding model file path (`servers.models_dir` + `substrate.embedding.file`) — `None`
    /// when unconfigured (Ring-4 then degrades gracefully; never a guessed path).
    pub fn embed_model_path(&self) -> Option<String> {
        let (dir, file) = (&self.servers.models_dir, &self.substrate.embedding.file);
        (!dir.is_empty() && !file.is_empty()).then(|| join_path(dir, file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
tiers:
  local:     { adapter: local_llama, model: qwen3.5-9b, vision: true, price: { miss: 0.0, hit: 0.0, out: 0.0 }, default_effort: { n: 8 } }
  cheap-API: { adapter: deepseek, model: deepseek-v4-pro, api_key_env: DEEPSEEK_API_KEY, price: { miss: 0.435, hit: 0.003625, out: 0.87 }, default_effort: { n: 2, thinking: high } }
router:
  default_tier: local
  escalate_after_oracle_failures: 2
"#;

    #[test]
    fn parses_sample() {
        let m = Manifest::from_yaml(SAMPLE).unwrap();
        assert_eq!(m.tiers.len(), 2);

        let local = m.tier("local").unwrap();
        assert_eq!(local.adapter, "local_llama");
        assert!(local.vision);
        assert_eq!(local.default_effort.n, 8);
        assert_eq!(local.max_tokens, 2048); // default when unspecified in keel.lock
        assert_eq!(local.price.to_price().input_miss, 0.0);
        assert!(local.capabilities().vision);

        let cheap = m.tier("cheap-API").unwrap();
        assert_eq!(cheap.api_key_env.as_deref(), Some("DEEPSEEK_API_KEY"));
        assert_eq!(cheap.default_effort.thinking.as_deref(), Some("high"));
        assert_eq!(cheap.price.to_price().output, 0.87);

        assert_eq!(m.router.default_tier, "local");
        assert_eq!(m.router.escalate_after_oracle_failures, 2);
        // a field absent from the router block falls back to the sovereign-first default
        assert_eq!(m.router.sovereign_forces, "local");
    }

    #[test]
    fn defaults_apply_for_absent_sections() {
        let m = Manifest::from_yaml("tiers: {}").unwrap();
        assert!(m.tiers.is_empty());
        assert_eq!(m.router.default_tier, "local");
        assert_eq!(m.cost.budget_per_task, 5.0);
        assert_eq!(m.tier_effort("nope").n, 1); // neutral default for an unknown tier
        // absent servers/substrate -> the launch-path helpers return None (the resolver fails honestly)
        assert!(m.llama_exe().is_none());
        assert!(m.llm_model_path().is_none());
        assert!(m.embed_model_path().is_none(), "no embedding config -> Ring-4 silently off, never a guessed path");
    }

    #[test]
    fn reads_the_real_keel_lock() {
        // Diff against the operator's actual substrate file (the bench-diff for this slice).
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../keel.lock");
        let m = Manifest::load(path).expect("keel.lock must parse");

        assert!(m.tier("local").is_some());
        assert!(m.tier("cheap-API").is_some());
        assert!(m.tier("frontier").is_some());
        assert_eq!(m.tier("local").unwrap().adapter, "local_llama");
        assert!(m.tier("local").unwrap().vision);
        assert_eq!(m.tier("frontier").unwrap().model, "claude-opus-4-8");
        assert_eq!(m.router.default_tier, "local");
        assert_eq!(m.router.escalate_after_oracle_failures, 2);
        // the cheap-API key is referenced by env name, never inlined
        assert_eq!(m.tier("cheap-API").unwrap().api_key_env.as_deref(), Some("DEEPSEEK_API_KEY"));
        // config-from-keel.lock: launch paths + max_tokens are keel.lock-driven (no hardcoded consts).
        // local = 4096 (A7.5: thinking maintenance turns need the headroom; 2048 truncated mid-think).
        assert_eq!(m.tier("local").unwrap().max_tokens, 4096);
        assert_eq!(m.servers.llama_cpp.exe, "llama-server.exe");
        assert!(m.servers.models_dir.contains("models"));
        assert_eq!(m.substrate.llm_vision.file, "Qwen3.5-9B-Q5_K_M.gguf");
        assert_eq!(m.substrate.llm_vision.mmproj_file, "mmproj-F16.gguf");
        // derived launch paths join the dir + the on-disk filename (id is normalized, not a path)
        assert!(m.llama_exe().unwrap().ends_with("llama-server.exe"));
        assert!(m.llm_model_path().unwrap().ends_with("Qwen3.5-9B-Q5_K_M.gguf"));
        assert!(m.llm_mmproj_path().unwrap().ends_with("mmproj-F16.gguf"));
        // A7.3: the embed organ is keel.lock-driven (its own server on its own port; fingerprint id+dim)
        assert_eq!(m.substrate.embedding.id, "qwen3-embedding-0.6b-q8");
        assert_eq!(m.substrate.embedding.file, "qwen3-embedding-0.6b-q8_0.gguf");
        assert_eq!(m.substrate.embedding.dim, 1024);
        assert_eq!(m.substrate.embedding.port, 8090);
        assert!(m.embed_model_path().unwrap().ends_with("qwen3-embedding-0.6b-q8_0.gguf"));
        // A7: memory-autopilot defaults hold when the lock has no `memory:` section (config, not pins)
        assert!(m.memory.recall, "Ring-4 recall is on by default (wired only when the substrate resolves)");
        assert_eq!(m.memory.recall_k, 3);
        assert_eq!(m.memory.recall_window, 4096);
        assert_eq!(m.memory.consolidate_every_turns, 24);
        assert_eq!(m.memory.cold_eyes_every, 4);
        assert_eq!(m.memory.budget.ring2, 2000);
        assert_eq!(m.memory.budget.to_budget().ring3, 1000);
    }
}
