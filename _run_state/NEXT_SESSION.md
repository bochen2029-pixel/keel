# NEXT SESSION — handoff brief (written 2026-07-10, session end)

> **How to use:** read this FIRST, then run the ⛑ reconstitution protocol in `_run_state/STATE.md`
> (canon + CLAUDE.md + ROADMAP + the last two WORKLOG entries). This file carries only the deltas
> the standing docs don't; on any conflict, trust git + the gate over this snapshot. Supersede or
> delete this file at the next session's end.

## Where 2026-07-10 ended (verify, never recall)

`git -C C:\KEEL log --oneline -5` → HEAD = this session's commit, pushed, tree clean.
`cargo test` (PowerShell, from `C:\KEEL`) → **165 passed / 7 ignored**, clippy clean, seal `db4377b3` green.

One slice delivered: **C1/C2 design-review + harness** — proposal
`docs/proposals/golden-recall-set.md` · DRAFT labeled set `tests/recall/golden-recall.json`
(42 docs / 30 queries / 6 families, fictional, `ratified:false`, thresholds in-file) · `Rerank`
seam + `IdentityRerank` · `keel-adapters::Reranker` (`/v1/rerank`) · IR metrics + stub-tested
`run_recall_bench` · **`keel recall-bench`** CLI (DRAFT-stamped until ratified) · keel.lock/manifest
`substrate.rerank {file, port: 8091}`. Zero contract/golden edits. **ISSUE-11 filed** (the operator
gate to the live run).

## Session-specific state the standing docs don't carry

- **C1/C2 is now operator-gated (ISSUE-11), not build-gated.** Operator: (1) edit + ratify
  `tests/recall/golden-recall.json` (flip `ratified:true` + by/date; content edits can't break CI —
  the lint is structure-only); (2) provision `all-MiniLM-L6-v2` GGUF into `C:\models` (~25–45 MB;
  it's already the keel.lock `embedding.fallback`) for the C2 leg; (3) contingent: re-provision the
  reranker GGUF if step-0 shows it lacks the rank head.
- **The focused live run (after ratification), 3 legs + smoke:** step-0 = launch
  `llama-server --reranking` on `qwen3-reranker-0.6b-q8_0.gguf` :8091 (ISSUE-8 pattern: bounded,
  file-redirect, kill) + one `/v1/rerank` POST — refusal ⇒ ISSUE, not a C1 verdict. Then:
  `keel recall-bench` (embed :8090 up) → baseline artifact · `keel recall-bench --rerank
  --baseline <baseline>` → C1 input · restart :8090 on MiniLM → `keel recall-bench --embed-model
  all-minilm-l6-v2 --baseline <baseline>` → C2 input (note: `--baseline` warns comparing across
  embedders is a deliberate cross-variant read, k must match). Decisions → WORKLOG + keel.lock
  flips (`rerank.default` / `embedding.id`); C1-ON additionally pulls the Ring-4 rerank wiring +
  lifecycle rerank-launch (deliberately unbuilt until earned).
- **Standing local procs** (all self-reviving): llama-server `:8080` (`--jinja`) · embed server
  `:8090` · `keel-serve` `:7070`. After rebuilding the `keel` crate, restart `keel-serve`.
- **`git stash@{0}`** in C:\KEEL = the June-17 anomaly (forensics only — never pop casually).
- **The `nul` junk file** reappears after cargo runs. Delete: `cmd /c del "\\?\C:\KEEL\nul"`.
  A side-task chip exists to hunt the creator; don't chase it inline.
- **NightScribe commits live in `C:\ClaudeCode\photo2deck` — LOCAL-ONLY repo (no remote).**
- **A7 honest residual** (unchanged): cold-eyes single-judge recall on adversarial plants is
  stochastic on the 9B; 2-of-3 vote is in; upgrade trigger = a stronger local judge.

## The queue (ROADMAP order; first unblocked `[ ]`/`[?]` wins)

1. **C1/C2 live run** — gated on ISSUE-11 (operator). If still gated, route around:
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
