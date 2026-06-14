# KEEL — Recent Turns (REVERSE-CHRONOLOGICAL recency tail)

**⚠ READ ORDER: most-recent-FIRST. Top = newest, bottom = oldest.** You are reading memory *backwards on
purpose* — newest context first, because it's most relevant to "where we are now." This is the **recency
register** (REEL Ring 2): the immediate conversational thread at higher resolution. Reconstruct the causal
order from `forward-arc.md` (forward narrative). Trust git + contracts/goldens (the factual register) over this.

---

### T-0 (NOW) — mid memory-handoff
Operator: **"proceed."** Outgoing instance is writing THIS file — step 2 of a 3-step pre-compaction handoff:
(1) `forward-arc.md` ✓ committed `5083c41`; (2) recent-turns = **now**; (3) persist/wire the reconstitution flow.
Context ~91%, compaction imminent (operator can't force it). On resume, pick up from the wired flow.

### T-1 — "write forward arc, then recent memories, then persist" (at 91%)
Operator sent a context-window screenshot (914.2k/1.0M, 91%) and laid out the plan: split the memory write so I
don't hit the output-token limit — **forward arc this turn, recent memories next, then persist**, then resume after
compaction. **Confirmed the resume point: "wire the engine so it all self-drives" is exactly where they wanted to
go next** before the memory tangent — *"note and log that too."* Endorsed the forward+reverse ("both sides")
approach. → I wrote `forward-arc.md`, committed `5083c41`, pushed.

### T-2 — my memory brainstorm + proposal captured
Honest Skeptic-pass assessment of the operator's by-hand memory techniques: write-to-disk→rehydrate (strongest;
proven here via STATE.md); reverse-recency refeed (good for recency but **pair with a forward narrative**; the
meta-LABEL is the load-bearing part); the **dialogic handshake** (most novel; doesn't fit auto-compaction → adapt as
a **pre-compaction self-interview**); self-curated compression (right for the **narrative** register, but I5 → keep
critical facts in the lossless **Tape** + add cold-eyes validation). Biggest fix over manual: **continuous append**
kills the "I forget to dump" failure. Reframe: not perpetual memory, **perfect re-constructability**. → captured
`docs/proposals/perpetual-memory.md` (`1cb6d8c`). **TWO OPEN QUESTIONS I asked the operator (still pending):**
(a) draft the `/checkpoint`+`/rehydrate` **skill now** (helps this session) or proposal-only until `svc::memory`?
(b) is the **dialogic-handshake-as-self-interview** worth prototyping, or over-engineering?

### T-3 — operator's memory-techniques audio brainstorm
Long audio transcript describing the hand-developed continuity kit (memory-dump/continuation prompt, write-to-disk
rehydrate, reverse-order verbatim refeed, the talking/calibrated handshake handoff, REEL's five rings, self-curated
compression). Asked: can this be a **skill** / make it into the **harness config** / benefit KEEL — and **capture the
insight in the project so it's not lost**, noting (ironically) I'm near compaction myself. "Brainstorm for now."

### T-4 — SIRP review accepted → build the router
Operator: *"go with your own best recommendations and then continue on and proceed before I interrupted you."* →
built `keel-services::router::DifficultyRouter` (`f620cb1`), validated against frozen `GOLDEN_ROUTER`, as a swappable
`Router` policy (the trait is the seam; `SirpRouter` slots in later). Reported Stage 1 begun; teed up "wire the engine."

### T-5 — SIRP shared + the swappable-router question
Operator shared `SIRP_v1.md` (their March router spec, built w/ Opus 4.5) and asked whether the router's *core logic*
could be swappable like models. I downloaded + read it fully, gave an honest review — strong (intent-multiplexer
framing, three-axis model, Layer-2 semantic-abstraction preprocessor); weak (the quality signal rests on a non-model
score = a JOINT_WRONG/I5 risk; cooling-vs-safety tension; federation premature) — and synthesized: KEEL's `Router`
trait is the policy seam; a `SirpRouter` is a future swappable policy **only once its quality signal rests on a
non-model oracle (I5)**, which KEEL supplies and SIRP lacks.

### T-6 — Opus 4.8 wired + Stage 0 finished
Operator gave the **Anthropic API key** (KEEL workspace key), *"wire up opus 4.8... then resume the stage 0
remainder."* → `keel-adapters::anthropic` (`07c0787`): Opus 4.8 live via the Messages API (its own protocol = thin
gateway), real cost validated, keel.lock frontier price confirmed correct ($5/$0.50/$25 per MTok). Then
`store::sqlite`/`Spine` I2 (`6580744`). **Three-tier economy complete; Stage 0 spine done.** Keys in env
(`DEEPSEEK_API_KEY`, `ANTHROPIC_API_KEY`, User-level), verified absent from the repo tree.

---
*Older context → `forward-arc.md` (full arc) + `STATE.md` (per-slice state). Verify everything by `git`/`cargo`.*
