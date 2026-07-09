# C:\KEEL — The Full Picture (consolidated)

> Produced 2026-06-17 by fanning out 6 parallel Explore sub-agents across the codebase and consolidating their reports. Survey was READ-ONLY; no KEEL files were changed during reconnaissance.

## TL;DR (read this first)

**KEEL** is Bo Chen's single-operator, **sovereign, native-Rust AI-harness "genome"** — *not* an agent or a loop, but a *"trust-and-cost economy with a tiny immortal core."* Its thesis: **rented cognition, owned self.** One frozen kernel loop + ten frozen contracts, written once in Rust, scaled purely by toggling modules (never rewriting the loop) — from a minimal embeddable AI bundle for the operator's own apps/games, up to a personal harness, aspirationally an org-scale orchestrator. *"The .NET of my AI apps."*

- **State:** 7-crate Cargo workspace (Rust 2021, v0.2.0, MIT), edition-strict 6-layer architecture (L0→L5).
- **Build status (committed):** GREEN. `6ac319d` is HEAD. ~129–130 tests pass / 6 ignored (live tests). `cargo clippy` clean. Golden freeze-seal `db4377b3` green.
- **QC:** A 48-agent audit (`docs/AUDIT-2026-06-15.md`) returned **GREEN — zero critical/high**, 5 mediums actioned in `6ac319d`.
- **⚠️ IMPORTANT CAVEAT — the working tree does NOT match HEAD and does NOT compile.** There's an uncommitted, incomplete revert on top of `6ac319d` (deletes `embed.rs` + `conformance-coverage.md`, drops `pub mod recall`, rewrites run-state docs backward) but leaves `recall.rs` and `memory.rs`'s Ring-4 references dangling. **Trust the committed HEAD as the real state, not the working tree.** Also: a stray junk file `nul` should be removed. (More in §6 below.)
- **Single open blocker:** ISSUE-10 — the Qwen3-Embedding-0.6B GGUF is absent from `C:\models`, blocking live embed/recall benchmarks (C1/C2/A3-live/Ring-4-live). Routed around per "pivot-when-stuck."
- **Next major effort:** D1 — re-home NightScribe (C#/.NET) on KEEL as the first real "cell," the controlled cross-language proof.

---

## 1. What KEEL is, and why

**Mission (from `README.md` + `KEEL_ARCHITECTURE.md` §0):** a *persistent, sovereign self that perceives (eyes and ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model to punch above its weight, and grounds every critical output in an assertion no model authored.* The model that thinks is interchangeable (rented, per-token); KEEL is the persistent self (owned, portable across providers and years).

**Why it exists:** the operator kept rebuilding the same AI substrate by hand across projects (a local vision-LLM, Whisper ears, privacy filtering, cheap-vs-frontier routing). KEEL builds that common core **once, from first principles, scale-invariant and reusable — and never rewrites it.** You can't embed OpenCode/Claude Code inside a game; KEEL is a *substrate you embed*, not an agent you talk to.

**The linchpin invariant is I5** — "ground truth lives outside the model." Every critical output carries ≥1 assertion no model authored. The project proved this thesis on its own history: a lossy compaction dropped things, the lossless transcript held them, and the one place an instance reasoned from memory instead of the artifact, it **confabulated** — and the artifact caught it. That's why the entire record/reconstitution apparatus exists.

**Three destinies from one frozen core** (differ only by which modules toggle, never by rewriting the loop):
1. An embeddable, reusable AI bundle for the operator's apps/games.
2. A sovereign personal harness (local-first, grounded by a non-model oracle).
3. Aspirationally, the orchestration kernel of an intelligence running an org.

**Genome/cell metaphor:** the frozen core is the *genome*; specializations are *cells* (genome + periphery, never edits to the core). *If a cell ever forces a kernel/contract edit, the boundary is wrong — fix KEEL first.*

---

## 2. Architecture & ecosystem

**Language/ecosystem:** Rust, edition 2021, rustc 1.96.0, target `x86_64-pc-windows-msvc`. A native C/C++ port is a *contemplated future* — explicitly "the most reversible decision" because the frozen goldens are language-neutral. Polyglot *above* the core: Rust embeds in-process; Python/C# consume over HTTP/MCP/OpenAI-egress.

**Workspace (`Cargo.toml`)** — 7 members, one per layer:

| Crate | Layer | Role |
|---|---|---|
| `keel-contracts` | L0 | The frozen joints: core types + the **ten contracts** (the genome's load-bearing surface). Imports nothing. |
| `keel-kernel` | L1 | The spine: manifest, context, registry, chain, lifecycle+resolver, engine, lock. Imports only L0. |
| `keel-adapters` | L2 | Tier adapters under OpenAI Chat Completions (local_llama; deepseek/anthropic/whisper) + mic/screen capture organs. |
| `keel-store` | L2 | The SQLite index (bundled, behind a Store seam): backs the Spine (I2 checkpoint/resume) + metrics. |
| `keel-middleware` | L3 | Invariant middleware: I1 audit · I3 privacy · I4 cost. |
| `keel-services` | L4 | Services: route · amplify · verify · memory · perception · driver · distill · trace_sink · recall. |
| `keel` | L5 | Apps: the `keel` CLI + `keel-serve` (OpenAI-compatible server on `:7070`), sharing one wiring lib. |

**Layer-import rule is law** (enforced as a bug, not style): `contracts ← kernel ← {adapters, middleware, store} ← services ← apps`. Middleware may never import a service.

**The ten contracts** (`keel-contracts/src/traits.rs`): `ModelTier`, `ToolHost`, `Next`, `Middleware`, `Router`, `Oracle`, `Memory`, `Spine`, `Driver`, `PerceptionSource`.

**The five invariants + reversibility gate:**
- **I1 audit** — every call logged (even blocked ones); redactions audited labels-never-values.
- **I2 durable** — file Tape (system of record) + disposable SQLite index (derived, rebuildable); engine checkpoints each turn.
- **I3 sovereign** — deterministic PII mask: egress rung (cloud tiers) + output rung (every tier, so model-authored PII never lands in the Tape).
- **I4 cost** — pre-call hard-stop gate; engine folds cost into Context each turn; daemon re-seeds per-tick budget.
- **I5 externalized** — the whole point. `verify()` runs every turn; unresolved `golden_ref` → fail-closed; `critical` step with no oracle → config-fault.
- **Reversibility gate** — never train on a secret (the `TraceSink` scrubs before write).

**The engine loop (canon §8):** `assemble → route → chain → verify → checkpoint → emit`. `Engine::run()` = one turn; `tick()` = driver select-loop step; `run_until_idle()` = bounded burst. Tier ladder `["local","cheap-API","frontier"]`; escalation after oracle failures; down-ladder fallback when a tier is unplugged.

**Substrate (resolved at runtime, pinned in `keel.lock`, never embedded):** `C:\llama.cpp` (llama-server b9627), `C:\models` (Qwen3.5-9B-Q5_K_M + mmproj-F16 vision; whisper large-v3-turbo; OpenAI privacy-filter; **NOT the Qwen3-Embedding GGUF — ISSUE-10**), `C:\whisper.cpp`. GPU: RTX 4070 Ti SUPER 16GB. Cloud keys `DEEPSEEK_API_KEY`/`ANTHROPIC_API_KEY` in env (User scope), never committed. Three tiers: local $0, cheap-API DeepSeek ~$0.435/$0.003625/$0.87 per 1M, frontier Opus $5/$0.5/$25.

---

## 3. Domain glossary (where each lives in code)

- **Tape / Spine / ledger** — the append-only lossless JSONL factual register of every `Trace`. "The Tape is the Spine." Impl: `FileMemory` in `keel-services/src/memory.rs`; path const `TAPE_PATH = ".keelstate/tape/tape.jsonl"` in `keel/src/lib.rs`. SQLite index is derived/disposable.
- **Memory (ringed)** — contract `Memory`; impl `FileMemory`. Five rings in `AssembledContext`/`TokenBudget` (`types.rs`): Ring-0 soul · Ring-1 exemplars · Ring-2 working turns · Ring-3 compressed narrative · Ring-4 retrieved (semantic recall). Two registers: **narrative** (lossy, voice-shaped) vs **factual** (lossless Tape) — never conflated.
- **Embedding/recall** — `Embed` trait + `Fingerprint` + `cosine()`/`recall_top_k()`/`should_rebuild()` in `keel-services/src/recall.rs`; HTTP embedder organ was `keel-adapters/src/embed.rs::Embedder` (⚠️ deleted in the uncommitted tree). Brute-force cosine (ISSUE-1 decided: **no `sqlite-vec`**); format-committing (fingerprint mismatch ⇒ rebuild-from-ledger).
- **Agent/Driver** — initiative seam `poll() → Option<Step>`. Three default drivers in `keel-services/src/driver.rs`: `UserTurnDriver` (FIFO), `HeartbeatDriver` (perpetual tick), `WatchDriver` (poll-on-change). Select-loop in `keel-kernel/src/engine.rs`; perpetual wrapper in `keel/src/main.rs::run_daemon`.
- **QC / conformance** — operator-frozen, language-neutral goldens at `tests/golden/golden.json` + `.frozen.json` seal. The freeze-gate test `keel-contracts/tests/golden_freeze.rs` re-hashes and fails the build on any golden change. 21 cases across 6 families: ROUTER, ORACLE, PERCEPTION, MODEL_TIER, RECALL, PRIVACY. `JOINT_WRONG` = code + its own tests agree but a frozen golden disagrees (the most dangerous failure).
- **Perception (eyes/ears)** — `PerceptionSource`/`Percept`/`Modality`/`SampleSpec`; change-gate + retinas in `keel-services/src/perception.rs` (dHash frames, energy-based VAD — silence is free); organs `Whisper`/`Microphone`(gated)/`ScreenCapture`(gated) in `keel-adapters`.
- **Verifiers (I5)** — `keel-services/src/verifier.rs`: `PropertyOracle`, `GoldenOracle` (the JOINT_WRONG detector), `SourceOracle` (trace-to-canon Truth Gate), `SchemaOracle` (JSON Schema Draft 2020-12 pinned, in-memory), `GoldenDispatchOracle`.
- **Tiers/routing** — `ModelTier` + `DifficultyRouter` (`keel-services/src/router.rs`): cheapest tier clearing the trust bar; raw perception/sovereign/PHI → forced local; over-budget → BLOCK; repeated oracle failure escalates up the ladder.
- **TraceSink/distill (flywheel)** — `FileTraceSink` (scrubs secrets before write) → `distill-export` (chat-format training pairs for out-of-band LoRA). KEEL §16-refuses to be its own trainer.
- **Spine (I2)** — `SqliteStore` (`keel-store/src/lib.rs`): checkpoint/resume + off-loop `metrics()` rollup (turns, escalation_rate, rework_rate, by_tier, total_cost).

---

## 4. Build, test, CI, ops

- **Build:** plain cargo workspace, resolver 2, no profiles, no `[workspace.dependencies]`. No justfile/Makefile/build.rs/.cargo/rust-toolchain.toml/rustfmt.toml/clippy.toml/deny.toml. Default-run `keel`; second bin `keel-serve`.
- **Build-from-PS rule:** must build from a **native MSVC PowerShell** shell, NOT git-bash (git-bash mangles `$LASTEXITCODE` and false-fails cargo). Do not mutate the global toolchain.
- **Tests:** ~135 inline `#[cfg(test)]` tests across all 7 crates + 1 integration freeze-gate. Model-free stub-driven (live model calls are `#[ignore]`). The engine loop (`engine.rs`, 22 tests), perception (15), memory (10), privacy (10), verifier (8) are the densest. Fixtures shared at workspace root `tests/golden/`.
- **CI: ⚠️ NONE committed.** No `.github/`, no `.gitlab-ci.yml`, etc. The docs repeatedly reference a "Stage-0 CI gate" that enforces the freeze-seal — but **today it only fires if a human runs `cargo test --workspace` locally.** This is a real gap.
- **Lints:** no clippy/deny config. The only lint-like discipline is ASCII-only checks on operator-facing strings (route reasons, I5 alarms) so they render on any codepage.
- **Scripts:** `tools/keel-autoloop.ps1` — the external respawn supervisor that self-perpetuates autonomous build sessions (stops on `.keelstate/DONE`/`STALLED`, a stall, or session cap; meant to be replaced by the in-process `keel daemon`). `chunker/*.py` — token-aware document chunker (aux, not build).
- **Daemon:** `keel daemon [--max-ticks N] [--interval MS] [--watch PATH] [--prompt …] [--consolidate-every N] [--kind core-wire] [--sovereign]`. Bounded by default (`--max-ticks 1`); `--max-ticks 0` or bare `--watch` → perpetual. Wires HeartbeatDriver (+ optional WatchDriver). Each tick = distinct `{base}-{n}` trace_id. Per-turn stderr report (tier, cost, verdict, answer). Config from `keel.lock` + CLI flags; no separate daemon TOML. Runtime state under `.keelstate/`.
- **"--tier guard" (commit `6ac319d`):** NOT a routing concept — it's the I3/I5 safety guard in `keel/src/main.rs:96-110`. The manual `--tier` override skips the router (I3 force-local gate) AND the engine (I5 verifier), so it now **refuses** `--sovereign` with a non-local tier, and refuses `--critical`/`--golden-ref` outright, instead of silently voiding them.

---

## 5. Project status & roadmap (from committed `_run_state/`)

ROADMAP is structured `DONE → NOW→DONE plan → DONE definition → perpetual-polish → ISSUES`, with an explicit autonomy contract (§0): reconstitute → pick next unblocked `[ ]` slice → build against frozen contracts → gate → commit+push → mark `[x]` → loop until ~90% context, exit for the supervisor to respawn.

**Stage 0 (spine): complete** — three-tier economy through one invariant chain, file Tape + SQLite Spine (I2), self-resolving substrate, consumable embedded (CLI) AND over protocol (`keel-serve`).

**Stage 1 (amplification & senses): largely landed** — `DifficultyRouter` (golden-green), self-driving `kernel::engine`, perception change-gates, Whisper ears, `xcap` screen eyes → native Qwen vision, the Driver seam + daemon select-loop.

**Stage 2 (correctness & memory): largely complete** — `svc::verifier` (I5, golden-green) in the loop; ringed `svc::memory` (Tape + Rings 0/2/3/4 + narrative register + consolidate/cold-eyes); metrics as off-loop SQLite reader; golden freeze-gate active. **Still ahead: privacy rung-3** (the ONNX OpenAI Privacy Filter — A5, the operator's stated last item).

**Stage 3 (flywheel): started** — `FileTraceSink` (scrubbed distill corpus) + `distill-export` landed; out-of-band LoRA training deferred.

**Slice status (committed HEAD):**
- `[x] A1` perception retinas · `[x] A2` driver daemon (+`--consolidate-every`) · `[x] A4` I3 output rung (resolves ISSUE-9) · `[x] A6.1` Ring-3 narrative register · `[~] A3` embedder + Ring-4 recall (first pass + Ring-4 DONE; **live-served measurement blocked by ISSUE-10**) · `[~] A6.2` consolidate/cold-eyes (LIVED; cold-eyes **caught real narrative drift**; remaining: cold-eyes as periodic Step, policy swap, Ring-1/Ring-4 hardening) · `[G] A5` privacy rung-3 (operator's last item, ISSUE-2).
- `[x] B2` FileTraceSink · `[x] B4` distill-export · `[?] B1` amplify (built OFF; §23 falsifier decides) · `[?] B3` flywheel metric (prelim 0.000 over 18 turns; needs trainer running).
- `[?] C1` reranker (blocked ISSUE-10) · `[?] C2` embedder vs MiniLM floor (blocked ISSUE-10) · `[?] C3` privacy model (after A5) · `[?] C4` rework_rate<10% (**prelim PASS 5.6%**) · `[?] C5` economic KEEL vs cheap-API-everything (**prelim KEEL-favorable, ~78% cheaper**).
- `[ ] D1` re-home NightScribe on KEEL (**scoped**, commit `e127ada`) · `[ ] D2` SEXTANT on KEEL (the canon's designated first cell) · `[ ] D3` ToolHost (MCP) adapter.
- `[x] E1` conformance-coverage map (ADR #5) · `[ ] E2` the DONE review.

**ISSUES register (committed):** ISSUE-1 (embedder shape — RESOLVED: brute-force cosine, no sqlite-vec) · ISSUE-2 (A5 privacy, operator's LAST) · ISSUE-3 (A6 narrative = highest-risk seam) · ISSUE-4 (B1 amplify benchmark) · ISSUE-5 (B3/C4 trend data) · ISSUE-6 (operator-only: substrate-hash lock dormant until sha256 pinned) · ISSUE-7 (cache middleware deferred) · ISSUE-8 (Windows shell-capture hang — tooling, not a KEEL defect; workaround proven) · ISSUE-9 (A4 privacy policy — RESOLVED: mask-all-output) · **ISSUE-10 (Qwen3-Embedding GGUF absent from `C:\models` — blocks C1/C2/A3-live/Ring-4-live; the one open code-path blocker).**

**Canon versioning:** `KEEL_ARCHITECTURE.md` is the canon, **v0.2.0**, 23 sections. References like "canon 10.2" / "canon 11" are **section citations** (§10.2 = the cold-eyes I5 anti-pattern; §11 = Memory; §8 = engine loop; §5.1 = privacy mask; §16 = distill refusal; §22 = anti-patterns; §23 = falsifiers), NOT version numbers.

**Authority hierarchy:** (1) `KEEL_ARCHITECTURE.md` (canon — design ground truth) → (2) `_run_state/STATE.md` + git (live slice state) → (3) `docs/AUDIT-2026-06-15.md` (QC verdict) → (4) `docs/PROJECT-STATE.md` (orientation, explicitly disclaimed) → (5) `docs/RUN-2026-06-15.md` (run report) → (6) `docs/proposals/*` (non-binding RFCs). `CLAUDE.md` is the build constitution/rules but its build-state block is **known stale** (audit M4/M5) — trust STATE+git for state, CLAUDE.md for rules only.

---

## 6. ⚠️ The working-tree anomaly (operationally critical)

`git status` on `main` (up to date with `origin/main`) shows **uncommitted changes that do NOT represent a deliberate new direction** — they look like an in-progress, incomplete revert that was left mid-flight:

- **`D crates/keel-adapters/src/embed.rs`** — the A3 embedder organ (added in `9d27a56`) deleted.
- **`D docs/conformance-coverage.md`** — the E1 conformance map (added in `52c4dd2`) deleted.
- **`M crates/keel-adapters/src/lib.rs`** — drops `pub mod embed;` + `pub use embed::Embedder;`.
- **`M crates/keel-services/src/lib.rs`** — drops `pub mod recall;` + `pub use recall::{cosine, recall_top_k, should_rebuild, Fingerprint};`.
- **`M _run_state/{ROADMAP,STATE,WAKE_UP_part4,WORKLOG}.md`** — rewrites the narrative *backward*: A3→`[G]` gated (re-proposing the rejected `sqlite-vec`), A6.2/B3/C1–C5/D1/E1 back to open, ISSUE-10 removed, invariant scorecard reverted to a less-advanced state, freeze-gate back to "dormant."
- **`?? nul`** — a 54-byte junk file (Windows reserved name) containing a stray shell error; should be deleted, not committed.

**Why this matters — the tree does not compile as-is.** `recall.rs` is still on disk (still tracked), and `memory.rs:37` still does `use crate::recall::{cosine, Embed, Fingerprint};` with ~16 live references including 3 tests — but `recall` is no longer a declared module of `keel-services` and `Embedder` no longer exists in `keel-adapters`. This is a half-revert: either `recall.rs` + the Ring-4 code/tests should also be removed, or the `pub mod recall;`/re-exports should be restored.

**The reflog shows no commits after `6ac319d` and no stash.** The committed HEAD `6ac319d` is the canonical current state (GREEN, 129–130/6, seal green, QC GREEN, embedder present, E1 present, ISSUE-10 recorded). Anyone resuming work should reconcile or discard the uncommitted tree deliberately — exactly the "verify by artifact" discipline the project is built around. This is the single most important thing to know about the repo right now.

---

## 7. Recent commit through-line

The arc `807b62f` (genesis-arc) → `6ac319d` (QC fix) is **one sustained push** under a self-perpetuating perpetual-run mandate: take KEEL from "a sovereign router + substrate" to a **self-driving, self-remembering genome whose externality loop is wired in the running binary.** Themes, in order:

1. **Perception organs** (canon §12): config-from-`keel.lock` → dHash/FrameGate retina → whisper ears → cpal mic → xcap screen → `listen()`/`see_screen()` wrappers (A1).
2. **Self-driving engine + daemon** (§7/§8): `svc::driver` → daemon select-loop → A2 `keel daemon` → `--consolidate-every` self-consolidation.
3. **Perpetual memory** (§11, densest cluster): `svc::memory` → afferent change-gate → A6.1 Ring-3 narrative + self-interview consolidate → A3 embedder + cosine recall + GOLDEN_RECALL → A3 Ring-4 wired → A6.2 `keel consolidate` (LIVED) → `keel cold-eyes` (LIVED, caught real drift).
4. **Flywheel** (§8/§16): B2 `FileTraceSink` (scrubbed corpus) → B4 `svc::distill` + `keel distill-export`.
5. **Privacy** (§5.1): A4 I3 output rung masks response PII on every tier.
6. **Autonomy plumbing:** AUTOSTART + `keel-autoloop.ps1` respawn supervisor + reconstitution wiring + the standing operator directive baked to survive compaction.
7. **Conformance + validation docs:** E1 conformance map → "whole flywheel LIVED" + prelim C4/C5 → ISSUE-10 → D1 cell scoping.
8. **QC capstone:** 48-agent audit (`f9c031a`, GREEN) → fix commit (`6ac319d`, actioned M1–M3 + doc-drift + poison-policy).

Every feature slice lands with a heavy "self-narrating run-state" doc tail (ROADMAP/STATE/WORKLOG/WAKE_UP updates) — consistently heavier than the code itself.

---

## 8. What's next (per committed narrative)

The operator-directive's end-condition is **MET**: "the bulk + low-hanging fruit are done."

1. **D1 cell build** — re-home NightScribe (C#/.NET) on KEEL over `serve_openai`, the cross-language controlled proof. Boundary is clean (eyes/ears/perception-gate/route/oracle/memory/constrained-decode come FROM KEEL; only meeting periphery is cell-written). Best begun with fresh full context.
2. **A5 privacy rung-3** (operator's explicit last/least-urgent item; ISSUE-2) — ONNX OpenAI Privacy Filter via `ort`, behind `GOLDEN_PRIVACY`.
3. **C1/C2 recall-uplift benchmarks** — blocked on ISSUE-10 (operator downloads Qwen3-Embedding GGUF to `C:\models`).
4. **E2 the DONE review** — all phases done/decided, ISSUES resolved-or-accepted, scorecard all-green, flip `keel.lock stage:` to done, write `.keelstate/DONE`.
5. Then **perpetual-polish** (§4): code-review, raise coverage, re-check falsifiers, reconcile doc drift, completeness-critic, harden+simplify — it does not stop.

---

**Bottom line:** KEEL is a validated, self-perpetuating, audit-GREEN sovereign Rust harness — spine, three-tier economy, I5 verifier (both directions lived), perception (eyes+ears), persistent ringed memory (Tape + Ring-3 narrative + Ring-4 recall + cold-eyes), self-driving daemon with self-consolidation, secret-scrubbed flywheel feed + distill-export, and a port-readiness conformance map. The committed HEAD `6ac319d` is the real, clean, gate-green state. The uncommitted working tree is an incomplete revert that breaks compilation and should be reconciled before any further work. The one open code-path blocker is ISSUE-10 (a substrate-provisioning step, routed around); the next major effort is the D1 cell build.
