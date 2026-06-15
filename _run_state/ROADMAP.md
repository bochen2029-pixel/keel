# KEEL — ROADMAP (the single forward plan: NOW → DONE → perpetual polish)

> **What this is.** The durable, machine-followable blueprint from the current state to KEEL's
> completion, then a self-improvement loop that never "stops." **Any session reads this (after
> WAKE_UP + STATE) and knows the exact next action — with zero operator re-explanation.** It is the
> forward complement to `STATE.md` (the live you-are-here cursor) and `WORKLOG.md` (the chronological
> trail). **Trust `git` + `cargo` over this doc for *state*; trust this doc for the *plan + completion
> criteria*.** Author: the autonomous build loop, 2026-06-14; maintained every slice.

---

## 0 · THE AUTONOMY CONTRACT (how a session uses this file — the loop)

You are an autonomous KEEL build session operating under the operator's standing grant (WORKLOG
"AUTONOMOUS RUN" entries + `AUTONOMY_CHARTER.md`). Run this loop without asking:

1. **Reconstitute.** Read `WAKE_UP.md` → `STATE.md` → **this ROADMAP** → the latest `WORKLOG.md`
   entries. **Verify by artifact, never recall:** `git -C C:\KEEL log --oneline -10`, `git status`,
   `cargo test` from PowerShell, the freeze seal. Keep the "lived vs reconstructed" line honest.
2. **Pick the next slice.** The first `[ ]` (todo) entry below that is **unblocked** (its deps are
   `[x]`) and **not `[G]`/`[!]`** (operator-gated / blocked). Respect dependency order.
3. **Build it** against the **frozen** contracts (never bend a joint/golden to ease an impl — if a
   slice seems to need that, the boundary is wrong: flag it in ISSUES and skip). Follow the slice
   protocol (CLAUDE.md §Session-protocol).
4. **Gate, bank, push.** `cargo test && cargo clippy` zero-warning → **secret-scan the staged diff**
   → one commit (one-line intent + trailer `Co-Authored-By: Claude Opus 4.8 (1M context)
   <noreply@anthropic.com>`) → **push**. Feature-gated deps: also build/clippy `--features <f>`.
5. **Record.** Mark the slice `[x]` here, update `STATE.md` (the cursor + banner), append `WORKLOG.md`
   (decision + commit result). Keep WAKE_UP §0/§5.1 current when the plan shifts. Doc edits in the
   **same commit** as the slice. (WAKE_UP.md is CRLF = the LF parts re-stitched — edit the parts +
   `WAKE_UP_part1..5.md`, then regenerate WAKE_UP.md; STATE/WORKLOG/ROADMAP are LF.)
6. **Loop** to step 2 until **~90% context**, then **checkpoint and exit** (the supervisor respawns a
   fresh session that reconstitutes and continues). Never continue through a forced compaction.
7. **On compaction** (mid-session): rehydrate via the ⛑ protocol (`STATE.md` §⛑) — re-read +
   re-verify before any further action.
8. **Decide-and-document on EVERYTHING** (use the §5 operator-calibration: `GENESIS-ARC.md` §5 +
   the `keel-operator-calibration` memory) **EXCEPT the operator-only acts**, which you NEVER
   self-authorize: **edit a frozen contract · ratify/change/re-stamp a golden or the seal · mutate
   the global Rust toolchain.** Those → the ISSUES register; **route around them, never block the
   rest.** A non-operator-only blocker: resolve it, or decide + press forward. **Do not stop the run.**

**Operator-touch is required exactly once** (the env wiring that turns this loop on) — see
`tools/keel-autoloop.ps1` + the SessionStart/PreCompact hooks. After that one-time setup the loop is
self-perpetuating; the operator reviews the ISSUES queue whenever he chooses, never to keep it moving.

**Why this works (not more context — a tighter loop):** the self is the durable record + the
discipline of re-reading it (WAKE_UP §3). This file is that record for the *build*; the gate + I5 are
the guardrails that make unsupervised iteration safe; the supervisor is the temporary external
`Driver` — until KEEL is complete enough to host its own loop (then the scaffold dissolves).

## Status legend
`[ ]` todo · `[x]` done · `[~]` in progress · `[!]` blocked (see ISSUES) · `[G]` operator-gated
(needs an operator-only act / review) · `[?]` unknown (needs a falsifier/benchmark to decide ON/OFF)

---

## 1 · DONE (the foundation — do not redo; verify by `git log`)
Stage 0 (spine: kernel · invariant middleware I1/I3/I4 · 3-tier economy local/DeepSeek/Opus · file
ledger + SQLite Spine · CLI + `serve_openai` · substrate resolver) **✅**. Stage 1 (`DifficultyRouter`
· `kernel::engine` L1 §8 loop · `svc::verifier` I5 + freeze-gate · perception change-gate + whisper
ears + `hear()` · `svc::driver` initiative + the daemon `select`/`tick`/`run_until_idle`) **✅**.
Stage 2 partial (`metrics` · `svc::memory` minimal persistent Tape · config-from-`keel.lock` · ears
**cpal mic** capture + eyes **xcap screen** capture, both feature-gated → the native Qwen vision /
Whisper) **✅**. **112 tests green / 5 ignored; seal `db4377b3`; public.** (Latest commits: see `git`.) Phase A: A1 ✅.

---

## 2 · THE PLAN (NOW → DONE), dependency-ordered

### Phase A — Stage 2 completion
- `[x] A1` · **`listen()` + `see_screen()` retina wrappers** (svc::perception) — DONE 2026-06-14.
  `listen()` (`#[cfg(feature="mic")]`) = mic(cpal)→`voiced_ms` VAD-gate (silence short-circuits)
  →`resample_to_16k` (no-dep linear)→`write_wav`→whisper→Audio `Percept`; `see_screen()`
  (`#[cfg(feature="screen")]`) = screen(xcap)→`FrameGate`→`see()`→Image `Percept`. Factored the
  hardware-free `listen_from_samples` so the silence-gate is unit-tested without a mic; live paths
  `#[ignore]`'d. `mic`/`screen` features forwarded keel-adapters→keel-services. **No new dep.** +2
  unit + 2 feature-gated live. Gate: 112/5 green, clippy clean (default + both features).
- `[ ] A2` · **the Driver daemon (L5)** — the continuously-running select-loop over `run_until_idle`:
  `keel daemon [--max-ticks N | --watch] [--interval ms]` polls the wired drivers → runs each emitted
  `Step` through the engine → idles. **Done =** a bounded live daemon runs N ticks end-to-end (lived),
  the §8 loop self-drives; the perpetual sleep-loop form documented but the bound is the default.
  **Deps:** none (the loop logic exists). **No new dep.** *(This is the in-KEEL twin of the build
  supervisor — the dogfood of "KEEL is the loop.")*
- `[ ] A4` · **re-home the no-SSN baseline → an I3 output rung** (`mw::privacy` output-side check) so
  the engine's `EngineConfig.baseline` STOPGAP retires. **Done =** local-turn output PII is masked by
  an I3 rung, not the I5 baseline; the baseline slot is dropped. **No new dep.**
- `[G] A3` · **embedder + `GOLDEN_RECALL`** — **format-committing (ADR #13) → ISSUE-1 operator
  design-review FIRST.** Proposed shape: embed adapter = HTTP to llama-server `/v1/embeddings`
  (Qwen3-Embedding-0.6B, model at `C:\models`) reusing the `openai` mapping; `sqlite-vec` vector index
  in `keel-store`; the index **fingerprint** `(id,dim)` with **rebuild-from-ledger on mismatch**;
  reranker ships **OFF**/identity (Qwen3-Reranker-0.6B). **Done =** `GOLDEN_RECALL` green + the
  fingerprint guard + sovereign-local (vectors never egress, I3). **New dep:** `sqlite-vec`.
- `[G] A5` · **privacy rung-3** — OpenAI Privacy Filter via `ort`/ONNX (in-process, NOT GGUF), behind
  `GOLDEN_PRIVACY`; additive recall, **never the guarantee** (rungs 1-2 carry it). **Operator: least
  urgent / LAST.** **New dep:** `ort` (heavy native). → ISSUE-2.
- `[G] A6` · **memory narrative register + consolidation** — model-authored narrative register +
  `consolidate()` generation + Ring-1/Ring-4. **The canon's highest-risk seam → ISSUE-3 operator-review**
  (design vs `docs/proposals/perpetual-memory.md`). The factual register (Tape) already exists.

### Phase B — Stage 3 (the flywheel; size to the base case, ignition is upside)
- `[ ] B2` · **`TraceSink` file impl** — passed verdicts → an append-only distill corpus
  (`.keelstate/traces`), **secrets scrubbed before feedstock** (reversibility gate — no secret
  fossilized into a LoRA). **Done =** the engine's emit-on-pass writes scrubbed `VerifiedTrace`s.
  **No new dep.** **Deps:** none.
- `[?] B1` · **`svc::amplify` (best-of-N + verifier-select)** — build the structure **clamped OFF**
  (n=1). The §23 falsifier: does verified best-of-N beat single-pass on a fixed benchmark? → ISSUE-4
  (run the benchmark; decide ON/OFF). **No new dep** (uses local tier + the verifier).
- `[?] B3` · **flywheel metric** — `escalation_rate` trend over runs (needs A2 daemon producing
  multi-turn data); §23: flat after N cycles → flywheel doesn't compound. → ISSUE-5 (measure).
- `[ ] B4` · **`svc::distill` (out-of-band)** — KEEL only **emits + stores** the verified-trace corpus;
  the LoRA training is external (Unsloth Studio, an operator step, §16-refused from the core). **Done =**
  the corpus is distill-ready + the hand-off documented.

### Phase C — the §23 falsifiers (check + DECIDE each; a decision is the deliverable)
- `[?] C1` reranker vs identity on `GOLDEN_RECALL` → keep OFF or turn ON. (after A3)
- `[?] C2` embedder vs the MiniLM floor → keep floor or upgrade. (after A3)
- `[?] C3` privacy model vs deterministic-only on `GOLDEN_PRIVACY`. (after A5)
- `[?] C4` `rework_rate` < 10% with oracles on. (needs A2 data)
- `[?] C5` economic: KEEL overhead vs cheap-API-single-pass-for-everything. (needs cost data)
- *(Each falsifier trip says "revise, don't extend" — blast radius one adapter. Record the decision +
  rationale in WORKLOG; flip the relevant default in `keel.lock` if warranted [config, not a pin].)*

### Phase D — the first real cell (the §17/§21 proof that the genome is at the right altitude)
- `[ ] D1` · **(controlled experiment) re-home NightClerk or NightScribe on KEEL** — known shape, clean
  boundary signal. NightScribe (`C:\ClaudeCode\photo2deck\labs\nightscribe`) is the eyes+ears reference
  (Qwen-vision screenshots + Whisper, timestamp-fused). **Done =** its eyes/ears/memory/route come from
  KEEL **unchanged**; only domain periphery written. (Consumes KEEL over `serve_openai` or `embed`.)
- `[ ] D2` · **SEXTANT on KEEL** (the canon first cell) — done = Conductor (`engine`) / Router /
  Gate (Truth Gate `Oracle`, `INSUFFICIENT_SOURCE`→human) / Canon (factual `Memory`) / State (`Store`)
  / ToolHost (Gmail MCP) / vision retina — **all from KEEL unchanged**; only job-domain periphery.
  **If a cell forces a kernel/contract edit → KEEL's boundary is wrong: FIX KEEL FIRST** (a §23 trip).
- `[ ] D3` · **`ToolHost` (MCP) adapter** — a §3 protocol bet, unbuilt; **pulled by D2** (SEXTANT's
  Gmail MCP). Build when the cell needs it. **New dep:** an MCP client crate (vet at the time).
- *(The Backrooms Director at `C:\backrooms` is the parallel dogfood **consumer** over `serve_openai`
  — NOT a cell; it can start anytime and does not block D1/D2.)*

### Phase E — completion gates
- `[ ] E1` · **C++-port-readiness** — confirm the goldens are a complete language-neutral conformance
  layer (a future C/C++ port re-passes them, ADR #5). Doc + a coverage check (every joint/invariant has
  a golden or a documented gap).
- `[ ] E2` · **the DONE review** — all phases done/decided, ISSUES resolved-or-accepted, the §4.2
  invariant scorecard all-green; write the completion account; flip `keel.lock` `stage:` to `stage3`/done.

---

## 3 · DONE definition
KEEL is **complete** when **all** hold: Stage 0–3 functionally done (amplify/reranker/privacy-model/
embedder each **ON or OFF per its falsifier — decided, not skipped**) · the first cell (D2, or at least
D1) is built on KEEL **with zero kernel/contract edits** · every Phase-C falsifier is checked-and-decided
· the operator-only ISSUES are resolved or explicitly accepted · E1 + E2 pass. **Then the loop does not
stop — it enters perpetual-polish mode (§4).** Write `.keelstate/DONE` only when E2 passes (the supervisor
reads it to wind down to polish cadence, not to halt).

## 4 · Perpetual-polish mode (post-DONE; the self-improvement loop, until quota/power)
When §2 is exhausted, shift to continuous improvement: (1) run `/code-review` on the tree → fix
findings; (2) raise test/golden coverage where thin; (3) re-check the §23 falsifiers with fresh data;
(4) reconcile any doc drift; (5) a **completeness-critic** pass — "what is unverified, missing, or
stale?" → new polish slices; (6) harden + simplify (smaller, never larger). Each polish item is a
gated/banked/pushed slice like any other. Honest about diminishing returns — bounded by the gate + I5,
not a promise of literal perfection. *(Also slot a completeness-critic pass every ~N build slices, not
just post-DONE, to catch drift early.)*

## 5 · ISSUES / BLOCKERS register (the operator-only + unknown queue — route AROUND; never block the rest)
- **ISSUE-1 [operator design-review]** — A3 embedder is format-committing (ADR #13). A session must
  PROPOSE the fingerprint / `sqlite-vec` / embed-adapter design (append it here) for the operator's OK
  **before** committing the index format. Until OK'd → skip A3; do A1/A2/A4/B2/etc. Models +
  HF links: Qwen3-Embedding-0.6B, Qwen3-Reranker-0.6B (operator's note; at `C:\models`).
- **ISSUE-2 [operator · least-urgent/LAST]** — A5 privacy rung-3 needs `ort` (heavy native) + is the
  operator's explicit last item. Defer to the end; `openai/privacy-filter` model at `C:\models`.
- **ISSUE-3 [operator-review]** — A6 memory narrative register = the highest-risk seam-cut; reserved
  for the operator. Propose the design here first (vs `perpetual-memory.md`).
- **ISSUE-4 [unknown/benchmark]** — B1 amplify ON/OFF needs a verified-best-of-N-vs-single-pass
  benchmark on a fixed set. Build OFF; run it; decide + record.
- **ISSUE-5 [unknown/data]** — B3/C4 `escalation_rate` + `rework_rate` trends need the A2 daemon
  producing multi-turn data over time (and ideally the flywheel running).
- **ISSUE-6 [operator-only]** — `kernel::lock` (substrate-hash verify) is a no-op until the operator
  pins the `sha256: TODO` fields in `keel.lock`. Build the verify-logic; it stays dormant until pinned.
- **ISSUE-7 [deferred — no trigger yet]** — `mw::cache` (cache-prefix discipline) waits until
  cache-hit-rate matters (scale + the daemon running). §22 anti-pattern to build it speculatively.
- *(Append new issues as discovered, each: `ISSUE-N [type] — description · what unblocks it`. If the
  loop STALLS — only `[G]`/`[!]`/`[?]` slices remain and none can advance — write `.keelstate/STALLED`
  with the reason so the supervisor stops respawning, and the operator resolves the queue on next look.)*

## 6 · The cursor
`STATE.md` is the live you-are-here (per-slice banner + the ⛑ protocol). **This ROADMAP is the map;
STATE is the pin.** A session: reconstitute → find the next actionable `[ ]` here → go.
