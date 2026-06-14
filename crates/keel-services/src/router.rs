//! keel-services::router — the difficulty router (canon §9): the **fusion point**, where cost and
//! trust meet. `route(step, ctx) → Decision` picks the cheapest tier that clears the bar — a cheap
//! rules heuristic, never a model call.
//!
//! **The `Router` trait is a policy seam** (the SIRP-review takeaway). `DifficultyRouter` is the
//! rules policy. A *learned/cooling* policy — a SIRP-style `SirpRouter` — is just another
//! `impl Router` that slots in behind the same trait and earns its place only by passing
//! `GOLDEN_ROUTER`. One caveat carried from that review: a learned policy's quality signal MUST be
//! re-grounded on a **non-model oracle (I5)** before it is trusted — cooling that trusts an
//! automated score is exactly the `JOINT_WRONG` risk the externality layer exists to close.
//!
//! Axes the policy reads today: **reasoning demand** (`kind`) and **data sensitivity**
//! (`data_class` + raw perception content). SIRP's third axis — **latency tolerance** — would be a
//! first-class input but needs a `latency` field on the frozen `Step` contract (an operator
//! action); it is noted here, not faked.

use keel_contracts::{Content, Context, DataClass, Decision, Effort, Kind, Router, Step};

/// The tier ladder, cheapest first. Escalation climbs it.
const LADDER: [&str; 3] = ["local", "cheap-API", "frontier"];

/// The rules routing policy (canon §9): sovereign/perception force local, cost governs, `kind`
/// picks the tier, repeated oracle failure escalates up the ladder.
pub struct DifficultyRouter {
    escalate_after: u32,
}

impl DifficultyRouter {
    /// `escalate_after` = oracle failures on a tier before escalating up (canon default 2).
    pub fn new(escalate_after: u32) -> Self {
        Self { escalate_after }
    }
}

impl Default for DifficultyRouter {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Router for DifficultyRouter {
    fn route(&self, step: &Step, ctx: &Context) -> Decision {
        // 1. raw perception (image/clip/audio) is sovereign-by-default → local (I3). Overrides all.
        if step.content.iter().any(|c| !matches!(c, Content::Text { .. })) {
            return forced_local("raw perception is sovereign — forced local (I3, perception)");
        }
        // 2. sovereign / PHI data → local (I3).
        if matches!(step.data_class, DataClass::Sovereign | DataClass::Phi) {
            return forced_local("sovereign data — privacy forces local (I3)");
        }
        // 3. cost governor: projected cost breaches the remaining budget → BLOCK to operator (I4).
        if let (Some(pc), Some(rem)) = (step.projected_cost, ctx.budget_remaining()) {
            if pc > rem {
                return Decision {
                    tier: "BLOCK".to_string(),
                    effort: Effort::default(),
                    reason: format!("projected cost ${pc:.2} over budget ${rem:.2} — BLOCK (I4)"),
                };
            }
        }
        // 4. trust × difficulty: kind picks the base tier; repeated oracle failure escalates up.
        let base = match step.kind {
            Kind::Scaffolding => 0,
            Kind::CoreWire => 1,
        };
        let climbed = step
            .tier_history
            .iter()
            .filter_map(|t| LADDER.iter().position(|l| l == t))
            .max()
            .unwrap_or(0);
        let mut idx = base.max(climbed);
        let escalated = step.oracle_failures >= self.escalate_after && idx + 1 < LADDER.len();
        if escalated {
            idx += 1;
        }
        let tier = LADDER[idx];
        let reason = if escalated {
            format!("escalated to {tier} after {} oracle failures", step.oracle_failures)
        } else {
            format!("{} → {tier}", kind_label(step.kind))
        };
        Decision { tier: tier.to_string(), effort: effort_for(idx), reason }
    }
}

fn forced_local(reason: &str) -> Decision {
    Decision { tier: "local".to_string(), effort: effort_for(0), reason: reason.to_string() }
}

/// Effort by tier economics (canon §9.3): local is electricity → crank best-of-N and run **lean**
/// (thinking off — local amplifies by sampling *width*, not by reasoning *depth*, and a bare `None`
/// would defer to the server's thinking-on default and ramble into empty content); cheap-API leans
/// on the model's own thinking; frontier is single-pass. (Best-of-N ships OFF until `amplify` earns it.)
fn effort_for(idx: usize) -> Effort {
    match idx {
        0 => Effort { n: 8, thinking: Some("low".to_string()) },
        1 => Effort { n: 2, thinking: Some("high".to_string()) },
        _ => Effort { n: 1, thinking: None },
    }
}

fn kind_label(k: Kind) -> &'static str {
    match k {
        Kind::Scaffolding => "scaffolding",
        Kind::CoreWire => "core-wire",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::Trust;
    use serde_json::Value;

    /// Build a `(Step, Context)` from a language-neutral GOLDEN_ROUTER `input` object. The golden
    /// uses `core-wire` (hyphen) and lists raw modalities as strings — the impl interprets them.
    fn build_step(input: &Value) -> (Step, Context) {
        let kind = match input["kind"].as_str() {
            Some("core-wire") | Some("core_wire") => Kind::CoreWire,
            _ => Kind::Scaffolding,
        };
        let data_class = match input["data_class"].as_str() {
            Some("sovereign") => DataClass::Sovereign,
            Some("phi") => DataClass::Phi,
            _ => DataClass::Normal,
        };
        let content = input["content"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| match v.as_str() {
                        Some("image") => Some(Content::Image { source: String::new() }),
                        Some("audio") => Some(Content::Audio { source: String::new() }),
                        Some("text") => Some(Content::Text { text: String::new() }),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let tier_history = input["tier_history"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let step = Step {
            kind,
            ty: input["ty"].as_str().unwrap_or_default().to_string(),
            trust_required: Trust::Normal,
            data_class,
            tier_history,
            oracle_failures: input["oracle_failures"].as_u64().unwrap_or(0) as u32,
            projected_cost: input["projected_cost"].as_f64(),
            critical: false,
            source: None,
            content,
            golden_refs: vec![],
        };
        let mut ctx = Context::default();
        if let Some(rem) = input["budget_remaining"].as_f64() {
            ctx.task_budget = Some(rem); // cost.total = 0 ⇒ budget_remaining() == rem
        }
        (step, ctx)
    }

    /// The conformance test: the router MUST satisfy every frozen GOLDEN_ROUTER case (I5).
    #[test]
    fn passes_golden_router() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/golden.json");
        let raw = std::fs::read_to_string(path).expect("read golden.json");
        let golden: Value = serde_json::from_str(&raw).expect("parse golden.json");
        let cases = golden["router"].as_array().expect("router cases");
        assert!(!cases.is_empty());
        let router = DifficultyRouter::default();
        for case in cases {
            let name = case["name"].as_str().unwrap_or("?");
            let (step, ctx) = build_step(&case["input"]);
            let d = router.route(&step, &ctx);
            let expect = &case["expect"];
            if let Some(t) = expect["tier"].as_str() {
                assert_eq!(d.tier, t, "case '{name}': tier");
            }
            if let Some(arr) = expect["tier_in"].as_array() {
                let allowed: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
                assert!(allowed.contains(&d.tier.as_str()), "case '{name}': {} not in {allowed:?}", d.tier);
            }
            if let Some(sub) = expect["reason_contains"].as_str() {
                assert!(
                    d.reason.to_lowercase().contains(&sub.to_lowercase()),
                    "case '{name}': reason '{}' missing '{sub}'",
                    d.reason
                );
            }
        }
    }

    #[test]
    fn escalates_up_the_ladder_on_repeated_failure() {
        let r = DifficultyRouter::default();
        let mut step = Step {
            kind: Kind::CoreWire,
            ty: "reason:x".into(),
            trust_required: Trust::Normal,
            data_class: DataClass::Normal,
            tier_history: vec!["cheap-API".into()],
            oracle_failures: 2,
            projected_cost: None,
            critical: false,
            source: None,
            content: vec![],
            golden_refs: vec![],
        };
        // already failed twice on cheap-API → escalate to frontier
        assert_eq!(r.route(&step, &Context::default()).tier, "frontier");
        // no failures → stays at the base tier
        step.oracle_failures = 0;
        step.tier_history = vec![];
        assert_eq!(r.route(&step, &Context::default()).tier, "cheap-API");
    }
}
