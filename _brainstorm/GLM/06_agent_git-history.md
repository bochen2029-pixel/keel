# Sub-agent 6 — Git history & current working-tree state (verbatim)

> Original task: analyze recent commits, current uncommitted diff, decode commit `6ac319d`. READ-ONLY.

---

# KEEL — Recent Work & Current Working-Tree State

## 1. What the project has been doing recently (last ~40 commits)

KEEL is a Rust "AI-genome" substrate (a sovereign routing + memory + verification engine meant to underpin "cells," AI apps that consume it over `serve_openai`). The recent commit arc (`807b62f` "genesis arc" through `6ac319d`) is a single sustained push, authored by BoChen co-driven by "Claude Opus 4.8 (1M context)", running under a self-perpetuating "perpetual run" mandate (ROADMAP §0 + AUTOSTART hooks + a `keel-autoloop.ps1` respawn supervisor). Themes, grouped:

**Perception organs (canon §12)** — `e2b2fdd` config-from-keel.lock → `e4c2f1e` dHash/FrameGate retina → `771bcc6`/`b221bf0` whisper "ears" → `07e118b` cpal mic capture → `e7fe79a` xcap screen grab → `04d9dbe` `listen()`/`see_screen()` retina wrappers (A1). The eyes/ears are feature-gated (`mic`/`screen`) and dependency-light (no-dep VAD/WAV, dHash).

**The self-driving engine + daemon (canon §7/§8)** — `3ae8626` svc::driver (the initiative seam) → `886cda8` the daemon select-loop → `154b0fa` A2 `keel daemon` self-driving select-loop → `a7bfeef` `keel daemon --consolidate-every N` autonomous self-consolidation.

**Perpetual memory (canon §11) — the densest cluster** — `5f42831` svc::memory first cut → `1801aa1` afferent change-gate → `a40fba5` A6.1 Ring-3 narrative register + self-interview consolidate → `9d27a56` A3 embedder organ + cosine recall + GOLDEN_RECALL fingerprint → `e10cd09` A3 Ring-4 semantic recall wired into `assemble` via the `Embed` seam → `6635130` A6.2 `keel consolidate` closes the loop → `830a8f2` records it LIVED end-to-end → `6fe0ad0` `keel cold-eyes` (validate the narrative against the Tape, I5 capstone).

**The flywheel / distillation (canon §8/§16)** — `56d0dde` B2 FileTraceSink (scrubbed distill corpus) → `747109f` B4 svc::distill + `keel distill-export` (chat-format training pairs).

**Privacy (canon §5.1)** — `fe26129` A4 I3 output rung masks response PII on every tier; retires the no-SSN I5 stopgap.

**Autonomy plumbing** — `8c9b5bb`/`34b334e`/`0b93199` the self-perpetuating build mechanism (AUTOSTART + respawn supervisor + reconstitution wiring) → `ab5e0f0` INIT_PROMPT refresh → `acf9f9b` bakes the standing operator directive into the reconstitution chain so it survives compaction.

**Conformance + validation docs** — `52c4dd2` E1 conformance-coverage map (ADR 5) → `457986f`/`830a8f2`/`65e16ae` "whole flywheel LIVED" + preliminary C4/C5 falsifier decisions → `04a6acd` ISSUE-10 (Qwen3-Embedding GGUF absent from `C:\models`) → `e127ada` D1 cell scoping (NightScribe-on-KEEL boundary map).

**QC pass (the capstone)** — `f9c031a` docs(qc): a 48-agent ultracode audit + run report + project-state → `6ac319d` fix(qc): addresses the confirmed audit findings.

## 2. Shape of the most recent 15 commits (`git log --stat -15`)

Two shapes dominate. **Feature slices** (`6635130`, `e10cd09`, `9d27a56`, `a7bfeef`, `6fe0ad0`) each touch 3–5 files: the implementing crate file (`memory.rs`, `main.rs`, `recall.rs`, `embed.rs`) plus the four run-state docs (`ROADMAP.md`/`STATE.md`/`WORKLOG.md`, sometimes `WAKE_UP_partN.md`) with large insertions (40–200 lines each) — every slice lands with a heavy "self-narrating run-state" tail. **Doc-only commits** (`457986f`, `830a8f2`, `65e16ae`, `04a6acd`, `e127ada`, `acf9f9b`) touch only the run-state docs (3–21 lines each). The two largest single-commit file deltas are `f9c031a` (3 docs, +506 lines: the audit/run/project-state trio) and `9d27a56` (+198 lines: the embedder + recall module). The QC fix `6ac319d` is small and surgical (8 files, +55/−12). Net: code lands in small tested slices; the doc tail is consistently heavier than the code.

## 3. Current working-tree state (`git status`)

Branch `main`, up to date with `origin/main`. **All changes are unstaged; nothing is staged or committed.** Tracked changes (matching the starting snapshot, now confirmed):

- modified: `_run_state/ROADMAP.md`, `_run_state/STATE.md`, `_run_state/WAKE_UP_part4.md`, `_run_state/WORKLOG.md`
- modified: `crates/keel-adapters/src/lib.rs`, `crates/keel-services/src/lib.rs`
- deleted: `crates/keel-adapters/src/embed.rs`
- deleted: `docs/conformance-coverage.md`
- untracked: `nul` — a **junk file**, not intentional content. Its sole contents are the string `ls: cannot access 'C:KEEL': No such file or directory` — a Windows reserved-name artifact created when an earlier `git -C "C:\KEEL"` command's backslashes got eaten and the error was redirected to a file named `nul`. It should be deleted, not committed.

## 4. What the uncommitted diff actually does (`git diff --stat`: 8 files, +46/−243)

This is a **rollback / reversion of the entire A3 Ring-4 semantic-recall feature and the E1 conformance doc**, plus a run-state narrative rewrite. Semantically, four things are happening:

**(a) `crates/keel-adapters/src/embed.rs` deleted (−83 lines).** The `Embedder` struct (HTTP `/v1/embeddings` client for Qwen3-Embedding-0.6B) — added in commit `9d27a56` — is removed entirely. `crates/keel-adapters/src/lib.rs` drops both `pub mod embed;` and `pub use embed::Embedder;`.

**(b) `crates/keel-services/src/lib.rs` drops `pub mod recall;` and `pub use recall::{cosine, recall_top_k, should_rebuild, Fingerprint};`.** But — critically — `crates/keel-services/src/recall.rs` is **still on disk and still tracked** (not deleted; `git status` does not list it). And `crates/keel-services/src/memory.rs` (also not in the diff) still contains at line 37 `use crate::recall::{cosine, Embed, Fingerprint};` and still references `Embed`, `Fingerprint`, `with_embedder`, `recall_k`, etc. (~16 references across the file, including 3 live tests at lines 563/584/588).

**The working tree does not compile as-is:** `memory.rs` imports from `crate::recall`, but `recall` is no longer declared as a module in `lib.rs`. This is an inconsistent half-revert — either `recall.rs` and the `memory.rs` Ring-4 code/tests should also be removed, or the `pub mod recall;`/re-exports should be restored. Right now it is broken.

**(c) `docs/conformance-coverage.md` deleted (−48 lines).** This was the E1 deliverable from commit `52c4dd2` — the golden-coverage map for C/C++-port-readiness (ADR 5).

**(d) Run-state docs rewritten to unwind the "A3/D1/E1 are done" narrative.** The diffs to ROADMAP/STATE/WAKE_UP_part4/WORKLOG are large prose rewrites that downgrade completed items back to not-started:
- A3 flips from `[~] A3 FIRST PASS DONE / Ring-4 WIRED` to `[G] A3` (gated) — "format-committing (ADR #13) → ISSUE-1 operator design-review FIRST," re-proposing `sqlite-vec` (the very dep the committed A3 had explicitly *rejected* in favor of brute-force cosine).
- A6.2 flips from "DONE / LIVED" back to "remaining (deferred — model-dependent)".
- B3/C1/C2/C4/C5 flip from prelim-PASS `[~]` back to open `[?]` (e.g. C4 `rework_rate < 10%` goes from "PRELIM PASS 0.056" to "needs A2 data"; C5 from "~78% saved KEEL-favorable" to "needs cost data").
- D1 (the NightScribe cell) flips from `[~] D1 SCOPED` to `[ ] D1` open.
- E1 flips from `[x] E1 DONE` to `[ ] E1` open.
- ISSUE-10 (missing Qwen3-Embedding GGUF) and the ISSUE-8 live-workaround detail are removed from the register.
- WAKE_UP_part4.md §4.2–4.4 reverts the invariant scorecard and the "RESOLVED" debt register back to an *earlier, less-advanced* state (e.g. I5 back to "Engine::run never calls it — THE gap"; the freeze-gate back to "dormant, operator-only re-stamp pending"; CLAUDE.md build-state back to "STALE — operator-governed fix on the roadmap"). This is older prose that predates several already-committed fixes — it contradicts both the committed code and the audit doc.

**Through-line of the uncommitted change:** it is a wholesale *revert of the A3/D1/E1/QC-pass narrative* (and the embedder code) back to a pre-`9d27a56` state, while leaving `recall.rs` and `memory.rs`'s Ring-4 code dangling — i.e. an in-progress, incomplete revert that has not yet been made to compile.

## 5. Other branches / WIP

None. `git branch -a` shows only `* main` and `remotes/origin/main`; `git log --oneline --all -20` is identical to `main`'s log. There is no feature branch, no stash, no WIP ref — all WIP is the uncommitted working tree on `main`.

## 6. The most recent commit `6ac319d` — theme and sub-items

`6ac319d fix(qc): address confirmed audit findings` (2026-06-15 02:52). Its body is empty beyond the subject; the sub-items are decoded by the audit doc `docs/AUDIT-2026-06-15.md` (committed in the preceding `f9c031a`) and the actual code diff. The commit is the code-fix half of a two-commit QC pass (docs `f9c031a` + fixes `6ac319d`); stat: 8 files, +55/−12. Each sub-item:

- **I3/I5 `--tier` guard (audit M1 + L3):** the manual `--tier` override skips both the router (the I3 force-local *gate*) and the engine (the I5 verifier), so `--sovereign`/`--critical`/`--golden-ref` were silently voided on that path — a false-sense-of-protection footgun. Fix in `crates/keel/src/main.rs:96-110`: the override now **refuses** (`exit(2)`) when `--sovereign` is set with a non-local tier, and refuses `--critical`/`--golden-ref` outright, with explanatory messages.

- **Tape excludes maintenance turns (audit M2):** `keel consolidate`/`keel cold-eyes`/daemon self-consolidation ran model-authored maintenance prompts through `engine.run`, which recorded them to the lossless factual Tape — violating the A6.1 register separation (canon §22.8) and creating a self-ingest loop. Fix in `crates/keel-services/src/memory.rs:310-318`: `FileMemory::record` now early-returns `Ok(())` when `step.source == "memory"` or `step.ty` starts with `"memory:"`. A regression test `memory_maintenance_turns_are_not_recorded_to_the_tape` was added (`memory.rs:449`).

- **Daemon per-tick budget (audit M3):** `run_daemon` built one `ctx` and reused it across all ticks, so a perpetual *paid* daemon would climb one shared per-task budget to a permanent `BudgetExceeded` and die. Fix in `crates/keel/src/main.rs:238-242`: each tick re-seeds `ctx.task_budget = Some(ctx.cost.total + manifest.cost.budget_per_task)` (cumulative `cost.total` preserved for the final report; only headroom resets).

- **doc-drift (audit M4/M5/L4/L6):** `CLAUDE.md` refreshed — freeze-gate now stated "ACTIVE + green" (seal `db4377b3`, was wrongly "#[ignore]-dormant"); `svc::memory` dropped from "Still ahead" (it shipped); metrics described as the off-loop SQLite-Spine reader it actually is (not `mw::metrics`); Stage-3 flywheel line added. `crates/keel/src/lib.rs:115` docstring fixed ("`TraceSink` stays `None` until Stage 3" → "wired just below").

- **poison-policy (audit L5):** `crates/keel-adapters/src/mic.rs:45,68` and `crates/keel-services/src/driver.rs:59,63,75,106,158` switched from `.lock().expect("…poisoned")` (panic) to `.lock().unwrap_or_else(|p| p.into_inner())` (recover), unifying with the poison-tolerant pattern already used in `memory.rs`/`trace_sink.rs`.

## Through-line

The project's recent arc is one continuous push to **take KEEL from "a sovereign router + substrate" to a self-driving, self-remembering genome whose externality loop is wired in the running binary**, narrated by an unusually heavy run-state doc tail and capped by a multi-agent QC audit that found it GREEN. The committed HEAD (`6ac319d`) is the clean, gate-green, audit-fixed tip.

The **uncommitted working tree is a break from that arc** — an in-progress, incomplete revert that unwinds the A3 embedder/recall feature and the E1 conformance doc and rewrites the run-state narrative back to a less-advanced state (re-proposing `sqlite-vec`, re-opening A6.2/B3/C1–C5/D1/E1, deleting ISSUE-10). It is incomplete in a load-bearing way: `pub mod recall;` was removed from `keel-services/src/lib.rs` and `embed.rs` was deleted, but `recall.rs` remains on disk and `memory.rs:37` still does `use crate::recall::{cosine, Embed, Fingerprint};` with ~16 live references including tests — so **the tree as it stands will not compile**. There is also a stray `nul` junk file that should be removed. This looks like a re-think/revert of the Ring-4 embedder direction (perhaps pivoting A3 back to the operator-design-review gate per the rewritten ROADMAP) that was left mid-flight before the code was made consistent.
