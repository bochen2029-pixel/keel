//! keel-kernel::engine — the canonical closed loop (canon §8). L1.
//!
//! The kernel ships **one** cycle as a default; exotic cells compose their own from the same joints
//! and still inherit the invariants, because the invariants live in the *chain* and the *spine*, not
//! here. This engine is **injection-only**: it imports nothing but L0 (`keel-contracts`) and its own
//! kernel modules, and is handed the concrete policies — the `Router`, the I5 `Oracle`, the I2
//! `Spine`, optionally `Memory`/`TraceSink`, and the operator-frozen golden registry — as L0 trait
//! objects + data via [`EngineConfig`]. The same dyn-injection the [`Registry`](crate::Registry)
//! uses for `ModelTier`, applied to the loop.
//!
//! One turn (canon §8): **assemble → route → chain → verify → checkpoint → emit.** The engine **owns
//! the [`Context`]** (it folds in `result.cost` after the chain returns — I4 — because the chain only
//! sees `&Context`) and the [`Step`]'s history (it appends `tier_history` and bumps `oracle_failures`
//! on a failure), so the escalation ladder (canon §9) fires across turns.
//!
//! **I5 teeth (canon §8/§10):** the loop resolves `step.golden_refs` against the injected registry
//! and verifies against the resolved cases. Two guards keep "vacuous on a plain chat turn" *safe*:
//! (a) an **unresolved** `golden_ref` → **fail-closed** (a named-but-missing assertion is a hole,
//! never a vacuous pass); (b) a **`critical`** step with **no correctness assertion** (the I3 baseline excluded; empty verdict
//! evidence) → **config fault**, never a silent pass. A non-critical, no-ref turn still passes
//! silently — the teeth bite on critical/ref'd work, not on plain chat.
//!
//! **Per-tier chains (I3, canon §8 footnote):** the privacy mask differs by destination, so the
//! engine holds one egress-correct [`Chain`] per tier and runs the routed tier through *its* chain.
//!
//! **The driver select-loop (canon §8 `select(drivers).poll()`):** above the single turn sits the
//! multi-turn loop — [`select`] polls the drivers in priority order, [`Engine::tick`] runs the one
//! emitted Step, and [`Engine::run_until_idle`] repeats until every driver is idle. This is what turns
//! KEEL from "responds to a user turn" into a self that *acts*. The continuously-running daemon (idle
//! = sleep, run forever) is a thin L5 wrapper, deliberately not started here — this is the loop logic.

use crate::Chain;
use keel_contracts::{
    Assertion, Content, Context, Decision, Driver, Effort, GenerateRequest, GenerateResult, GoldenCase, KeelError,
    Memory, Message, ModelTier, Oracle, Result, Role, Router, Spine, Step, StepOutput, Trace, TraceSink,
    Verdict, VerifiedTrace,
};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
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

/// Everything the engine is injected with (canon §6 — the wiring layer builds the concrete services
/// and hands them in as L0 trait objects + data). A config struct rather than a long argument list,
/// so new seams (`Memory`, `TraceSink`, `goldens`, …) don't churn the constructor signature.
pub struct EngineConfig {
    pub slots: BTreeMap<String, TierSlot>,
    pub router: Box<dyn Router>,
    pub oracle: Arc<dyn Oracle>,
    /// The **I3 sovereignty baseline** (currently no-SSN-on-output): an always-on check that runs and
    /// folds into the verdict but is **excluded from the critical-step guard (#3)** — a privacy
    /// baseline is not a correctness oracle. **STOPGAP:** `mw::privacy` masks only on *egress* (cloud),
    /// so a LOCAL turn's output PII is otherwise unguarded; this covers that gap until the Stage-2
    /// privacy rung re-homes it as an output-side I3 check (then this slot is dropped). Never satisfies #3.
    pub baseline: Option<Arc<dyn Oracle>>,
    pub spine: Arc<dyn Spine>,
    pub memory: Option<Arc<dyn Memory>>,
    pub trace_sink: Option<Arc<dyn TraceSink>>,
    pub default_tier: String,
    /// Operator-frozen ground truth keyed by case name; the loop resolves `step.golden_refs` against
    /// it. **Read-only at runtime** — populated from `golden.json` / a cell's goldens by the wiring
    /// layer (the engine never writes it).
    pub goldens: HashMap<String, GoldenCase>,
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
    baseline: Option<Arc<dyn Oracle>>,
    spine: Arc<dyn Spine>,
    memory: Option<Arc<dyn Memory>>,
    trace_sink: Option<Arc<dyn TraceSink>>,
    default_tier: String,
    goldens: HashMap<String, GoldenCase>,
}

impl Engine {
    /// Build the engine from an [`EngineConfig`]. The wiring layer (L5) constructs the concrete
    /// services (`DifficultyRouter`, the `Verifier` as a composite `Oracle`, the SQLite `Spine`, the
    /// golden registry, …) and hands them in as L0 trait objects — the kernel imports none of them
    /// (the layer rule, canon §6). Errors if no tier is wired (`SUBSTRATE_UNRESOLVED`).
    pub fn new(config: EngineConfig) -> Result<Engine> {
        if config.slots.is_empty() {
            return Err(KeelError::SubstrateUnresolved(
                "engine: no tier wired (local substrate down and no cloud keys)".into(),
            ));
        }
        let EngineConfig { slots, router, oracle, baseline, spine, memory, trace_sink, default_tier, goldens } = config;
        Ok(Engine { slots, router, oracle, baseline, spine, memory, trace_sink, default_tier, goldens })
    }

    /// The tiers actually wired (sorted).
    pub fn available(&self) -> Vec<String> {
        self.slots.keys().cloned().collect()
    }

    /// Run one turn of the canonical loop. Mutates `step` (history/failure feedback) and `ctx` (cost
    /// accumulation) — the engine owns both. Honors `BLOCK` (I4 → `BudgetExceeded`); falls back DOWN
    /// the ladder when the routed tier is unplugged (local is always present).
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

        // (5) resolve the step's golden_refs against the injected registry (I5). An unresolved ref is
        //     a MISSING ASSERTION — tracked here and failed-closed below (a step that names ground
        //     truth we cannot supply must never verify vacuously).
        let mut golden_cases = Vec::new();
        let mut missing = Vec::new();
        for name in &step.golden_refs {
            match self.goldens.get(name) {
                Some(gc) => golden_cases.push(gc.clone()),
                None => missing.push(name.clone()),
            }
        }

        // (6) verify (I5). The injected `oracle` is the CORRECTNESS surface (the golden-ref dispatch +
        //     a cell's domain oracles); ITS evidence is what satisfies the critical-step guard (#3).
        let output = StepOutput { content: result.content.clone(), artifact: Value::Null };
        let mut verdict = self.oracle.verify(&output, &golden_cases, ctx).await?;
        // capture BEFORE folding the baseline: only a CORRECTNESS assertion counts for #3.
        let correctness_asserted = !verdict.evidence.is_empty();

        // (6a) the I3 sovereignty baseline (no-SSN-on-output) runs always and folds into the verdict —
        //      but it is a privacy check, NOT a correctness oracle, so it is **excluded** from #3.
        if let Some(baseline) = &self.baseline {
            let bv = baseline.verify(&output, &[], ctx).await?;
            verdict.passed &= bv.passed;
            verdict.joint_wrong |= bv.joint_wrong;
            verdict.failures.extend(bv.failures);
            verdict.evidence.extend(bv.evidence);
        }

        // (6b) fail-closed on any unresolved golden_ref (a named-but-missing assertion is a hole).
        //      ASCII-only: this line lands in the verdict, the checkpoint, and the ledger — the I5
        //      alarm must render on any console/codepage, not just a UTF-8 shell.
        if !missing.is_empty() {
            verdict.passed = false;
            let detail = format!("unresolved golden_ref(s) {missing:?} - missing assertion, fail-closed (-> operator)");
            verdict.failures.push(detail.clone());
            verdict.evidence.push(Assertion { kind: "golden_ref".into(), detail });
        }

        // (6c) a CRITICAL step needs a CORRECTNESS assertion — a resolved golden_ref that fired, or a
        //      domain oracle — **not** the I3 baseline. None ⇒ config fault, never a vacuous pass
        //      (canon 8/10). Resolving a ref is not asserting one: a ref to a conformance-only golden
        //      produces no correctness evidence and lands here too.
        if step.critical && !correctness_asserted {
            verdict.passed = false;
            let detail =
                "critical step with no applicable correctness oracle - config fault (canon 8/10), not a vacuous pass".to_string();
            verdict.failures.push(detail.clone());
            verdict.evidence.push(Assertion { kind: "critical".into(), detail });
        }

        // (7) feed the verdict back onto the Step so the next route can escalate (canon §9).
        step.tier_history.push(tier_used.clone());
        if !verdict.passed {
            step.oracle_failures = step.oracle_failures.saturating_add(1);
        }

        // (8) checkpoint the run-state (I2) — the `Trace` is the durable unit (the index; the file
        //     ledger remains the system of record).
        let trace = Trace {
            step: step.clone(),
            decision: decision.clone(),
            result: result.clone(),
            verdict: verdict.clone(),
        };
        let state = serde_json::to_value(&trace).map_err(|e| KeelError::Other(format!("trace encode: {e}")))?;
        self.spine.checkpoint(&ctx.trace_id, &state).await?;

        // (8a) append the full Trace to the Memory **Tape** — the lossless system of record (canon
        //      §11); the SQLite checkpoint above is the derived, rebuildable index. A `None` memory
        //      (the off-Tape default) skips. This is what `assemble` reads back as Ring-2 next turn.
        if let Some(mem) = &self.memory {
            mem.record(&trace).await?;
        }

        // (9) a passed verdict is flywheel feedstock (the sink scrubs secrets before distill, §5).
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

    /// One driver tick (canon §8): [`select`] a Step from the drivers (priority order), then run it
    /// through the full loop. `Ok(None)` => every driver was idle (nothing ran). The request is built
    /// from the Step via [`request_from_step`]; `step`/`ctx` are mutated as in [`run`](Engine::run).
    pub async fn tick(&self, drivers: &[Arc<dyn Driver>], ctx: &mut Context) -> Result<Option<Outcome>> {
        let Some(mut step) = select(drivers, ctx).await? else {
            return Ok(None); // every driver idle
        };
        let req = request_from_step(&step);
        Ok(Some(self.run(&mut step, ctx, req).await?))
    }

    /// The **bounded** driver loop (the testable form of the §8 daemon): [`tick`](Engine::tick) up to
    /// `max_ticks` times, stopping early the first time every driver is idle. Each turn gets a distinct
    /// `trace_id` (`{base}-{n}`) so it checkpoints as its own run (N rows for `metrics`) and appends its
    /// own Tape entry, while cost accumulates across the run in the shared `ctx` (I4). Returns each
    /// turn's [`Outcome`]. This is **not** the continuously-running daemon — that wrapper (idle = sleep,
    /// run forever) is a thin L5 add and is deliberately not started here; this is the loop logic, and
    /// it always terminates.
    pub async fn run_until_idle(
        &self,
        drivers: &[Arc<dyn Driver>],
        ctx: &mut Context,
        max_ticks: usize,
    ) -> Result<Vec<Outcome>> {
        let base = ctx.trace_id.clone();
        let mut outcomes = Vec::new();
        for n in 0..max_ticks {
            ctx.trace_id = format!("{base}-{n}"); // distinct run per turn: N metrics rows + N Tape entries
            match self.tick(drivers, ctx).await? {
                Some(o) => outcomes.push(o),
                None => break, // all drivers idle; a live daemon would sleep here instead of returning
            }
        }
        Ok(outcomes)
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

/// Poll the drivers in **priority order** (array order = priority: user-turn before heartbeat before
/// watch) and return the first `Some(Step)` — the §8 `select(drivers).poll()`. `None` = every driver
/// is idle (nothing to do now). A driver error short-circuits (the caller decides whether to continue).
pub async fn select(drivers: &[Arc<dyn Driver>], ctx: &Context) -> Result<Option<Step>> {
    for d in drivers {
        if let Some(step) = d.poll(ctx).await? {
            return Ok(Some(step));
        }
    }
    Ok(None)
}

/// Build the base [`GenerateRequest`] for a driver-emitted [`Step`]: its `content` becomes the user
/// turn (empty content => no user message — the Ring-0 system preamble still applies via `assemble`).
/// The engine fills `model` from the routed tier; effort/grammar default. A cell needing a richer
/// mapping composes its own loop — this is the genome default.
pub fn request_from_step(step: &Step) -> GenerateRequest {
    let messages = if step.content.is_empty() {
        Vec::new()
    } else {
        vec![Message {
            role: Role::User,
            content: step.content.clone(),
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        }]
    };
    GenerateRequest {
        messages,
        model: String::new(),
        tools: Vec::new(),
        grammar: None,
        effort: Effort::default(),
        cache_prefix_len: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use keel_contracts::{AssembledContext, Capabilities, DataClass, Kind, RunId, State, Trust};
    use serde_json::json;
    use std::collections::VecDeque;
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

    /// Passes with **no evidence** (the vacuous case — an empty effective oracle set).
    struct PassOracle;
    #[async_trait]
    impl Oracle for PassOracle {
        async fn verify(&self, _o: &StepOutput, _g: &[GoldenCase], _c: &Context) -> Result<Verdict> {
            Ok(Verdict { passed: true, ..Default::default() })
        }
    }
    /// Passes **with** an assertion (a real oracle actually applied to the step).
    struct EvidenceOracle;
    #[async_trait]
    impl Oracle for EvidenceOracle {
        async fn verify(&self, _o: &StepOutput, _g: &[GoldenCase], _c: &Context) -> Result<Verdict> {
            Ok(Verdict {
                passed: true,
                failures: Vec::new(),
                joint_wrong: false,
                evidence: vec![Assertion { kind: "stub".into(), detail: "asserted".into() }],
            })
        }
    }
    struct FailOracle;
    #[async_trait]
    impl Oracle for FailOracle {
        async fn verify(&self, _o: &StepOutput, _g: &[GoldenCase], _c: &Context) -> Result<Verdict> {
            Ok(Verdict { passed: false, failures: vec!["nope".into()], joint_wrong: false, evidence: vec![Assertion { kind: "stub".into(), detail: "failed".into() }] })
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

    /// Counts records — proves the engine appends the Trace to the Memory Tape each turn (canon §11).
    #[derive(Default)]
    struct RecMemory {
        records: Arc<Mutex<u32>>,
    }
    #[async_trait]
    impl Memory for RecMemory {
        async fn assemble(&self, _s: &Step, _c: &Context) -> Result<AssembledContext> {
            Ok(AssembledContext::default())
        }
        async fn record(&self, _t: &Trace) -> Result<()> {
            *self.records.lock().unwrap() += 1;
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
    /// Build an engine with an empty golden registry (the common case for these tests).
    fn engine_with(
        slots: BTreeMap<String, TierSlot>,
        router: Box<dyn Router>,
        oracle: Arc<dyn Oracle>,
        spine: Arc<dyn Spine>,
        memory: Option<Arc<dyn Memory>>,
        trace_sink: Option<Arc<dyn TraceSink>>,
    ) -> Engine {
        Engine::new(EngineConfig {
            slots,
            router,
            oracle,
            baseline: None,
            spine,
            memory,
            trace_sink,
            default_tier: "local".into(),
            goldens: HashMap::new(),
        })
        .unwrap()
    }

    #[tokio::test]
    async fn passing_turn_folds_cost_checkpoints_and_emits() {
        let spine = Arc::new(RecSpine::default());
        let sink = Arc::new(RecSink::default());
        let engine = engine_with(
            one_local(echo(0.25)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            spine.clone(),
            None,
            Some(sink.clone()),
        );

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
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(FailOracle),
            spine.clone(),
            None,
            Some(sink.clone()),
        );

        let mut s = step();
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();

        assert!(!out.verdict.passed);
        assert_eq!(s.oracle_failures, 1); // I5 feedback (failure count)
        assert_eq!(spine.checkpoints.lock().unwrap().len(), 1); // still checkpointed
        assert_eq!(*sink.emitted.lock().unwrap(), 0); // no flywheel emit on a failed verdict
    }

    #[tokio::test]
    async fn cost_accumulates_across_turns() {
        let engine = engine_with(
            one_local(echo(0.1)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
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
        let engine = engine_with(
            one_local(echo),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            Some(Arc::new(SoulMemory)),
            None,
        );
        engine.run(&mut step(), &mut ctx(), req()).await.unwrap();
        let msgs = seen.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0].role, Role::System));
        assert!(matches!(&msgs[0].content[0], Content::Text { text } if text == "SOUL"));
    }

    #[tokio::test]
    async fn memory_records_the_trace_each_turn() {
        // I2/§11: with a Memory wired, the engine appends the full Trace to the Tape every turn.
        let inner = RecMemory::default();
        let records = inner.records.clone();
        let mem: Arc<dyn Memory> = Arc::new(inner);
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            Some(mem),
            None,
        );
        let mut s = step();
        let mut c = ctx();
        engine.run(&mut s, &mut c, req()).await.unwrap();
        engine.run(&mut s, &mut c, req()).await.unwrap();
        assert_eq!(*records.lock().unwrap(), 2, "the Tape is appended every turn (canon §11)");
    }

    #[tokio::test]
    async fn block_decision_is_budget_exceeded() {
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("BLOCK")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let err = engine.run(&mut step(), &mut ctx(), req()).await.unwrap_err();
        assert_eq!(err.code(), "BUDGET_EXCEEDED");
    }

    #[tokio::test]
    async fn unplugged_tier_falls_back_down_the_ladder() {
        // router wants frontier; only local is wired → substitute down to local.
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("frontier")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
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
        let engine = engine_with(slots, Box::new(EscalatingRouter), Arc::new(FailOracle), Arc::new(RecSpine::default()), None, None);

        let mut s = step();
        let mut c = ctx();
        let t1 = engine.run(&mut s, &mut c, req()).await.unwrap();
        assert_eq!(t1.tier_used, "local"); // first turn routes local
        assert_eq!(s.oracle_failures, 1); // …and fails its oracle
        let t2 = engine.run(&mut s, &mut c, req()).await.unwrap();
        assert_eq!(t2.tier_used, "cheap-API"); // next turn escalates because the Step carried it
    }

    // ── I5 teeth: the resolver + the two hardenings ──

    #[tokio::test]
    async fn unresolved_golden_ref_fails_closed() {
        // a step names a golden absent from the (empty) registry → missing assertion → fail-closed,
        // never a vacuous pass; it also feeds the escalation ladder.
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let mut s = step();
        s.golden_refs = vec!["does_not_exist".into()];
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();
        assert!(!out.verdict.passed, "unresolved golden_ref must fail closed");
        assert!(out.verdict.failures.iter().any(|f| f.contains("unresolved golden_ref")));
        assert_eq!(s.oracle_failures, 1);
    }

    #[tokio::test]
    async fn resolved_golden_ref_does_not_fail_closed() {
        // a known ref resolves → no fail-closed; the case is available to the oracle.
        let mut goldens = HashMap::new();
        goldens.insert("g1".to_string(), GoldenCase { name: "g1".into(), input: json!({}), expect: json!({}) });
        let engine = Engine::new(EngineConfig {
            slots: one_local(echo(0.0)),
            router: Box::new(FixedRouter("local")),
            oracle: Arc::new(EvidenceOracle),
            baseline: None,
            spine: Arc::new(RecSpine::default()),
            memory: None,
            trace_sink: None,
            default_tier: "local".into(),
            goldens,
        })
        .unwrap();
        let mut s = step();
        s.golden_refs = vec!["g1".into()];
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();
        assert!(out.verdict.passed, "resolved ref + asserting oracle → pass");
        assert!(!out.verdict.failures.iter().any(|f| f.contains("unresolved")));
    }

    #[tokio::test]
    async fn critical_step_with_no_oracle_is_config_fault() {
        // PassOracle emits NO evidence; a CRITICAL step with nothing asserting is a config fault.
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let mut s = step();
        s.critical = true;
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();
        assert!(!out.verdict.passed, "critical + no applicable oracle must fail (not a vacuous pass)");
        assert!(out.verdict.failures.iter().any(|f| f.contains("config fault")));
    }

    #[tokio::test]
    async fn critical_step_with_an_asserting_oracle_passes() {
        // a critical step is fine when an oracle actually asserts (evidence present).
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let mut s = step();
        s.critical = true;
        assert!(engine.run(&mut s, &mut ctx(), req()).await.unwrap().verdict.passed);
    }

    #[tokio::test]
    async fn plain_turn_passes_silently() {
        // no false alarms: a non-critical, no-ref chat turn still passes vacuously — verify runs but
        // nothing fails. The teeth bite on critical/ref'd work, not on plain chat.
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(PassOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let mut s = step(); // critical:false, golden_refs:[]
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();
        assert!(out.verdict.passed, "plain non-critical no-ref turn passes silently");
        assert!(out.verdict.failures.is_empty(), "no false alarm on a plain turn");
    }

    #[tokio::test]
    async fn baseline_does_not_satisfy_critical_step() {
        // the un-neutering: a critical step with NO golden_ref, where ONLY the I3 baseline asserts,
        // must still config-fault — a privacy baseline is not a correctness oracle.
        let engine = Engine::new(EngineConfig {
            slots: one_local(echo(0.0)),
            router: Box::new(FixedRouter("local")),
            oracle: Arc::new(PassOracle),             // correctness: no evidence
            baseline: Some(Arc::new(EvidenceOracle)), // baseline: asserts — but must NOT count for #3
            spine: Arc::new(RecSpine::default()),
            memory: None,
            trace_sink: None,
            default_tier: "local".into(),
            goldens: HashMap::new(),
        })
        .unwrap();
        let mut s = step();
        s.critical = true;
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();
        assert!(!out.verdict.passed, "the baseline must not satisfy a critical step's #3");
        assert!(out.verdict.failures.iter().any(|f| f.contains("config fault")));
    }

    #[tokio::test]
    async fn critical_resolved_ref_with_no_assertion_config_faults() {
        // resolving a ref is not asserting one: a critical step names a golden that RESOLVES, but the
        // correctness oracle produces no evidence for it (as GoldenDispatchOracle does for a
        // conformance-only family) → #3 config-faults, not a vacuous pass, and not the #2 path.
        let mut goldens = HashMap::new();
        goldens.insert("conf-only".to_string(), GoldenCase { name: "conf-only".into(), input: json!({ "usage": {} }), expect: json!({}) });
        let engine = Engine::new(EngineConfig {
            slots: one_local(echo(0.0)),
            router: Box::new(FixedRouter("local")),
            oracle: Arc::new(PassOracle), // mimics the dispatch skipping a conformance-only case: no evidence
            baseline: None,
            spine: Arc::new(RecSpine::default()),
            memory: None,
            trace_sink: None,
            default_tier: "local".into(),
            goldens,
        })
        .unwrap();
        let mut s = step();
        s.critical = true;
        s.golden_refs = vec!["conf-only".into()]; // resolves (no #2), but yields no correctness assertion
        let out = engine.run(&mut s, &mut ctx(), req()).await.unwrap();
        assert!(!out.verdict.passed, "resolved-but-unasserted ref on a critical step config-faults");
        assert!(out.verdict.failures.iter().any(|f| f.contains("config fault")));
        assert!(!out.verdict.failures.iter().any(|f| f.contains("unresolved golden_ref")), "the ref resolved — not the #2 path");
    }

    #[tokio::test]
    async fn the_failure_alarm_is_ascii() {
        // the I5 alarm lands in the verdict, the checkpoint, and the ledger — it must render on any
        // console/codepage. Drive both engine-authored failure paths and assert the strings are ASCII.
        let engine = engine_with(one_local(echo(0.0)), Box::new(FixedRouter("local")), Arc::new(PassOracle), Arc::new(RecSpine::default()), None, None);
        let mut s = step();
        s.golden_refs = vec!["nope".into()]; // #2 unresolved-ref alarm
        for f in &engine.run(&mut s, &mut ctx(), req()).await.unwrap().verdict.failures {
            assert!(f.is_ascii(), "alarm must be ASCII: {f:?}");
        }
        let mut s2 = step();
        s2.critical = true; // #3 critical config-fault alarm
        for f in &engine.run(&mut s2, &mut ctx(), req()).await.unwrap().verdict.failures {
            assert!(f.is_ascii(), "alarm must be ASCII: {f:?}");
        }
    }

    // ── the driver select-loop (canon §8: select(drivers).poll()) ──

    struct QueueDriver {
        q: Mutex<VecDeque<Step>>,
    }
    impl QueueDriver {
        fn with(steps: Vec<Step>) -> Self {
            Self { q: Mutex::new(steps.into_iter().collect()) }
        }
    }
    #[async_trait]
    impl Driver for QueueDriver {
        async fn poll(&self, _ctx: &Context) -> Result<Option<Step>> {
            Ok(self.q.lock().unwrap().pop_front())
        }
    }
    /// Always idle — nothing to do.
    struct IdleDriver;
    #[async_trait]
    impl Driver for IdleDriver {
        async fn poll(&self, _ctx: &Context) -> Result<Option<Step>> {
            Ok(None)
        }
    }
    /// Never idle (like a zero-interval heartbeat) — proves the `max_ticks` bound terminates the loop.
    struct AlwaysDriver;
    #[async_trait]
    impl Driver for AlwaysDriver {
        async fn poll(&self, _ctx: &Context) -> Result<Option<Step>> {
            Ok(Some(step()))
        }
    }

    #[tokio::test]
    async fn select_polls_in_priority_order() {
        // an idle higher-priority driver is skipped; the first driver with work wins.
        let mut s = step();
        s.ty = "work".into();
        let drivers: Vec<Arc<dyn Driver>> = vec![Arc::new(IdleDriver), Arc::new(QueueDriver::with(vec![s]))];
        let picked = select(&drivers, &ctx()).await.unwrap().expect("a driver had work");
        assert_eq!(picked.ty, "work");
    }

    #[tokio::test]
    async fn select_all_idle_is_none() {
        let drivers: Vec<Arc<dyn Driver>> = vec![Arc::new(IdleDriver), Arc::new(IdleDriver)];
        assert!(select(&drivers, &ctx()).await.unwrap().is_none(), "every driver idle -> nothing to do");
    }

    #[test]
    fn request_from_step_maps_content_to_a_user_turn() {
        let mut s = step();
        s.content = vec![Content::Text { text: "hi".into() }];
        let req = request_from_step(&s);
        assert_eq!(req.messages.len(), 1);
        assert!(matches!(req.messages[0].role, Role::User));
        // empty content -> no user message (the Ring-0 preamble still applies via assemble).
        assert!(request_from_step(&step()).messages.is_empty());
    }

    #[tokio::test]
    async fn tick_runs_one_turn_then_idle() {
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let drivers: Vec<Arc<dyn Driver>> = vec![Arc::new(QueueDriver::with(vec![step()]))];
        let mut c = ctx();
        assert!(engine.tick(&drivers, &mut c).await.unwrap().is_some(), "one queued turn ran");
        assert!(engine.tick(&drivers, &mut c).await.unwrap().is_none(), "queue drained -> idle");
    }

    #[tokio::test]
    async fn run_until_idle_drains_distinct_runs_and_accumulates_cost() {
        let spine = Arc::new(RecSpine::default());
        let engine = engine_with(
            one_local(echo(0.1)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            spine.clone(),
            None,
            None,
        );
        let drivers: Vec<Arc<dyn Driver>> = vec![Arc::new(QueueDriver::with(vec![step(), step()]))];
        let mut c = ctx(); // trace_id "t1"
        let outs = engine.run_until_idle(&drivers, &mut c, 10).await.unwrap();
        assert_eq!(outs.len(), 2, "drained both queued turns, then stopped at idle");
        // each turn checkpoints under its OWN run_id -> metrics sees N rows, not an upsert of 1.
        let cps = spine.checkpoints.lock().unwrap();
        assert_eq!(cps.len(), 2);
        assert_eq!(cps[0].0, "t1-0");
        assert_eq!(cps[1].0, "t1-1");
        // cost accumulates across ticks in the shared ctx (I4).
        assert!((c.cost.total - 0.2).abs() < 1e-9);
    }

    #[tokio::test]
    async fn run_until_idle_respects_max_ticks() {
        // a never-idle driver (heartbeat-like) is bounded by max_ticks -> the loop always terminates.
        let engine = engine_with(
            one_local(echo(0.0)),
            Box::new(FixedRouter("local")),
            Arc::new(EvidenceOracle),
            Arc::new(RecSpine::default()),
            None,
            None,
        );
        let drivers: Vec<Arc<dyn Driver>> = vec![Arc::new(AlwaysDriver)];
        let mut c = ctx();
        let outs = engine.run_until_idle(&drivers, &mut c, 3).await.unwrap();
        assert_eq!(outs.len(), 3, "bounded by max_ticks (no infinite loop)");
    }
}
