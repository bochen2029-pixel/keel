//! keel-kernel::registry — tier name → brain adapter (canon §14).
//!
//! A container of `Arc<dyn ModelTier>` keyed by tier name. **The kernel may not import L2
//! adapters** (the layer rule, canon §6) — so, unlike the Marrow-L1 bench (whose `registry.py`
//! imports the adapter classes directly), here the wiring/app layer reads the `Manifest`,
//! constructs each adapter, and `register`s it. The kernel only ever holds the L0 trait object.

use keel_contracts::{KeelError, ModelTier, Result};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Maps a tier name (`local` | `cheap-API` | `frontier` | …) to its brain adapter.
#[derive(Default, Clone)]
pub struct Registry {
    tiers: BTreeMap<String, Arc<dyn ModelTier>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register (or replace) the adapter for a tier. Returns `&mut self` for chaining.
    pub fn register(&mut self, name: impl Into<String>, tier: Arc<dyn ModelTier>) -> &mut Self {
        self.tiers.insert(name.into(), tier);
        self
    }

    /// The adapter for a tier, or an error if none is registered (a wiring/config fault).
    pub fn get(&self, tier: &str) -> Result<Arc<dyn ModelTier>> {
        self.tiers
            .get(tier)
            .cloned()
            .ok_or_else(|| KeelError::Other(format!("no adapter registered for tier '{tier}'")))
    }

    pub fn contains(&self, tier: &str) -> bool {
        self.tiers.contains_key(tier)
    }

    /// Registered tier names (sorted, since the backing store is a `BTreeMap`).
    pub fn names(&self) -> Vec<String> {
        self.tiers.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.tiers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use keel_contracts::{Capabilities, Context, GenerateRequest, GenerateResult};

    /// A no-op tier — exercises the registry without needing a runtime (generate is never awaited).
    struct DummyTier;

    #[async_trait]
    impl ModelTier for DummyTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, _req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            Ok(GenerateResult::default())
        }
    }

    #[test]
    fn register_get_contains_names() {
        let mut r = Registry::new();
        assert!(r.is_empty());
        r.register("local", Arc::new(DummyTier))
            .register("cheap-API", Arc::new(DummyTier));

        assert_eq!(r.len(), 2);
        assert!(r.contains("local"));
        assert!(r.get("local").is_ok());
        assert!(r.get("frontier").is_err()); // unregistered tier
        assert_eq!(r.names(), vec!["cheap-API".to_string(), "local".to_string()]);
    }

    #[test]
    fn register_replaces() {
        let mut r = Registry::new();
        r.register("local", Arc::new(DummyTier));
        r.register("local", Arc::new(DummyTier));
        assert_eq!(r.len(), 1);
    }
}
