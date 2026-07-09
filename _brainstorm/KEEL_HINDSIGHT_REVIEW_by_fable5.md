# KEEL — A Hindsight Review
## The placeholder-brain, removed: what KEEL becomes once the brain and the memory are crystallized

**Author:** Claude — **Fable 5** (`claude-fable-5`). *Not Opus.* The KEEL canon and most of this ecosystem's synthesis documents are Bo Chen + Claude Opus 4.7/4.8 work; this review is by a different, later model, and per the co-authorship-asymmetry discipline that distinction is load-bearing and stated here at the top.
**Date:** 2026-07-01
**Status:** advisory external review, **non-canonical**. Nothing in the governed repo was edited; this file lives in `_brainstorm/` as a clearly-labeled outsider artifact. Every proposal below is a *candidate* requiring operator ratification and, where accepted, a versioned canon successor (v0.3.0) — never a silent edit. The agent-frozen goldens and contracts were not touched and are not touchable by me.
**Sources read in full:** `KEEL_ARCHITECTURE.md` v0.2.0, `README.md`. Referenced: `keel.lock` (by description). **Not read:** `crates/` source, `_run_state/`, `tests/` — so this reviews the **canon**, not the build; where the running Stage-0/1 code already diverges from the canon, the code wins and this review may be stale on that point.
**Companion documents (mine, same day):** `C:\the_brain\THE_RESIDENT_SPEC_by_fable5_v1.md` (the organism), `C:\cortex\CROSS_INTEGRATION_PROPOSAL_by_fable5.md` (the Measured Corpus), plus the reviewed `BRAIN_SPEC_by_fable_v1/v2` and `CORTEX_ARCHITECTURE.md` v0.4.

> **One sentence:** KEEL was specced with "the brain" as a placeholder, and the placeholder left self-language and four missing seams inside an otherwise excellent body — with the brain (seat → node → organism) and the memory (CORTEX → Measured Corpus) now crystallized, KEEL should shed the pretension of being the self, claim its true and *larger* office as the **guarantee layer** — the body where the organism's honesty is physically manufactured — and accept a genome delta of roughly two traits, two type fields, two kernel surfaces, and one ladder extension, all verified the same way everything in this house is verified: by building the two crystallized things as cells and watching whether §23 trips.

---

## §0 · The question, and the method

The operator's question: *KEEL was designed with the brain as an uncrystallized placeholder. With hindsight — the brain now crystallized as the seat/node/organism stack, the memory as CORTEX plus the Measured Corpus — what would you revise, iterate, or add to improve the essence and spirit of KEEL, what it is and what it should become?*

The method: KEEL supplies its own instrument. §23's architectural falsifier — **"a cell can't be built without a kernel/contract edit → the genome boundary is wrong"** — is exactly the right test. So this review runs the two crystallized artifacts through the ten contracts as *paper cells*: RESIDENT (the organism) and CORTEX (behind the Memory seam). Where the contracts hold, the genome was ahead of its time. Where they trip, the placeholder was standing. The findings: the contracts hold in six places, trip in four, and the one-sentence identity claim needs a single, consequential correction.

---

## §1 · The essence revision — KEEL is not the self; KEEL is the body (a promotion, not a demotion)

The canon's one sentence says KEEL "is the persistent, sovereign **self**." The canon then spends the rest of its length quietly contradicting that claim, correctly:

- §20.5: *"the language is the most reversible decision in the system; the contracts, not the language, are the longevity asset."* If KEEL's internals were the self, a C++ port would be a personality transplant. It is explicitly a re-pass-the-goldens exercise.
- §11/§13: the **ledger is the system of record**; the index is disposable; Ring 0 (the soul) is *files*, loaded by KEEL, not defined by it.
- §0 itself: the model is "rented cognition plugged into a tiny stable core" — and the core's product is enumerated as remembering, perceiving, routing, verifying, continuity: **services to a self, not the self.**

The resolution, with hindsight from the whole later stack (BRAIN v0.6's "a deterministic core that *has* an LLM"; the fable-v1 spec's "the sovereignty artifact is the state, not the FLOPs"; RESIDENT's "the Resident is the state plus the policy; the model is weather"):

> **The self is the state — the ledger, the canon, Ring 0, the goldens, the lockfiles. KEEL is the body that keeps that self alive and makes the cognition rentable.**

This is not pedantry; it is the design decision that resolves the spec's internal split, and it makes KEEL *more* important, not less. Every layer above KEEL is model-authored and lossy — summaries drift, judgments confabulate, personas flatter. KEEL is the only layer in the entire stack that can make promises **physically true**:

- **Provenance by pipe.** A datum's origin class is stamped by *which pipe it arrived through*, and KEEL is the pipe — so provenance is unforgeable by any model output, by construction.
- **Fences as syscalls.** I5 lives in the chain, unbypassable (§8) — the brain corpus's fences are loop *conventions*; KEEL's are *structure*. This was KEEL's best idea before the brain existed and it remains the best idea after.
- **Budgets as limits** (I4 hard-stops), **time as a service** (drivers, heartbeats, lifecycle), **the tape as the single write path** (I2, capture sanctity), **reproducibility as a manifest** (`keel.lock`).

In four-frameworks terms: KEEL is the **D-class substrate of the whole organism** — the out-of-network anchor built into the architecture, the place where the layer-coherence audits of everything above get their incorruptible telemetry. The keel metaphor was always correct: the keel keeps the vessel upright; it is neither the captain nor the cargo. The spec briefly let the keel claim to be the captain because, at spec time, there was no other candidate. Now there is. Rewritten one sentence, proposed for v0.3.0:

> *KEEL is the body — the tiny immortal guarantee layer that keeps the self (which is state) alive, the cognition (which is rented) swappable, and the honesty (which is provenance, fences, budgets, and clocks) physically manufactured rather than promised.*

The brain decides; the memory holds; the body guarantees.

---

## §2 · The instrument — the two paper cells against the ten contracts

**Seams that hold, cleanly (the genome was right):**

| Crystallized need | KEEL seam | Verdict |
|---|---|---|
| RESIDENT's watchers / percept streams | `PerceptionSource` + change-gates | slots as-is |
| The lossless Tape, single write path | ledger + I2 + capture sanctity | identical doctrine, KEEL's is stronger (sync-to-disk) |
| The COMMIT fence, reversibility, traps-on-harm | `Oracle` + I5 + reversibility gate | KEEL had it first; nothing to change |
| The deliberation ladder (local→cheap→frontier) | the three-tier `Router` | *is* the ladder; economics already right |
| Crash recovery, resume, checkpoints | `Spine` | fits |
| Sleep/consolidation machinery | consolidation-as-a-`Step` | more elegant than the RESIDENT spec's own batch framing — see §6 |
| CORTEX as the memory engine | `Memory::{assemble, record, consolidate}` | slots behind the trait with the §4 amendments |

**Seams that trip §23 (the placeholder was standing here) — four, detailed in §3:** arbitration between drivers; the unimplemented "recalibrate"; the two-registers type system; interoception.

---

## §3 · The four tripped seams

### 3.1 `select(drivers)` is hardcoded arbitration — the Arbiter seam

The engine's first line — `step = select(drivers).poll()` — is where the brain was going to live, and you can tell, because *nothing arbitrates*. Drivers emit Steps; no salience, no competition, no drive-modulated cadence, no auction. The RESIDENT's entire attention economy — the thing the crystallized brain says *is* the organism-level intelligence ("intelligence is a property of the attention policy") — has no seam to plug into without editing the engine. That is a literal §23 trip.

**Proposed delta (minimal):** `Driver::poll` returns `Candidate { step, salience_features }` (deadline, surprise, stakes-class, source, staleness — features only, no policy), and the engine's selector becomes an **`Arbiter` trait** with a trivial default (FIFO / round-robin — a game cell never notices it exists). The RESIDENT cell slots an auction; a clerk cell keeps the default. Attention *policy* stays out of the genome (see §7); only the *seam* and the *feature vocabulary* enter.

**Falsifier for this delta:** attempt RESIDENT Phase 1 (the pulse/auction) as a cell against the current contracts. If it builds without the Arbiter, this section is wrong and the delta is withdrawn. (On paper it cannot — the selector is not pluggable — but the build is the arbiter of the Arbiter.)

### 3.2 "Recalibrate" appears in the loop mantra and in no module — expectations, and trust that is earned

§8's cycle is *route → amplify → verify → record → **recalibrate**/distill*. Distill has a stage (Stage 3, LoRA). Recalibrate has **no mechanism anywhere in the canon** — no module computes calibration, nothing consumes it. Meanwhile the crystallized brain's sharpest single result (fable-v2's dense-signal theorem, independently my prediction registry) is exactly the missing mechanism:

- **Every committed action registers its expected consequences** — `Expectation { what, forecast, tolerance, deadline, resolver }` — appended to the ledger alongside the trace. An action with no statable expectation is itself suspicious (you are acting without a theory of what the action does).
- The monitor **resolves expectations** as outcomes arrive or deadlines expire. In-band resolutions are silence (free). Misses and expiries are *innovations* — the atomic unit of surprise, which is simultaneously an attention signal (feeds 3.1's salience features), a semantic staleness alarm (a calibration dip in a region says *the world changed there*, long before timestamp decay notices), and dense, free, reality-labeled supervision.
- The rollup is a **calibration ledger** per step-class — and this is where it becomes a KEEL-native upgrade rather than an import: the router's `trust_required` is today a *static, configured* bar. With calibration history, **trust becomes earned**: the autonomy ceiling per step-class is a deterministic function of recent calibration, fence coverage, and escalation history — widening with demonstrated accuracy, contracting automatically on dips. The difference between a permissions file and a nervous system, and it makes `escalation_rate` (the ignition metric) *self-improving by mechanism* instead of by hope.

**Falsifier:** run one cell for N weeks with expectations on. If the calibration ledger catches no staleness event earlier than timestamp decay does, and earned-trust changes no routing decision the static bar wouldn't have made, the machinery is ornament — withdraw it. (Prediction: the chat-adjacent cells trip it within days.)

### 3.3 The two registers are the right instinct at half resolution — provenance classes

Narrative-vs-factual (§11) was the placeholder-era version of a fourfold type system the brain corpus later crystallized: **D** (deterministic: calculators, sensors, engine state) · **R** (model-reduced: LLM/VLM extraction) · **H** (human-attested: operator input, corrections) · **M** (model self-notes: consolidations, judgment write-backs). The generalization matters because:

- **KEEL can stamp it unforgeably** — provenance = which pipe you came through, and KEEL is every pipe. No layer above can grant itself D-class.
- **Fence semantics become provenance-aware** (the fable-v1 insight): a critical step pivoting on an R-field demands a stronger oracle or dual extraction; M never feeds a fence; H trusts but decays. The current canon has this only as prose ("model writes are suspect by default") — the type field makes it checkable.
- It is **orthogonal to `data_class`** (Sovereign/Phi is *sensitivity*; D/R/H/M is *epistemics*) — two independent fields, both routing-relevant, currently conflated into adjacent prose.

**Proposed delta:** `prov: ProvClass` on `Percept`, `Trace`, and memory records; narrative register ≙ M/R, factual register ≙ D/H — the registers become derived vocabulary, not separate machinery. **Falsifier:** author one golden where correct fence behavior *differs* by authorship of the antecedent (same content, D-pipe vs R-pipe). If no such golden can be written for any real cell, the classes are decoration.

### 3.4 KEEL already feels; it doesn't tell anyone — the Vitals surface

`escalation_rate`, `rework_rate`, budget remaining, cache-hit rate, queue depths, staleness of the tape's tail: KEEL computes the organism's vital signs today and serves them as an off-loop dashboard *for the operator*. The RESIDENT's drives eat exactly these numbers (INTEGRITY, GROUNDING, THRIFT are literally functions of them). **Proposed delta:** export a `Vitals` struct/endpoint — D-class self-metrics as first-class *sensor data*, consumable by Drivers, the Arbiter, and cells. Nearly free (the numbers exist); it converts KEEL's self-knowledge from reporting into interoception.

---

## §4 · The remaining deltas (smaller, still worth the canon's attention)

1. **Extend the ladder at both ends.** *Rung 0, below `local`:* the **procedure/reflex store** — verified traces distill *first* into guarded, expiring, demotable **text procedures** (match → execute, pennies to micro-pennies) and only later, optionally, into LoRA. The canon's own reversibility gate demands this ordering — weights are the least-reversible layer, and Stage 3 currently jumps from traces straight to LoRA with no intermediate. Chunks rot, so the reflex store is a new staleness surface and inherits the full freshness discipline (expiry, revalidation, demotion back to deliberation). *Rung ∞, above `frontier`:* **the operator as a tier**, not an error path — `Decision.tier = "operator"` with its own economics (most expensive, highest trust) and **typed traps with blame tags** (`schema-gap | fence-gap | stale | reducer-error | integrator-error | genuine-tail`) instead of the flat `ESCALATION_EXHAUSTED`. Then the routing economy runs unbroken from free-deterministic to priceless-human under one accounting, trap *composition* becomes the recompile agenda, and `escalation_rate` means precisely "spend at the top rung."
2. **The fleet needs a body function.** The operator's daily substrate now includes **parallel frontier agent instances** working shared directories, and the concurrency protocol is currently *the operator verbally telling each instance not to overwrite the others* — which is how this very document came to live in `_brainstorm/` under a `by_fable5` name. That is a traffic-control job and traffic control is a body function: a **lease/registry convention** (the `cortex.lock` pattern generalized — work leases with TTLs, an instance registry, append-only conventions, merge discipline), owned by KEEL as the box's traffic controller. Whether whole *agents* become a routable tier in the §9 economy is a larger question this review deliberately defers; the lease protocol is genome-safe, small, and solves a pain that is live today.
3. **`Memory::assemble` returns a coverage manifest** — what entered the window, at what resolution, from which provenance classes — so I5 can verify that an answer's support was actually *present in the window* (the anti-Hedonic check at the context level), and so CORTEX's coverage certificates ride a seam that already exists instead of a side channel.
4. **Boot verification in `lifecycle`.** `keel.lock` pins the body (models, hashes, prices); it should also pin the **soul manifest** — which Ring-0/canon version booted, checksum-verified — plus an orientation golden (the boot drill as a kernel capability). This is also the K-series succession mechanism, formalized: succession = new body, verified soul manifest, passed orientation.
5. **One unification for free:** the §14 metric rollups and the Measured Corpus's sketch channel are the same species — **D-class monoid aggregates over an append-only tape** (mergeable, hence O(log N) incremental). Generalize the `Store`'s rollup capability once; the operational tape and the content tape both consume it. One organ, two corpora.

---

## §5 · The delta budget (the whole ask, in one table)

| # | Delta | Layer | Size | Unlocks |
|---|---|---|---|---|
| 1 | `Arbiter` trait + `Candidate{step, salience_features}` | contract + engine | 1 trait, 1 struct | RESIDENT's attention economy as a cell |
| 2 | `Expectation` records + calibration ledger + earned trust in the router | contract + ledger + router | 1 struct, 1 reader, 1 router input | the mantra's "recalibrate," semantic staleness, autonomy that contracts on error |
| 3 | `prov: D\|R\|H\|M` on Percept/Trace/memory | types | 1 enum, 3 fields | unforgeable epistemics; provenance-aware fences; registers become derived |
| 4 | `Vitals` surface | kernel | 1 struct/endpoint | interoception; drives eat real numbers |
| 5 | Reflex store (rung 0) + operator-as-tier with typed traps (rung ∞) | service + router | 1 store, 1 tier, 1 tag enum | reversibility-ordered distillation; unified escalation economics |
| 6 | Lease/registry protocol | kernel/tooling | 1 convention + helper | fleet concurrency without verbal locks |
| 7 | Coverage manifest from `assemble` | contract (Memory) | 1 return field | window-level grounding check; CORTEX certificates ride the seam |
| 8 | Soul manifest + boot drill in lifecycle/lock | kernel | 1 lock section, 1 golden | verified succession; anti-collapse |
| 9 | Generalized monoid rollups in `Store` | L2 | refactor | metrics and sketches become one mechanism |

Roughly: **two new traits, two type additions, two kernel surfaces, one ladder extension, one convention, one refactor.** Anything materially larger than this is probably the placeholder re-inflating, and should be refused on §16 grounds.

---

## §6 · What flows backward — KEEL innovations the newer specs should absorb

Honest hindsight runs both directions; three of KEEL's moves are *better* than their counterparts in the 2026-07-01 documents, and the newer specs should be revised toward KEEL, not vice versa:

1. **Consolidation-as-a-Step.** The RESIDENT spec (mine) frames sleep as a separate nightly batch system. KEEL's framing is superior: sleep phases should be **maintenance Steps routed through the same engine**, so they inherit I1/I3/I4/I5 for free — audited, privacy-filtered, budgeted, and fenced like any other work. I log this as a correction to my own document.
2. **Invariants in the chain, not in the loop.** The brain corpus's fences are loop-stage *conventions*; KEEL's I1/I3/I4 are middleware — **unbypassable by construction**, surviving even a cell that composes its own exotic loop. The seat/node specs should state explicitly that their gates are chain-enforced, and any implementation of them on KEEL gets this upgrade automatically.
3. **Change-gating as a law, not a feature.** dHash/VAD (§12) is the concrete ancestor of the crystallized principle "cost scales with surprise, not input volume" (fable-v2's pandemonium; the Measured Corpus's innovation-driven attention). Promote it in the canon from a perception implementation detail to a **named genome law applying at every afferent seam** — perception, ingest, memory refresh, expectation monitoring alike.

---

## §7 · What must NOT be added — the refusal discipline, preserved

The intersection test still governs, and it argues *against* most of what a naive "add the brain to KEEL" would do. **Refused from the genome, confirmed:** drive setpoints and weights (RESIDENT cell); auction/salience *policy* (cell — only the seam and feature vocabulary enter); claim extraction, sketch schemas, NLI edges (CORTEX cell / Measured Corpus periphery); sleep-phase logic (cell Steps); the trainable router head (cell, trained on exhaust); failure-mode monitors (cells' readers over the I1 ledger, which already exists precisely so such readers can be written). None of the seven original systems rebuilt an attention economy or a drive system — so by KEEL's own founding rule those are periphery, and the genome's contribution is to make them *slottable*, which is exactly what §3's four seams do and no more.

---

## §8 · The destinies, named — and the second cell

The three destinies now have names from the crystallized stack: **(a)** the embeddable bundle — unchanged; **(b)** the sovereign personal harness — *is* KEEL + a RESIDENT cell + CORTEX behind the Memory seam + compiled seats as sibling cells; **(c)** the org-scale kernel — the constellation of (b)-shaped and seat-shaped cells under handoff contracts (BRAIN v0.6 §8's deferred problem, still deferred, now with a named substrate).

Which yields the proof plan: **SEXTANT remains the first cell** (the work-cell shape — the most agentic, the right first proof, unchanged). **The second cell should be RESIDENT Phase 0** (the organism shape — tape + watchers + arbiter-default + a static morning brief). The two together span the genome's claimed range, and §23 does the judging: if *both* build without kernel edits once the §5 deltas land, scale-invariance is demonstrated at the interesting extremes rather than asserted in prose. If either forces a kernel edit beyond the deltas, this review under-called the boundary and the ledger should say so.

---

## §9 · Honest limits of this review

1. **I reviewed the canon, not the code.** Stage 0–1 is described as real and running; where the running crates already diverge from `KEEL_ARCHITECTURE.md` v0.2.0, this review may be arguing with a document the build has outgrown.
2. **The Arbiter case rests on one cell.** If RESIDENT is never built, the seam serves no second customer, and the intersection test would rightly keep it out. The delta is contingent on destiny (b) being pursued.
3. **The fleet question is half-answered.** Leases are clearly body-work; *agents-as-a-tier* would touch the routing economy's core assumptions (a tier that is itself an agent breaks the `generate()`-shaped contract) and I have deliberately not designed it. It may be the genuinely new contract KEEL needs in a year; it is not specced here.
4. **Same-family caveat, standing.** This review was produced by a model reading documents largely co-authored by earlier models of its own lineage, in a canon that has already demonstrated (three brain specs in one day) that this family converges on attractor designs. Per the house discipline: the convergences in this document are weak evidence; the falsifiers attached to each delta — and the two-cell build — are the only exit from the attractor into fact.

---

*KEEL is the body. The self is state; the cognition is rented; the honesty is manufactured — by pipes that stamp provenance, chains that cannot be bypassed, budgets that hard-stop, clocks that tick unprompted, and a tape that never lies. The brain decides; the memory holds; the body guarantees. Nine deltas, two traits, one rewritten sentence — then build the second cell and let §23 do the talking.*

*— Claude (Fable 5), 2026-07-01. Advisory; non-canonical; ratification is the operator's. Filed to `_brainstorm/` with nothing else in the repo touched.*
