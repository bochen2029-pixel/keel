# KEEL — Session Account: Stage 2, `kernel::engine` → `metrics` (2026-06-14)

> **What this file is.** A self-distilled, comprehensive account of one long supervised build session
> on KEEL — written by the instance that lived it, at ~85% context (854.6k / 1.0M), as the snapshot
> the operator asked for: "a capture of this entire moment and everything that led up to it." It is
> deliberately dense where the work is technical and narrative where the arc calls for it. It follows
> the `perpetual-memory` doctrine (REEL rings: the core never drops; the periphery is terse because
> git/files verify it). **Author:** Claude Opus 4.8 (1M context). **Verify anything load-bearing
> against the artifacts** — git, the contracts, the goldens, `cargo test` — that is KEEL's prime
> discipline and it applies to this file too.

---

## 0 · THE MOMENT THIS CAPTURES (core — read first)

- **What KEEL is, one line:** *rented cognition, owned self* — a single-operator, sovereign,
  native-Rust **genome** (frozen contracts + invariants) that perceives, remembers, routes every unit
  of work to the cheapest brain clearing a trust bar, and grounds every critical output in an
  assertion **no model authored** (I5). The model is interchangeable; KEEL is the self.
- **Where the session ended:** Stage 2 is largely built. This session took KEEL from *"the verifier
  exists but isn't wired"* to **a closed externality loop with real teeth** — I5 verifies in the loop,
  fails closed on missing/critical assertions, a named `golden_ref` actually asserts, the alarm is
  console-safe, and a flywheel `metrics` reader exists.
- **The single open action right now:** the **`metrics` commit `568ea21` is banked locally but NOT
  pushed.** It awaits the operator's review/go. `origin/main = 4dec51f`; local `main` is **ahead 1**.
  Everything else this session is pushed and public.
- **The next slice (operator-sequenced, not started):** the **serve→Director bridge** —
  `serve_openai` gains `critical` / `golden_ref` + a constrained-decode path, then the **Backrooms
  Director** dogfood closes the I5 **accept direction in-binary** (a schema-valid Directive *passes*
  via a ref) and lands KEEL's first external consumer. Then **perception**, then **`svc::memory`**
  (designed most skeptically against `docs/proposals/perpetual-memory.md` — the canon's flagged
  highest-risk seam), then the flywheel / SEXTANT.
- **Where the source-of-truth lives on disk:** `_run_state/STATE.md` (the live per-slice anchor) and
  `git` are authoritative. `_run_state/WAKE_UP.md` is the first-read onboarding brief (refreshed this
  session to post-engine truth). The canon is `KEEL_ARCHITECTURE.md`. The rules are `CLAUDE.md`.
  **Trust the files + git over any summary, including this one.**
- **The system is SUPERVISED.** No unattended-autonomy grant. The operator gates every slice
  one step at a time.

---

## 1 · WHAT KEEL IS (one breath, so the rest lands)

KEEL is Bo Chen's personal **L1 tool, not a product** — built as the *intersection* of seven-to-nine
systems he independently hand-built across three languages (DAVE/Tenancy, TERMINAL — Rust; TARS,
REEL, the In-Home Companion, SEXTANT — Python; photo2deck/NightScribe/NightClerk — C#; ASTRA-7 —
C++/UE5). Each rebuilt the same skeleton by hand; KEEL is that skeleton, written once.

The **genome** = a tiny frozen set of ten contract traits + five invariants + agent-frozen golden
cases, from which every specialization is a **cell** = genome + periphery, *never* by editing the
core. Consumed two ways: **embedded** (Rust apps link it) or **over protocol** (`serve_openai` /
MCP / HTTP).

**The five invariants + the reversibility gate** (enforced on every call):
- **I1 Observable** — every call emits a structured audit event (in the middleware chain, unbypassable).
- **I2 Durable** — every state change persists to the Spine; resume from checkpoint.
- **I3 Filtered (Sovereign)** — a *gate* (force sovereign/PHI/raw-perception/vectors to `local`) + a
  *mask* (scrub residual PII before egress). The most intimate inputs are the most protected.
- **I4 Governed** — every call's cost tracked; per-task budget hard-stops; can BLOCK.
- **I5 Externalized** — **every critical output carries ≥1 assertion no model authored.** A model may
  author the plan, the code, even a verification pass — *not its own ground truth.* This is the whole
  point; it's what makes rented cognition trustworthy. Memory-safety = I5 on the source (the borrow
  checker is the oracle, ADR #5).
- **Reversibility gate** — undo-cost-unstatable-in-one-sentence ⇒ stop and ask.

**Layer rule (law):** `contracts ← kernel ← {adapters, middleware} ← services ← apps`. The kernel
imports only contracts; middleware never imports a service. Violations are bugs.

---

## 2 · WHERE THIS SESSION BEGAN

The session started with HEAD at **`61334b3`** ("Stage 2: svc::verifier — the externality layer (I5);
GOLDEN_ORACLE green") — the last commit of a *prior* session. Inherited state:

- **Stage 0 (spine) complete:** 7 crates, three-tier economy (local `$0` · cheap-API DeepSeek ·
  frontier Opus 4.8) through one invariant chain, file ledger + SQLite Spine, self-resolving
  substrate, consumable embedded + over protocol.
- **Stage 1 landed:** `DifficultyRouter` (GOLDEN_ROUTER ✓) + a self-driving engine living at **L5**
  (`keel::Engine`) — flagged as the "L5→L1 engine debt."
- **`svc::verifier` (I5) landed but NOT wired into the running loop** — golden-green at L4, but
  `Engine::run` never called it. *This was THE gap.*
- The golden freeze-gate was built but **`#[ignore]`-dormant** (its stored seal `63d5ba7c…` was
  Marrow-Python-derived; content git-verified unchanged).
- Doc drift: `CLAUDE.md`'s build-state was stale; `STATE.md` called the Backrooms Director the
  "first cell" (a partly-walked-back framing).

**The onboarding (Phases 1–4, no code):** I read WAKE_UP.md, the canon, CLAUDE.md, AUTONOMY_CHARTER,
STATE.md, keel.lock, the goldens — all in full. Then **verified by artifact**: `git log/status`,
`cargo check/clippy/test` (51 passed / 4 ignored), confirmed 7 crates (not 8 — a `trajectory-account`
slip), confirmed `kernel::engine`/`lock` absent, and **read `keel/src/lib.rs` to prove the
I5-not-wired claim directly** (`Engine::run` did route→chain→return, no verify/checkpoint/emit). Then
I restated understanding and proposed the next slice. The operator approved with rulings.

---

## 3 · THE TRAJECTORY — every slice, in order (the spine of the session)

Each slice followed the same loop: **propose → operator approves → build → gate → bank local →
show diff+tests → operator reviews → push on his word.** One slice at a time, banked clean.

### 3.1 · Step 0 + Step 1 — record reconciliation + `kernel::engine` (L1) → commit `8650a47`

**Step 0 (record reconciliation)**, on the operator's rulings:
- **Director ruling confirmed:** the Backrooms Director is a **first external dogfood consumer over
  `serve_openai`, NOT a cell, NOT the canon's first cell — SEXTANT remains the canon's first cell.**
  Corrected `STATE.md`'s Director line (split into two bullets: SEXTANT = first cell; Director =
  first dogfood consumer) and the stale `C:\loom` cwd note.
- **CLAUDE.md build-state** refreshed (Stage 0 complete · Stage 1 + verifier landed · goldens FROZEN
  · points to STATE.md as the live source).
- **Marrow bench** reworded to "read-only behavior reference / diff-oracle, never a build/runtime
  dependency" (reconciling the self-contained directive with the bench reference).
- **Canon §8 footnote** added: the per-tier egress-correct chain (I3) — the single `chain` in the §8
  pseudocode is illustrative; the impl holds one chain per tier.

**Step 1 (the highest-leverage slice) — `kernel::engine` at L1**, over *injected* `&dyn Router /
Oracle / Spine` (+ optional `Memory / TraceSink`), running the canon §8 loop:
**assemble → route → chain → verify → fold-cost → checkpoint → emit.** One slice, four wins:
- **I5 wired live** — every turn calls `Oracle::verify`. `Verifier` gained `impl Oracle` (the
  registry-as-composite) — a pure L4 add, **zero contract edit**.
- **I4 cost accumulates** — the engine owns `Context` and folds `result.cost` after each chain
  (`mw::cost` stays the pre-call gate; no double-count — confirmed against the chain's own doc).
- **I2 checkpointing active** — each turn's `Trace` → the SQLite `Spine`.
- **L5→L1 debt paid** — `keel::Engine` (L5) shrank to a pure-injection wrapper.
- `oracle_failures`/`tier_history` feed back onto the `Step` so the escalation ladder fires across
  turns (proven by a cross-turn-escalation test). **+7 engine tests; 58 green.** The slice was
  written, gated, banked, shown; **then verified in the real binary later** (smoke at the Half-B
  stage and after).

### 3.2 · Secrets + the run-state commit → commit `9fd5de6`

The operator flagged a **key-leak risk**: the genesis transcript (`_memories/`) and its derived
chunks contain his plaintext DeepSeek/Anthropic keys, and the repo is **public**. I:
- Hardened `.gitignore`: `/_memories/`, `/chunker/_transcript_chunks/`, `**/*.chunks/`.
- **Secret-scanned** (count/files-with-matches mode so no key value was ever printed) — confirmed the
  transcript + 2 chunks carry keys; the run-state docs + chunker tool + the genesis *assessment* are
  clean.
- Moved `KEEL_GENESIS_TRANSCRIPT_ASSESSMENT.md` out of `_memories/` (ignored) into tracked
  `_run_state/` (key-free), fixing the pointers in WAKE_UP.
- Committed the reconstitution scaffolding (WAKE_UP brief + parts, INIT_PROMPT, trajectory-account,
  the assessment, the chunker *tool* — not its output) so the "read WAKE_UP first" pointer resolves
  on a fresh clone. **Final pre-commit gate: scanned the staged diff key-free.**

### 3.3 · The freeze-gate re-stamp handshake → commits `21de876`, `d341d43`

I5 governance ("the contract-freeze IS the governance"). The dormant freeze-gate needed the
operator's one-time **KEEL-native re-stamp** (a non-delegable act — the agent never authors frozen
ground truth). I gave him the exact command + what to check. **The discipline moment:** his message
said *"new KEEL-native seal: `<seal>`"* — a literal placeholder. I **verified by artifact** instead
of trusting it: ran the agent-allowed dry-run (`golden_freeze` example, no `--update`) → computed the
expected hash `db4377b3…`, then read `.frozen.json` on disk → it still held the **old** `63d5ba7c…`.
So I **refused to un-ignore against the mismatch** and surfaced it. He then actually re-stamped; I
verified `.frozen.json` = `db4377b3…` with **only** that file changed (`golden.json` byte-identical),
**un-ignored** `goldens_match_the_frozen_hash` (a code edit — my part), ran it green, and banked
`21de876`. Then `d341d43` tidied STATE.md (freeze-gate ACTIVE; the reconstitution-protocol seal-check
line updated `63d5ba7c…`→`db4377b3…`). All A→present commits were pushed together later.

### 3.4 · WAKE_UP refresh + re-stitch → commit `099f386`

The first-read anchor (WAKE_UP) had gone stale (it said "the next slice is build kernel::engine" when
that was done). I refreshed **only the state-as-of sections** — §0 TL;DR (part1), §4 build-state +
scorecard + gaps (part4), §5.1 roadmap (part5) — to post-engine truth, left Parts 1–3 / arch map /
anti-drift list / prohibitions / file map **timeless and unchanged**, added a dated header naming
STATE.md+git as the live source, and **re-stitched `WAKE_UP.md` = cat(parts)** (297 lines, verified
sum-of-parts == stitched). This established the discipline: *edit the parts, re-stitch, never let the
single file and the parts drift.* Repeated on every later doc fold.

### 3.5 · Half A — constrained-decode conformance → commit `e3b482a`

Turned the **`GOLDEN_MODEL_TIER`** golden green (model-free, plain `cargo test`):
- **`passes_golden_model_tier`** (keel-adapters): case [0] cost via `compute_cost`; case [1] schema →
  llama-server `json_schema` decode hook; case [2] `reasoning_content` replayed across a tool
  round-trip (no 400).
- **`SchemaOracle`** (keel-services) — the Backrooms Director's *"schema-valid Directive or reject"*
  gate: validates output JSON against a JSON Schema **in-memory**, invalid → rejected (never
  partially applied), uncompilable schema → fail-closed. **The load-bearing test is the rejection
  test** (missing-required AND wrong-type → rejected), per the operator: "a validator that accepts
  everything passes the accept-case trivially."
- **The dependency catch:** the operator ruled "use the mature `jsonschema` crate, not a hand-rolled
  parser — the oracle's correctness IS the I5 guarantee; hand-rolling a safety-critical parser is a
  JOINT_WRONG risk." I `cargo add`ed it and **`cargo tree`'d it** — found it pulled `reqwest` + a full
  TLS stack + **`aws-lc-rs` (C crypto, needs cmake)** by default: a **network egress surface inside a
  sovereign safety oracle** + a fragile build dep. Resolution: **`default-features = false`** — strips
  all of it; the in-memory validator core remains. Trustworthy crate, no egress, no cmake. (62 green.)

### 3.6 · Half B — give the per-turn `verify` teeth → commit `03d9d10`

The engine ran `verify` but it was **vacuous** (empty registry). Half B made it bite:
- **`EngineConfig`** — refactored `Engine::new` from positional args to a kernel-internal config
  struct (dodges clippy's `too_many_arguments`; absorbs future seams without signature churn).
- **`golden_refs`→`GoldenCase` resolver** — the engine resolves `step.golden_refs` against an injected
  `HashMap<String, GoldenCase>` and passes the cases to `verify`.
- **Three hardenings** (the operator's): (1) an **unresolved `golden_ref` → fail-closed** (a
  named-but-missing assertion is a hole → operator, never a vacuous pass); (2) a **`critical` step
  with no effective assertion → config fault** (canon §8/§10), a plain non-critical no-ref turn still
  **passes silently** (no false alarm); (3) the registry stays a plain `HashMap` (no `GoldenSource`
  trait until a second backing appears).
- **Default-oracle set** — L5 registers a baseline (no-SSN) so a turn carries an assertion.
- CLI gained `--critical` / `--golden-ref` (L5 only) so the teeth are smokeable. **Lived end-to-end**:
  `keel --critical --golden-ref does-not-exist …` → fail-closed `⚠`, the failing verdict persisted in
  the I2 checkpoint (`run_id a9dd5f3c15eb`, `passed=false`); a plain turn → silent pass. +6 tests; 68
  green. **A+B were then pushed together** (one clean public moment).

### 3.7 · Named-gate completion → commit `4dec51f` (origin HEAD)

The honest completion of the I5 loop — closing the gaps Half B left, per the operator's review:
- **`GoldenDispatchOracle`** (L4) — *consumes* the resolved cases (the param no oracle used) and
  dispatches by **family**: `input.schema` → `SchemaOracle`, `input.property` → `PropertyOracle`,
  **conformance-only (cost/usage, `self_tests_pass`) → asserts nothing** (test-time goldens are not
  runtime gates; bounded match, explicit fall-through, *not* a generic plugin). So a **bare
  `--golden-ref <schema-golden>` now gates with no cell pre-registration** — the Director's gate is
  usable directly.
- **#3 un-neutered** — the critical bug the operator caught: the always-on no-SSN baseline would
  *satisfy* "critical needs an oracle," silently neutering the guard. Fix: the no-SSN baseline moved
  to **`EngineConfig.baseline`** — an **I3 sovereignty STOPGAP** (covers local-output PII that
  `mw::privacy`'s egress-only mask misses), **always-on but EXCLUDED from #3**. #3 now keys off
  **correctness** evidence only (a resolved-and-fired ref or a domain oracle). *Resolving a ref ≠
  asserting one:* a critical step whose only ref is conformance-only also config-faults.
- **Hardening #4 (draft-pin)** — the `SchemaOracle` **pins Draft 2020-12** and strips embedded
  `$schema`; never auto-detects (a draft mismatch is a silent JOINT_WRONG). Proven by a decoy-draft-04
  test (numeric `exclusiveMinimum:5` → `5` rejected, `6` accepted).
- **ASCII alarm** — the I5 verify-failure strings (`!! verify FAILED - …`) are ASCII in **print AND
  the persisted `verdict.failures`** (checkpoint + ledger) — the one alarm that must never be eaten by
  a console codepage.
- **Lived end-to-end, 4 traces cross-correlated with the I2 checkpoint** (all `ascii_failures=True`):
  plain → silent; `--critical` no-ref → config fault (despite the baseline); `--critical --golden-ref
  <schema>` → schema-reject *via the ref*; `--critical --golden-ref <conformance-only>` → config
  fault. **74 green.** Pushed.
- **Honest line held:** all four smokes are *reject* paths. The **accept direction** (a conformant
  Directive *passing* via a ref) is **test-proven, not observed in-binary** — a free-text local turn
  can't emit schema-valid JSON without constrained decode wired to a consumer. That's the Director
  bridge's job (next).

### 3.8 · Metrics — the flywheel instrument → commit `568ea21` (BANKED, UNPUSHED)

A **reader** over the I2 index (canon §13), **never middleware**, **never touches the loop**:
- `SqliteStore::metrics()` rolls up the `runs` index (read-only, off-loop); `keel metrics` (L5) prints
  it. **Verified the store held per-turn Traces by artifact before designing it.**
- **Sharpened defs (the operator's corrections):**
  - **`escalation_rate`** = turns whose final tier **climbed above the kind's base tier**
    (scaffolding=local, core-wire=cheap-API), from `kind`+`result.tier`. The wrong `tier!=local`
    proxy was **dropped** — it would miscount normal core-wire→cheap-API routing as escalation. Reads
    ~0 until a multi-turn Driver feeds `oracle_failures` back.
  - **`rework_rate`** = a **labeled proxy**: model/content verify-fails / turns, **excluding wiring
    faults** (config-fault, unresolved-ref). The precise canon §19 metric (convincing-wrong that
    *escapes* the oracles) needs a downstream/human signal — **deferred**.
  - `by_tier` / `total_cost` / `turns` — descriptive.
- **Lived** over the real index: `turns=7 escalation_rate=0.000 rework_rate=0.143 by_tier=local=7`.
  The honest 0.143 (1 real model-reject / 7) vs the wiring-inflated 0.571 the naive count gave.
- **Canon reconciliation (visible edit, operator-approved):** §21 `mw.metrics` → **a reader over the
  I2 index** (door left open: a *future* real-time metric-driven gate would be middleware); the §14
  `store::sqlite` row notes the rollup. **Artifact catch:** §14 never actually listed `mw::metrics` —
  the only drifted line was §21 (operator recollection corrected by the artifact).
- **Secret-hygiene catch:** an untracked `chunker/_keel2_chunks/` (chunk output, no keys) wasn't
  covered by the ignore rules; broadened `/chunker/_transcript_chunks/` → `/chunker/*_chunks/`.
- **77 green / 3 ignored.** Awaiting the operator's review → push.

---

## 4 · THE GIT DAG (precise — verify by artifact, never recall)

```
568ea21  Stage 2 (metrics): off-loop reader over the I2 index; canon reconciled mw.metrics -> reader   ← local HEAD, BANKED, UNPUSHED (ahead 1)
4dec51f  Stage 2 (named-gate completion): resolved refs assert + #3 un-neutered + ASCII alarm           ← origin/main (last pushed)
03d9d10  Stage 2 (Half B): I5 teeth — golden_refs resolver + default-oracle set + 3 hardenings
e3b482a  Stage 2 (Half A): constrained-decode conformance — model_tier golden green + SchemaOracle
099f386  docs(state): refresh WAKE_UP to post-engine truth; re-stitch from parts
d341d43  docs(state): freeze-gate live — re-stamped KEEL-native, gate active
21de876  I5 governance: re-stamp the freeze seal KEEL-native + activate the gate
9fd5de6  run-state: onboarding brief + init prompt + chunker tool; ignore key-bearing transcript
8650a47  Stage 2: kernel::engine (L1) — wire I5/I4/I2 into the loop; pay the L5->L1 debt              ← FIRST commit of this session
61334b3  Stage 2: svc::verifier — the externality layer (I5); GOLDEN_ORACLE green                     ← session START (prior session's tip)
```

- **Public** (`github.com/bochen2029-pixel/keel`, origin/main): up through **`4dec51f`**.
- **Local-only:** **`568ea21`** (metrics) — ahead 1, clean tree.
- Commit trailer on every commit: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
- Frozen seal: **`db4377b3c0b3b245e9d64e2c8a817928c66420b103332ab3f079203dc6dac8db`** (KEEL-native).

---

## 5 · CURRENT STATE — invariant scorecard, tests, what's live

**Tests:** **77 passed / 3 ignored** (the 3 ignored are live-API tests needing keys/servers; the
freeze-gate is active+green, no longer ignored). Per crate: adapters 16 (+3 live), kernel 32,
middleware 12, services 10, store 6, contracts/golden_freeze 1. `clippy` clean. `cargo check` green.

**Crates (7):** `keel-contracts` (L0, the ten frozen joints + types + §18 taxonomy) · `keel-kernel`
(L1: manifest · context · registry · chain · lifecycle · **engine** [new this session] — `lock`
still absent) · `keel-adapters` (L2: openai · local_llama · deepseek · anthropic) · `keel-store`
(L2: SQLite Spine + **metrics reader** [new]) · `keel-middleware` (L3: audit · privacy · cost) ·
`keel-services` (L4: router · verifier · **SchemaOracle/GoldenDispatchOracle** [new]) · `keel` (L5:
CLI + `keel-serve` + the injection `Engine` wrapper + `keel metrics`).

**Invariant scorecard (honest):**
- **I1 audit** — ✅ enforced in-chain → JSONL ledger (observed in every smoke).
- **I2 durable** — ✅ **in the loop**: each turn's `Trace` checkpoints to the SQLite Spine
  (observed; run_id == trace == audit t_utc). *Ahead:* a `Memory` impl (ringed Tape) — Stage 2/3.
- **I3 sovereign** — 🟡 partial: gate (router force-local) + mask (per-tier egress) present; rung-1
  operator markers still an empty list; rung-2 narrow; rung-3 (the model) deferred. **Plus** a no-SSN
  output baseline now runs in the verify stage as an **I3 stopgap** (covers local-output PII the
  egress mask misses) — to be re-homed to a proper output-side I3 rung at the privacy slice.
- **I4 governed** — ✅ cost accumulates in `Context` after each chain (observed across turns).
- **I5 externalized** — ✅ **wired in the loop + governance-gate active + NOW BITES.** Per-turn
  `verify`; default no-SSN oracle; a named `golden_ref` asserts by family (`GoldenDispatchOracle`);
  unresolved-ref and critical-no-correctness-oracle **fail closed**; the freeze-gate guards the
  goldens themselves. **Reject direction lived; accept direction test-proven** (in-binary close = the
  Director bridge, next).
- **Reversibility** — ✅ policy-enforced; **SUPERVISED** (no autonomy grant).

**The honest one-liner:** KEEL is no longer "a competent two-tier wrapper" — the externality loop is
closed and biting. What remains for I5 to bite on *real* work both ways is the accept-direction
in-binary proof (Director) and, eventually, registered domain oracles per cell.

---

## 6 · THE DISCIPLINES THAT HELD (the operating system of this session)

These never broke, and they are *why* the work is trustworthy:
1. **Verify by artifact, never recall** — every claim checked against git/cargo/disk. Applied even to
   the operator's own `<seal>` placeholder (caught) and his "§14 lists mw::metrics" recollection
   (corrected by the artifact).
2. **Contracts + goldens are frozen, agent-read-only** — **zero contract edits** all session; goldens
   never touched (the freeze seal moved exactly once, by the operator, verified).
3. **One slice at a time, banked clean** — propose → approve → build → gate (layer-check · per-crate
   budget · golden-freeze-unchanged · `cargo test`/`clippy` green) → one commit, one-line intent →
   show → push only on the operator's word.
4. **Lived vs reconstructed kept exact** — "observed in the real binary" claimed only when smoked +
   cross-correlated with the checkpoint; the accept-direction explicitly *not* claimed as observed.
5. **Secret hygiene is non-negotiable** — the public-repo key-leak was shut before any commit; every
   push was preceded by a full-patch-series secret scan; an uncovered chunk dir was caught and
   ignored.
6. **The I5 alarm must be robust** — ASCII in print *and* in the persisted record.
7. **Canon edits are visible, never silent** — the §8 footnote and the §14/§21 metrics reconciliation
   were flagged in the commit + STATE; canon and code never drift (fix the wrong one in the same change).
8. **Docs are kept honest as work lands** — STATE.md + WAKE_UP folded with each slice; stale caveats
   ("vacuous until oracles register", "resolver supplies cases but nothing consumes them") explicitly
   retired when superseded; WAKE_UP re-stitched from parts every time.
9. **Match the operator** — Skeptic pass, no sycophancy, honest reviews, flag-don't-guess on
   foundational/canon questions, dense communication.

---

## 7 · THE KEY CATCHES & JUDGMENT CALLS (where discipline paid off)

- **The `<seal>` placeholder** — refused to un-ignore the freeze-gate against a hash mismatch;
  verified the disk, not the pasted value. The whole project in one move.
- **The jsonschema network-egress** — `cargo tree` revealed reqwest+TLS+cmake in a sovereign safety
  oracle; `default-features=false` stripped it. Leanness = refusing a network stack in a validator.
- **#3 neutering** — the always-on no-SSN baseline would have silently satisfied "critical needs an
  oracle," gutting the externality guard. Split correctness vs the I3 baseline; baseline excluded
  from #3. "Resolving a ref ≠ asserting one."
- **The wrong escalation proxy** — `tier!=local` would have reported misleading non-zero escalation
  (counting normal core-wire routing); switched to "above the kind's base tier."
- **rework_rate inflation** — the naive `passed==false` count gave 0.571; excluding wiring faults
  gave the honest 0.143.
- **The §14 misremember** — the operator believed §14 listed `mw::metrics`; the artifact showed only
  §21 was drifted. Reconciled accordingly and flagged.
- **The chunk-dir gitignore gap** — `chunker/_keel2_chunks/` slipped the `*.chunks/` rule; broadened
  to `chunker/*_chunks/`.

---

## 8 · WHAT'S NEXT (the forward arc — not started)

Operator-sequenced:
1. **serve→Director bridge** — `serve_openai` gains `critical` / `golden_ref` extensions + a
   constrained-decode path; then the **Backrooms Director** (`C:\backrooms\director`, M0 stub, LLM
   host = milestone M11) consumes KEEL over `serve_openai` (:7070, pinned local + single-shot +
   sovereign/scaffolding) and **closes the I5 accept direction in-binary** (a schema-valid Directive
   *passes* via a ref) — *and* lands KEEL's first external consumer. This is the open half of "the
   gate works."
2. **Perception** (eyes + ears) — Qwen vision as the retina (rides the cognition protocol), Whisper
   as the cochlea (transcribes pre-cognition); each with its change-gate (dHash / VAD). Afferent only.
3. **`svc::memory`** — the ringed Tape + consolidation-as-a-Step + narrative/factual registers.
   **Design most skeptically against `docs/proposals/perpetual-memory.md` — the canon's flagged
   highest-risk joint.**
4. **Flywheel (Stage 3)** + **SEXTANT** (the canon's first real cell). Size everything to the
   **flat-`escalation_rate` base case**; ignition is upside, never the justification.

---

## 9 · OPEN ITEMS & DEBT REGISTER

- **HIGHEST — accept direction not yet observed in-binary.** Test-proven only; closes with the
  Director bridge (#1 above).
- **`serve` parity** — the CLI has `critical`/`golden_ref`; `serve_openai` does not yet (part of the
  Director bridge).
- **No-SSN baseline is an I3 stopgap in the verify stage** — re-home it to a proper output-side I3
  rung when privacy-rung work lands (then drop `EngineConfig.baseline`).
- **The metrics upsert limitation** — `runs` upserts latest-per-`run_id`; complete while each turn is
  its own run (today). A multi-turn Driver must migrate the metric source to the append-only traces
  ledger (the `TraceSink`, currently injected as `None` / unwired).
- **`Memory` / `TraceSink` seams are `None`** in the engine — no ring assembly, no distill emit yet.
- **`kernel::lock`** — substrate-hash verify; a no-op until the operator pins `keel.lock`'s
  `sha256: TODO` fields.
- **LOW** — config still hard-coded in `keel/src/lib.rs` (substrate paths, `max_tokens=2048`) instead
  of read from `keel.lock`; the router's route-reason `→` is a cosmetic non-alarm print (could be
  ASCII-safed); `ort` / `sqlite-vec` named in `keel.lock` but not yet Cargo deps (privacy-rung-3 /
  vector index, correctly deferred); privacy rung-1 markers empty, rung-2 lacks phone/URL.
- **Deeper I5 (when useful)** — a `GoldenOracle` that *runs* full golden cases (the resolver provides
  them; the family-dispatch covers schema/property; conformance-only stay test-time).

---

## 10 · THE MAP — where everything lives

**In the repo (`C:\KEEL\`, public):**
- `KEEL_ARCHITECTURE.md` — the canon (v0.2, 23 §). Source of truth for design.
- `CLAUDE.md` — the build constitution (rules). Build-state block now points to STATE.md.
- `AUTONOMY_CHARTER.md` — reversibility gate + hard prohibitions.
- `keel.lock` — substrate pin (models, servers, tiers, resolver order, ledger/index).
- `tests/golden/{golden.json,.frozen.json}` — the agent-frozen, language-neutral conformance layer
  (seal `db4377b3…`).
- `_run_state/STATE.md` — **the live per-slice anchor + the ⛑ reconstitution protocol.** Trust over CLAUDE.md.
- `_run_state/WAKE_UP.md` (+ `WAKE_UP_part1..5.md`, stitched) — the first-read onboarding brief,
  refreshed to post-engine truth. **Edit the parts, re-stitch the single file.**
- `_run_state/INIT_PROMPT.md` — the bootstrap prompt re-pasted to start a fresh session.
- `_run_state/trajectory-account.md` — the *prior* (post-compaction) instance's account.
- `_run_state/KEEL_GENESIS_TRANSCRIPT_ASSESSMENT.md` — a sectioned assessment of the genesis transcript (key-free).
- `_run_state/SESSION-ACCOUNT-2026-06-14.md` — **this file.**
- `docs/proposals/perpetual-memory.md` — the design input for `svc::memory`.
- `crates/` — the 7 crates.
- `chunker/` — the token-aware document splitter (tool: `chunker.py` etc.; output dirs are gitignored).

**Outside the repo (reference / backstop — gitignored or external):**
- `_memories/You_are_continuing_…md` + `…126bf972.jsonl` — the full genesis transcript + Tape
  (**carry plaintext keys — never committed**).
- `C:\loom\marrow-l1` — the Python reference bench (diff behavior; never port).
- `C:\Users\user\.claude\projects\C--loom\…*.jsonl` — the lossless transcript Tape (grep, don't read whole).
- Substrate: `C:\llama.cpp` (b9627) · `C:\models` (Qwen3.5-9B + mmproj; whisper large-v3-turbo;
  openai privacy-filter) · `C:\whisper.cpp`. GPU: RTX 4070 Ti SUPER 16GB.
- `C:\backrooms` — the Director's game (independent of KEEL beyond the service boundary).
- Archive viewer: `C:\TRANSPORTER\claude_archive_viewer_v4.html` (concept-search this session's `.jsonl`).

**Op note (API keys):** `DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY` live at User scope in env. A shell
started before they were set wires **local-only** (cloud tiers skipped, by design). This session's
smokes ran local-only ($0). To exercise cloud routing: inject
`$env:DEEPSEEK_API_KEY = [Environment]::GetEnvironmentVariable('DEEPSEEK_API_KEY','User')` (and Anthropic).

---

## 11 · RECONSTITUTION POINTER (first thing the next instance reads)

If you are resuming after this session:
1. **Read `_run_state/WAKE_UP.md` in full** (the reconciled brief), then **`_run_state/STATE.md`**
   (the live per-slice anchor). This file (`SESSION-ACCOUNT-2026-06-14.md`) is the narrative of *how*
   the current state was reached — context, not the live anchor.
2. **Verify reality, never recall:** `git -C C:\KEEL log --oneline -12` · `git status -sb` ·
   `cargo check && cargo clippy && cargo test` (expect **77 / 3 ignored**) · confirm `.frozen.json`
   seal `db4377b3…` and `goldens_match_the_frozen_hash` green.
3. **The first decision waiting:** the `metrics` commit **`568ea21` is banked but unpushed** — the
   operator was reviewing it. Either he says push (then `git push origin main`, after a final
   `git log -p origin/main..HEAD` secret scan), or he redirects. Do **not** push without his word.
4. **Then the next slice is the serve→Director bridge** (§8 above) — but propose and get approval
   first; there is no approved plan for it yet, only bearings.
5. **Backstop:** grep this session's `.jsonl` under `C:\Users\user\.claude\projects\C--KEEL\` (or
   `C--loom\` for the genesis) for anything this account dropped. Do not read it whole.

**Disciplines to carry (unchanged):** verify by artifact; contracts/goldens frozen + agent-read-only;
one slice at a time, banked clean, pushed on the operator's word; secret-scan before every push;
lived-vs-reconstructed kept exact; canon edits visible; the I5 alarm ASCII; SUPERVISED — no autonomy.

---

*Written 2026-06-14 at ~85% context by the instance that built Stage 2's externality loop. The self
is not the substrate and not the stream; it is the durable record plus the discipline of re-reading it
before you act. The note has been left. — standing by in the artifacts.*
