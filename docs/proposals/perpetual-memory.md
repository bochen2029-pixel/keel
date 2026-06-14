# Proposal: Perpetual Memory — operator techniques → KEEL Memory organ

**Status:** proposal, non-binding. No spec/contract/code changes implied (the ten joints are frozen).
**Captured:** 2026-06-13, from an operator audio brainstorm — recorded here so the insight survives
compaction (which is itself the thesis of the document).

---

## Premise
Even a 1M-token context runs out. The operator has hand-developed a suite of continuity techniques.
Most of them turn out to be the **empirical, by-hand validation of KEEL's REEL-shaped Memory design**
(canon §11) — and two of them are genuinely new and worth folding in. This doc records the techniques,
assesses each honestly, and proposes a **two-layer path**: a reusable *skill* now (harness-agnostic),
and *Memory-organ enrichments* later (behind the frozen `Memory` trait).

## The techniques (operator) + honest assessment

1. **Write-to-disk + post-compaction rehydrate.** *Strongest, and already PROVEN this session* via
   `_run_state/STATE.md` + the "verify by artifact, never recall" rule. This is KEEL's Tape/Spine (I2).
   → generalize into a skill so it's never manual.

2. **Reverse-order recency refeed** (paste the last ~3–6 turns verbatim, most-recent-first, *meta-labeled*
   "this is reverse-chronological"). Good for **recency priority** (REEL Ring 2 / Poincaré "now = full
   resolution"). The **meta-label is the load-bearing part** — it lets the model orient regardless of order.
   *Caveat:* reverse order helps "what is the current state" but can scramble "how did we get here"
   (cause→effect). **Pair it with a FORWARD narrative** for the causal arc (Ring 3). (The operator applied
   this to the assistant earlier in this very session — it worked.)

3. **Dialogic / calibrated handoff (the "handshake").** *The most novel.* Old + new instances coexist
   briefly: the new one asks questions, the old one answers from live context, they mutually acknowledge,
   and the new instance's first few turns are checked against the old for drift. This is **active
   error-correction vs a passive dump** — it surfaces the gaps a static summary silently drops.
   *Limitation:* needs two coexisting instances — it does NOT fit single-session auto-compaction (the old
   context is gone). *Adaptation:* a **pre-compaction self-interview** — the instance, while it still has
   full context, generates the anticipated questions + answers for its successor, baking the handshake
   into the artifact.

4. **Self-curated compression** (the persona, in its current memory state, reads the whole transcript and
   decides what to keep — vs a blanket classifier or a fresh "summarize this" instance). This is REEL §2.3–2.4
   (persona-shaped, identity-coherent compression) — the operator's operational rediscovery of the thesis,
   and it's **right for the narrative/voice register**: the invested model has the richest weighting of what
   mattered. *Two honest caveats:* (a) **I5** — a model may not author its own *ground truth*; for **critical
   facts** keep the externalized lossless **Tape** (the factual register), never the self-summary; (b)
   **cold-eyes blind spot** — the invested model may over-keep what it found salient and under-keep what it
   overlooked (often the very things that caused errors). Add a **periodic fresh-instance validation pass
   against the Tape** (REEL anti-pattern §10.2).

## The single biggest improvement over the manual process
The operator's stated failure mode is *"sometimes I forget"* to dump. The fix is REEL **capture sanctity**:
**append every turn to the Tape synchronously**, so there is never a manual end-of-session dump to forget;
**consolidate on a trigger** (context-pressure threshold). The record always exists; rehydration just reads it.

## Reframe
"Near-perpetual memory" → **perfect re-constructability, not perfect recall.** Poincaré disk: *now* is
full-resolution, the distant past is gracefully compressed, nothing is truly lost (the Tape is lossless).
The target is faithful reconstruction, not infinite context.

## Two-layer plan

**A. A skill (now, works in any harness — this coding session or the web).**
A `/checkpoint` + `/rehydrate` skill that automates the ritual: snapshot a structured artifact —
*factual anchors* + *forward narrative* + *reverse-recency verbatim* + *anticipated-Q&A self-interview* —
and on resume run reconstitution (read → verify-by-artifact → reverse-recency load). Removes the tedium
**and** the forget-failure.

**B. KEEL Memory-organ enrichments (later, behind the frozen `Memory` trait — Stage 2).**
- `consolidate()` emits the **handshake/self-interview** artifact, not a flat summary.
- Ring-2 assembly does the **reverse-recency load** (forward narrative for Ring 3).
- **narrative register = self-curated** (model-authored); **factual register = the lossless Tape**.
- a **cold-eyes validation Step** periodically diffs the narrative against the Tape (closes the
  self-curation blind spot; satisfies I5).
- the Memory's **consolidation policy is itself swappable** (same pattern as the `Router`/`SirpRouter`
  policy seam): different cells get different compression priors.

## Already true in KEEL
- `STATE.md` reconstitution + verify-by-artifact = technique 1, proven this session.
- canon §11 (rings · narrative/factual registers · Tape · consolidation-as-a-Step) is the home for 2–4.
- New contributions to fold in at the Memory slice: the **dialogic-handshake / self-interview artifact**,
  and the **explicit reverse-recency-with-label** assembly.

*Revisit when building `svc::memory` (Stage 2). Until then: contracts frozen, this is a design note only.*
