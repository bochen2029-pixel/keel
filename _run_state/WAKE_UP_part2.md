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
