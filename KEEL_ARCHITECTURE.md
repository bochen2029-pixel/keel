# KEEL — Architecture Canon

**Codename:** KEEL — *the backbone laid first, running the whole length, that everything is built upon and that keeps the vessel upright in any sea.*
**Version:** 0.2.0 · **Status:** canon (spec-first; pre-implementation)
**Author:** Bo Chen — Dallas–Fort Worth — June 2026
**Lineage:** distilled as the *intersection* of seven independently-built systems across three languages (Tenancy, Terminal — Rust; TARS, REEL, the In-Home Companion, SEXTANT — Python; photo2deck, NightScribe, NightClerk — C#). KEEL is what every one of them rebuilt by hand. This document owes nothing to any prior framing; it is the thing itself, written once.

> **One sentence:** *KEEL is a single-operator substrate that is the persistent, sovereign **self** — it perceives (eyes and ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model to punch above its weight, and grounds every critical output in an assertion no model authored — while the model that does the thinking is interchangeable, rented cognition plugged into a tiny stable core.*

---

## §0 · What KEEL Is

The API (local or cloud) is **rented cognition** — stateless, interchangeable, billed per token. KEEL is **the self** — persistent, user-owned, sovereign, portable across providers and across years. The model provides the thinking. KEEL provides the *remembering, the perceiving, the routing, the verifying, and the continuity.*

KEEL is not an agent and not a loop. It is a **trust-and-cost economy with a tiny immortal core**, inside which a closed *verify-and-distill cycle* lets the cheapest available brain clear a bar that something no model authored has set. Off-the-shelf harnesses orchestrate a single expensive brain well; none treats trust as a routable quantity, keeps a sovereign local self, runs a distillation flywheel, or enforces an externality principle. That gap is the entire reason KEEL exists, and it is where every line of original code is spent.

KEEL is the **genome**: a frozen set of contracts and invariants from which every specialization (a companion, an OCR clerk, a job-search conductor, a meeting scribe) is grown by composing periphery — *never* by editing the core. The genome is written once and reused forever; the cells are many and disposable.

---

## §1 · Intent & The Bet

**Why build, not buy.** In the age of capable coding models it is now feasible to build custom software for nearly everything; the *harness* is the one primitive worth getting exactly right, because it is load-bearing under everything else and because it must encode *this operator's* judgment — the core-wire/scaffolding routing doctrine, the externality principle, the reversibility gate, the trust threshold. No off-the-shelf harness encodes that doctrine; bending one to fit is permanent impedance mismatch. So KEEL is built custom **at the core** and composes the periphery ruthlessly.

**The atomic bet (applied recursively).** Make every interface a protocol; make every protocol uniform across providers; keep the kernel tiny and the seams sharp. Then capability accumulates by writing adapters, not by rewriting the kernel — and adaptability becomes a property of *smallness plus clean seams*, never of pre-built generality. The future is absorbed cheaply at the edges; it is never anticipated in the center.

**Future-proofing, honestly stated.** You cannot future-proof against "whatever AI becomes." You can (a) bet on the most durable protocols available, (b) keep the core small enough that a wrong bet is cheap to redo, and (c) write down the exact signals that say *re-architect* (§23). "Never re-architect" is a fantasy that breeds over-abstraction. "Re-architect rarely, cheaply, and only on a falsifier trip, with a blast radius one adapter wide" is the achievable, healthy version.

---

## §2 · Scope & Non-Goals (the honest envelope)

**In scope:** one operator, one box (an RTX-class GPU + DeepSeek/Anthropic accounts). KEEL is the persistent self + the routing economy + the externality layer + the senses + the flywheel. It is consumed embedded (in-process) or over protocol (any language).

**Non-goals (each a refusal, not an omission):**
- **Not a product, not multi-tenant, not a framework for resale.** "Universal/extensible" means *the things this operator needs* — never a platform surface for others.
- **Not the union of its verticals.** The genome is the **intersection** of the seven systems, never their union. The moment a vertical's heavy apparatus (an appliance's HIPAA escrow, a companion's inter-persona graph) migrates into the core, KEEL stops being able to serve the others. Genome stays small; periphery stays peripheral (§16).
- **Not a new inference engine.** KEEL sits *above* the inference layer (llama.cpp, whisper.cpp, cloud), routing to and supervising it. It is the runtime between apps and inference — the way .NET is the runtime between apps and the CPU — not the inference itself.
- **Not making the small model smarter.** KEEL makes it *trustworthy and productive* via scaffolding + externality. The model's reasoning ceiling is fixed; only its reliability and reach are amplified.
- **Not (yet) expressive.** KEEL perceives (afferent: eyes, ears) as a first-class capability. It does *not* speak (TTS) or generate images (diffusion) — those are *efferent* and arrive later as adapters on a seam that already exists (`ToolHost`), at zero cost to the core (§12).

---

## §3 · The Three Protocol Bets (the foundation)

The kernel speaks only these. Everything is an adapter under one of them; a new provider changes nothing above the adapter.

1. **OpenAI Chat Completions — all cognition.** Every model tier (local, cheap-API, frontier) speaks it natively or through a thin gateway. **Native multimodal vision rides this protocol** as multi-part message content (`image_url`); vision is therefore *not a fourth protocol* — it is a capability of this one.
2. **Model Context Protocol (MCP) — all tools, context, and resources.** Under Linux Foundation governance and broadly adopted; the durable "not-invented-here" tool membrane. KEEL is both an MCP *client* (consuming the world's tools) and, when consumed over protocol, an MCP *server* (exposing its own capabilities).
3. **OpenAPI / HTTP — everything else.** Search, vector stores, embeddings, ASR servers, the long tail. Standard REST, standard auth, standard errors.

**Forbidden:** inventing a fourth wire protocol. Marrow-internal data shapes (`Step`, `Decision`, `Percept`) are *contracts*, not protocols; they never cross the wire as a new dialect.

---

## §4 · Ubiquitous Glossary (forbidden synonyms — CI-enforceable)

| Canonical term | Definition | Forbidden / never |
|---|---|---|
| **genome** | the frozen contracts + kernel + invariants; the core written once, shared by every specialization | not "framework"; not "platform" |
| **cell / specialization** | a vertical built as *genome + periphery* (a companion, a clerk, a conductor) | not "plugin"; not "tenant" |
| **tier** | a routable cognition source: `local` \| `cheap-API` \| `frontier` | never "the model" ambiguously |
| **local** | the on-box workhorse: Qwen3.x via llama-server (text **and** native vision). The sovereign default. | never called "weak" in code/comments |
| **cheap-API** | the metered low-cost reasoning tier (canonically DeepSeek) | **never** "frontier"; never "the API" generically |
| **frontier** | the escalation tier (Anthropic-class, or a larger local model for sovereign hard reasoning) | not "DeepSeek"; not "the big model" |
| **scaffolding-step** | a mechanical, separable, cheaply-verifiable unit of work (read, grep, transform, see, hear) | not "easy step" |
| **core-wire-step** | an irreducible-judgment unit needing reasoning depth the small model lacks | not "hard step" |
| **oracle** | a NON-model assertion of correctness (golden case, property/metamorphic test, runtime behavior, deterministic gate, model-diverse review) | **never** an LLM verifier |
| **verification pass** | a cold-context LLM re-check; catches *drift*, not *systematic* error | not "oracle"; not "proof" |
| **golden case** | an operator-authored, agent-frozen `input → expected` pair; ground truth | not "unit test"; not "contract test" |
| **trust** | the required confidence level of a step; a first-class routable quantity carried on every `Step` | not "temperature"; not "quality" |
| **effort** | the amplification dial: best-of-N width × verification depth × replan budget | not "temperature" |
| **escalation** | routing a step *up* a tier after oracle failure or high difficulty | not "retry" (retry is same-tier) |
| **percept** | a timestamped, modality-tagged, source-attributed unit of perception (a frame description, a transcript segment) | not "observation" loosely |
| **retina / cochlea** | the local sense organ that converts a raw modality → compact text *locally* (Qwen-vision = retina, Whisper = cochlea) | — |
| **afferent / efferent** | sensory-in (perception) vs motor-out (expression/actuation). KEEL is afferent-first. | — |
| **ledger** | append-only, lossless, human-readable files; the system of record (the Tape, the audit log, traces, artifacts) | not "the database" |
| **index** | the queryable, mutable, *derived and disposable* store (SQLite); rebuildable from the ledger | not "the source of truth" |
| **substrate resolver** | the kernel capability that discovers / launches / supervises / routes to whatever local inference the box has | — |
| **verified trace** | a step whose output passed an oracle; the feedstock for distillation | not "log"; not "transcript" |
| **JOINT_WRONG** | implementation + its own tests agree and a golden case disagrees (systematic error; everything looked green) | not "test failure" |
| **narrative register / factual register** | model-authored lossy memory (for voice/relationship) vs lossless/externalized memory (for critical facts) | never conflate the two |

---

## §5 · The Five Invariants + the Reversibility Gate (structural law)

Enforced by type and by CI, on every call. *Where* each is enforced is part of the design:

- **I1 — Observable.** Every protocol call emits a structured audit event. *Enforced in the middleware chain (§8) — unbypassable.*
- **I2 — Durable.** Every state change persists to the run-state spine; any step resumes from checkpoint. *Enforced by the `Spine` + the ledger.*
- **I3 — Filtered (Sovereign).** Two distinct jobs: a **gate** (is this payload sensitive enough to force `local`?) and a **mask** (this is cleared to egress — scrub residual PII/secrets first). Sovereign/PHI data and **raw perception** (screen, webcam, microphone, scanned documents) hit the gate and are forced local — they may never leave the box. **Embeddings are an egress surface too:** a cloud embedder egresses the raw text, and vectors can be partially inverted back to source, so sovereign content must be embedded *locally* and its vectors stay in the local index. The mask is the three-rung privacy layer (§5.1). *Gate enforced in the router; mask + audit in the chain.*
- **I4 — Governed.** Every call's cost is tracked; per-task budgets are hard-stopped. *Enforced in the chain.*
- **I5 — Externalized.** **Every critical output carries at least one assertion no model authored.** The model may author the plan, the code, the contract interpretation, and a verification pass — it may **not** author its own ground truth. *Enforced as a loop stage (`Oracle`), with a pluggable registry.*

**§5.1 · The privacy layer (I3's mask, made concrete).** A safety invariant may never rest on a model judging its own boundary — so the mask is layered, deterministic-first, mapping onto the §10 taxonomy:
1. **Operator sovereign markers** — your own name, addresses, accounts, keys: exact strings, matched deterministically, **agent-frozen like the goldens** (the agent may never edit the privacy policy). 100% recall on *you* — the highest-value rung for a single operator, no model.
2. **Structured regex + checksums** — emails, phones, Luhn-validated cards, key formats (`sk-…`, `AKIA…`), URLs. Deterministic. Rungs 1–2 are the privacy **oracle**: a non-model assertion that PII is present.
3. **The OpenAI Privacy Filter** (a local token-classifier, §13) — the context-aware sweep for what 1–2 structurally can't catch (third-party names, context-private dates, novel secrets). It is a **verification pass**, not an oracle: probabilistic (~98% recall), additive, **never sole** — it shares the model-class blind spot and so can never be the guarantee. Per-cell fine-tunable (general PII vs medical PHI vs code/secrets). The model lands at Stage 2 behind a `GOLDEN_PRIVACY` falsifier; rungs 1–2 ship in Stage 0.

Composition is the reversibility gate applied to privacy: **egress only if all rungs clear; redaction is the union of every rung's spans; the gate fails toward `local`** (leak-uncertain ⇒ treat as sovereign). The privacy model is *necessarily local* — it must see the unredacted payload to redact it, so a cloud privacy filter is a contradiction. Every redaction decision is an **I1-audited event**, so a miss is forensically traceable. For *critical* outputs, prefer forcing `local` over redact-and-egress — redaction-induced context loss can silently corrupt the result (the choice rides the `critical` flag). The mask's real scope is narrow: most sensitive content is gated to `local` and never egresses, so the mask targets the residual `Normal`-class payloads that legitimately reach the cheap-API/frontier tiers.

**The reversibility gate (the standing sixth doctrine).** Before any action whose undo cost cannot be stated in one sentence, KEEL stops and asks the operator — regardless of how pre-authorized the category seemed. Reversibility-uncertain ⇒ treat as irreversible. **The flywheel is the sharpest case: baking a secret into a distilled LoRA is irreversible (undo = retrain; a key sent to a provider can be rotated, a key fossilized into weights cannot) — so the trace→distill path must scrub secrets before a verified trace becomes feedstock, and that corpus becomes an I3 egress the moment a LoRA is ever shared.**

**"Critical" =** any step whose wrong output would silently corrupt the operator's substrate, capital, published output, reputation, or KEEL's own state. The default for code-writing, data-mutating, and anything carrying the operator's name (an email, an application, a financial entry) is *critical*. Non-critical steps ship single-pass.

These six are the line between a production substrate and a prototype, and they are the **closure conditions** of the loop (§8): observable + durable means you can feed it back; filtered means it is safe on your corpus; governed means it cannot run away; externalized means the feedback is trustworthy ground truth, not model self-delusion.

---

## §6 · The Layered Architecture

Dependencies point **down**. Layer N never imports Layer N+1. Enforced by static analysis; violations are bugs, not style.

```
L5  apps          ─┐  consume services; expose KEEL (CLI, OpenAI egress, MCP server, embed lib)
L4  services      ─┤  compose adapters under middleware (route, amplify, verify, memory, perception, driver)
L3  middleware    ─┤  cross-cutting invariants applied to every call (audit, privacy, cost, cache)
L2  adapters      ─┤  thin shims under a protocol (local_llama+vision, deepseek, anthropic, whisper, mcp, store)
L1  kernel        ─┤  manifest · context · registry · chain · lifecycle (+substrate resolver) · engine · lock
L0  contracts     ─┘  the joints: types + the ten traits + the agent-frozen golden cases
```

`{adapters, middleware}` share a layer; neither imports the other. A **service may import middleware** (it composes them); **middleware may never import a service.** The kernel imports only contracts. The contracts import nothing.

---

## §7 · The Joints — Boundary Contracts (frozen)

These ten traits are the genome's load-bearing surface. *Get the joints right and the bones can be swapped.* All are async. Sketched in Rust (the core's language, §20); the shapes are language-neutral.

```rust
// ---- core types (compact) ----
struct Step {
  kind: Kind,                 // Scaffolding | CoreWire
  ty: String,                 // "tool_call:read_file" | "reason:multi_constraint" | "see" | "hear" | ...
  trust_required: Trust,      // the routable trust bar
  data_class: DataClass,      // Normal | Sovereign | Phi   (Image/Audio default → Sovereign)
  tier_history: Vec<String>,
  oracle_failures: u32,
  projected_cost: Option<f64>,
  critical: bool,
  source: DriverId,           // which Driver emitted it (user turn? heartbeat? watch?)
  content: Vec<Content>,      // multi-part: Text | Image | Clip | AudioRef
  golden_refs: Vec<GoldenRef>,
}
enum Content { Text(String), Image(Bytes|Ref), Clip(VideoRef, FrameSpec), AudioRef(AudioRef) }
struct Decision { tier: String, effort: Effort, reason: String }          // tier name or "BLOCK"
struct Verdict  { passed: bool, failures: Vec<String>, joint_wrong: bool, evidence: Vec<Assertion> }
struct Percept  { content: Text|Json, t_utc: Time, modality: Modality, source: SourceId, confidence: f32 }
struct Context  { trace_id, cost_acc, redaction_state, budget_remaining, effort_budget, golden_refs }

// ---- the ten traits ----
trait ModelTier   { async fn generate(&self, req: GenReq, ctx: &Context) -> GenResult;     // cognition (incl. multimodal)
                    fn caps(&self) -> Caps; }                                               // {vision, video, thinking, ...}
trait ToolHost    { async fn list(&self) -> Vec<ToolDef>;                                   // MCP client
                    async fn call(&self, name: &str, args: Json, ctx: &Context) -> ToolResult; }
trait Middleware  { async fn handle(&self, req: GenReq, ctx: &Context, next: Next) -> GenResult; }   // I1/I3/I4 live here
trait Router      {       fn route(&self, step: &Step, ctx: &Context) -> Decision; }
trait Oracle      { async fn verify(&self, out: &Output, refs: &[GoldenCase], ctx: &Context) -> Verdict; }
trait Memory      { async fn assemble(&self, step: &Step, ctx: &Context) -> AssembledContext;        // ringed + budgeted
                    async fn record(&self, trace: &Trace);                                           // → the ledger
                    async fn consolidate(&self) -> Step; }                                           // returns a maintenance Step
trait Spine       { async fn checkpoint(&self, run: &RunId, state: &State);                          // I2
                    async fn resume(&self, run: &RunId) -> Option<State>; }
trait Driver      { async fn poll(&self, ctx: &Context) -> Option<Step>; }                           // initiative / heartbeat
trait TraceSink   { async fn emit(&self, t: VerifiedTrace); }                                        // flywheel feed
trait PerceptionSource { async fn percepts(&self, spec: SampleSpec) -> Stream<Percept>; }            // eyes + ears (afferent)
```

**Proposed golden cases** (operator ratifies and freezes; the agent may never edit a frozen golden):

```jsonc
GOLDEN_ROUTER = [
  { "name": "file-read scaffolding → local",        "in": {kind:"scaffolding", ty:"tool_call:read_file"},          "expect": {tier:"local"} },
  { "name": "multi-constraint reasoning → cheap",   "in": {kind:"core-wire",  ty:"reason:multi_constraint"},        "expect": {tier:"cheap-API"} },
  { "name": "local failed oracle twice → escalate", "in": {tier_history:["local","local"], oracle_failures:2},      "expect": {tier_in:["cheap-API","frontier"]} },
  { "name": "sovereign forces local",               "in": {kind:"core-wire", data_class:"sovereign"},               "expect": {tier:"local", reason_contains:"privacy"} },
  { "name": "image content forces local",           "in": {content:["Image"]},                                      "expect": {tier:"local", reason_contains:"perception sovereign"} },
  { "name": "projected cost over budget → BLOCK",   "in": {projected_cost:9.99, budget_remaining:1.00},             "expect": {tier:"BLOCK", reason_contains:"budget"} },
]
GOLDEN_ORACLE = [
  { "name": "SSN pattern fails a property oracle",  "in": {output:"...123-45-6789...", property:"no_ssn_pattern"},  "expect": {passed:false, joint_wrong:false} },
  { "name": "tests pass but golden fails = JOINT_WRONG", "in": {self_tests_pass:true, golden_violated:true},        "expect": {passed:false, joint_wrong:true} },
  { "name": "claim not traceable to canon → flag",  "in": {claim:"led team of 12", canon_supports:false},          "expect": {passed:false, reason_contains:"INSUFFICIENT_SOURCE"} },
]
GOLDEN_PERCEPTION = [
  { "name": "identical frame is change-gated (dHash≤4)", "in": {frame_delta:2},  "expect": {emitted:false} },
  { "name": "slide change emits a percept (dHash≥15)",   "in": {frame_delta:18}, "expect": {emitted:true} },
  { "name": "silence is VAD-gated",                      "in": {voiced_ms:0},    "expect": {emitted:false} },
]
GOLDEN_MODEL_TIER = [
  { "name": "cost from usage + price (deepseek)", "in": {input:1000, output:500, cache_hit:800, price:{miss:.435,hit:.003625,out:.87}}, "expect": {cost:0.0005249, tol:1e-6} },
  { "name": "tool call valid against schema under constrained decode", "in": {schema:"{path:string}"}, "expect": {tool_call_valid:true} },
]
GOLDEN_RECALL = [    // retrieval quality — the reranker/embedder earn their place here or stay off
  { "name": "reranker beats identity on recall@k", "in": {query:"...", expected_memory_ids:["m7"]}, "expect": {recall_at_5_uplift_over_identity:">= threshold"} },
  { "name": "embedder upgrade beats the MiniLM floor", "in": {set:"golden-recall"},               "expect": {ndcg_uplift_over_floor:">= threshold"} },
]
GOLDEN_PRIVACY = [   // the model earns its place only on what markers + regex miss
  { "name": "operator key caught deterministically (no model)", "in": {text:"...sk-abc123..."},     "expect": {redacted:true, by:"rung2_regex"} },
  { "name": "third-party name the model adds over deterministic", "in": {text:"...met Dana at..."},  "expect": {model_adds_span:true} },
  { "name": "secret never reaches the distill corpus",          "in": {trace_contains_secret:true}, "expect": {distill_feedstock:false} },
]
```

---

## §8 · The Closed Loop (the Engine)

The kernel ships **one canonical cycle** as a default — exotic verticals compose their own from the same traits and still inherit the invariants, because the invariants live in the *chain* and the *spine*, not the engine.

```text
loop {                                          // kernel::engine
  step  = select(drivers).poll()                // user turn OR heartbeat / outreach / watch / consolidation
  win   = memory.assemble(step)                 // ringed, budgeted context — inject only what this step needs
  dec   = router.route(step)                    // cheapest tier that clears the TRUST bar → or BLOCK (I4 / reversibility)
  res   = amplify?(req(win, dec.effort),        // best-of-N + verifier-select if enabled (ships OFF), else single pass
                   |r| chain.run(r, terminal = adapter.generate))   // ← I1 audit, I3 privacy, I4 cost happen HERE
  vrd   = verifier.verify(res, step.golden_refs)// I5 — a non-model assertion; flags JOINT_WRONG; may escalate to human
  spine.checkpoint(run, state)                  // I2
  if vrd.passed { trace_sink.emit(verified(step, dec, res, vrd)) }   // fuel for the flywheel
}
```

This is the self-sustaining cycle: **route → amplify → verify (externally) → record → recalibrate/distill**, each move feeding the next, each verified trace making the next run cheaper and safer. The metric that proves the loop *ignited* is **`escalation_rate` trending down** as the flywheel makes the local model handle more (§19). Below ignition, the externality layer is a tax; above it, the loop runs on its own fuel.

`amplify?` is optional and ships disabled: best-of-N + verifier-selection is the one genome capability the other six systems did *not* independently rediscover (they bought reliability with trained weights and oracle gates), so it earns its place only by passing the §23 amplification falsifier — never by assumption.

> **Implementation note (I3 — per-tier chains).** The single `chain` above is illustrative. Because the privacy mask (I3) differs by destination — `local` stays on the box (pass-through) while every cloud tier is scrubbed before egress — the engine holds **one egress-correct chain per tier** and runs the routed tier through *its* chain, not one shared chain with a static flag. The loop shape is unchanged; `chain` is resolved per `Decision.tier`.

---

## §9 · The Router (the fusion point — the single most important module)

`route(step, ctx) → Decision`, in priority order:

1. **Hard constraints first.** Sovereign/PHI data — *including any raw image or audio content* — forces `local`; non-local tiers are blocked. I3 overrides cost and difficulty.
2. **Trust × difficulty.** A cheap rules heuristic (not a model call): `kind` + step-type + `tier_history` + `trust_required`. Scaffolding, no prior failure → `local`. Core-wire → `cheap-API`. Local that failed its oracle ≥ `ESCALATE_AFTER` (default 2) → escalate one tier. cheap-API that failed → `frontier`.
3. **Effort by tier economics.** `local` is electricity → crank best-of-N. `cheap-API` → N=1–3, lean on the model. `frontier` → N=1. Effort is routable, not constant.
4. **Cost governor.** Projected cost breaches the budget → prefer the cheaper tier, or **BLOCK to operator** (I4 hard-stop).
5. **Cache affinity.** On a quality tie, prefer the tier whose cacheable prefix is already warm (a warm DeepSeek prefix is ~100× cheaper than a cold one).

`Decision.reason` is logged. The router is where the two requirements fuse: requirement #1 (amplify the local model) lives in *how the scaffolding tier is made reliable*; requirement #2 (maximize the cheap API) lives in *how the core-wire tier is made cheap*.

---

## §10 · The Externality Layer (correctness — I5)

| Error class | Caught by | NOT caught by |
|---|---|---|
| **Drift** (impl diverged from contract, rationalized) | a cold-context **verification pass** (fresh LLM) | — |
| **Systematic** (impl + reader misread the spec identically) | an **oracle only** (golden case → property/metamorphic → deterministic gate → model-diverse review → observed runtime) | any same-model-class verification pass — it shares the blind spot |

**Every critical step passes both** a verification pass *and* ≥1 oracle no model authored. The **oracle registry is pluggable** — each cell registers its own: a companion registers a chattiness gate, an autonomous agent a tool-justification gate, a clerk clinical-safety + cross-footing, a job conductor a "every claim traces to the Canon" gate with an `INSUFFICIENT_SOURCE` escape to human. Same slot, different assertions.

**Operator golden cases** are the cheapest true externality and the highest value-per-operator-minute lever. They are **agent-frozen**: the agent may never edit a `GOLDEN_*` constant or the freeze hash; if a golden test fails, fix the *code*. Changing a golden is an operator action.

**The joint-wrong detector** (`svc.verifier`) runs golden cases against an output with *contract + golden + diff but NOT the implementer's reasoning*. If the code passes its own tests yet fails a golden case → **JOINT_WRONG** — the most dangerous finding, because everything looked green. Surfaced to the operator immediately. `mw.oracle` *flags* it from the booleans; `svc.verifier` *derives* it by running the cases.

**The mirror rule:** the cheaper/smaller the model on a step, the more KEEL leans on oracles. This is exactly the local-model case.

---

## §11 · Memory (the self that persists)

Memory is not a store; it is a **ringed, budgeted assembly + a lossless ledger + model-authored consolidation + pluggable retrieval.** The trait is thin (`assemble / record / consolidate`); the stores behind it are swappable.

**The rings** (concentric context, each with its own persistence and budget):
- **Ring 0 — Soul + config** (identity; never compressed; the system message). *Persona lives here, never in the core.*
- **Ring 1 — Calibration exemplars** (voice anchoring).
- **Ring 2 — Working memory** (the live task/conversation; full resolution; trimmed at consolidation).
- **Ring 3 — Compressed history** (rolling narrative; recursively re-compressed so it never overflows — *graceful compression, never a hard wall*).
- **Ring 4 — Retrieved memories** (demand-loaded recall, injected per-cycle).

**The ledger is the Tape, and the Tape is the Spine.** Every turn and percept is appended to a lossless, human-readable, synchronous-to-disk file before anything downstream runs (capture sanctity: a crash loses seconds, never the record). This one append-only ledger serves crash-recovery, resume (I2), the retrieval source, the audit (I1), and the flywheel feedstock (`TraceSink`). The **index** (§13) is derived from it and disposable.

**Consolidation is just a Step.** When Ring 2 crosses its threshold, `consolidate()` returns a *maintenance Step* the engine routes → generates → records like any other — so it inherits I1/I3/I4 for free, and the model writes its *own* persona-shaped summary.

**The two registers (the load-bearing distinction).** Model-authored consolidation is the **narrative register** — lossy, voice-shaped, right for a companion's sense of self. But for *critical* work, model-summarized memory is an I5 violation (a model authoring its own ground truth, which can silently drift). So critical facts live in the **factual register** — the lossless ledger and externalized facts (a job conductor's Canon, a clerk's validated rows), never the persona-shaped summary. Companions lean narrative; work leans factual. KEEL carries both; it never conflates them.

**Retrieval** is a pluggable sub-seam: a `rerank()` that defaults to identity (pass-through) and upgrades to a cross-encoder (optionally persona-weighted). Embedding search by default; dense/sparse/hybrid backend behind the seam; a vector DB only at scale.

**The embedder and reranker are Memory organs, not a tier — the router never routes to them** (they serve `assemble`, off the routing economy). The **reranker is stateless** and ships **off** (`identity`), earning its place per-cell only by beating pass-through on an operator-authored `GOLDEN_RECALL` set — *the `amplify` of retrieval*. The **embedder is format-committing** — the one substrate exception to tier-interchangeability (§20 #13): the index is stamped with an embedder fingerprint `(id, revision, dim)`, and `store` **refuses to serve or rebuilds-from-ledger on mismatch** — a non-model assertion (I5) that the vectors match the resolved embedder, closing a silent JOINT_WRONG on the recall path. The embedder is sovereign-by-default (local; vectors stay local — §5). It may *optionally* double as a model-free routing classifier (embed → kNN vs labeled exemplars), but only as a tie-breaker: the router's hard constraints stay pure rules and never depend on the embedder being resolved.

---

## §12 · Perception (eyes & ears — the afferent senses)

KEEL grows a **sensorium**. The senses are *afferent only* (perception in); *efferent* expression (speech, image generation, actuation) is the `ToolHost`/INTENT seam and is deferred (§2). This boundary is principled — sensory vs motor — and future-proof: voice-out later is a TTS adapter on a seam that already exists, at zero core change.

**Two senses, one seam:**
- **Eyes (retina) — vision.** Qwen3.x is *natively* multimodal (early-fusion), so the eye and the brain can be the **same model**: a frame goes as multi-part content over the *cognition* protocol (§3). *(Operational note: llama.cpp serves Qwen vision via an `mmproj` projector file even though the model is architecturally native — so `mmproj` is a tier-config field the adapter passes when present; native-architecture ≠ no-projector-file.)*
- **Ears (cochlea) — audio.** These weights carry no audio encoder, so the ear is a **separate organ**: Whisper (whisper.cpp/whisper-server) transcribes audio → text *before* it reaches the cognition model. Speaker attribution is **capture topology, not a model** (mic = "me", loopback = "them").

**Both emit a `Percept`** (timestamped, modality-tagged, source-attributed, with confidence) into the same route → amplify → verify → remember loop; perception frames become timestamped events fused into the memory rings (a transcript × screenshots timeline *is* Ring assembly over a multimodal stream).

**Each sense ships with its own change-gate** — the cost control without which perception bankrupts the budget:
- **dHash perceptual dedup** for frames (the model is consulted only on visual change; a static screen is GPU-free; calibrated threshold ~4).
- **VAD** (voice-activity detection) for audio (transcribe only on speech; silence is free).

**The retina pattern — why this is genome-level, not a bolt-on.** Seeing and hearing are *scaffolding* (mechanical, high-volume, sovereign); reasoning about what was seen/heard is *core-wire* (routed). The local sense converts pixels/audio → compact **text** locally, and the expensive or token-capped reasoning brain consumes only the text. This gives *any* brain — local, cheap-API, or a frontier agent with a hard image budget — **sovereign, near-free eyes and ears.** Frames handed onward are pre-resized (`--max-px`, default 1600).

**Sovereignty (I3) applies hardest here.** The screen, the webcam, and the microphone are the most sensitive inputs that exist; raw perception is sovereign-by-default and the router force-locals it. Vision/audio egress is structurally forbidden unless the operator explicitly opts a resized, scrubbed artifact out.

**Scope (first cut):** vision (image/video/screen/screenshot-series) + speech→text. Out: non-speech sound-event classification, voiceprint diarization (topology suffices). A non-LLM capture oracle (TTS speaks a known phrase → loopback → ASR → string-match) verifies the pipeline itself.

---

## §13 · The Substrate (primitives, the resolver, the ledger/index split)

KEEL assumes four **canonical primitives** and *resolves* them rather than embedding weights:

| Primitive | Role | Disposition |
|---|---|---|
| **Qwen3.x (text + native vision)** via llama-server (+ mmproj) | brain + eyes | shared system asset (download-once, pinned in `keel.lock`) |
| **Whisper** via whisper.cpp / whisper-server | ears | shared system asset, pinned |
| **An embedding model** (capable default + a tiny CPU floor) | recall / cheap classification | floor *embedded* (never blind); capable default *shared* — and **format-committing** (§20 #13) |
| **SQLite** | the index / working-state | **baked into the binary** (~1 MB, embeddable) |

**The asymmetry is the whole answer:** SQLite is the one primitive small enough to truly embed; the models are the ones worth *sharing* across every app on the box. KEEL bakes in *knowledge + machinery*, not multi-GB weights — the .NET-runtime model, not the Chrome-embeds-a-model model.

**Two more local organs ride the same resolver:** the **privacy classifier** (the OpenAI Privacy Filter — I3's recall rung, §5.1) and the optional **reranker** (Memory's recall-attention, §11). Both are sovereign-local, pinned in `keel.lock`, behind seams that already exist — neither is a tier, and the router never routes to either. *(Runtime note, to forestall a category error: the privacy classifier is a bidirectional token-classifier with a KEEL-owned Viterbi span-decoder — it runs **in-process via ONNX (`ort`), not llama.cpp**; no GGUF exists or is needed. The Qwen embedder/reranker, by contrast, can use llama-server's embedding/rerank endpoints. "Runs in-process" is a separate axis from "weights baked into the binary": the privacy model's ~1GB weights are a shared download, executed in-process — not embedded like the ~80 MB MiniLM floor.)*

**The substrate resolver** (a kernel capability — formalizes "use whatever exists"):
1. An OpenAI-compatible server already running? (LM Studio `:1234`, Ollama `:11434`, llama-server `:8080`) → use it.
2. Can KEEL launch llama-server / whisper-server from the known paths (`C:\llama.cpp`, `C:\whisper.cpp`, `C:\models`)? → launch + supervise (one long-lived subprocess per server: auto-port, `/health` poll, 3-strike restart, OOM→error mapping — *never* a process per request).
3. A cloud tier configured (`DEEPSEEK_API_KEY`, `ANTHROPIC_API_KEY`)? → route per §9.
4. Nothing → the embedded tiny model for trivial work, else fail honestly.

**The ledger/index split** (storage, decided):
- **Ledger = append-only files** (JSONL/markdown): the Tape, audit, verified traces, the Canon, output artifacts. Lossless, human-readable, user-owned, portable, crash-safe, no schema migration, survives the app. **The system of record.**
- **Index = SQLite** (a `Store` seam): dedup-by-content-hash, the queue, the status pipeline, joins, analytics, retrieval pointers, cost/metric rollups. Transactional (correctness for resume/idempotency). **Derived from and rebuildable from the ledger — disposable.**
- KEEL owns *genome state* (spine/audit/traces/memory-index/metrics) in this Store; *cells own their domain schema* in their own Store. KEEL provides the primitive, never the schema.

**`keel.lock`** pins the substrate (model ids + hashes, server endpoints, tier prices) so a fresh box is reproducibly provisioned and a run is reproducible.

---

## §14 · Module Inventory (the genome — the intersection of the seven)

`[C]` contract · `[K]` kernel · `[M]` invariant middleware · `[S]` default service (swappable) · `[A]` self-exposure. "Seen in" = how many of the seven systems independently built it (intersection evidence).

| Module | L | Role | Seen in |
|---|---|---|---|
| `contracts::types` | C | the core structs + `Content` (multimodal) + `Percept` + `Trust` | 7/7 |
| `contracts::{model_tier, tool, middleware, router, oracle, memory, spine, driver, trace, perception}` | C | the ten joints (§7) | 7/7 |
| `kernel::manifest` | K | declarative config → behavior | 7/7 |
| `kernel::context` | K | the object that flows every call | 7/7 |
| `kernel::registry` | K | tier/provider → adapter factory | 7/7 |
| `kernel::chain` | K | middleware executor — **I1/I3/I4 become unbypassable** | 7/7 |
| `kernel::lifecycle` (+ **substrate resolver**) | K | discover / launch / supervise inference; subprocess health & restart | 7/7 |
| `kernel::engine` | K | the canonical closed loop (§8) — optional to use | 7/7 |
| `kernel::lock` | K | `keel.lock` reproducibility / substrate pin | 4/7 |
| `mw::audit` | M | I1; append-only behind an `AuditSink` trait (heavy hash-chain swaps in) | 7/7 |
| `mw::privacy` | M | I3 mask: 3 rungs (operator markers + regex/checksums = the oracle; OpenAI Privacy Filter = recall verification-pass); force-local gate on sovereign + raw perception; redactions I1-audited | 7/7 |
| `mw::cost` | M | I4; budget cap, hard-stop, reversibility BLOCK | 6/7 |
| `mw::cache` | M | cache-prefix discipline (the 100× lever) | 4/7 |
| `svc::router` | S | the §9 router; degrades to pass-through for single-model cells | 6/7 |
| `svc::verifier` | S | oracle registry runner + joint-wrong detector | 7/7 |
| `svc::amplify` | S | best-of-N + verifier-select — **ships OFF** | 1/7 |
| `svc::memory` | S | ringed assembly, the Tape ledger, consolidation, pluggable retrieval | 7/7 |
| `svc::perception` | S | capture + change-gate (dHash/VAD) + sample + the `see()`/`hear()` retinas | 5/7 |
| `svc::driver` | S | user-turn + heartbeat/watch/outreach drivers (initiative) | 4/7 |
| `store::sqlite` | L2 | the index behind a `Store` seam (+ `sqlite-vec`); the ledger is files | 6/7 |
| `adapters::{local_llama(+vision), whisper, deepseek, anthropic, mcp, embedding, reranker, privacy}` | L2 | the tiers + ears + tools + the Memory organs (embed/rerank) + the privacy classifier, under their protocols | 7/7 |
| `app::serve_openai` | A | OpenAI-compatible egress — apps consume KEEL as a brain | 3/7 |
| `app::serve_mcp` | A | expose KEEL itself as an MCP server | 2/7 |
| `app::embed` | A | in-process library API (Rust apps link this) | 2/7 |
| `app::cli` | A | the daily-driver `keel` command | 3/7 |

**Honest accounting:** ~6–9k lines of Rust — smaller than a product because it is the *intersection*. Every module is comprehensible in one session; none is permitted to grow into a God Module (a per-module token budget, warn/fail, is CI-enforced).

---

## §15 · How KEEL Is Consumed (embed *or* protocol)

Consumed two ways — which is *why* the self-exposure is core, not an app, and why one core serves Rust, Python, and C# alike:
- **Embedded** (in-process, native): Rust apps (Tenancy, Terminal) link `app::embed` and call the engine directly.
- **Over protocol** (any language): Python/C# apps (TARS, REEL, the Companion, SEXTANT; photo2deck, NightScribe, NightClerk) talk to a KEEL binary via `serve_openai` (OpenAI-compatible egress), `serve_mcp` (KEEL's capabilities as MCP tools), or plain HTTP.

The contracts (§7) are the fixed thing; the binary is the fixed thing. The genome is consumed, not copied.

---

## §16 · The Refusal List (intersection, not union)

What is *not* in the genome — periphery, mapped to the cell that pulls it:

| Refused from the core | Pulled by | Lives in |
|---|---|---|
| ASR/TTS/avatar/UE5; STAGE telemetry render; **all efferent expression** | Companion, TARS | perception/actuation *adapters* + app |
| Vector RAG stack (Qdrant, reranker, BM25) | Companion | a `Memory`/`Store` *impl* |
| Heavy trust (Shamir escrow, post-quantum signing, pfSense, encryption-at-rest, fleet telemetry) | Companion | heavy `AuditSink`/`EgressFilter` impls + *deployment* |
| Training pipeline (LoRA/DPO rounds) | Tenancy, TARS, Companion | out-of-band; core only *emits + stores* verified traces |
| Sandbox / code execution | TARS | an **MCP tool** |
| Multi-persona, inter-persona channels, federation | Tenancy | services *built on* the genome |
| Document generation (DOCX/PDF), ATS/form-fill, dispatch | SEXTANT | domain periphery + reused fleet tools |
| Specific UI (CRT shaders, React, MetaHuman) | all | app layer |

If a cell ever needs a *kernel or contract* edit to be built, the abstraction is wrong (§23) — that is the test that keeps this list honest.

---

## §17 · Differentiation — the seven (and SEXTANT next)

Each cell = **genome + periphery**:
- **Terminal** = `serve_openai` + a CRT app. *(Pure L5 client — built waiting for exactly this.)*
- **Tenancy/Dave** = `embed` + persona bundles + an outreach `Driver` + a consolidation `Memory` impl + LoRA flywheel + Tauri UI.
- **TARS** = genome + STAGE/INTENT perception+actuation adapters + a REEL `Memory` impl + a PULSE `Driver` + a sandbox tool + self-retraining + a personality LoRA.
- **In-Home Companion** = genome (heavy audit/egress impls) + ASR/TTS/avatar + a vector `Memory` + clinical `Oracle`s + multi-provider routing + fleet/trust periphery.
- **photo2deck / NightClerk** = genome + the vision retina + a `Store` of structured rows + schema-constrained `Oracle`s + recipes.
- **NightScribe** = genome + eyes **and** ears + timeline-fused `Memory` + a `watch` `Driver`.
- **SEXTANT** *(the first real build-on-KEEL)* = genome's `engine` (the Conductor) + `Router` (Claude/Cerebras/local) + `Oracle` (the Truth Gate, `INSUFFICIENT_SOURCE`→human) + factual-register `Memory` (the Canon) + `Store` (its SQLite schema) + `ToolHost` (Gmail MCP) + the vision retina (JD/DOM reading) + I3/reversibility (PII filter + approval gate). Its build surface collapses to job-domain periphery — discovery resolvers, tailoring, document generation, dispatch — once KEEL supplies the rest. It is the most agentic of the seven and therefore the ideal first proof.

That seven systems across three languages each rebuilt this skeleton by hand *is* the empirical proof the genome is at the right altitude — and the cost of not having had it (duplicated routers, hand-rolled memory, bolted-on harnesses) is the value proposition, already demonstrated.

---

## §18 · Error Taxonomy

| Code | Meaning | Retryable | Operator-visible |
|---|---|---|---|
| `ROUTE_NO_TIER` | no tier clears the bar within budget | no | yes (block) |
| `TIER_UNAVAILABLE` | provider/server down or OOM | yes (failover) | no |
| `ORACLE_FAIL` | output failed an oracle | yes (retry/escalate) | on exhaust |
| `JOINT_WRONG` | tests+code agree, golden disagrees | no | **yes, immediately** |
| `BUDGET_EXCEEDED` | task cost cap hit | no | yes (hard-stop) |
| `ESCALATION_EXHAUSTED` | all tiers failed the oracle | no | yes (block) |
| `REVERSIBILITY_BLOCK` | undo cost unstatable in one sentence | no | yes |
| `INSUFFICIENT_SOURCE` | a claim cannot be grounded in ground truth | no | yes (→ human) |
| `PERCEPT_LOW_CONFIDENCE` | a sense returned below its confidence floor | no | yes (→ review lane) |
| `SUBSTRATE_UNRESOLVED` | no inference layer found or launchable | no | yes |
| `GRAMMAR_VIOLATION` | constrained decode produced invalid output (should be impossible) | yes | on repeat (adapter bug) |

---

## §19 · Non-Functional Requirements

- **Footprint & cold-start:** a single static binary; starts in milliseconds; runs anywhere with the substrate present. (The reason for native — §20 — is this, not orchestration CPU speed.)
- **Local latency:** p50 single-pass < 3 s at ≤8K context; best-of-8 in parallel < 6 s wall-clock.
- **Cache-hit rate (cheap-API):** > 70% of input tokens on repeated/session work served as cache hits.
- **`escalation_rate`:** measured per run; **target = downward trend** as the flywheel runs. A flat curve after N cycles falsifies the flywheel (§23).
- **`rework_rate`:** < 10% and stable; rising means convincing-wrong output is slipping past the oracles (I5 failing).
- **Sovereignty:** zero sovereign-marked bytes — *including raw frames and audio* — egress to a non-local tier. A single violation is a P0.
- **Perception thrift:** a static screen and silence cost zero inference (change-gating works).
- **Post-compaction TTP:** < 60 s — the run is compaction-indifferent (plan + state live in the spine, not the window).

---

## §20 · Technology Decisions (ADRs, condensed)

1. **OpenAI Chat Completions as the one cognition protocol** — wire-level isomorphism across all tiers; new providers are adapters; the kernel never changes.
2. **MCP as the tool protocol** (LF-governed, durable) — KEEL is client and (over-protocol) server. Never invent a fourth protocol.
3. **Native vision rides chat-completions** as multimodal content; it is a *capability*, not a protocol. Audio (Whisper) is a separate organ that pre-transcribes to text.
4. **Afferent-only senses now; efferent via `ToolHost`, deferred.** Eyes and ears are genome; mouth and image-gen are periphery on an existing seam.
5. **A native, memory-safe, single-binary, embeddable core — Rust, from Stage 0 (decided; not "earned into later").** Two consumption modes are *both* primary: **(a)** a standalone, owned harness for work a closed vendor's coding agent structurally can't reach (your own job-applier, home-assistant — sovereignty, no vendor-roadmap risk), and **(b)** an in-process primitive embedded across your polyglot apps (UE5/C++, Tauri/Rust, .NET/C#) — the *.NET-of-your-AI-apps* reuse model. Mode (b) requires native + C-ABI-embeddable, which rules out C#/Python for the core. Among native options it is **Rust over C/C++**, and the deciding argument *strengthens* in an agent-authored world: when the model writes *and* reviews the code, you want more machine-checked guarantees it cannot talk past — the borrow checker is a **non-model oracle (I5) for the memory/concurrency bug class**, the one a 24/7 persistent self can least afford (a data race that passes every test and review is a `JOINT_WRONG` inside the kernel). C/C++ trades that compile-time oracle for runtime, probabilistic sanitizers — a *verification pass, not an oracle* (§10). C++ stays the documented fallback (craft / UE5 zero-FFI / longevity); if taken, ASan/UBSan/TSan-in-CI + the golden gates are the substitute oracle. **The language is the most reversible decision in the system:** the frozen golden cases are a *language-neutral conformance layer* (§7), so any reimplementation merely re-passes the same goldens — the contracts, not the language, are the longevity asset. **Python (Marrow-L1) is retained as the reference bench** the Rust core diffs against (the ASTRA-textverse pattern), never as a shipped artifact.
6. **Polyglot above the core via the protocol membranes** — Rust embeds; Python/C# consume over HTTP/MCP/egress. One genome, three+ languages.
7. **Hot paths only, surgically, via FFI** (tokenizer, grammar compiler, PII scanner, metrics writer) — if a profiled one ever appears. It probably won't.
8. **SQLite baked in as the index; append-only files as the ledger.** The ledger is truth; the index is a disposable, rebuildable cache. `sqlite-vec` for vectors; a vector DB only at scale.
9. **Substrate resolver, not embedded weights.** KEEL rides whatever local inference the box has (its own llama-server, an existing LM Studio/Ollama, or cloud), and pins it in `keel.lock`.
10. **Constrained decoding** (GBNF / JSON-schema) on the local tier — the small-model reliability lever; an API-tier fallback uses JSON-mode + schema validation.
11. **Best-of-N + verifier-select ships disabled** — it is a hypothesis (§23), not an assumption.
12. **Quantization bound Q4_K_M–Q5_K_M** for local models; ship the matching `mmproj` for vision; pin a Whisper model + a fast fallback.
13. **Interchangeability has a boundary: stateless cognition is swappable; format/corpus-committing components are pinned harder.** The generator, the reranker, and the senses' models are stateless — swap freely (§0). The **embedder is not** — it commits the index to its vector space, so swapping it is an *index migration*, not a hot-swap (de-risked by the ledger/index split: re-embed from the Tape). The same rule governs the prompt-cache prefix and the distilled LoRA. Pin these in `keel.lock` harder than the generator; the index carries an embedder fingerprint (§11).
14. **Privacy is layered, deterministic-first (§5.1):** operator markers + regex/checksums are the non-model *oracle* and ship in Stage 0; the OpenAI Privacy Filter is a local *verification pass* that lands in Stage 2 behind a falsifier and may never be the guarantee. The privacy model is necessarily local and per-cell fine-tunable.

---

## §21 · Staged Build Plan (each stage independently useful; falsifier-gated)

Stop at any stage where the lift stops justifying the time — that judgment is itself a deliverable. **Bank clean rather than build speculatively on an unconfirmed foundation.**

- **Stage 0 — The spine.** contracts (frozen) + kernel (manifest, context, registry, chain, lifecycle+resolver, engine, lock) + `local_llama`/`deepseek` adapters + `mw.audit`/`mw.privacy`/`mw.cost` + the file ledger + `store::sqlite` + `app.cli` + `serve_openai`. **Outcome:** a binary that resolves the substrate, talks to both tiers, logs every call, and is consumable embedded or over protocol. **Falsifier:** if this exceeds ~2 weeks, the protocol-first/native-core thesis is wrong for this build.
- **Stage 1 — Amplification & senses.** constrained decoding; `svc.router` (difficulty + trust + escalation); `svc.amplify` (best-of-N + selector); `svc.perception` (eyes + ears, change-gated); `mw.cache` (prefix discipline); the `Driver` seam. **Outcome:** the local model does multi-step work reliably, sees and hears sovereignly, and cheap-API calls are cache-cheap. **Falsifier:** if verified best-of-N can't beat single-pass on a fixed benchmark, ship `amplify` OFF permanently and keep local as a pure tool/perception tier.
- **Stage 2 — Correctness & memory.** the agent-frozen golden registry + freeze check; `svc.verifier` (oracle registry + joint-wrong); ringed `svc.memory` (Tape ledger, consolidation-as-a-Step, narrative/factual registers); `mw.metrics` (`escalation_rate`, `rework_rate`). **Outcome:** critical outputs carry a non-model assertion; the substrate measures itself. **Falsifier:** if `rework_rate` can't be driven < 10% with oracles on, the oracle taxonomy needs rework before more autonomy.
- **Stage 3 — The flywheel.** `svc.distill` (verified-trace store → LoRA; teacher = cheap-API/frontier, student = local); cache-prefix tuning; the multi-chain persistent context at scale. **Outcome:** `escalation_rate` trends *down* over cycles. **Falsifier:** flat after N cycles → the flywheel doesn't compound; keep the local model as a workhorse and route hard reasoning to cheap-API permanently (still a complete, useful system).
- **Proof: build SEXTANT on KEEL.** The first real cell. Done = SEXTANT's Conductor/Router/Gate/Canon/State all come *from* KEEL unchanged; only job-domain periphery is written. If SEXTANT forces a kernel edit, KEEL's boundary is wrong — fix KEEL, in the same change, before continuing.

---

## §22 · What Will Kill This Build

1. **Building a product instead of an L1 tool.** The moment "universal/extensible" grows features for hypothetical users, it is the general-agent-framework this doctrine forbids. Defense: every feature checked against §1–§2.
2. **Letting the genome become the union.** A vertical's heavy apparatus migrating into the core. Defense: the §16 refusal list + the §23 "kernel-edit" falsifier.
3. **Over-abstracting for an imagined future.** Generality machinery built for futures that never arrive. Defense: clean seams + reserved-optional defaults, never speculative plumbing; the falsifiers are the future-proofing.
4. **Loyalty to local over economics.** Squeezing the local model on steps that should route up. Defense: the router decides on cost + trust + oracle, never on "it should be local."
5. **Cache-discipline rot.** Prompts scramble, cache misses, "cheap" evaporates. Defense: `mw.cache` is structural; cache-hit-rate is an NFR.
6. **The oracle becoming an LLM.** A same-model verification pass quietly standing in for a real oracle on a critical step. Defense: I5 + the §10 taxonomy + the agent-frozen golden CI check.
7. **Perception with no change-gate.** Re-captioning identical frames / transcribing silence until the budget burns. Defense: dHash + VAD ship *with* the senses, not after.
8. **Confusing the registers.** Trusting model-authored narrative memory for critical facts. Defense: the factual register is lossless/externalized; critical facts never live in a persona-shaped summary.
9. **Embedding what should be shared.** Baking multi-GB weights into the binary. Defense: the substrate resolver + `keel.lock`; only SQLite (and maybe a tiny embedder) is embedded.
10. **Theorizing instead of shipping Stage 0.** Defense: Stage 0 is weeks; the next artifact is a running binary that resolves the substrate and talks to both tiers.

---

## §23 · Falsification Conditions (when to revise, not extend)

- **Architectural:** a capability shift requires editing the kernel or any non-adapter layer → the protocol-first thesis was wrong; the abstraction was at the wrong level. Adding a provider takes > 1 day → the adapter abstraction is too thin. A middleware change forces adapter edits → the chain leaks. **A cell can't be built without a kernel/contract edit → the genome boundary is wrong.**
- **Seam-specific:** one `Driver` can't express idle-outreach *and* a perpetual heartbeat *and* watch-on-change → the initiative abstraction is wrong. `Memory` can't host a ring archive *and* a hash-chained canon *and* SQLite-consolidation behind one trait → the memory seam is mis-cut (the highest-risk seam; design it most skeptically). Rust apps can't embed the core in-process and are forced to a sidecar → the native/embeddable bet failed (fall back to protocol-only).
- **Amplification:** verified best-of-N on local doesn't beat single-pass on a fixed benchmark → demote local to a pure tool/perception tier; ship `amplify` OFF.
- **Recall & privacy:** the cross-encoder reranker can't beat `identity` on `GOLDEN_RECALL` enough to justify its latency → ship `rerank: identity` permanently (embedding-search-only is complete). The capable embedder can't beat the MiniLM floor enough to justify the download + full re-index → keep the floor. The privacy *model* can't catch enough that markers+regex miss, on `GOLDEN_PRIVACY`, to justify the dependency → deterministic-only is a complete privacy layer.
- **Correctness:** `rework_rate` can't be held < 10% with oracles on → I5 isn't earning its keep; rework the taxonomy before more autonomy.
- **Economic:** KEEL's own overhead (extra sampling + verification + perception) costs more than it saves vs. calling cheap-API single-pass for everything → the amplification economics don't close at current prices; collapse to a cheap-API-first router and keep local for sovereign/offline/perception only.
- **Flywheel:** `escalation_rate` flat after N LoRA cycles → the verified-trace loop doesn't compound; keep teacher-distillation only if it independently moves the number.

---

*KEEL v0.2.0 — the backbone written once. The router fuses cost and trust; the senses perceive sovereignly; the amplification raises the floor; the oracle layer sets the bar; the memory is the self that persists; the flywheel compounds. The contracts are frozen and outlive every language; the implementations refine forever behind them. Build Stage 0 first. The spec is finished the moment it stops being a substitute for a running binary.*
