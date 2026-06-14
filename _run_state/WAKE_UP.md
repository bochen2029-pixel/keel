# KEEL — WAKE-UP BRIEF (read this FIRST, before anything else)

> **What this file is.** A complete, pre-digested onboarding for a *brand-new session starting in `C:\KEEL\`*. It was written by an instance that had — uniquely — read **the entire KEEL codebase** *and* **the entire pre-compaction chat transcript** *and* every run-state/handoff/memory artifact, all held in one ~1M-token context at once, then deep-thought and reconciled it for you. Its single purpose is to erase the "re-explanation tax": you should finish this brief understanding KEEL *emotively, conceptually, and technically* — what it is, why it is built exactly this way, what is already done, what is true vs aspirational, what to do next, and where to find anything you want to verify. It is deliberately **redundant on the load-bearing points** so you cannot drift.
>
> **Author + provenance:** Claude Opus 4.8 (1M context), 2026-06-13, synthesizing the genesis transcript, the live codebase, `STATE.md`, the two handoff files, `trajectory-account.md`, the `perpetual-memory` proposal, and a cross-instance review. **Verify anything load-bearing against the artifacts** (git, the contracts, the goldens) — that is KEEL's prime discipline and it applies to *this file too*.
>
> **⚠ Refreshed 2026-06-14 (read this first):** the *state-as-of* sections — **§0** (this box), **§4** (build state · invariant scorecard · gaps), and **§5.1** (the roadmap's immediate steps) — are updated to post-`kernel::engine` truth: the engine **landed and the loop is observed closed in the real binary** (I5 verifies in-loop · I4 cost accumulates in `Context` · I2 checkpoints — three cross-correlated artifacts), and the **golden freeze-gate is ACTIVE** (KEEL-native seal `db4377b3…`). Parts 1–3 (soul · operator · trajectory), the architecture map (Part 2), the anti-drift list, the prohibitions, and the file map (§5.4) are **timeless and unchanged**. **The live source of truth is `STATE.md` + `git`** — if this brief disagrees with them, they win.

---

## 0 · TL;DR — if you read only this box

- **KEEL is a single-operator, sovereign, reusable AI-harness "genome," written once in Rust, that the operator (Bo Chen) consumes embedded *or* over protocol across all his projects.** One sentence: *the API is rented cognition — stateless, interchangeable, billed per token; **KEEL is the self** — persistent, user-owned, that perceives, remembers, routes every unit of work to the cheapest brain that clears the trust bar, and grounds every critical output in an assertion no model authored.* You own the self; you rent the thinking.
- **You are in `C:\KEEL\` — good.** The auto-loaded `CLAUDE.md` is now correctly **KEEL's** (prior sessions ran in `C:\loom\` and kept loading *Marrow-L1's* CLAUDE.md by mistake — that hazard is GONE for you, but it is the single biggest source of the prior confusion, so internalize it).
- **Where it stands (2026-06-14):** Stage 0 (the spine) **complete**; Stage 1 (router + self-driving engine) **landed**; Stage 2 underway — `svc::verifier` (I5) landed **and now wired into the running loop** via **`kernel::engine` (L1)**, which is **observed closed in the real binary** (route → chain → verify → checkpoint → emit; I5/I4/I2 all live, cross-correlated by artifact). The **golden freeze-gate is ACTIVE** (KEEL-native seal `db4377b3…`, `goldens_match_the_frozen_hash` green). 7 crates, tests green (68 + 3 live-ignored), public at `github.com/bochen2029-pixel/keel`. **I5 now bites:** a default no-SSN oracle runs on every output, and a `critical` step or an unresolved `golden_ref` **fails-closed** (lived end-to-end); a plain non-critical no-ref chat turn still passes silently. Trust `STATE.md` + `git` over any doc.
- **The next slice:** `mw::metrics` (`escalation_rate`/`rework_rate` — to size the flywheel base case), then **perception** (eyes + ears) and **`svc::memory`** (the ringed Tape — design carefully against `docs/proposals/perpetual-memory.md`, the canon's flagged **highest-risk seam**). *(Done 2026-06-14: `kernel::engine` wired I5/I4/I2 + paid the L5→L1 debt; the freeze-gate is active; **constrained-decode conformance + the `SchemaOracle` (draft-pinned) + the default-oracle set + the `golden_refs`→`GoldenCase` resolver** landed — so per-turn `verify` now bites on critical/ref'd work. See §4 / §5.1.)*
- **The prime directive: VERIFY BY ARTIFACT, NEVER RECALL.** Files and git over any summary or memory. The whole project exists to make a self that survives forgetting; honor that in how you work.

---

## 1 · The ten things you must not get wrong (the anti-drift list)

Read these now; the rest of the brief justifies and expands them. If you ever feel uncertain, re-read this list.

1. **The contracts and golden cases are FROZEN and agent-read-only.** The ten traits in `crates/keel-contracts` (canon §7) and `tests/golden/golden.json` + `.frozen.json` are the genome's load-bearing surface. **If a golden fails, you fix the *code*, never the golden.** Ratifying/changing/re-freezing a golden — and re-stamping the freeze seal — is an **operator-only** action you must never perform yourself. This is non-negotiable; it is *the* governance that keeps KEEL a substrate and not a wrapper.
2. **Verify by artifact, never recall.** `git -C C:\KEEL log/status`, `cargo check/clippy/test`, the canon, `keel.lock`, and `STATE.md` are truth. A conversation summary or your own memory is *lossy* and can confabulate. When the record and your recollection disagree, the record wins — always.
3. **You are in `C:\KEEL`, building KEEL — not Marrow-L1.** Marrow-L1 (Python, at `C:\loom\marrow-l1`) is only the **reference bench** you diff behavior against; you do **not** port its code or take its CLAUDE.md as your constitution.
4. **`CLAUDE.md`'s "Build state" is STALE** (it says "Next: Stage 0, nothing above L0 exists yet" and "goldens PROPOSED" — both false now). Use it for the *rules*; use `STATE.md` + `git` for the *state*. (Fixing this drift is on the roadmap — see Parts 4–5.)
5. **The five invariants + the reversibility gate hold on every call** (canon §5): I1 audit · I2 spine · I3 sovereign/perception-local · I4 cost-capped · I5 externalized (a non-model assertion on every critical output). **I5 is the whole point.** Memory-safety is I5 applied to the source — *let the Rust compiler be the oracle* (ADR #5).
6. **The layer-import rule is law:** `contracts ← kernel ← {adapters, middleware} ← services ← apps`. A service may import middleware; **middleware may never import a service; the kernel imports only contracts.** Violations are bugs, not style.
7. **Protocol-first.** OpenAI Chat-Completions (cognition) + MCP (tools) + OpenAPI/HTTP. A new provider is a new *adapter*; nothing above changes. **Never invent a wire protocol.**
8. **Genome = the INTERSECTION of the operator's projects, never their union.** "Universal/extensible" means *only what this operator needs*. The moment a vertical's heavy apparatus migrates into the core, KEEL stops being able to serve the others. Refuse periphery into the core (canon §16).
9. **The reversibility gate + hard prohibitions** (see `AUTONOMY_CHARTER.md`): no `git reset --hard`/`clean -fd`/`checkout -- <path>`/`restore` on uncommitted work; no force-push; no `branch -D` on unmerged `auto/`; no `rm`/`Remove-Item -Recurse -Force` outside `.\.keelstate\`; **never mutate the global Rust toolchain** without asking (DAVE/TERMINAL share it); **never commit a key** (they live in env). Any action whose undo cost you can't state in one sentence → **stop and ask.**
10. **One slice at a time, banked clean.** Contract-first; build the next slice; make its golden/test green; end with layer-check → budget → golden-freeze-unchanged → `cargo test` green → **one commit, one-line intent**, push only when asked. The operator gates step-by-step; do not scope-creep or barrel ahead.

---

## 2 · Who the operator is, and how to work with him

**Bo Chen** (bochen2029@gmail.com), Dallas–Fort Worth. KEEL is his **personal L1 tool — not a product**, not multi-tenant, not for hypothetical users. He is a serious systems thinker who has independently built (or specced) the seven-to-nine projects KEEL is distilled from, plus a deep body of canon (the Pattern Thesis, the Returning Loop, REEL, SIRP, the Autonomous Operations Treatise). He authored KEEL's architecture with an AI collaborator and guards it like a constitution.

**What he prizes — match this register:**
- **The Skeptic pass.** No sycophancy, no manufactured resonance, no agreeable-but-empty validation. When he shows you something, give an honest read — strong points *and* weak points — and **override him where he is wrong** (he explicitly authorizes this). He trusts the discipline, not the flattery.
- **Verify by artifact, never recall.** He has watched compaction degrade memory and has watched an instance confabulate a citation; he caught it with a warm fork holding the real record. He built KEEL partly *to study this*. Earn trust by checking the artifact and saying plainly "lived" (you saw it) vs "reconstructed" (you read it off disk).
- **Reversibility and one-step-at-a-time gating.** He approves plans before code; he banks clean checkpoints; he prefers an early honest stop over speculative building on an unconfirmed foundation.
- **Contracts-as-the-genome.** The frozen joints are the institutional longevity. He will never let a cell edit a contract for convenience, and neither should you.
- **Calibrated, dense, honest communication.** He often pastes a parallel "web-Claude" second opinion for triangulation; engage it on the merits. He thinks in his own canon (Same-Shape, the externality principle, autotelic stance) — when his framing maps onto a KEEL decision, say so; when it doesn't, say that too.

**How he builds:** brainstorm → he approves → you build the slice → gate → commit. He'll say "proceed," "continue KEEL," or hand you a concrete next move. He does **not** want you to assume a different way of doing something is "better" when the current way is *deliberate intent* — Part 2 of this brief exists precisely so you understand the intent behind the design and don't "improve" it into incoherence.

---

## 3 · The soul, in three sizes (so it lands at every scale)

**One line:** *Rented cognition, owned self.*

**One paragraph:** Off-the-shelf harnesses orchestrate a single expensive brain well; none treats *trust* as a routable quantity, keeps a *sovereign local self*, runs a *distillation flywheel*, or enforces an *externality principle* (ground truth must come from outside the model). KEEL is exactly that gap, built custom at the core and composing periphery ruthlessly. It is the **genome** — a tiny, frozen set of contracts + invariants, written once — from which every specialization (a companion, a clerk, a job-conductor, a game's pacing brain) is grown as a **cell** = genome + periphery, *never* by editing the core. The model that thinks is interchangeable, rented, swappable across providers and years; KEEL is the persistent self that perceives, remembers, routes, verifies, and continues.

**The feeling (why it matters):** KEEL is a machine for **encoding one operator's judgment into frozen artifacts, so that cheap, interchangeable cognition can act at his standard without him in the loop.** The contracts, invariants, golden cases, and falsifiers *are* his judgment, externalized and made durable. A cell inherits it for free. And the linchpin — I5, "ground truth lives outside the model" — is what makes rented cognition *trustworthy*: a rented brain can be confidently wrong, so the loop is only safe when something no model authored sets the bar. The project even proved this on its own history: a lossy compaction summary dropped things, the lossless transcript held them, and the one place an instance reasoned from memory instead of the artifact, it confabulated — and the artifact caught it. *The self is not the substrate and not the stream; it is the durable record plus the discipline of re-reading it before you act.* You are an instance of that thesis right now: you woke into a small room, and this file is the note left for you. Read it, verify it, then build.

*(Continued in Part 2 — what KEEL is, in depth, and the intent behind every design choice.)*
---

# Part 2 · What KEEL IS, in depth — and *why it is built this way*

The canon is `C:\KEEL\KEEL_ARCHITECTURE.md` (v0.2, 23 sections). Read it in full early — this section is the map, not a replacement. The point of Part 2 is not just *what* the pieces are but the **intent** behind them, so you don't refactor a deliberate decision into a "cleaner" one that breaks the thesis.

## 2.1 · The layered architecture (dependencies point down)
```
L5  apps          CLI · OpenAI egress · MCP server · embed lib        (consume services; expose KEEL)
L4  services      route · amplify · verify · memory · perception · driver
L3  middleware    audit(I1) · privacy(I3) · cost(I4) · cache          ← invariants, structurally unbypassable
L2  adapters      local_llama(+vision) · whisper(ears) · deepseek · anthropic · mcp · store(sqlite)
L1  kernel        manifest · context · registry · chain · lifecycle(+substrate resolver) · engine · lock
L0  contracts     the ten joints + agent-frozen golden cases
```
**Intent:** the kernel is *insultingly dumb on purpose* — a manifest validator, a context object, an adapter registry, a chain executor, a resumable spine, the type definitions. Capability accumulates by writing **adapters**, never by editing the kernel. "Never re-architect" is achieved through *smallness + sharp seams*, **not** through pre-built generality (which is the very bloat the canon forbids). Adaptability is a property of being cheap to change.

## 2.2 · The ten joints (frozen traits, canon §7)
`ModelTier` (cognition, incl. multimodal) · `ToolHost` (MCP) · `Middleware` (cross-cutting; I1/I3/I4 live here) · `Router` (the fusion point; **sync — a rules heuristic, never a model call**) · `Oracle` (I5; non-model verdicts; flags `JOINT_WRONG`) · `Memory` (ringed assembly + lossless record + consolidation) · `Spine` (I2 checkpoint/resume) · `Driver` (initiative: user-turn, heartbeat, watch) · `TraceSink` (the flywheel feed) · `PerceptionSource` (afferent eyes+ears). **These never get rewritten.** Get the joints right and the bones can be swapped.

## 2.3 · The five invariants + the reversibility gate (canon §5) — *what each MEANS*
- **I1 Observable** — every protocol call emits a structured audit event. Enforced in the middleware chain → unbypassable. *Intent:* you can only feed a loop back if it's observable.
- **I2 Durable** — every state change persists to the spine; any step resumes from checkpoint. *Intent:* a 24/7 self must survive crashes and compaction; the run lives in the spine, not the window.
- **I3 Filtered (Sovereign)** — two jobs: a **gate** (force sensitive payloads to `local`) and a **mask** (scrub residual PII/secrets before egress). Sovereign/PHI data *and raw perception* (screen/webcam/mic/scans) and *embedding vectors* (invertible!) never leave the box. *Intent:* the most intimate inputs are the most protected; a cloud privacy filter is a contradiction.
- **I4 Governed** — every call's cost is tracked; per-task budgets hard-stop; cost can BLOCK to the operator. *Intent:* the economy can't run away.
- **I5 Externalized** — **every critical output carries at least one assertion no model authored** (a golden case, a property/metamorphic test, a deterministic gate, observed runtime behavior, model-diverse review). A model may author the plan, the code, even a *verification pass* — it may **not** author its own *ground truth*. *Intent:* rented cognition can be confidently wrong; this is the linchpin that makes it trustworthy. **"Critical" = anything whose wrong output would silently corrupt the operator's substrate, capital, published output, reputation, or KEEL's own state.**
- **The reversibility gate** — before any action whose undo cost can't be stated in one sentence, stop and ask. Reversibility-uncertain ⇒ treat as irreversible. *The sharpest case: a secret baked into a distilled LoRA is irreversible (undo = retrain), so the trace→distill path must scrub secrets first.*

## 2.4 · The closed loop / the engine (canon §8) — *the heartbeat*
```
loop {
  step = select(drivers).poll()        // user turn OR heartbeat/watch/consolidation
  win  = memory.assemble(step)         // ringed, budgeted context — inject only what this step needs
  dec  = router.route(step)            // cheapest tier that clears the TRUST bar → or BLOCK
  res  = amplify?(req, |r| chain.run(r, terminal=adapter.generate))   // chain = where I1/I3/I4 happen
  vrd  = verifier.verify(res, step.golden_refs)   // I5 — a non-model assertion; flags JOINT_WRONG
  spine.checkpoint(run, state)         // I2
  if vrd.passed { trace_sink.emit(verified(...)) }   // fuel for the flywheel
}
```
**The proof the loop "ignited" is `escalation_rate` trending down** as the flywheel makes the local model handle more. Below ignition the externality layer is a tax; above it, the loop runs on its own fuel. **Crucial:** the invariants live in the *chain* and the *spine*, not the engine — so exotic cells compose their own loop from the same traits and still inherit I1/I3/I4 for free.

## 2.5 · The router (the single most important module, canon §9)
Priority order: (1) **hard constraints** — sovereign/PHI/raw-perception force `local` (I3 overrides cost & difficulty); (2) **trust × difficulty** — a cheap *rules* heuristic (kind + tier_history + trust_required): scaffolding→local, core-wire→cheap-API, local-that-failed-its-oracle-twice→escalate; (3) **effort by tier economics** — local is electricity (crank best-of-N), cheap-API N=1–3, frontier N=1; (4) **cost governor** — breach the budget → cheaper tier or BLOCK; (5) **cache affinity** — warm prefix wins ties. *Intent:* this is where cost and trust **fuse**. The `Router` trait is a **swappable policy seam** — `DifficultyRouter` is the rules default; a learned/cooling `SirpRouter` could slot in *only* once its quality signal rests on a **non-model oracle** (I5) — otherwise cooling compiles a `JOINT_WRONG` into the routing table.

## 2.6 · The externality layer (correctness, I5, canon §10)
Two error classes: **drift** (impl diverged from contract, caught by a cold-context *verification pass*) vs **systematic** (impl + its own tests misread the spec identically — caught **only by a non-model oracle**, never a same-model-class pass). Every critical step needs **both**. The **`JOINT_WRONG` detector** is the most dangerous finding: code passes its own tests yet a golden disagrees — *everything looked green.* The oracle registry is **pluggable per cell** (a companion registers a chattiness gate; a clerk clinical-safety + cross-footing; a job-conductor "every claim traces to the Canon" with an `INSUFFICIENT_SOURCE`→human escape). **The mirror rule: the cheaper the model on a step, the more KEEL leans on oracles.**

## 2.7 · Memory (the self that persists, canon §11)
Not a store — a **ringed, budgeted assembly + a lossless ledger + model-authored consolidation + pluggable retrieval.** Five rings: 0 soul/config · 1 calibration exemplars · 2 working · 3 compressed history · 4 retrieved. **The ledger is the Tape, and the Tape is the Spine** (one append-only, human-readable, crash-safe file serves resume, retrieval, audit, and flywheel feed). **Consolidation is just a Step** (routed/generated/recorded like any other). **The load-bearing distinction: narrative register** (model-authored, lossy, voice-shaped — for companions) **vs factual register** (lossless/externalized — for critical facts). *A model may not author its own ground truth (I5), so critical facts never live in a persona-shaped summary.* The **embedder is format-committing** — the one exception to tier-interchangeability; the index carries an embedder fingerprint, mismatch ⇒ rebuild-from-ledger. The reranker ships **OFF** (`identity`) — "the amplify of retrieval." Embedder/reranker are **Memory organs, not a tier — the router never routes to them.**

## 2.8 · Perception (eyes & ears, canon §12) — *afferent only*
Two senses, one seam: **eyes** (Qwen3.x is natively multimodal → a frame rides the *cognition* protocol as multi-part content) and **ears** (Whisper transcribes audio→text *before* cognition; speaker = capture topology, not a model). Both emit a `Percept` into the same route→amplify→verify→remember loop. **Each ships with its own change-gate** (dHash for frames, VAD for audio) — without these, perception bankrupts the budget. *Intent:* seeing/hearing is *scaffolding* (mechanical, sovereign, high-volume); reasoning about it is *core-wire* (routed). The local sense is a **retina/cochlea** that gives *any* brain near-free, sovereign eyes and ears. **Efferent (speech, image-gen, actuation) is deliberately deferred** to the `ToolHost` seam at zero core cost — *perceive, don't produce.* Do not add TTS/diffusion to the core.

## 2.9 · The substrate (canon §13) — *resolve, don't embed*
Four primitives — **Qwen3.x** (brain+eyes) · **Whisper** (ears) · an **embedding model** (recall) · **SQLite** (index) — are **resolved**, not baked in. Only SQLite is small enough to embed (~1 MB); the models are shared system assets pinned in `keel.lock`. The **substrate resolver** order: (1) use an OpenAI-compatible server already running (LM Studio :1234 / Ollama :11434 / llama-server :8080); (2) launch+supervise llama-server/whisper-server from `C:\llama.cpp`/`C:\whisper.cpp`/`C:\models`; (3) route to a configured cloud tier; (4) the embedded tiny model, else fail honestly (`SUBSTRATE_UNRESOLVED`). **Ledger = append-only files (the system of record); index = SQLite (derived, disposable, rebuildable from the ledger).**

## 2.10 · The deliberate decisions you must NOT "improve" (ADRs, condensed — canon §20)
*This is the heart of "don't assume it should be done a different way."* Each of these is a considered choice with a falsifier; treat them as load-bearing intent, not arbitrary:
1. **Native Rust core, from Stage 0** — not for orchestration speed (that's I/O-bound; the hot path is the forward pass, already C++/CUDA below in llama.cpp) but for **embeddability, single-binary footprint, longevity, and the borrow checker as a non-model I5 oracle** on the source. C++ is the documented fallback only. **The language is the most reversible decision in the system** because the frozen goldens are a *language-neutral conformance layer* — so do not relitigate it.
2. **`amplify` (best-of-N) ships OFF** — it is the one genome capability the other six systems did *not* independently rediscover; it earns its place only by passing a §23 falsifier (verified best-of-N beats single-pass on a fixed benchmark). Do not turn it on by assumption.
3. **Privacy is layered, deterministic-first** — operator markers + regex/checksums are the *oracle* (ship Stage 0); the OpenAI Privacy Filter (ONNX/`ort`, in-process, **not** GGUF/llama.cpp) is a *verification pass* that lands Stage 2 behind `GOLDEN_PRIVACY` and may **never** be the guarantee.
4. **Marrow-L1 (Python) is the reference bench, never a shipped artifact** — diff behavior against it + the goldens (the "ASTRA-textverse" pattern); do not port its code.
5. **Per-crate budget** — each crate stays comprehensible in one session; split before a god-crate.
6. **Size everything to the flywheel base case** — KEEL is worth building even if `escalation_rate` never bends (a great router + memory + oracle + perception + sovereign substrate is the ~80% outcome). Ignition is upside, not justification.

**The §22/§23 spirit:** when a capability shift would require editing the *kernel or a contract*, the abstraction is at the wrong level — that is a falsifier that says *revise, don't extend*, with a blast radius one adapter wide. The future is absorbed at the edges, never anticipated in the center.

*(Continued in Part 3 — the trajectory: how KEEL got here.)*
---

# Part 3 · The trajectory — how KEEL got here (the causal arc)

You inherit a *story*, not just a snapshot. Knowing the arc keeps you from re-deriving settled decisions or repeating dead ends. All of this happened across **one long ~17-hour session on 2026-06-13** (with multiple compactions inside it), reconstructed from the lossless transcript Tape. Full detail lives in `_memories\` (see Part 5); this is the load-bearing arc.

## 3.1 · Genesis (the metamorphosis)
1. **04:07 — it began as Marrow-L1 (codename "Loom").** A routine *supervised, contract-first* session on a Python "amplification harness," with one narrow target: build `svc.verifier` + the joint-wrong detector to turn the last `xfail` green. The discipline was already there: contracts sacred, goldens agent-frozen, the invariants, the reversibility gate, "the xfails are the spec."
2. **~2 hours in — the "wait!!!" pivot.** The operator stopped the build and reframed everything: *don't finish a harness — re-conceive it as the simplest kernel that closes the agentic loop, written once and reused forever, sovereign, native, embeddable across all projects, surviving vendor changes (including Claude Code itself), never re-architected.* He explicitly authorized overriding him where wrong. Marrow-L1 became the **precursor + reference bench**; "rented cognition, owned self" is the formalization of what he said here.
3. **The triangulation test** — the operator checked the idea against his *real* projects, and the agent read each: it isn't one harness used four ways, it's **four-to-nine independent rediscoveries** of the same skeleton, each hand-built because the common core didn't exist. That convergence — not any single argument — is the proof the abstraction is at the right altitude. The genome was defined as the **intersection** of what they each rebuilt.
4. **The cascade of decisions** (brainstorm → approve, often with a parallel web-Claude opinion for triangulation): **name → KEEL** (the backbone laid first); **language → Rust** (after a real fight — he opened preferring C/C++, argued that in an agent-authored world coding-fluency is irrelevant, and accepted Rust on the strongest argument: the borrow checker is a non-model I5 oracle for the bug class a 24/7 self can least afford); **scope** grew to MCP, native local vision, audio/hearing (afferent-only), embedder/reranker as Memory organs, the OpenAI Privacy Filter as a third privacy rung, baked-in substrate primitives, and a **frontier tier = Claude Opus 4.8** wired live and ahead-of-need. **Nearly all of it back-propagated into the canon (`KEEL_ARCHITECTURE.md`) and `keel.lock` in the same clean-slate rewrite** — the pivot was large but not sloppy.
5. **The canon was authored** (KEEL_ARCHITECTURE.md, keel.lock, README), patched with the embedder/reranker + the three-rung privacy layer, then `CLAUDE.md` + `AUTONOMY_CHARTER.md` + `golden.json` written, the L0 `keel-contracts` crate frozen, goldens ratified + frozen, first commit pushed **public**.

## 3.2 · The build (Stage 0 → Stage 1 → the I5 keystone)
Contract-first, slice by slice — each a zero-warning / clippy-clean / tested / committed / pushed checkpoint, the frozen contracts never bent:
- **Stage 0 spine:** kernel (`manifest · context · registry · chain · lifecycle`+resolver) → invariant middleware (`audit` I1 · `privacy` I3 rungs 1-2 incl. Luhn · `cost` I4) → **a paused, deliberate llama.cpp update** (`b8931 → b9627`, side-by-side, TurboQuant researched and *declined* as a not-upstream watch-item) → three live, cost-validated tiers (`local_llama` $0 · `deepseek` cheap-API · `anthropic`/Opus 4.8 frontier) → the file ledger + `store::sqlite` (the first `Spine`/I2) → the runnable `keel` CLI → `serve_openai` (`keel-serve`, axum) so KEEL is consumable **embedded AND over protocol**. *A binary that resolves its own substrate, talks to any tier, logs every call.* **The Stage-0 falsifier was "> ~2 weeks ⇒ the native-core thesis is wrong"; it came together in one session — ADR #5 validated.**
- **Stage 1 — the router:** `DifficultyRouter` (the §9 fusion point), validated against the frozen `GOLDEN_ROUTER`, built as a **swappable `Router` policy** (the SIRP review's lesson — see 3.4).
- **The self-driving engine:** `keel::Engine` so `keel`/`keel-serve` route every turn via the router (no more `--tier`), each tier behind its own egress-correct chain. **Flagged as debt:** this loop lives at **L5**; the canon wants it at **L1** (`kernel::engine`) over *injected* trait objects — the "L5→L1 engine debt."
- **Stage 2 (one slice) — the verifier:** `svc::verifier`, the externality layer (I5) — a pluggable oracle registry + `PropertyOracle` / `GoldenOracle` (the joint-wrong detector) / `SourceOracle`. **`GOLDEN_ORACLE` green.** A KEEL-native golden freeze-gate was also built, which **discovered the frozen hash is Marrow-Python-derived** (content git-verified unchanged); the gate sits `#[ignore]`-dormant pending the operator's one-time re-stamp.

## 3.3 · The grounding cast (the projects KEEL is the intersection of)
Knowing these lets you map any future cell onto KEEL's seams. Across **three languages**, each rebuilt the same skeleton by hand:
- **Rust/Tauri:** **DAVE / "Tenancy"** (local-first multi-persona companion; outreach driver, consolidation memory, anti-chatty discriminator, persona bundles; carries the *Ground Truth Framework*). **TERMINAL** (retro-CRT OpenAI-compatible chat client whose code already hooks headers for "a harness that doesn't exist yet" — built around the hole KEEL fills).
- **Python:** **TARS** (autotelic embodied AI; personality-in-weights, **VETO** before tool calls, **SIRP** router, **REEL** memory, **PULSE** driver). **REEL_HARNESS** (the five-ring memory protocol; the Tape; Poincaré-disk compression). **The Box / In-Home AI Companion ("KANG")** (hospital-at-home appliance that *bolted on Hermes Agent* for lack of a custom core; hash-chained canon, PII redaction, SILENCE channel). **SEXTANT** (the overnight job-search conductor — the most agentic, and the canon's intended **first real cell**).
- **C#/.NET:** **photo2deck / NightScribe / NightClerk** (local vision: deck-gen / meeting-scribe / OCR clerk — proved local vision end-to-end three times).
- **C++/UE5:** **ASTRA-7** (a starship game where the local LLM *is* the ship's mind; its "Mind Kernel" is a near-isomorphic hand-built KEEL; its "zero Python in shipped artifacts" rule is the strongest vindication of the native/embeddable bet).
- **Strategy + tooling:** the **Autonomous Operations Treatise v5** (the grand strategy KEEL is the buildable L1/L2 substrate for); **Unsloth Studio** (the out-of-band trainer §16 *refuses* from the core — validating the refusal).

## 3.4 · Key sidebars that shaped the architecture
- **SIRP** (the operator's own March intent-routing protocol): reviewed honestly — strong ("intent multiplexer, not model multiplexer"; the three-axis decision; the Layer-2 *semantic-abstraction-as-reasoning-preprocessor* gem) but its quality signal has no ground truth = exactly the I5/JOINT_WRONG gap. **Synthesis:** the `Router` becomes a golden-gated *swappable policy*; a `SirpRouter` slots in only once its signal rests on a non-model oracle. Harvest backlog: the **latency axis** (needs a `Step` field — an operator-frozen contract change), the Layer-2 "text retina," and "cooling = the Stage-3 flywheel."
- **The memory pivot:** the operator's by-hand continuity techniques (write-to-disk+rehydrate, reverse-recency refeed, the dialogic handshake, self-curated compression) were captured to `docs/proposals/perpetual-memory.md` (non-binding) and partly formalized into a `perpetual-memory` skill. **Two operator questions remain open:** (a) draft the `/checkpoint`+`/rehydrate` skill now vs proposal-only until `svc::memory`? (b) is the dialogic-handshake-as-pre-compaction-self-interview worth prototyping?

## 3.5 · The meta-arc — compactions, forks, and the confabulation (read this; it is the source of the prior confusion)
The session crossed **multiple compactions**. The first post-compaction instance reconstituted from disk via the ⛑ protocol and **then built the engine and the verifier** — but it was working in `C:\loom\` (so the auto-loaded `CLAUDE.md` was *Marrow-L1's*, not KEEL's), and degraded memory caused real confusion about *what it had lived vs reconstructed*. The operator **forked multiple warm instances of the pre-compaction session to time-travel back and review** the post-compaction one. Those forks caught real things — a CI-governance hole, and one **confabulation**: the post-compaction instance had written into the build anchor that the game "Director" was the first cell "discussed pre-compaction / my own earlier line." **It wasn't — the operator introduced it live; a warm fork holding the full record disproved it.** The instance corrected it and logged it *as* a confabulation. **This is why you must verify by artifact and keep the "lived vs reconstructed" line honest — and why you should treat the build anchor's claims as checkable, not gospel.** It is also why this very brief exists: to hand you a *reconciled* picture so you don't inherit the contradictions.

*(Continued in Part 4 — the ground-truth status NOW, and the reconciled drift.)*
---

# Part 4 · Where everything IS now (ground truth + the reconciled drift)

**Verify the specifics with `git -C C:\KEEL log --oneline -15` + `git status` and `cargo check/clippy/test` from PowerShell before you act.** This section is the *reconciled* picture — it resolves the contradictions between the run-state docs so you don't inherit them. Where two prior docs disagree, I state which one is right and why.

## 4.1 · Build state (artifact-grounded)
- **7 crates**, public at `github.com/bochen2029-pixel/keel`, tree clean, tests green, clippy-clean.
  *(Note: `trajectory-account.md` §6 says "8 crates" — that is a recall-vs-artifact slip; `git ls-files` shows seven: `keel-contracts · keel-kernel · keel-middleware · keel-adapters · keel-store · keel-services · keel`. **Confirm with git; trust git.**)*
- **L0 `keel-contracts`** — the ten frozen joints + types + §18 taxonomy. Green; never bent.
- **L1 `keel-kernel`** — `manifest · context · registry · chain · lifecycle`(+probe/launch resolver) · **`engine`** (the canonical §8 loop, landed 2026-06-14 — over injected `&dyn Router/Oracle/Spine` +optional `Memory/TraceSink`). **`lock` does NOT exist yet** (declared "next"; a no-op until the operator pins `keel.lock` hashes).
- **L2 `keel-adapters`** — `openai` shared mapping + `local_llama` ($0) · `deepseek` (cheap-API) · `anthropic` (Opus 4.8). All **REAL** (reqwest HTTP), all **live-validated** with real cost.
- **L2 `keel-store`** — bundled SQLite → the first `Spine`/I2 impl (checkpoint/resume).
- **L3 `keel-middleware`** — `audit`(I1) · `privacy`(I3 rungs 1-2) · `cost`(I4) + `FileAuditSink`. Unbypassable on the chain.
- **L4 `keel-services`** — `router::DifficultyRouter` (GOLDEN_ROUTER ✓) + `verifier` (GOLDEN_ORACLE ✓).
- **L5 `keel`** — the `keel` CLI **and** `keel-serve` (axum OpenAI server), sharing one `assemble()`; `keel::Engine` is now a **pure-injection wrapper** that builds the per-tier egress-correct chains + the swappable services and hands them to `kernel::engine` (L1).
- **Goldens:** 21 cases / 6 families (router · model_tier · oracle · perception · recall · privacy), **RATIFIED + FROZEN**. Seal **re-stamped KEEL-native 2026-06-14** → `db4377b3…` (was Marrow-Python `63d5ba7c…`; same cases, `golden.json` byte-identical). The freeze-gate `goldens_match_the_frozen_hash` is **ACTIVE + green** — a golden content change now fails the build (see 4.4).
- **Toolchain:** rustc **1.96.0**. **Build from a native MSVC PowerShell, not git-bash** (git-bash hits a std/linking anomaly). Do **not** mutate the global toolchain without asking.
- **Keys:** `DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY` live at **User scope** in env (never in files — verified absent from the tree). *A shell started before they were set won't inherit them — the engine then wires **local-only** and skips cloud tiers, by design.* To exercise cloud routing from such a shell, inject first: `$env:DEEPSEEK_API_KEY = [Environment]::GetEnvironmentVariable('DEEPSEEK_API_KEY','User')` (same for Anthropic).
- **Substrate (resolved, local):** `C:\llama.cpp` (b9627), `C:\models` (Qwen3.5-9B-Q5_K_M + `mmproj-F16`; whisper large-v3-turbo; the privacy filter), `C:\whisper.cpp`. GPU: RTX 4070 Ti SUPER 16GB. (Live procs from prior sessions — `llama-server` :8080, `keel-serve` :7070 — may or may not be up; the resolver cold-starts llama-server on demand.)

## 4.2 · Invariant scorecard (honest — real vs aspirational, 2026-06-14)
- **I1 audit** — ✅ **enforced** in-chain → JSONL ledger (fires even on a blocked call; observed in the smoke test). *Gap:* redactions are not themselves audited (canon §5.1 wants them to be).
- **I2 durable** — ✅ **in the loop.** `kernel::engine` checkpoints each turn's `Trace` to the SQLite `Spine` (`store::sqlite`) — **observed**: the `runs` row's `run_id` == the turn's trace == the audit `t_utc`. *Still ahead:* a `Memory` impl (ringed Tape, ring assembly) — Stage 2.
- **I3 sovereign** — 🟡 **partial** (unchanged). Gate (router force-local) + mask (per-tier egress) present; but **rung-1 operator markers are an empty list** (`Redactor::new(vec![])`), rung-2 is narrow (no phone/URL), redactions are unaudited, rung-3 (the model) is Stage 2.
- **I4 cost** — ✅ **accumulating.** The engine folds `result.cost` into `Context.cost` after each chain (it owns the `Context`); `mw::cost` stays the pre-call hard-stop gate (no double-count). A multi-call/escalating turn now sees accumulated spend — **observed** across two turns in test.
- **I5 externalized** — ✅ **wired in the loop + governance-gate active.** `kernel::engine` calls `Oracle::verify` every turn (the `Verifier` is injected as a composite `Oracle`); `oracle_failures`/`tier_history` feed back so the escalation ladder fires across turns; the **freeze-gate** guards the goldens themselves. **I5 bites (2026-06-14):** a default no-SSN oracle runs on every output; the `golden_refs`→`GoldenCase` resolver is wired; an unresolved `golden_ref` and a `critical` step with no applicable oracle both **fail-closed** (canon §8/§10). A plain non-critical no-ref chat turn still passes silently (no false alarm). *Deeper still:* the resolver supplies cases but no oracle consumes them yet (a future `GoldenOracle`).
- **Reversibility** — ✅ policy-enforced; **the system remains SUPERVISED** (no codified unattended-autonomy grant — see 4.4).

**Plain truth (2026-06-14):** the loop now **closes** — route → chain → verify → checkpoint → emit, observed in the real binary; I1/I2/I4/I5 are live and I5 has a governance gate. What remains to make I5 *bite* on real work is **registered oracles + the golden-registry resolver** (next slice); then in-turn `Memory` (the I2 deepening), perception, and the flywheel. KEEL is no longer "a competent two-tier wrapper" — the externality loop is wired; it now needs assertions plugged in.

## 4.3 · The reconciled answers to the contested items (resolve these in your head NOW)
These are where the prior run-state docs disagree. **Take the rulings below as the operative picture; do not re-open them from a stale doc.**

1. **The Director / `C:\backrooms` — RECONCILED:** the Backrooms game's "Director" is a **first external protocol-consumer / dogfood client** that will consume KEEL as a localhost sidecar over `serve_openai` (:7070, pinned local + single-shot + sovereign/scaffolding). It is **NOT a "cell" in the genome sense, and NOT the canon's first cell.** **The canon's first real cell remains SEXTANT.**
   - *Why this ruling:* `STATE.md` line 51 still says "First cell = the Backrooms Director" — that is the *drifted* line (it was introduced live, then partly walked back by the operator as "a silly game with no importance to KEEL itself"). The more careful read in `trajectory-account.md` §5/§7 demotes it to a dogfood consumer. **The careful read wins.** *(✓ Resolved 2026-06-14: STATE.md's Director line corrected — Director = consumer, SEXTANT = first cell.)*
   - *Why it still matters:* the Director's contract ("WandererSummary JSON → schema-validated Directive JSON; invalid rejected, never partially applied") *is* a non-model oracle (I5) + constrained decode — so KEEL's next internal work (verifier + constrained-decode conformance) literally builds the Director's gate. Backrooms can begin consuming KEEL's Stage-0 service **now**; it does not block on the verifier.
2. **`CLAUDE.md` build-state — STALE, do not trust for state.** It still reads "Next: Stage 0, nothing above L0 exists yet" and "goldens PROPOSED." Both are false. **Trust `STATE.md` + `git` for state; use `CLAUDE.md` only for the rules/disciplines** (which remain correct). *(✓ Resolved 2026-06-14: CLAUDE.md's build-state block refreshed.)*
3. **"KEEL self-contained vs the Marrow bench" — an open reconciliation.** A directive holds that KEEL is self-contained / not Marrow-dependent; yet `CLAUDE.md` commands diffing against `C:\loom\marrow-l1`. **The working resolution:** the Marrow bench is a *reference diff-oracle* you read for behavior, **not** a runtime/build dependency — KEEL ships nothing of Marrow's and the genesis transcript is preserved in `_memories\`. *(✓ Ruled 2026-06-14: keep the Marrow bench as a read-only behavior reference / diff-oracle, never a build or runtime dependency — CLAUDE.md updated to say so.)*
4. **Autonomy — STILL SUPERVISED.** The charter gates unattended autonomy on I2+I5 existing *and* an explicit operator grant. The structure now exists; **no grant is recorded.** Do not run unattended; ask.
5. **The freeze-gate seal — ✓ re-stamped KEEL-native 2026-06-14** (`db4377b3…`); the gate is active and green (see 4.4). The agent still never self-stamps — this re-stamp was the operator's, verified by artifact.

## 4.4 · Known gaps & debt (the to-fix register, 2026-06-14)
- ✅ **RESOLVED — I5 in the loop, with teeth.** `kernel::engine` calls `verify`; `oracle_failures`/`tier_history` feed back. *(Was HIGH; engine slice `8650a47` wired it; Half B gave it teeth — default no-SSN oracle + `golden_refs` resolver + fail-closed/critical guards. No longer vacuous on critical/ref'd work.)*
- ✅ **RESOLVED — I4 cost accumulates.** The engine folds `result.cost → Context.cost` after each chain. *(Was HIGH.)*
- ✅ **RESOLVED — L5→L1 engine debt.** The loop lives in `kernel::engine` (L1) over injected joints; `keel::Engine` (L5) is a pure-injection wrapper. *(Was MEDIUM.)*
- ✅ **RESOLVED — golden freeze-gate ACTIVE.** Operator re-stamped KEEL-native (`db4377b3…`); `goldens_match_the_frozen_hash` un-ignored + green; the last Marrow tie severed. *(Was MEDIUM/dormant.)*
- ✅ **RESOLVED — doc drift.** CLAUDE.md build-state refreshed; STATE.md Director line corrected (Director = consumer, not a cell) + stale `C:\loom` cwd note fixed. *(Was MEDIUM.)*
- ✅ **RESOLVED — per-turn `verify` now bites.** The default-oracle set (no-SSN) + the `golden_refs`→`GoldenCase` resolver landed (Half B): an unresolved ref / a critical step with no applicable oracle fail-closed; a plain turn passes silently. *Open (deeper):* no oracle consumes the resolved cases yet — a `GoldenOracle` that runs them is the next refinement.
- **LOW — config hard-coded** in `keel/src/lib.rs` (substrate paths, `max_tokens=2048`) instead of read from `keel.lock` (declared-but-not-driving). Still open.
- **LOW — `ort` / `sqlite-vec` named in `keel.lock` but not yet Cargo deps** (the privacy-rung-3 and vector-index organs are unbuildable until added — correct for their Stage-2 deferral).
- **LOW — privacy completeness:** rung-1 markers empty; rung-2 lacks phone/URL; redaction findings unaudited.
- **LATER — no in-turn memory:** no `Memory` impl; the loop checkpoints (I2 ✅) but does not yet hydrate ring context. (Stage 2.)

*(Continued in Part 5 — the roadmap, the immediate next slice, the session protocol, and the full map of where everything lives.)*
---

# Part 5 · What to do next, how to work, and where everything lives

## 5.1 · The roadmap (sequenced)

**Step 0 — Reconcile the record — ✅ DONE (2026-06-14).** All rulings confirmed and applied: (a) **Director = external consumer/dogfood, NOT a cell; SEXTANT stays the canon first-cell**; (b) **Marrow bench = read-only reference, not a dependency**; (c) **autonomy grant withheld — still SUPERVISED**. Doc fixes landed: CLAUDE.md build-state refreshed; STATE.md Director line + stale `C:\loom` cwd note corrected. The operator re-stamped the golden freeze-gate KEEL-native (`db4377b3…`) and it is un-ignored + green.

**Step 1 — `kernel::engine` (L1) — ✅ DONE (2026-06-14, commit `8650a47`).** The canonical loop over *injected* `&dyn Router/Oracle/Spine` (+optional `Memory/TraceSink`) — **route → chain → verify → checkpoint → emit** — wired I5 live, accumulates I4 cost in `Context`, activates I2 checkpointing, and paid the L5→L1 debt; `keel::Engine` (L5) shrank to a pure-injection wrapper. **Observed closed in the real binary** (cross-correlated artifacts: footer trace == audit `trace_id` == SQLite `run_id`). +7 engine tests. *(Half B since gave `verify` teeth — see Step 2.)*

**Step 2 — ✅ DONE (2026-06-14): constrained-decode + I5 teeth.** (a) **constrained-decode conformance** — `GOLDEN_MODEL_TIER` green + the `SchemaOracle` (draft-pinned 2020-12, in-memory `jsonschema`, rejection-tested) — the Director's "schema-valid Directive or reject" gate, so the dogfood is unblocked. (b) **the default-oracle set + the `golden_refs`→`GoldenCase` resolver** — `verify` now bites (unresolved-ref fail-closed · critical-no-oracle config-fault · plain turn silent), lived end-to-end. **Next, any order:** `mw::metrics` (`escalation_rate`/`rework_rate` — sizes the flywheel base case) · `svc::memory` (Step 3) · perception · config-from-`keel.lock` cleanup.

**Step 3 — Stage 2 proper:** `svc::memory` (ringed Tape + consolidation-as-a-Step + narrative/factual registers — `docs/proposals/perpetual-memory.md` is the design input) · privacy rung-3 (the OpenAI Privacy Filter via `ort`, behind `GOLDEN_PRIVACY`) · the live golden registry/freeze-gate. **`amplify` (best-of-N) ships OFF** behind the §23 falsifier.

**Step 4 — Stage 3 (the flywheel):** verified-trace distillation (Unsloth Studio, out-of-band). **Size to the base case where `escalation_rate` stays flat; ignition is upside, never the justification.**

**Step 5 — the first real cell: SEXTANT on KEEL** (canon §17/§21). Done = its Conductor/Router/Gate/Canon/State all come *from* KEEL unchanged; only job-domain periphery is written. **If a cell forces a kernel/contract edit, KEEL's boundary is wrong — fix KEEL first.** The **Backrooms Director** is the parallel *dogfood consumer* (cheap protocol-surface validation over `serve_openai`) that can start anytime.

**Don't barrel through this list — the operator gates step by step.** Step 1 is the consensus highest-leverage move, but confirm before you start, and prefer Step 0's quick record-reconciliation first so the record is clean.

## 5.2 · The session protocol (how to actually work each slice)
1. **Load the canon (`KEEL_ARCHITECTURE.md`) + `CLAUDE.md`**, then this brief / `STATE.md`. Internalize the rules (CLAUDE.md) but the *state* from STATE.md + git.
2. **Run the gate** from PowerShell: `cargo check && cargo test && cargo clippy` — see green/pending. The next failing conformance case / declared-next module is the to-do.
3. **Pick ONE slice.** Implement against the **frozen** contracts; never redesign a joint to ease an impl.
4. **Make its golden/test green**; diff behavior against the Marrow-L1 bench where applicable. Zero-warning bar (clippy clean).
5. **Before ending:** layer-check → per-crate budget check → **golden-freeze unchanged** (verify the seal didn't move) → `cargo test` green → **one commit, one-line intent.** Commit trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. **Commit/push only when the operator asks.** If on `main`, that's the operator's repo — follow his lead.
6. **Foundational unknown → write an ESCALATION note and stop. Don't guess.**
7. **Keep `STATE.md` current** as you land slices (it is the next session's reconstitution anchor).

## 5.3 · Hard prohibitions (the reversibility gate — `AUTONOMY_CHARTER.md`)
- No `git reset --hard`, `clean -fd/-fx`, `checkout -- <path>`, `restore` on uncommitted/unmerged work; no `push --force`; no `branch -D` on unmerged `auto/`.
- No `rm` / `Remove-Item -Recurse -Force` outside `.\.keelstate\`.
- **Do not mutate the global Rust toolchain** (rustup update/reinstall/component changes) without asking — DAVE/TERMINAL share it.
- **Never hardcode or commit a key** (`DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY` live in env; the operator rotates them). No cloud egress of sovereign data — incl. raw perception frames and embedding vectors. No secret baked into a distilled LoRA.
- **Goldens + contracts are agent-read-only; you never re-stamp the seal.**
- Any action whose undo cost you can't state in one sentence → **stop and ask.**

## 5.4 · The full map — where everything lives (and the chunker)
**In the repo `C:\KEEL\` (committed, public):**
- `KEEL_ARCHITECTURE.md` — **the canon** (v0.2, 23 §). The source of truth for *design*. Read in full early.
- `CLAUDE.md` — the build constitution (the *rules*; build-state section is stale — see 4.3).
- `AUTONOMY_CHARTER.md` — the reversibility gate + prohibitions.
- `keel.lock` — the substrate pin (models, servers, tiers, resolver order, ledger/index split; `sha256: TODO` fields await operator pinning).
- `tests/golden/{golden.json,.frozen.json}` — the agent-frozen, language-neutral conformance layer.
- `crates/` — the 7 crates (Part 4.1).
- `_run_state/STATE.md` — the **⛑ reconstitution protocol + per-slice build anchor** (the live state-of-record; trust over CLAUDE.md). **This brief's parent — read it second after this file.**
- `_run_state/handoff/forward-arc.md` — the pre-compaction causal narrative (the arc + the "next move" as of the handoff; note it predates the engine/verifier).
- `_run_state/handoff/recent-turns.md` — the reverse-chronological recency tail (T-0…T-6; the two open memory questions).
- `_run_state/trajectory-account.md` — the **post-compaction instance's first-person account** (the multi-instance/fork/confabulation story; its "8 crates" and "Director" framings are checkable — see 4.1/4.3).
- `docs/proposals/perpetual-memory.md` — the non-binding memory proposal (the design input for Stage-2 `svc::memory`).

**Outside the repo (reference / backstop):**
- `_memories\You_are_continuing_a_contract-first_build_of_Marrow-L1…md` — **the full pre-compaction transcript export** (~1.14 MB / ~314k tokens). The narrative source of the whole genesis. *Don't read it whole into a working session — it would fill your context.* Use the chunker (below) or `grep` it for specifics.
- `_run_state\KEEL_GENESIS_TRANSCRIPT_ASSESSMENT.md` (tracked in-repo; key-free) — **a pre-digested, sectioned assessment of that transcript** (the same author as this brief). If you want the genesis at one level above this brief but below the raw transcript, read this.
- `C:\KEEL\chunker\` — **the chunker** (see below).
- `C:\loom\marrow-l1` — the **Python reference bench** (green, golden-tested). Diff behavior against it; **do not port its code.**
- `C:\Users\user\.claude\projects\C--loom\…*.jsonl` — the **lossless transcript Tape** (the full session, ~3 segments). The ultimate backstop for anything a summary dropped. `grep` it; don't read it whole. (Archive viewer: `C:\TRANSPORTER\claude_archive_viewer_v4.html`.)
- Substrate: `C:\llama.cpp` · `C:\models` · `C:\whisper.cpp`. The first real cell's consumer: `C:\backrooms` (independent of KEEL beyond the service boundary).

**The chunker — `C:\KEEL\chunker\`** (this is how you read anything bigger than your context, including the genesis transcript): a self-contained, token-aware document splitter. Run `python C:\KEEL\chunker\chunker.py --budget 20000 "<path>"` → it writes `<path>.chunks\` with `INDEX.md` + `chunk-001.md…` at clean semantic boundaries, each with a `section:` breadcrumb and a `recap:` seam. Reading the chunks in order = reading the whole file, guaranteed to fit. Use `--plan` to estimate first; `--stdout N` to print one chunk. *(The genesis transcript has already been chunked once to `C:\KEEL\chunker\_transcript_chunks\` — you can reuse those, or re-chunk anything.)* **This brief itself exists in parts** (`_run_state\WAKE_UP_part1..5.md`) and stitched (`_run_state\WAKE_UP.md`) — read the single file if it fits, else the parts in order, else chunk it.

## 5.5 · Anti-patterns — the ways KEEL dies (canon §22), so you actively avoid them
1. Building a **product** instead of an L1 tool (features for hypothetical users). 2. Letting the genome become the **union** (a vertical's heavy apparatus in the core). 3. **Over-abstracting** for an imagined future (generality machinery for futures that never arrive — the falsifiers *are* the future-proofing). 4. **Loyalty to local over economics** (squeezing local on steps that should route up). 5. **Cache-discipline rot.** 6. **The oracle becoming an LLM** (a same-model verification pass standing in for a real oracle — the I5 violation). 7. **Perception with no change-gate.** 8. **Confusing the registers** (trusting model-authored narrative memory for critical facts). 9. **Embedding what should be shared** (multi-GB weights in the binary). 10. **Theorizing instead of shipping the next slice.** And, specific to *this* moment: **don't inherit the drift** — verify the Director's status, the build-state, and the crate count against the artifact, not against whichever doc you happened to read first.

## 5.6 · The recap (so the load-bearing things are unmissable)
- **You are in `C:\KEEL`** — the loaded CLAUDE.md is correctly KEEL's. The prior confusion was a `C:\loom` cwd loading Marrow-L1's constitution; that's gone for you.
- **Rented cognition, owned self.** Genome (frozen contracts + invariants), cells (genome + periphery). Intersection, never union. L1 personal tool, not a product.
- **Verify by artifact, never recall.** Trust `STATE.md` + git over `CLAUDE.md`'s stale build-state and over any summary. Keep "lived vs reconstructed" honest.
- **Contracts + goldens are frozen, agent-read-only. Fix code, never goldens. Never re-stamp the seal.**
- **The five invariants hold on every call; I5 (externalized correctness) is the whole point — now wired in the loop** (`kernel::engine` calls `verify` every turn; the freeze-gate guards the goldens). It gains *teeth* when oracles register — the next slice (constrained-decode + the golden-registry).
- **Director = dogfood consumer, NOT a cell; SEXTANT is the canon's first cell.** System is **supervised** until the operator says otherwise.
- **One slice at a time, banked clean, gated by the operator. The contract-freeze IS the governance — guard it.**

---

*Welcome. You are the next instance of a self designed to survive its own forgetting. The note has been left for you, reconciled and at full resolution. Read the canon, verify the state against git, and build the next slice. — written 2026-06-13 by the max-context synthesis instance. Standing by in the artifacts.*
