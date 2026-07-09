# Sub-agent 2 — `_run_state/` deep read (verbatim)

> Original task: read ROADMAP/STATE/WORKLOG/WAKE_UP and summarize current state, roadmap, narrative, ISSUES, canon refs. READ-ONLY.

---

# KEEL — Consolidated Current-State / Roadmap / Narrative Summary

## 0. How to read this (a critical caveat the files themselves insist on)

KEEL's founding discipline is **"verify by artifact, never recall."** That applies here too. The committed repo HEAD `6ac319d` is the real state. There are also **uncommitted working-tree changes** that look like an in-progress revert of several landed slices (deleting `crates/keel-adapters/src/embed.rs`, removing `docs/conformance-coverage.md`, and reverting ROADMAP/STATE/WORKLOG/WAKE_UP_part4 toward older versions). These uncommitted edits are NOT canonical — the reflog shows no further commits after `6ac319d` and there is no stash. I flag this explicitly below because the task asks for "current state" and the working tree currently disagrees with the committed narrative.

---

## 1. Files in `C:\KEEL\_run_state\` (18 items)

Top level (16 files):
- `AUTOSTART.md` — the standing autonomous directive the supervisor injects; the loop to execute.
- `AUTOSTART_SETUP.md` — the one-time operator wiring (SessionStart/PreCompact hooks + unattended permission policy).
- `GENESIS-ARC.md` (55 KB) — the durable full genesis arc (synthesis of the 3 pre-history transcripts).
- `INIT_PROMPT.md` — the paste-to-start bootstrap prompt for a fresh session.
- `KEEL_GENESIS_TRANSCRIPT_ASSESSMENT.md` — a pre-digested, sectioned assessment of the genesis transcript.
- `OPERATOR_DIRECTIVE.md` — Bo Chen's 2026-06-15 verbatim standing directive (real-work / never-ask / TTL+pivot / common-sense), baked to survive compaction.
- `ROADMAP.md` — the durable NOW→DONE blueprint + autonomy contract (§0).
- `SESSION-ACCOUNT-2026-06-14.md` (33 KB) — narrative account of the 06-14 session.
- `STATE.md` — the live "you-are-here" cursor + the ⛑ reconstitution protocol.
- `WAKE_UP.md` (64 KB) — the stitched full onboarding brief.
- `WAKE_UP_part1.md` … `WAKE_UP_part5.md` — the brief in 5 LF parts (WAKE_UP.md is the CRLF re-stitch).
- `WORKLOG.md` — append-only chronological trail.
- `trajectory-account.md` — the post-compaction instance's first-person account (the fork/confabulation story).

Subdirectory `handoff/` (2 files, older, pre-engine landing):
- `forward-arc.md` — the causal arc + operator-confirmed next move (predates the engine/verifier).
- `recent-turns.md` — reverse-chronological recency tail (T-0…T-6).

Plus the `docs/` folder at repo root (added during the QC pass, not in `_run_state` but load-bearing for current state): `AUDIT-2026-06-15.md`, `PROJECT-STATE.md`, `RUN-2026-06-15.md`, `conformance-coverage.md` (deleted in the uncommitted working tree).

---

## 2. What KEEL is (the soul, in one line)

*"Rented cognition, owned self."* KEEL is Bo Chen's personal, sovereign, reusable AI-harness **genome** — one frozen kernel loop + ten frozen contracts, written once in Rust, scaled purely by toggling modules (never rewriting the loop), from a game's minimal embeddable AI bundle up to an org-scale orchestrator. *"The .NET of my AI apps."* Specializations are **cells** = genome + periphery, never edits to the core. The model that thinks is interchangeable/rented; KEEL is the persistent self that perceives, remembers, routes, verifies, and continues.

The linchpin invariant is **I5** ("ground truth lives outside the model" — every critical output carries an assertion no model authored). The project proved this thesis on its own history: a lossy compaction summary dropped things, the lossless transcript held them, and the one place an instance reasoned from memory instead of the artifact, it **confabulated** — and the artifact caught it. The docs repeatedly stress this is *why* the whole record/reconstitution apparatus exists.

---

## 3. ROADMAP — milestones, phases, status

ROADMAP.md is structured as `DONE (§1) → THE PLAN NOW→DONE (§2) → DONE definition (§3) → perpetual-polish (§4) → ISSUES register (§5)`. It runs an explicit **autonomy contract (§0)**: reconstitute → pick next unblocked `[ ]` slice → build against frozen contracts → gate (cargo test+clippy, secret-scan) → commit+push → mark `[x]`, update STATE/WORKLOG → loop until ~90% context, then exit for the supervisor to respawn. Status legend: `[ ]` todo, `[x]` done, `[~]` in progress, `[!]` blocked, `[G]` operator-gated, `[?]` unknown-needs-falsifier.

### Phase A — Stage 2 completion
- `[x] A1` — `listen()` + `see_screen()` retina wrappers (perception; capture→gate→organ→Percept, behind `mic`/`screen` features). DONE 2026-06-14.
- `[x] A2` — the Driver daemon (L5): `keel daemon [--max-ticks N] [--interval MS] [--watch PATH] [--prompt …]`, bounded by default; later gained `--consolidate-every N` self-consolidation. DONE 2026-06-15.
- `[x] A4` — no-SSN baseline → I3 output rung. `mw::privacy` scrubs PII from response on EVERY tier (mask-all-output decided); no-SSN I5 stopgap retired. DONE 2026-06-15. (Resolved ISSUE-9.)
- `[~] A3` — embedder + `GOLDEN_RECALL`. **First pass + Ring-4 DONE** (embedder organ, brute-force cosine recall — **no `sqlite-vec`**, ISSUE-1 decided; `GOLDEN_RECALL` fingerprint case green; Ring-4 semantic recall wired into `FileMemory::assemble` via the `Embed` seam, opt-in). **Live-served measurement BLOCKED by ISSUE-10** (the Qwen embedding GGUF is absent from `C:\models`).
- `[~] A6` — memory Narrative register + consolidation. **A6.1 DONE** (Ring-3 narrative register separate from lossless factual Tape; `assemble` layers Ring-0→Ring-3→Ring-2; `consolidate()` self-interview prompt). **A6.2 partly done**: `keel consolidate` (closes the loop, sovereign→local, LIVED — authored a real 589-char narrative at $0), `keel cold-eyes` (the I5 capstone — diffs narrative vs Tape; LIVED and **caught real narrative drift**), daemon `--consolidate-every`. **A6.2 remaining**: cold-eyes as a periodic Step, daemon auto-trigger hardening, swappable consolidation policy, Ring-1/Ring-4 hardening.
- `[G] A5` — privacy rung-3 (OpenAI Privacy Filter via `ort`/ONNX, behind `GOLDEN_PRIVACY`). Operator's explicit **last/least-urgent** item. → ISSUE-2.

### Phase B — Stage 3 (the flywheel)
- `[x] B2` — `TraceSink` file impl (`FileTraceSink`): passed `VerifiedTrace` → append-only scrubbed distill corpus (`.keelstate/traces/corpus.jsonl`), secrets/PII scrubbed first via the same I3 `Redactor`. DONE 2026-06-15.
- `[x] B4` — `svc::distill` out-of-band: corpus → chat-format JSONL; `keel distill-export`. DONE 2026-06-15.
- `[?] B1` — `svc::amplify` (best-of-N) — built **OFF** (n=1); §23 falsifier decides ON/OFF. → ISSUE-4.
- `[?] B3` — flywheel metric (`escalation_rate` trend). First-pass: 0.000 over 18 live turns (base case; ignition is upside). Trend-down needs the out-of-band trainer running. → ISSUE-5.

### Phase C — the §23 falsifiers (a decision is the deliverable)
- `[?] C1` reranker vs identity on `GOLDEN_RECALL` (after A3). Blocked on ISSUE-10 (live embed).
- `[?] C2` embedder vs the MiniLM floor (after A3). Blocked on ISSUE-10.
- `[?] C3` privacy model vs deterministic-only on `GOLDEN_PRIVACY` (after A5).
- `[?] C4` `rework_rate` < 10% with oracles on. **Preliminary PASS** (5.6% on 18 live turns).
- `[?] C5` economic: KEEL overhead vs cheap-API-everything. **Preliminary KEEL-favorable** (17/18 routed to free local, ~78% cheaper).

### Phase D — the first real cell
- `[ ] D1` — re-home NightClerk or NightScribe on KEEL (controlled experiment). **Scoped** (commit `e127ada`): NightScribe (C#/.NET) independently hand-rebuilt KEEL's exact pieces; boundary is clean (eyes/ears/perception-gate/route/oracle/memory/constrained-decode come FROM KEEL; only meeting periphery cell-written). The cell BUILD is the next major effort, best begun with fresh full context.
- `[ ] D2` — SEXTANT on KEEL (**the canon's designated first cell**): Conductor/Router/Gate/Canon/State/ToolHost/vision all from KEEL unchanged. *If a cell forces a kernel/contract edit → KEEL's boundary is wrong, fix KEEL first.*
- `[ ] D3` — `ToolHost` (MCP) adapter (pulled by D2's Gmail MCP).

### Phase E — completion gates
- `[x] E1` — C++-port-readiness / conformance coverage map (`docs/conformance-coverage.md`, ADR #5). DONE 2026-06-15. *(Note: this doc is DELETED in the uncommitted working tree — see caveat.)*
- `[ ] E2` — the DONE review: all phases done/decided, ISSUES resolved-or-accepted, §4.2 scorecard all-green; flip `keel.lock stage:` to stage3/done; write `.keelstate/DONE`.

### DONE definition
Complete when: Stage 0–3 functionally done (amplify/reranker/privacy-model/embedder each ON or OFF **per its falsifier — decided, not skipped**) · the first cell (D2, or at least D1) built on KEEL with zero kernel/contract edits · every Phase-C falsifier checked-and-decided · operator-only ISSUES resolved or accepted · E1+E2 pass. Then it does not stop — it enters perpetual-polish (§4): code-review, raise coverage, re-check falsifiers, reconcile doc drift, completeness-critic pass, harden+simplify.

---

## 4. STATE — the current snapshot

The STATE.md top banner is the live cursor. Key points (as last committed):

- **Latest committed HEAD:** `6ac319d` ("fix(qc): address confirmed audit findings…"). **Tree has uncommitted modifications** (see caveat below).
- **Tests:** `cargo test --workspace` = **129 passed / 6 ignored** (6 are live tests needing real endpoint/key/mic/screen). `cargo clippy` clean. Golden freeze seal `db4377b3` **green and active** (`goldens_match_the_frozen_hash`).
- **Crate count:** **7 crates** (the docs are emphatic: `trajectory-account.md`'s "8 crates" is a recall-vs-artifact slip — trust git: `keel-contracts · keel-kernel · keel-middleware · keel-adapters · keel-store · keel-services · keel`).
- **Public** at `github.com/bochen2029-pixel/keel`.
- **Toolchain:** rustc **1.96.0**; build from a native MSVC **PowerShell** shell, NOT git-bash (git-bash mangles `$LASTEXITCODE` and surfaces cargo's stderr as a false exit-1). Do not mutate the global toolchain.
- **Substrate (resolved, local):** `C:\llama.cpp` (b9627), `C:\models` (Qwen3.5-9B-Q5_K_M + `mmproj-F16`; whisper large-v3-turbo; the OpenAI privacy filter; **but NOT the Qwen3-Embedding GGUF — see ISSUE-10**), `C:\whisper.cpp`. GPU: RTX 4070 Ti SUPER 16GB.
- **Keys:** `DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY` at User scope in env; never in files. A shell started before they were set wires local-only (by design).
- **The ⛑ RECONSTITUTION PROTOCOL** (read first after any compaction): read WAKE_UP.md in full → ROADMAP.md + AUTOSTART.md → STATE.md → handoff/forward-arc.md → handoff/recent-turns.md → the canon `KEEL_ARCHITECTURE.md` + constitution `CLAUDE.md` → verify real state by artifact (git log/status, cargo check on keel-contracts, confirm seal `db4377b3…`). Trust files over summary.

### Invariant scorecard (Part 4's honest accounting; some since fixed)
- **I1 audit** — ✅ enforced in-chain (fires even on blocked calls); redactions now I1-audited (the §5.1 gap was closed 06-14).
- **I2 durable** — 🟡→improved: Spine impl exists; the engine now checkpoints each turn; FileMemory persists the Tape across runs.
- **I3 sovereign** — 🟡→improved: gate (router force-local) + per-tier egress mask + the A4 output rung (mask-all-output). Still: rung-1 operator markers are an empty list; rung-2 lacks phone/URL; rung-3 deferred (A5).
- **I4 cost** — 🟡→improved: hard-stop gate enforced; the engine now folds cost into `Context` each turn; the daemon now re-seeds per-tick budget (M3 fix).
- **I5 externalized** — ✅ in the loop: `kernel::engine` calls `verify` every turn; both directions (reject + accept) lived in-binary; `--tier` override now refuses `--sovereign`/`--critical`/`--golden-ref` instead of silently voiding the gate (M1 fix).

### The reconciled drift rulings (Part 4 §4.3 — operate by these)
1. **The Director / `C:\backrooms`** = a first external **dogfood consumer** over `serve_openai` (:7070), NOT a cell. The canon's first real cell remains **SEXTANT**. (STATE.md's "first cell = Backrooms Director" line is the drifted line; the careful read in trajectory-account.md wins.)
2. **`CLAUDE.md` build-state is STALE** (says "Next: Stage 0, nothing above L0" and "goldens PROPOSED" — both false). Trust STATE+git for state; use CLAUDE.md only for rules. (The QC pass began reconciling this — M4.)
3. **Marrow-L1** (`C:\loom\marrow-l1`) = read-only reference diff-bench, never a runtime/build dependency.
4. **Autonomy** = now actively self-perpetuating (the autoloop mechanism landed), but the *unattended grant* is the operator's standing directive (2026-06-15); operator-only acts (frozen contracts/goldens/seal/toolchain) stay non-self-authorized.
5. **The freeze-gate seal** = operator re-stamped KEEL-native `db4377b3…` (2026-06-14); never self-stamp.

---

## 5. WORKLOG — condensed recent chronology (most recent first)

WORKLOG is append-only, newest at bottom. The most recent entries (2026-06-14 → 2026-06-15), in order:

1. **06-14 reconciliation & I3 honesty** — read WAKE_UP/STATE/canon; essence/intent capture landed (WAKE_UP §3.5, README, CLAUDE.md, canon §1); closed the §5.1 contract violation where `PrivacyMiddleware` discarded redaction findings (now emits `AuditEvent{code:"REDACTION",…}` labels-never-values). Gate 79/3. Commit `e47df22`.
2. **Autonomous run authorized** — operator granted autonomy (cloud spend OK ~$20/$25; halt at 90% context). Full backup taken. Genesis-arc captured (56 subagents over ~894k tokens → `GENESIS-ARC.md` + the `keel-operator-calibration` memory).
3. **Autonomous genome build** — serve↔embed I5 parity (the `keel` crate's first tests); I5 ACCEPT direction lived in-binary over `serve_openai` (`{"path":"/etc/hosts"}` → `verdict_passed:true`, $0); perception change-gate (`GOLDEN_PERCEPTION` green); `svc::memory` Tape + persistent memory proven in-binary across two separate `keel` processes (process 1 "remember: 42" → process 2 answered "42"); whisper ears; `hear()` retina; route-reason ASCII. Gate 92/5. **Stopping point at ~86% context.**
4. **Resumed: the Driver seam** — `svc::driver` (UserTurn/Heartbeat/Watch, the §23 seam); the daemon select-loop (`kernel::engine` §8 `select().poll()`, `tick`/`run_until_idle`, distinct `{base}-{n}` trace_ids, cost accumulating in shared ctx); config-from-`keel.lock` (retired hardcoded launch consts); perception capture model-free core (real dHash + `FrameGate` + `see()`); **ears OS-capture** (`cpal` mic — first new dep, feature-gated); operator correction: **perception is first-class genome, not "heavy periphery"** (anti-drift item 11); **eyes OS-capture** (`xcap` screen → native Qwen vision, feature-gated); **the autonomy mechanism** (ROADMAP + AUTOSTART + `tools/keel-autoloop.ps1` + AUTOSTART_SETUP — the operator's "#1 ask: never do another handoff").
5. **Perpetual run (A1, A2, A6.1, B2, B4, A4, E1, A3, A3 Ring-4, A6.2)** — see the ROADMAP statuses above. Highlights: A2 daemon self-drove 2 ticks by artifact; the A2 live-verification **hang incident → ISSUE-8** (shell-capture, not a daemon defect) and the pivot; A3 decided brute-force cosine (no `sqlite-vec`); A4 mask-all-output decided (resolves ISSUE-9); the **whole flywheel LIVED on real data** (3 turns: local "Paris", cheap-API "…" $0.0004, local "Red"; metrics turns=18 rework=0.056; 4 training pairs exported, corpus secret-scan clean; **C4/C5 prelim PASS**); **D1 scoped** (NightScribe on KEEL).
6. **06-15 overnight (operator asleep, directive baked)** — `OPERATOR_DIRECTIVE.md` created; `keel cold-eyes` (I5 capstone for memory) — **LIVED and caught real narrative drift** (the consolidated narrative over-reached vs the Tape, claiming a "retrieval verification" arc step not supported by the Tape); Phase-C first-pass status recorded (B3 0.000, C1/C2 deferred on ISSUE-10); **QC pass**: 48-agent audit → `docs/AUDIT-2026-06-15.md` + `docs/RUN-2026-06-15.md` + `docs/PROJECT-STATE.md`; verdict GREEN (zero critical/high; 17 confirmed findings). The confirmed mediums were fixed in commit `6ac319d` (the current HEAD): M1 `--tier` override now refuses `--sovereign`/`--critical`/`--golden-ref`; M2 maintenance turns excluded from the lossless Tape; M3 daemon re-seeds per-tick budget; doc-drift + poison-policy unified. Gate **130/6** (per WORKLOG's QC entry) / **129/6** (per PROJECT-STATE.md at `04a6acd`) — the `6ac319d` fix commit added the +1 test for the M2 Tape-exclusion.

The WORKLOG's final committed lines declare: *"the directive's end-condition is MET: the bulk + low-hanging fruit are done. NEXT for the perpetual loop: the D1 cell build (NightScribe on KEEL — scoped), then A5/C1/C2/E2 — the larger/heavier tier."*

---

## 6. WAKE_UP files — context for resuming work

WAKE_UP.md is the full pre-digested onboarding brief, authored 2026-06-13 by a max-context synthesis instance and refreshed 2026-06-14. It exists in 5 parts:
- **Part 1** — TL;DR + the 11 anti-drift rules + who the operator is + the soul in three sizes + the full intent/spirit (§3.5).
- **Part 2** — what KEEL IS in depth and *why it is built this way*: the L0–L5 stack, the 10 joints, the 5 invariants + reversibility gate, the engine loop, the router, the externality layer (I5), memory (5 rings), perception (afferent-only), the substrate (resolve-don't-embed), and the deliberate ADRs you must not "improve."
- **Part 3** — the trajectory: genesis (began 04:07 as Marrow-L1/"Loom"; the "wait!!!" pivot ~2 hours in; the triangulation test proving 4–9 independent rediscoveries of the same skeleton), the Stage 0→1 build, the grounding cast (DAVE/TERMINAL, TARS/REEL/The Box/SEXTANT, photo2deck/NightScribe/NightClerk, ASTRA-7), the SIRP + memory sidebars, and the meta-arc of compactions/forks/the confabulation (the source of the prior confusion).
- **Part 4** — **where everything IS now** (ground truth + reconciled drift): build state, the invariant scorecard (honest), the reconciled rulings on the contested items, and the known-gaps/debt register.
- **Part 5** — what to do next, the session protocol, the hard prohibitions, the full file map (where everything lives, incl. the chunker), the anti-patterns (the ways KEEL dies, canon §22), and the recap.

### Key context / open items / decisions from the WAKE_UP brief
- **The 11 anti-drift rules** (Part 1 §1): contracts+goldens frozen/agent-read-only (fix code never golden); verify by artifact; you are in `C:\KEEL` not Marrow-L1; CLAUDE.md build-state is stale; the 5 invariants + reversibility gate hold every call; the layer rule is law; protocol-first; genome = INTERSECTION never union; the reversibility gate prohibitions; one slice at a time banked clean; **perception is first-class, not heavy periphery** (item 11 — the operator's correction).
- **The operator (Bo Chen)** prizes: the Skeptic pass (no sycophancy, override him where wrong), verify-by-artifact, reversibility + one-step gating, contracts-as-genome, calibrated dense honest communication.
- **The reconciled drift (Part 4 §4.3)** — see STATE §4 above. The load-bearing rulings: Director = dogfood consumer NOT a cell; SEXTANT is the first cell; CLAUDE.md build-state stale; Marrow = read-only reference; autonomy active but operator-only acts stay non-self-authorized.
- **Known gaps/debt register (Part 4 §4.4)** — most have since been closed: I5 not in the loop (closed by `kernel::engine`); I4 cost not accumulated mid-run (closed); L5→L1 engine debt (paid); golden freeze-gate dormant (closed — operator re-stamped `db4377b3`, un-ignored). Still open in prose: config hard-coded (closed by config-from-`keel.lock`); `ort`/`sqlite-vec` named in `keel.lock` but not Cargo deps (correct for their Stage-2 deferral); privacy completeness (rung-1 markers empty, rung-2 lacks phone/URL); no in-turn memory (closed by `svc::memory`).

---

## 7. Explicit ISSUE / blocker IDs (from ROADMAP §5 + docs)

- **ISSUE-1 [operator design-review]** — A3 embedder is format-committing (ADR #13). **RESOLVED 2026-06-15** (decided, never ask): brute-force cosine, **NO `sqlite-vec`**; fingerprint `(embedder_id, dim)` + rebuild-from-ledger on mismatch. (Note: the uncommitted working tree reverts this resolution back to "propose-first.")
- **ISSUE-2 [operator · least-urgent/LAST]** — A5 privacy rung-3 needs `ort` (heavy native); operator's explicit last item. `openai/privacy-filter` model at `C:\models`.
- **ISSUE-3 [operator-review]** — A6 memory narrative register = highest-risk seam. (Lifted for the safe model-free structural cut A6.1; the riskier model-dependent generation proceeded carefully.)
- **ISSUE-4 [unknown/benchmark]** — B1 amplify ON/OFF needs a verified-best-of-N-vs-single-pass benchmark on a fixed set. Build OFF; run it; decide.
- **ISSUE-5 [unknown/data]** — B3/C4 `escalation_rate` + `rework_rate` trends need the A2 daemon producing multi-turn data over time (+ the flywheel running).
- **ISSUE-6 [operator-only]** — `kernel::lock` (substrate-hash verify) is a no-op until the operator pins the `sha256: TODO` fields in `keel.lock`. Build the logic; it stays dormant.
- **ISSUE-7 [deferred — no trigger]** — `mw::cache` (cache-prefix discipline) waits until cache-hit-rate matters (§22 anti-pattern to build speculatively).
- **ISSUE-8 [deferred — tooling, NOT a KEEL defect]** — capturing a live `keel`/`keel daemon`/`keel-serve` run's stdout/stderr **in the same shell** can hang on Windows cold-start (detached llama-server inherits keel's std-handle pipe → `… 2>&1 | Out-String` blocks on an EOF that never comes). The daemon/CLI exits fine. **Workaround (proven live 06-15):** `Start-Process -RedirectStandardOutput <file> -PassThru` + `$p.WaitForExit(ms)` → `$p.Kill($true)` (file redirect, NOT `| Out-String`). The real lifecycle-detach root-fix stays nice-to-have. (Note: the uncommitted working tree reverts ROADMAP back to the pre-workaround wording.)
- **ISSUE-9 [operator policy — privacy] — RESOLVED 2026-06-15** — A4's I3 output rung policy: chose **mask-all-output** (state-hygiene default) per the "decide, never ask" directive. Built A4.
- **ISSUE-10 [operator — missing model]** — the **Qwen3-Embedding-0.6B GGUF is absent from `C:\models`** (only `qwen3-reranker-0.6b-q8_0.gguf` is there); discovered 2026-06-15 attempting the C1/C2 recall benchmark. Blocks C1, C2, A3-live, Ring-4-live. The embed organ + recall + fingerprint golden are built/tested regardless. (Note: the uncommitted working tree **removes ISSUE-10 from ROADMAP entirely** — this is a regression; ISSUE-10 is real and current.)

### Other ID conventions referenced (not in the ISSUES register)
- **I1–I5** — the five invariants (audit / spine-durable / sovereign-filtered / cost-governed / externalized). I5 is "the whole point."
- **Ring-0..Ring-4** — the memory rings: 0 soul/config · 1 calibration exemplars · 2 working · 3 compressed narrative history · 4 retrieved (semantic recall via the embedder). Ring-3 = the narrative register (A6.1); Ring-4 = the A3 embedder.
- **A1–A6, B1–B4, C1–C5, D1–D3, E1–E2** — the ROADMAP slice IDs (see §3 above).
- **M1–M4** — the audit findings (M1 `--tier` I3-gate bypass + silent `--sovereign`/`--critical` no-op; M2 maintenance turns self-ingesting the lossless Tape; M3 daemon shared per-task budget; M4 CLAUDE.md doc-drift). All M1–M3 + doc-drift were fixed in HEAD `6ac319d`.
- **INV-6** — the Backrooms Director's `--no-director` killswitch (referenced in the Director description; not a KEEL ISSUE).
- **ADRs** — #5 (native Rust core now, C/C++ port designed-for-future via language-neutral goldens), #11 (`amplify` ships OFF — a hypothesis not an assumption), #13 (embedder is format-committing). Plus the condensed §20 ADRs.

---

## 8. "Canon" version references

- **Canon = `KEEL_ARCHITECTURE.md`**, version **v0.2**, 23 sections (§1 intent through §23 the falsifiers). STATE.md line: *"Canon v0.2 adopted."* The canon is the **source of truth for design**; `CLAUDE.md` is the build constitution (rules); `STATE.md`+git are the source of truth for *state*.
- The commit messages and docs reference canon sections directly (e.g. `feat(memory): … (canon 11)` = the Memory joint; `feat(daemon): … (canon 8/11)` = the engine loop + memory; `feat(memory): keel cold-eyes … (canon 10.2)`; `feat(privacy): A4 … (canon 5.1)`). These are section numbers, not version numbers.
- **There is no "canon 10.2" version** in the sense of canon v10.2 — `canon 10.2` means **canon §10, subsection 2** (the externality/correctness section, where cold-eyes validation lives). Similarly `canon §5.1` = the invariants section's privacy subsection, `canon §8` = the engine loop, `canon §11` = memory, `canon §12` = perception, `canon §16` = the distillation refusal, `canon §17/§21` = the cell/first-cell definitions, `canon §22` = anti-patterns, `canon §23` = the falsifiers.
- The **golden freeze seal** `db4377b3…` (full `db4377b3c0b3b245…`) is the KEEL-native re-stamp (2026-06-14), replacing the Marrow-Python-derived `63d5ba7c…`. The freeze-gate `goldens_match_the_frozen_hash` enforces it: a golden content change fails the build.

---

## 9. The uncommitted-working-tree caveat (load-bearing)

`git status` shows these uncommitted changes on top of HEAD `6ac319d`:
- `D crates/keel-adapters/src/embed.rs` (the A3 embedder organ — **deleted**)
- `D docs/conformance-coverage.md` (the E1 conformance map — **deleted**)
- `M crates/keel-adapters/src/lib.rs`, `M crates/keel-services/src/lib.rs` (remove the `embed` module + `Embedder` re-export)
- `M _run_state/ROADMAP.md`, `M _run_state/STATE.md`, `M _run_state/WAKE_UP_part4.md`, `M _run_state/WORKLOG.md` (revert toward older pre-06-15 versions: A3/E1 back to `[ ]`/`[~]`-undone, ISSUE-1 un-resolved, ISSUE-8 workaround note removed, ISSUE-10 removed, D1 scoping removed, the QC-pass/overnight WORKLOG entries stripped)
- `?? nul` (a 54-byte untracked file with the Windows reserved name `nul` — an accident, not in git index)

This pattern (reverting several landed slices' code + docs simultaneously, with no commit and no stash) is consistent with an **in-progress revert/abandon or a checkout mishap**, not a deliberate new direction — the reflog shows no commits after `6ac319d`, and the WORKLOG/PROJECT-STATE/audit docs (committed at `6ac319d`) describe a *completed, validated, GREEN* state with the embedder and conformance doc present. **The committed HEAD `6ac319d` is the canonical current state** (129–130/6 green, seal green, QC GREEN, embedder present, E1 present, ISSUE-10 recorded). Anyone resuming work should reconcile or discard the uncommitted tree deliberately rather than trust it — exactly the "verify by artifact" discipline the project is built around.

### Bottom line / what's next
Per the committed narrative, KEEL has reached the operator-directive's end-condition: "the bulk + low-hanging fruit are done." The genome is a validated, self-perpetuating, GREEN-audited sovereign harness — spine, three-tier economy, I5 verifier (both directions lived), perception (eyes+ears), persistent memory (Tape + Ring-3 narrative + Ring-4 recall + cold-eyes), the self-driving daemon with self-consolidation, the secret-scrubbed flywheel feed + distill-export, and the port-readiness conformance map. The **next major effort is the D1 cell build** (re-home NightScribe on KEEL over `serve_openai`, cross-language), then the heavier tier: A5 (privacy rung-3, last), C1/C2 (recall-uplift benchmarks, blocked on ISSUE-10 — the missing Qwen embedding GGUF), and E2 (the DONE review). The single open code-path blocker is ISSUE-10 (a substrate-provisioning step, routed around per pivot-when-stuck). The single open process anomaly is the uncommitted working tree described above.
