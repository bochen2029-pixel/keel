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
- **I5 externalized** — ✅ **wired in the loop + governance-gate active.** `kernel::engine` calls `Oracle::verify` every turn (the `Verifier` is injected as a composite `Oracle`); `oracle_failures`/`tier_history` feed back so the escalation ladder fires across turns; the **freeze-gate** guards the goldens themselves. **Honest caveat:** the default oracle registry is **empty**, so a plain chat turn verifies **vacuously** — the *mechanism* is live, but it gains teeth only when a cell registers oracles / the golden-registry resolver lands (the next slice's other half).
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
- ✅ **RESOLVED — I5 in the loop.** `kernel::engine` calls `verify`; `oracle_failures`/`tier_history` feed back. *(Was HIGH; fixed by the engine slice `8650a47`. Caveat: vacuous until oracles register — see the OPEN item below.)*
- ✅ **RESOLVED — I4 cost accumulates.** The engine folds `result.cost → Context.cost` after each chain. *(Was HIGH.)*
- ✅ **RESOLVED — L5→L1 engine debt.** The loop lives in `kernel::engine` (L1) over injected joints; `keel::Engine` (L5) is a pure-injection wrapper. *(Was MEDIUM.)*
- ✅ **RESOLVED — golden freeze-gate ACTIVE.** Operator re-stamped KEEL-native (`db4377b3…`); `goldens_match_the_frozen_hash` un-ignored + green; the last Marrow tie severed. *(Was MEDIUM/dormant.)*
- ✅ **RESOLVED — doc drift.** CLAUDE.md build-state refreshed; STATE.md Director line corrected (Director = consumer, not a cell) + stale `C:\loom` cwd note fixed. *(Was MEDIUM.)*
- **OPEN (the next slice) — per-turn `verify` is toothless.** The engine runs `verify` but the default registry is empty and there is no `golden_refs`→`GoldenCase` resolver, so it asserts nothing on a real turn. Fix: the default-oracle set + the golden-registry, paired with constrained-decode.
- **LOW — config hard-coded** in `keel/src/lib.rs` (substrate paths, `max_tokens=2048`) instead of read from `keel.lock` (declared-but-not-driving). Still open.
- **LOW — `ort` / `sqlite-vec` named in `keel.lock` but not yet Cargo deps** (the privacy-rung-3 and vector-index organs are unbuildable until added — correct for their Stage-2 deferral).
- **LOW — privacy completeness:** rung-1 markers empty; rung-2 lacks phone/URL; redaction findings unaudited.
- **LATER — no in-turn memory:** no `Memory` impl; the loop checkpoints (I2 ✅) but does not yet hydrate ring context. (Stage 2.)

*(Continued in Part 5 — the roadmap, the immediate next slice, the session protocol, and the full map of where everything lives.)*
