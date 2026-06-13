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
    #[serde(default)]
    pub price: PriceCfg,
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
    }
}
