//! keel-kernel::engine — the canonical closed loop (canon §8). L1.
//!
//! The kernel ships **one** cycle as a default; exotic cells compose their own from the same joints
//! and still inherit the invariants, because the invariants live in the *chain* and the *spine*, not
//! here. This engine is **injection-only**: it imports nothing but L0 (`keel-contracts`) and its own
//! kernel modules, and is handed the concrete policies — the `Router`, the I5 `Oracle`, the I2
//! `Spine`, and optionally `Memory`/`TraceSink` — as L0 trait objects by the wiring layer (L5). The
//! same dyn-injection the [`Registry`](crate::Registry) uses for `ModelTier`, applied to the loop.
//!
//! One turn (canon §8): **assemble → route → chain → verify → checkpoint → emit.** The engine **owns
//! the [`Context`]** (it folds in `result.cost` after the chain returns — I4 — because the chain only
//! sees `&Context`) and the [`Step`]'s history (it appends `tier_history` and bumps `oracle_failures`
//! on an oracle failure), so the escalation ladder (canon §9) fires across turns. Verification (I5)
//! is *recorded and surfaced* on the verdict; in-turn re-escalate-and-retry is a deliberate
//! follow-up — escalation is realized on the **next** turn via the persisted history.
//!
//! **Per-tier chains (I3, canon §8 footnote):** the privacy mask differs by destination, so the
//! engine holds one egress-correct [`Chain`] per tier and runs the routed tier through *its* chain.

use crate::Chain;
use keel_contracts::{
    Content, Context, Decision, Effort, GenerateRequest, GenerateResult, KeelError, Memory, Message,
    ModelTier, Oracle, Result, Role, Router, Spine, Step, StepOutput, Trace, TraceSink, Verdict, VerifiedTrace,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The tier ladder, cheapest first — the engine walks DOWN it to substitute an available tier when
/// the routed one is unplugged.
const LADDER: [&str; 3] = ["local", "cheap-API", "frontier"];

/// One wired tier: its brain adapter, the model id to send it, and the **egress-correct** invariant
/// chain for that tier (local pass-through vs cloud-masked, I3 — canon §8 footnote). Built by the
/// wiring layer and injected.
pub struct TierSlot {
    pub tier: Arc<dyn ModelTier>,
    pub chain: Chain,
    pub model: String,
}

/// The outcome of one routed, verified turn: the model output plus the routing + externality story
/// (for the operator / ledger / serve response).
#[derive(Debug)]
pub struct Outcome {
    pub result: GenerateResult,
    /// What the router chose (its tier + reason).
    pub decision: Decision,
    /// The tier actually run — equals `decision.tier` unless that tier was unplugged.
    pub tier_used: String,
    /// True when the chosen tier was unavailable and the engine fell back down the ladder.
    pub substituted: bool,
    /// The I5 verdict — a non-model assertion on this output (`passed`, `joint_wrong`, failures).
    pub verdict: Verdict,
}

/// The canonical engine: hold every available tier (behind its chain), let the injected [`Router`]
/// pick one per turn, run it, verify (I5), accumulate cost (I4), checkpoint (I2), and emit verified
/// traces (the flywheel feed). The model that thinks is interchangeable; this loop is the self.
pub struct Engine {
    slots: BTreeMap<String, TierSlot>,
    router: Box<dyn Router>,
    oracle: Arc<dyn Oracle>,
    spine: Arc<dyn Spine>,
    memory: Option<Arc<dyn Memory>>,
    trace_sink: Option<Arc<dyn TraceSink>>,
    default_tier: String,
}

impl Engine {
    /// Inject the wired tiers + the joints. The wiring layer (L5) builds the concrete services
    /// (`DifficultyRouter`, the `Verifier` as a composite `Oracle`, the SQLite `Spine`, …) and hands
    /// them in as L0 trait objects — the kernel imports none of them (the layer rule, canon §6).
    /// Errors if no tier is wired (`SUBSTRATE_UNRESOLVED`).
    pub fn new(
        slots: BTreeMap<String, TierSlot>,
        router: Box<dyn Router>,
        oracle: Arc<dyn Oracle>,
        spine: Arc<dyn Spine>,
        memory: Option<Arc<dyn Memory>>,
        trace_sink: Option<Arc<dyn TraceSink>>,
        default_tier: String,
    ) -> Result<Engine> {
        if slots.is_empty() {
            return Err(KeelError::SubstrateUnresolved(
                "engine: no tier wired (local substrate down and no cloud keys)".into(),
            ));
        }
        Ok(Engine { slots, router, oracle, spine, memory, trace_sink, default_tier })
    }

    /// The tiers actually wired (sorted).
    pub fn available(&self) -> Vec<String> {
        self.slots.keys().cloned().collect()
    }

    /// Run one turn of the canonical loop. Mutates `step` (history/failures feedback) and `ctx`
    /// (cost accumulation) — the engine owns both. Honors `BLOCK` (I4 → `BudgetExceeded`); falls
    /// back DOWN the ladder when the routed tier is unplugged (local is always present).
    pub async fn run(&self, step: &mut Step, ctx: &mut Context, mut req: GenerateRequest) -> Result<Outcome> {
        // (1) assemble — Ring-0 soul → system message. The seam only; full ring assembly is
        //     `svc::memory` (Stage 2). A `None` memory (today) is a no-op.
        if let Some(mem) = &self.memory {
            let assembled = mem.assemble(step, ctx).await?;
            if !assembled.system.is_empty() {
                req.messages.insert(
                    0,
                    Message {
                        role: Role::System,
                        content: vec![Content::Text { text: assembled.system }],
                        name: None,
                        reasoning_content: None,
                        tool_call_id: None,
                    },
                );
            }
        }

        // (2) route — the cheapest tier that clears the trust bar, or BLOCK (I4 / reversibility).
        let decision = self.router.route(step, ctx);
        if decision.tier == "BLOCK" {
            return Err(KeelError::BudgetExceeded(decision.reason));
        }

        // (3) resolve down to an available tier, then run it through *its* egress-correct chain (I3).
        let tier_used = self.resolve_down(&decision.tier);
        let substituted = tier_used != decision.tier;
        let slot = self
            .slots
            .get(&tier_used)
            .ok_or_else(|| KeelError::TierUnavailable(format!("no slot for routed tier '{tier_used}'")))?;
        req.model = slot.model.clone();
        // The router picks thinking depth per tier; a caller's explicit thinking overrides it.
        // best-of-N (n>1) ships OFF until `amplify` earns it (§23) — single-shot.
        let thinking = req.effort.thinking.take().or_else(|| decision.effort.thinking.clone());
        req.effort = Effort { n: 1, thinking };
        let result = slot.chain.run(req, ctx, slot.tier.clone()).await?;

        // (4) fold cost into the Context (I4). `mw::cost` is the pre-call gate; the engine owns the
        //     post-call total, so a multi-call turn (escalation) sees accumulated spend.
        ctx.cost.add(&result.tier, result.cost);

        // (5) verify (I5) — a non-model assertion. The `golden_refs`→`GoldenCase` resolver is a
        //     Stage-2 runtime golden-registry seam; for now the registry runs its registered
        //     (property / source / joint-wrong) oracles. An empty registry passes vacuously, so the
        //     *mechanism* is live even before a cell plugs in its assertions.
        let output = StepOutput { content: result.content.clone(), artifact: Value::Null };
        let verdict = self.oracle.verify(&output, &[], ctx).await?;

        // (6) feed the verdict back onto the Step so the next route can escalate (canon §9).
        step.tier_history.push(tier_used.clone());
        if !verdict.passed {
            step.oracle_failures = step.oracle_failures.saturating_add(1);
        }

        // (7) checkpoint the run-state (I2) — the `Trace` is the durable unit (the index; the file
        //     ledger remains the system of record).
        let trace = Trace {
            step: step.clone(),
            decision: decision.clone(),
            result: result.clone(),
            verdict: verdict.clone(),
        };
        let state = serde_json::to_value(&trace).map_err(|e| KeelError::Other(format!("trace encode: {e}")))?;
        self.spine.checkpoint(&ctx.trace_id, &state).await?;

        // (8) a passed verdict is flywheel feedstock (the sink scrubs secrets before distill, §5).
        if verdict.passed {
            if let Some(sink) = &self.trace_sink {
                sink.emit(VerifiedTrace { trace }).await?;
            }
        }

        Ok(Outcome { result, decision, tier_used, substituted, verdict })
    }

    /// Manual override: run `req` directly on `tier` through its chain, **skipping route + verify**.
    /// Keeps the full invariant chain. Errors if that tier isn't wired.
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use keel_contracts::{AssembledContext, Capabilities, DataClass, GoldenCase, Kind, RunId, State, Trust};
    use std::sync::Mutex;

    // ── stub tiers / joints — all model-free; the engine *wiring* is what's under test ──

    /// Echoes a fixed cost; records the messages it was handed (to prove Ring-0 prepend).
    struct EchoTier {
        cost: f64,
        seen: Arc<Mutex<Vec<Message>>>,
    }
    #[async_trait]
    impl ModelTier for EchoTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            *self.seen.lock().unwrap() = req.messages.clone();
            Ok(GenerateResult { content: "echo".into(), cost: self.cost, tier: "stub".into(), model: req.model, ..Default::default() })
        }
    }

    /// Always routes to a fixed tier (or "BLOCK").
    struct FixedRouter(&'static str);
    impl Router for FixedRouter {
        fn route(&self, _step: &Step, _ctx: &Context) -> Decision {
            Decision { tier: self.0.into(), effort: Effort::default(), reason: "fixed".into() }
        }
    }

    /// Escalates: `local` until the Step has failed an oracle, then `cheap-API` — proving the
    /// engine's `oracle_failures` feedback reaches the next route.
    struct EscalatingRouter;
    impl Router for EscalatingRouter {
        fn route(&self, step: &Step, _ctx: &Context) -> Decision {
            let tier = if step.oracle_failures == 0 { "local" } else { "cheap-API" };
            Decision { tier: tier.into(), effort: Effort::default(), reason: "esc".into() }
        }
    }

    struct PassOracle;
    #[async_trait]
    impl Oracle for PassOracle {
        async fn verify(&self, _o: &StepOutput, _g: &[GoldenCase], _c: &Context) -> Result<Verdict> {
            Ok(Verdict { passed: true, ..Default::default() })
        }
    }
    struct FailOracle;
    #[async_trait]
    impl Oracle for FailOracle {
        async fn verify(&self, _o: &StepOutput, _g: &[GoldenCase], _c: &Context) -> Result<Verdict> {
            Ok(Verdict { passed: false, failures: vec!["nope".into()], ..Default::default() })
        }
    }

    #[derive(Default)]
    struct RecSpine {
        checkpoints: Mutex<Vec<(RunId, State)>>,
    }
    #[async_trait]
    impl Spine for RecSpine {
        async fn checkpoint(&self, run: &RunId, state: &State) -> Result<()> {
            self.checkpoints.lock().unwrap().push((run.clone(), state.clone()));
            Ok(())
        }
        async fn resume(&self, _run: &RunId) -> Result<Option<State>> {
            Ok(None)
        }
    }

    #[derive(Default)]
    struct RecSink {
        emitted: Mutex<u32>,
    }
    #[async_trait]
    impl TraceSink for RecSink {
        async fn emit(&self, _t: VerifiedTrace) -> Result<()> {
            *self.emitted.lock().unwrap() += 1;
            Ok(())
        }
    }

    /// Returns a Ring-0 soul string; `consolidate` is never reached in these tests.
    struct SoulMemory;
    #[async_trait]
    impl Memory for SoulMemory {
        async fn assemble(&self, _s: &Step, _c: &Context) -> Result<AssembledContext> {
            Ok(AssembledContext { system: "SOUL".into(), ..Default::default() })
        }
        async fn record(&self, _t: &Trace) -> Result<()> {
            Ok(())
        }
        async fn consolidate(&self) -> Result<Step> {
            Ok(step())
        }
    }

    // ── fixtures ──
    fn step() -> Step {
        Step {
            kind: Kind::Scaffolding,
            ty: "t".into(),
            trust_required: Trust::Normal,
            data_class: DataClass::Normal,
            tier_history: vec![],
            oracle_failures: 0,
            projected_cost: None,
            critical: false,
            source: None,
            content: vec![],
            golden_refs: vec![],
        }
    }
    fn req() -> GenerateRequest {
        GenerateRequest {
            messages: vec![],
            model: String::new(),
            tools: vec![],
            grammar: None,
            effort: Effort::default(),
            cache_prefix_len: None,
        }
    }
    fn ctx() -> Context {
        Context { trace_id: "t1".into(), ..Default::default() }
    }
    fn one_local(tier: Arc<dyn ModelTier>) -> BTreeMap<String, TierSlot> {
        let mut s = BTreeMap::new();
        s.insert("local".to_string(), TierSlot { tier, chain: Chain::new(vec![]), model: "m".into() });
        s
    }
    fn echo(cost: f64) -> Arc<EchoTier> {
        Arc::new(EchoTier { cost, seen: Arc::new(Mutex::new(vec![])) })
    }

    #[tokio::test]
    async fn passing_turn_folds_cost_checkpoints_and_emits() {
        let spine = Arc::new(RecSpine::default());
        let sink = Arc::new(RecSink::default());
        let engine = Engine::new(
            one_local(echo(0.25)),
            Box::new(FixedRouter("local")),
            Arc::new(PassOracle),
            spine.clone(),
            None,
            Some(sink.clone()),
            "local".into(),
        )
        .unwrap();

        let mut s = step();
        let mut c = ctx();
        let out = engine.run(&mut s, &mut c, req()).await.unwrap();

        assert!(out.verdict.passed);
        assert_eq!(out.tier_used, "local");
        assert!(!out.substituted);
        assert_eq!(s.tier_history, vec!["local".to_string()]); // I5 feedback (history)
        assert_eq!(s.oracle_failures, 0);
        assert!((c.cost.total - 0.25).abs() < 1e-9); // I4 fold
        assert_eq!(spine.checkpoints.lock().unwrap().len(), 1); // I2 checkpoint
        assert_eq!(*sink.emitted.lock().unwrap(), 1); // flywheel emit on pass
    }

    #[tokio::test]
    async fn failing_oracle_bumps_failures_and_skips_emit() {
        let spine = Arc::new(RecSpine::default());
        let sink = Arc::new(RecSink::default());
        let engine = Engine::new(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(FailOracle),
            spine.clone(),
            None,
            Some(sink.clone()),
            "local".into(),
        )
        .unwrap();

        let mut s = step();
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();

        assert!(!out.verdict.passed);
        assert_eq!(s.oracle_failures, 1); // I5 feedback (failure count)
        assert_eq!(spine.checkpoints.lock().unwrap().len(), 1); // still checkpointed
        assert_eq!(*sink.emitted.lock().unwrap(), 0); // no flywheel emit on a failed verdict
    }

    #[tokio::test]
    async fn cost_accumulates_across_turns() {
        let engine = Engine::new(
            one_local(echo(0.1)),
            Box::new(FixedRouter("local")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
            "local".into(),
        )
        .unwrap();
        let mut s = step();
        let mut c = ctx();
        engine.run(&mut s, &mut c, req()).await.unwrap();
        engine.run(&mut s, &mut c, req()).await.unwrap();
        assert!((c.cost.total - 0.2).abs() < 1e-9);
        assert_eq!(s.tier_history, vec!["local".to_string(), "local".to_string()]);
    }

    #[tokio::test]
    async fn memory_prepends_ring0_system() {
        let echo = echo(0.0);
        let seen = echo.seen.clone();
        let engine = Engine::new(
            one_local(echo),
            Box::new(FixedRouter("local")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            Some(Arc::new(SoulMemory)),
            None,
            "local".into(),
        )
        .unwrap();
        engine.run(&mut step(), &mut ctx(), req()).await.unwrap();
        let msgs = seen.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0].role, Role::System));
        assert!(matches!(&msgs[0].content[0], Content::Text { text } if text == "SOUL"));
    }

    #[tokio::test]
    async fn block_decision_is_budget_exceeded() {
        let engine = Engine::new(
            one_local(echo(0.0)),
            Box::new(FixedRouter("BLOCK")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
            "local".into(),
        )
        .unwrap();
        let err = engine.run(&mut step(), &mut ctx(), req()).await.unwrap_err();
        assert_eq!(err.code(), "BUDGET_EXCEEDED");
    }

    #[tokio::test]
    async fn unplugged_tier_falls_back_down_the_ladder() {
        // router wants frontier; only local is wired → substitute down to local.
        let engine = Engine::new(
            one_local(echo(0.0)),
            Box::new(FixedRouter("frontier")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
            "local".into(),
        )
        .unwrap();
        let out = engine.run(&mut step(), &mut ctx(), req()).await.unwrap();
        assert_eq!(out.tier_used, "local");
        assert!(out.substituted);
    }

    #[tokio::test]
    async fn oracle_failure_escalates_on_the_next_turn() {
        // two tiers wired; the stub router escalates once the Step records an oracle failure — this
        // is the cross-turn escalation the slice activates (canon §8/§9).
        let mut slots = BTreeMap::new();
        slots.insert("local".to_string(), TierSlot { tier: echo(0.0), chain: Chain::new(vec![]), model: "m-local".into() });
        slots.insert("cheap-API".to_string(), TierSlot { tier: echo(0.0), chain: Chain::new(vec![]), model: "m-cheap".into() });
        let engine = Engine::new(
            slots,
            Box::new(EscalatingRouter),
            Arc::new(FailOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
            "local".into(),
        )
        .unwrap();

        let mut s = step();
        let mut c = ctx();
        let t1 = engine.run(&mut s, &mut c, req()).await.unwrap();
        assert_eq!(t1.tier_used, "local"); // first turn routes local
        assert_eq!(s.oracle_failures, 1); // …and fails its oracle
        let t2 = engine.run(&mut s, &mut c, req()).await.unwrap();
        assert_eq!(t2.tier_used, "cheap-API"); // next turn escalates because the Step carried it
    }
}
