# Memory Autopilot — "solve memory" (proposal, 2026-07-09)

**Provenance.** Operator directive 2026-07-09 (live, supervised session): *"if I really start using KEEL,
I don't want to worry about memory management — it needs to functionally-forever manage memory in a
smart, contextually relevant way without my intervention. Not the flywheel — just functionally perpetual
memory per the KEEL idea and techniques. Solve memory first, before D1."* This document extends
`docs/proposals/perpetual-memory.md` (the operator's by-hand techniques) and targets REEL-v1.0 Tier-2
fidelity (`C:\OUTREACH\BC_Canon\MAY2026\Deeper\REEL_PROTOCOL_v1_0.md`) with KEEL's work-first register
correction (canon §11: the Tape validates the narrative, never the reverse).

**The goal, stated as an invariant:** *a KEEL consumer (CLI, serve, daemon, or a cell) gets unbounded
retention with bounded context and zero memory-management surface — no flags, no manual consolidation,
no manual drift checks — while every critical fact stays in the lossless factual register (I5).*

---

## 1 · The verified gap (why memory is not "solved" today)

What exists and is lived: the Tape (lossless, append-only, capture-sane) · Ring-0 soul slot · Ring-2
recent-N turns · Ring-3 rolling narrative via `consolidate()` (LIVED, $0 sovereign-local) · Ring-4
cosine recall + fingerprint gate (`GOLDEN_RECALL` green) · `keel cold-eyes` (LIVED — caught real drift).

What makes it hand-cranked (verified at the call sites, 2026-07-09):
1. **Ring-4 is inert in production** — `with_embedder` is called only in tests; all four L5
   `FileMemory::new` sites wire Ring-0/2/3 only (AUDIT-2026-06-15 L8). The embed GGUF is now on disk
   (ISSUE-10 resolved) but nothing resolves/launches an embed server or wires recall.
2. **No automatic triggers** — consolidation runs only via `keel consolidate` or the daemon's
   `--consolidate-every N` flag (default 0 = off). CLI and serve never consolidate. Cold-eyes is manual.
3. **No budget enforcement** — `assemble` layers rings by count (recent-N), not by a context budget;
   a long Tape/narrative can grow the preamble without bound. Canon §7 calls `assemble` "ringed +
   budgeted"; the budget half is not real yet.
4. **Single rolling narrative** — Ring-3 is overwritten each consolidation. The arc survives but
   mid-term episodic detail has no durable home between "recent turns" and "raw Tape."
5. **No self-correction loop** — cold-eyes *detects* drift but nothing acts on the verdict.

## 2 · Design thesis

KEEL already has the right bones; what is missing is the **autopilot layer**: budgets, triggers, a
mid-resolution register, default-on recall, and closing the cold-eyes loop. Everything lands in
**L4/L5 behind the frozen `Memory` trait (`assemble/record/consolidate`)** — zero contract edits, zero
golden edits, no new frozen surface. The Tape remains the single source of truth; every sidecar
(narrative, episodes, vectors) is derived and rebuildable from it (reversibility: deleting any sidecar
is always safe).

**The rings after A7:**

| Ring | Content | Persistence | Context cost |
|---|---|---|---|
| 0 soul | identity (cell-authored; genome default empty) | constitutional, never trimmed | fixed |
| 1 exemplars (A7.6) | calibration pool, anchor + rotation | never auto-deleted | budgeted |
| 2 working | recent turns from the Tape | volatile | budgeted |
| 3 narrative | one rolling model-authored arc (topology) | overwritten per consolidation, drift-checked | budgeted (small) |
| 3.5 episodes (new) | five-field digests per consolidation | **append-only, never overwritten** | zero (retrieval targets only) |
| 4 recall | vectors over episodes + recent turns | derived sidecar, fingerprint-gated | budgeted (top-k) |
| Tape | everything | immutable | zero (retrieval source) |

**Why narrative + episodes instead of REEL's full L1–L4 cascade + pruning pass:** the cascade's *goal*
(multi-resolution history, graceful compression, no overflow) is met by narrative (topology) +
episodes (mid-tier, append-only) + Tape (full) + hard budgets — with no pruning engine, no immunity
flags, no rebalancing pass to build or trust. Append-only episodes also cannot suffer
compression-of-compression loss (REEL anti-pattern §10.2) because they are written once from
near-in-time material and never re-compressed. *Falsifier: if lived use shows recall quality degrading
on mid-term history or episodes bloating the sidecars uselessly, revisit the cascade.*

## 3 · The slices (dependency-ordered; each gated, banked, pushed)

### A7.1 — Budgeted assembly (the context governor)
`MemoryBudget { ring1_max, ring2_max, ring3_max, ring4_max }` in chars (a ~4-chars/token proxy — no
tokenizer dep; falsifier below). `assemble` enforces: Ring-0 verbatim always (identity protection,
REEL §3.4) · narrative trimmed to `ring3_max` · Ring-2 newest-first until `ring2_max` · Ring-4 top-k
until `ring4_max`. Defaults derived from the keel.lock tier `max_tokens` with REEL §4.7-shaped ratios.
**Outcome: O(1) context per turn regardless of Tape size — the "functionally forever" precondition.**
Model-free tests: over-budget Tape stays under caps · newest survives trimming · Ring-0 never trimmed.
No new dep.

### A7.2 — The episodes register (mid-resolution, append-only)
Consolidation now also **appends** an episode digest to `<tape_stem>.episodes.jsonl`: the REEL §6.2
five-field schema `{what_happened, what_changed, what_matters, unresolved, anchors}` + a time range +
turn refs. The consolidation prompt is extended to ask for the five fields plus the rolling arc (one
generation, two artifacts). Episodes are never loaded wholesale — they are retrieval targets (A7.3)
and the cold-eyes diff substrate (A7.5). Model-free tests: append/read/schema plumbing; generation
validated live (bounded, ISSUE-8 pattern). No new dep.

### A7.3 — Ring-4 ON by default (recall autopilot)
- `kernel::lifecycle` resolves the **embed substrate** per keel.lock's `embedded_tiny` resolver rung:
  probe a configured embed endpoint; cold-start `llama-server --embeddings --pooling last` on its own
  port from `substrate.embedding` when absent (reuse the proven launch/health machinery; explicit std
  handles per ISSUE-8).
- L5 wires `with_embedder` **at all four `FileMemory::new` sites** when the substrate resolves;
  otherwise graceful degrade to Ring-0/2/3 (never blocks a turn; one I1 log line). The genome default
  flips from recall-off to **recall-when-resolvable** — zero operator surface.
- **Two-tier index:** embed-on-record for turns AND episodes. Recall scans all episode vectors +
  a bounded recent-turn window (default ~4k turns). Keeps ISSUE-1's brute-force decision fast
  functionally-forever: the hot set is O(episodes + window), not O(Tape). Older detail stays reachable
  because episodes carry anchors/turn-refs. *Falsifier (re-opens `sqlite-vec`): recall p95 > ~50 ms at
  the default window.*
- Cold-start backfill: missing/mismatched sidecar → rebuild-from-Tape (extends the existing
  fingerprint rebuild).
Tests: resolver probe/launch parse · degrade path · two-tier scan bound · backfill. No new dep.

### A7.4 — The autopilot policy (triggers; the zero-intervention core)
An L4 `MaintenancePolicy` (svc::memory — an internal seam like `recall::Embed`, NOT a frozen joint):
`due(stats) -> Option<Maintenance>` over `{turns_since_consolidation, ring2_overflow, session_end,
consolidations_since_cold_eyes}`. Defaults (keel.lock-configurable — config, not pins): consolidate
every N=24 turns OR on Ring-2 budget overflow OR at session end with ≥K=4 new turns; cold-eyes every
M=4 consolidations. Wiring: the daemon generalizes `--consolidate-every` onto the policy; the CLI
runs a due check post-turn (session end = the one-shot exit); serve checks post-turn (bounded, inline).
All maintenance turns remain sovereign→local, Tape-excluded, I1-audited (already the case).
Model-free tests: the policy decision table. No new dep.

### A7.5 — Self-correcting narrative (close the I5 loop)
Parse the cold-eyes verdict (CONSISTENT vs drift findings). On drift: **one** bounded
regenerate-from-ground-truth consolidation — the prompt carries the drift findings + the episodes +
recent Tape and instructs correction (the REEL §10.2 fix: regenerate from source, not from the drifted
copy) — then an I1 `MEMORY_DRIFT_CORRECTED` audit event (labels, never content). Persistent drift
(fails again next cadence) → surfaced in `keel metrics` + the ISSUES register; **never a retry loop**.
Tests: verdict parsing · policy fires exactly one regenerate · loop bound. No new dep.

### A7.6 — Persona-grade polish (optional; work cells skip it)
Ring-1 exemplar pool (`<tape_stem>.exemplars.md`; operator/cell-authored; anchor + rotation-N into
`assemble` under `ring1_max`; never auto-deleted — REEL §3.4; auto-promotion from episodes deferred).
Ring-2 as real `user/assistant` conversation messages instead of a system preamble (the deferred
memory.rs item — improves model behavior). Both additive; empty defaults like the soul.

## 4 · What this deliberately does NOT include (each with its re-open trigger)

- **The flywheel** (operator: out of scope). Episodes/corpus stay compatible with it.
- **Full REEL L1–L4 cascade + pruning pass + immunity flags** — superseded by narrative+episodes+budgets
  (§2 falsifier re-opens it).
- **`sqlite-vec`** — ISSUE-1 stands; the A7.3 latency falsifier re-opens it.
- **A tokenizer dep** — char-proxy budgets; re-open if lived runs overflow the window despite caps.
- **Model-native (Tier-3) memory behavior** — that IS the flywheel/LoRA path; out of scope.
- **A significance-threshold trigger** (REEL §6.2.4) — model-dependent; the cheap proxy (oracle-failure
  or high-cost turns flagging an episode) may ride A7.2 as a field, but no detector is built.

## 5 · Invariant constraints (unchanged, restated)

Zero frozen-contract/golden edits (all behind `Memory` + L4/L5 config). I3: every memory operation is
sovereign→local; vectors and Tape content never egress. I1: every maintenance turn and drift correction
is audited. I5: the Tape is ground truth; the narrative is validated against it; episodes are
append-only. Reversibility: sidecars are derived — rebuild-from-Tape is always available; no memory
operation can lose Tape data.

## 6 · Acceptance — "memory is solved" falsifiers

- **F-M1 (perpetual):** seed a Tape with ≥5k synthetic turns → every `assemble` stays under budget and
  recall latency stays bounded. O(1) context per turn, measured.
- **F-M2 (relevant):** plant a fact, run ≥N consolidations of unrelated turns (beyond Ring-2/narrative
  reach) → a query about the fact surfaces it via Ring-4/episodes. Lived.
- **F-M3 (zero-intervention):** a bounded multi-session daemon+CLI run with NO memory flags/commands →
  consolidations + cold-eyes fire per policy; verified by artifact (Tape · episodes · audit ledger ·
  `keel metrics`), per the kill-switch discipline.
- **F-M4 (honest):** plant narrative drift → the cadence catches it → auto-correction → the next
  cold-eyes reads CONSISTENT, with the I1 event on the ledger.

C1/C2 (recall-uplift falsifiers) remain separate Phase-C items — A7.3 is what makes them runnable.

## 6.5 · RESULTS (2026-07-09 — built + lived, same session)

All six slices landed (commits `56d116a` → A7.1 · `877f410` → A7.2 · `34d86ad` → A7.3 · `36d3485` →
A7.4 · `569a430` → A7.5 · `b17e22b` → A7.6 · one hardening commit) — **zero contract/golden edits,
seal unmoved, 155/6 green, clippy clean.**

**Acceptance:** **F-M1 PASS** (5k-turn Tape → bounded assemble, <2 s scan; a permanent unit test).
**F-M2 PASS lived** (BLUEFIN planted, buried beyond Ring-2 by six sessions, recalled correctly —
carried by both Ring-3 and Ring-4). **F-M3 PASS lived** (zero-flag CLI sessions: policy consolidated
at exactly the right turns, real five-field episodes appended, cursor tracked; the embed server
cold-started itself on :8090 and backfilled the existing Tape). **F-M4: the detect → correct →
confirm machinery is PROVEN lived** — two different planted drifts erased, `MEMORY_DRIFT_CORRECTED`
/ `MEMORY_DRIFT_PERSISTENT` I1 events on the ledger, the correction bounded (never a loop), and the
pending-drift confirm transition ("the drift correction held") exercised. **Honest residual:**
single-judge recall on *adversarial* plants is stochastic on Qwen3.5-9B (organic-drift catches — the
2026-06-15 lived case — remain solid); a 2-of-3 majority vote is implemented; the upgrade trigger is
a stronger local judge model. Cloud judging is declined by default — sovereignty beats judge recall
on personal memory (I3).

**Lived lessons hardened in the same session (each caught by the falsifier runs, each now guarded):**
1. **Maintenance-turn contamination** — the engine was injecting the assembled narrative into
   cold-eyes/corrective turns (the narrative rode into its own validation; a correction re-anchored
   on the drifted copy). Fixed in the kernel: `source == "memory"` steps skip ring assembly — the
   read-side twin of the Tape-exclusion. +1 kernel test.
2. **Cold-eyes window-blindness** — the reviewer saw only the recent Tape window, so legitimate
   old-arc claims read as drift forever. Fixed: episodes join the reviewer's ground truth.
3. **Reasoning-envelope leakage** — `<think>` blocks polluted stored registers and verdict parsing;
   a 2048-token cap truncated a thinking pass mid-reasoning. Fixed: strip closed think-blocks,
   treat an unclosed one as no-answer; local tier `max_tokens` → 4096 (keel.lock).
4. **Template parroting on thin substrates** — the model copied layout placeholders verbatim.
   Fixed: format contract moved to the end of the prompt (recency anchoring), parenthesized
   never-copy hints, and a store-side placeholder guard.
5. **Corrector degeneracy/sprawl** — a lean corrective pass stored a 10-char stub; a thinking one
   wrote an essay quoting the false claims. Fixed: corrector thinks + a 40-char floor (a stub never
   clobbers the narrative) + a never-mention-the-claims contract.
6. **Judge stochasticity** — the same plant was caught twice and missed once. Fixed structurally:
   2-of-3 majority vote (local, $0), short-circuiting.

## 7 · Sequencing

A7.1 → A7.2 → A7.3 → A7.4 → A7.5 (each independently useful; ~a session or less each), A7.6 optional
after. **A7 precedes D1** (operator's word); D1 (NightScribe) then *consumes* the autopilot — its
intent-memory becomes Ring-4 lived in a real cell — which makes D1 the memory autopilot's field
validation rather than a competing priority.
