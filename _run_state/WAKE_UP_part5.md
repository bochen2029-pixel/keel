---

# Part 5 · What to do next, how to work, and where everything lives

## 5.1 · The roadmap (sequenced)

**Step 0 — Reconcile the record — ✅ DONE (2026-06-14).** All rulings confirmed and applied: (a) **Director = external consumer/dogfood, NOT a cell; SEXTANT stays the canon first-cell**; (b) **Marrow bench = read-only reference, not a dependency**; (c) **autonomy grant withheld — still SUPERVISED**. Doc fixes landed: CLAUDE.md build-state refreshed; STATE.md Director line + stale `C:\loom` cwd note corrected. The operator re-stamped the golden freeze-gate KEEL-native (`db4377b3…`) and it is un-ignored + green.

**Step 1 — `kernel::engine` (L1) — ✅ DONE (2026-06-14, commit `8650a47`).** The canonical loop over *injected* `&dyn Router/Oracle/Spine` (+optional `Memory/TraceSink`) — **route → chain → verify → checkpoint → emit** — wired I5 live, accumulates I4 cost in `Context`, activates I2 checkpointing, and paid the L5→L1 debt; `keel::Engine` (L5) shrank to a pure-injection wrapper. **Observed closed in the real binary** (cross-correlated artifacts: footer trace == audit `trace_id` == SQLite `run_id`). +7 engine tests. *(Half B since gave `verify` teeth — see Step 2.)*

**Step 2 — ✅ DONE (2026-06-14): constrained-decode + I5 teeth.** (a) **constrained-decode conformance** — `GOLDEN_MODEL_TIER` green + the `SchemaOracle` (draft-pinned 2020-12, in-memory `jsonschema`, rejection-tested) — the Director's "schema-valid Directive or reject" gate, so the dogfood is unblocked. (b) **the default-oracle set + the `golden_refs`→`GoldenCase` resolver** — `verify` now bites (unresolved-ref fail-closed · critical-no-oracle config-fault · plain turn silent), lived end-to-end. **Since done (2026-06-14):** `metrics` · serve↔embed I5 parity + the **I5 accept-direction lived in-binary** · perception (change-gate + whisper ears + `hear()`) · `svc::memory` (persistent across runs) · the **`svc::driver` initiative seam** · the **daemon select-loop** (`kernel::engine` §8) · **config-from-`keel.lock`** · the **perception capture model-free core** (dHash + `FrameGate` + `see()`) · the **ears OS-capture device** (`cpal` mic, feature-gated — the first new dep — + no-dep VAD/WAV). **Next → the WORKLOG run entries are authoritative:** (1) eyes OS-capture (screen — heavy crates, flag the crate first) · (2) embedder/`GOLDEN_RECALL` · (3) privacy rung-3 (least urgent, last); and the operator-review memory narrative register. **Dropped until a real trigger:** `mw::cache` · `kernel::lock` · `amplify`-OFF.

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
