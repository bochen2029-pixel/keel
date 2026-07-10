# NEXT SESSION — handoff brief (written 2026-07-10, session end)

> **How to use:** read this FIRST, then run the ⛑ reconstitution protocol in `_run_state/STATE.md`
> (canon + CLAUDE.md + ROADMAP + the last two WORKLOG entries). This file carries only the deltas
> the standing docs don't; on any conflict, trust git + the gate over this snapshot. Supersede or
> delete this file at the next session's end.

## Where 2026-07-10 ended (verify, never recall)

`git -C C:\KEEL log --oneline -5` → HEAD = this session's commit, pushed, tree clean.
`cargo test` (PowerShell, from `C:\KEEL`) → **165 passed / 7 ignored**, clippy clean, seal `db4377b3` green.

One slice delivered: **C1/C2 design-review + harness** (`73430c7`) — proposal
`docs/proposals/golden-recall-set.md` · DRAFT labeled set `tests/recall/golden-recall.json`
(42 docs / 30 queries / 6 families, fictional, `ratified:false`, thresholds in-file) · `Rerank`
seam + `IdentityRerank` · `keel-adapters::Reranker` (`/v1/rerank`) · IR metrics + stub-tested
`run_recall_bench` · **`keel recall-bench`** CLI (DRAFT-stamped until ratified) · keel.lock/manifest
`substrate.rerank {file, port: 8091}`. Zero contract/golden edits. **ISSUE-11 filed.** **Then the
whole harness was SMOKED LIVE (DRAFT, $0)** — see the last WORKLOG entry; step-0 retired; the
recall@5-saturation finding drives a v2 hardening pass before ratification.

## Session-specific state the standing docs don't carry

- **C1/C2 sequence now: v2 hardening → ratification → live run.** The DRAFT smoke proved the
  harness end-to-end AND showed identity recall@5 saturates (0.975) — C1's golden-named measure
  (recall@5 uplift ≥ 0.10) has no headroom on the draft set. **Next session (or on the operator's
  word): author the v2 hardened corpus** (proposal §7 Remedy A — grow ~2–3× with near-topic
  confusables; engineer identity recall@5 into ~0.6–0.8 using the per-query data in
  `.keelstate/bench/recall-qwen3-embedding-0.6b-q8-identity.json` as the feedback loop). Then the
  operator ratifies v2 (flip `ratified:true`; content edits can't break CI — the lint is
  structure-only) + provisions `all-MiniLM-L6-v2` GGUF (~25–45 MB) for the C2 leg.
- **Step-0 is RETIRED:** `qwen3-reranker-0.6b-q8_0.gguf` loads under `--reranking` (up in 6 s) and
  `/v1/rerank` scores correctly (0.9975 vs 1.3e-05). Draft rerank leg: nDCG +0.061 / MRR +0.040 /
  recall@5 +0.025 at p95 551 ms (≤ the 1500 ms budget) — improves every family. The C1 live run is
  3 commands once v2 is ratified: `keel recall-bench` → `--rerank --baseline <id-artifact>` →
  (MiniLM on :8090) `--embed-model all-minilm-l6-v2 --baseline <id-artifact>`. Decisions → WORKLOG
  + keel.lock flips; C1-ON additionally pulls Ring-4 rerank wiring + lifecycle rerank-launch
  (deliberately unbuilt until earned).
- **Negative-control floor data (recorded, no change made):** top-1 cosines 0.689/0.528/0.443 on
  no-answer queries — the live `cos <= 0` Ring-4 floor would inject all three; a future floor sits
  ~0.7 on this embedder or stays score-relative.
- **Standing local procs** (all self-reviving): llama-server `:8080` (`--jinja`) · embed server
  `:8090` · `keel-serve` `:7070` (**restarted this session on the fresh build, User-scope keys
  injected**; the :8091 rerank smoke server was killed after — launch it per ISSUE-8 when needed).
  After rebuilding the `keel` crate, restart `keel-serve` (it locks the sibling bin — stop it
  first or the build fails with os error 5).
- **`git stash@{0}`** in C:\KEEL = the June-17 anomaly (forensics only — never pop casually).
- **The `nul` junk file** reappears after cargo runs. Delete: `cmd /c del "\\?\C:\KEEL\nul"`.
  A side-task chip exists to hunt the creator; don't chase it inline.
- **NightScribe commits live in `C:\ClaudeCode\photo2deck` — LOCAL-ONLY repo (no remote).**
- **A7 honest residual** (unchanged): cold-eyes single-judge recall on adversarial plants is
  stochastic on the 9B; 2-of-3 vote is in; upgrade trigger = a stronger local judge.

## The queue (ROADMAP order; first unblocked `[ ]`/`[?]` wins)

1. **C1/C2 v2 set hardening** (agent-doable now) → then ISSUE-11 ratification (operator) → the
   live run. If gated at the ratification step, route around:
2. **B1 — `svc::amplify`** built clamped-OFF + the ISSUE-4 best-of-N benchmark → decide ON/OFF.
   (Model-free structure is buildable now; the benchmark is a live session.)
3. **B3/ISSUE-5 — flywheel ignition:** out-of-band LoRA (Unsloth) over the exported corpus;
   measure `escalation_rate` trend (flat is an acceptable *decided* outcome).
4. **A5 — privacy rung-3** (`ort`/ONNX) — operator's explicit LAST. Then **C3**.
5. **D2 — SEXTANT** + **D3 — ToolHost** (pulled by D2).
6. **E2 — the DONE review.**

**Operator-only ISSUES open:** ISSUE-11 (ratify golden-recall set + MiniLM provisioning) ·
ISSUE-6 (`sha256:` pins) · the Fable-5 v0.3.0 hindsight ruling (piecemeal, non-blocking) ·
autonomy re-grant (sessions run SUPERVISED until re-granted).

## Disciplines (unchanged, load-bearing)

Contracts + goldens frozen (agent read-only; seal `db4377b3`). Layer rule. Zero-contract-edit bar
for cells. TTL every live run (file-redirect + `Start-Process`/`WaitForExit`/`Kill`, never
`| Out-String`); verify by artifact. Use the **PowerShell tool** for cargo/git (never git-bash).
One gated commit per slice (test + clippy green), push. Decide-and-document; operator-only acts
go to the ISSUES register.
