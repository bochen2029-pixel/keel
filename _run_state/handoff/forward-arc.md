# KEEL — Session Forward-Arc Handoff (max resolution)

*Written 2026-06-13 at ~91% context, pre-compaction, by the outgoing instance for the incoming one.
This is the **narrative/forward register** — the causal "how we got here + where we're going." It pairs
with `_run_state/handoff/recent-turns.md` (the verbatim recent tail, reverse-chronological — written next
turn) and `_run_state/STATE.md` (per-slice build state + the ⛑ verify-by-artifact protocol).*
**Trust git + the contracts/goldens (the factual register) over this narrative where they conflict.**

## Read order after compaction
1. `_run_state/STATE.md` ⛑ protocol (verify by artifact, never recall).
2. **This file** (the arc + the next move).
3. `_run_state/handoff/recent-turns.md` (recent verbatim, reverse-chronological).
4. Verify real state: `git -C C:\KEEL log --oneline -10` + `git status`; `cargo check`/`clippy`/`test` from PowerShell.

## Frame (who / what / where)
- **KEEL** = Bo Chen's sovereign genome harness — *"rented cognition, owned self."* Native Rust, L1 personal
  tool (not a product). Repo `C:\KEEL`, public at `github.com/bochen2029-pixel/keel`. Canon = `KEEL_ARCHITECTURE.md` v0.2.
- ⚠ **Session cwd is `C:\loom`, so the auto-loaded `CLAUDE.md` is Marrow-L1's, NOT KEEL's.** The active project is KEEL.
- The **ten contracts** (`crates/keel-contracts`) + **golden cases** (`tests/golden/`) are **frozen, agent read-only**.
- Operator: Bo Chen (bochen2029@gmail.com). Prizes: the Skeptic pass (no sycophancy/manufactured resonance),
  verify-by-artifact, reversibility, one-step-at-a-time gating, contracts-as-the-genome.

## The arc of THIS session (causal, forward)
1. **Resumed post-compaction** (a *prior* session). Reconstituted via STATE.md; verified git/cargo green; wrote a
   ~2000-word essay tying six BC-canon docs (REEL, Pattern Thesis, Dead Geometry, Pattern-Trajectory-Cursor,
   Nested Incompleteness, Returning Loop) to KEEL — central thread: KEEL is the engineering instantiation of the
   harness `H` in the `(W,C,H,O)` personhood spec; "verify by artifact, never recall" = I5 = the load-bearing gap.
2. Operator **manually re-fed recent turns in reverse order** (confirming the artifact-based reconstitution matched
   — zero drift; the two registers cross-validated). Then said **"continue KEEL."**
3. **Built Stage 0 slice by slice**, each a zero-warning / tested / committed / pushed checkpoint, contracts never bent:
   kernel (`manifest · context · registry · chain · lifecycle`+resolver) → middleware (`audit I1 · privacy I3 · cost I4`)
   → adapter `local_llama`.
4. Operator paused to **update llama.cpp**: researched TurboQuant (real, KV-cache quant, NOT upstream — PR closed
   2026-06-03; watch-item). Updated `b8931 → b9627` (CUDA 12.4) **side-by-side** (old at `C:\llama.cpp-b8931-april`).
   Validated text + reasoning_content + thinking-toggle + **vision** on Qwen3.5-9B. Pinned in `keel.lock`.
5. **Three tiers live + cost-validated**: `local_llama` ($0), `deepseek` (cheap-API, real $; fixed to honor
   `Effort.thinking` — v4-pro defaults thinking ON), `anthropic` (frontier = **Claude Opus 4.8**, real $; the
   Messages API is its own protocol = the thin-gateway adapter). Keys in **env** (`DEEPSEEK_API_KEY`,
   `ANTHROPIC_API_KEY`, User-level), never in files (verified absent from the tree).
6. **`serve_openai`** (`keel-serve`, axum) — KEEL consumable **over protocol**; refactored a shared `keel` wiring lib
   (`assemble()`) so the CLI and server share one assembly. **`store::sqlite`** (`keel-store`, bundled) — the index +
   first **`Spine` (I2)** impl (checkpoint/resume). → **Stage 0 spine complete.**
7. **Stage 1 began**: `keel-services::router::DifficultyRouter` — the §9 fusion point — **validated against frozen
   `GOLDEN_ROUTER`** (all 6 cases). Built as a **swappable `Router` policy** (the trait is the seam).
8. Operator sidebar: reviewed **SIRP** (their March router spec). I gave an honest Skeptic-pass review (strong:
   intent-multiplexer framing, three-axis model, Layer-2 semantic-abstraction; weak: the quality-signal rests on a
   non-model score = a JOINT_WRONG/I5 risk; cooling-vs-safety tension; federation premature). Synthesis: the router
   policy seam = where a future `SirpRouter` slots in, *only* once its quality signal rests on a non-model oracle (I5).
9. Operator sidebar: **memory techniques** brainstorm → captured `docs/proposals/perpetual-memory.md` (non-binding).
10. **NOW**: practicing the memory technique itself — this forward-arc handoff — at ~91% context, pre-compaction.

## Where we are RIGHT NOW
- **7 crates**, ~22 commits, all pushed; tree clean. Latest commit `1cb6d8c` (the memory proposal) on `main`.
- Stage 0 spine **complete** (self-sufficient substrate · three-tier economy · invariant chain · ledger + SQLite
  index · embedded + protocol) **+ Stage 1's router** (golden-validated).
- **Live processes**: `llama-server` on `:8080` (cold-started by `keel` earlier), `keel-serve` on `:7070`. Both keep
  running; `Stop-Process` to stop. (Note: a running `keel-serve` holds `keel-serve.exe` — kill it before rebuilding that bin.)
- rustc **1.96.0**; build from PowerShell/MSVC (not git-bash). LF→CRLF git warnings are cosmetic.

## THE NEXT MOVE — operator-confirmed resume point
Before the memory tangent, the operator said this is exactly where they wanted to go next:
**wire the minimal `engine` so `keel "…"` (and `keel-serve`) SELF-DRIVE** instead of taking `--tier`.
Concretely:
- In the shared `keel::assemble` (or a new `engine`), build a **registry with ALL available tiers** (local always;
  cheap-API/frontier when their env keys exist), not just one.
- Per turn: construct the `Step` (kind from the request — default scaffolding, or a flag for core-wire), call
  `DifficultyRouter::route(step, ctx)` → `Decision.tier`, `registry.get(tier)`, run through the existing chain.
- Print the routing reason (e.g., "scaffolding → local"). Honor `BLOCK` (I4). Keep `--tier` as a manual override.
- This makes `keel "read the config"` auto-land local, `keel "weigh these tradeoffs" --kind core-wire` auto-land
  cheap-API, and a twice-oracle-failed step escalate to Opus. (Escalation needs the loop to feed `oracle_failures`/
  `tier_history` back — that's the verifier + a real loop; a first cut can route single-shot and add escalation with `svc::verifier`.)
Then: `amplify` (best-of-N, **ships OFF** behind the §23 falsifier) · perception (eyes+ears: Qwen retina + whisper cochlea).

## Open threads / pending operator decisions
- **Memory skill**: operator is weighing whether I draft a `/checkpoint`+`/rehydrate` **skill now** (helps immediately)
  vs proposal-only until `svc::memory` (Stage 2). And whether the **dialogic-handshake-as-pre-compaction-self-interview**
  (the genuinely novel idea) is worth prototyping vs over-engineering. (This handoff partly *is* the technique in action.)
- **SIRP harvest backlog**: latency axis (needs a `Step` contract field — operator-frozen) · Layer-2 semantic-abstraction
  preprocessor (future I3 "text retina") · cooling = the Stage-3 flywheel.
- **Deferred Stage-0**: kernel `lock` (substrate-hash verify — no-op until the operator pins `sha256: TODO`); `engine`
  = the Stage-1 wiring above.

## Disciplines (do NOT drop)
- Contracts + goldens **frozen** (agent read-only). Fix code, never a golden.
- **Verify by artifact, never recall.** Layer rule `contracts ← kernel ← {adapters,middleware} ← services ← apps`.
- Five invariants + reversibility gate every call. **Keys live in env, never a file/commit.** No sovereign/vector egress.
- No `git reset --hard`/`clean -fd`/`restore` on uncommitted work; no force-push; `rm -rf` only under `.\.keelstate\`.
- Don't mutate the global Rust toolchain without asking (DAVE/TERMINAL share it). Build from PowerShell, not git-bash.
- End each slice: layer-check → clippy/test green → one commit (one-line intent) → push. Commit trailer:
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
