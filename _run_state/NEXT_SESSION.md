# NEXT SESSION — handoff brief (written 2026-07-10, session end)

> **How to use:** read this FIRST, then run the ⛑ reconstitution protocol in `_run_state/STATE.md`
> (canon + CLAUDE.md + ROADMAP + the last three WORKLOG entries — 2026-07-10 was a three-slice day).
> This file carries only the deltas the standing docs don't; on any conflict, trust git + the gate
> over this snapshot. Supersede or delete this file at the next session's end.

## Where 2026-07-10 ended (verify, never recall)

`git -C C:\KEEL log --oneline -6` → HEAD = the C1/C2-decision commit, pushed, tree clean.
`cargo test` (PowerShell, from `C:\KEEL`) → **167 passed / 7 ignored**, clippy clean, seal `db4377b3` green.

Three slices in one day: **(1)** C1/C2 design + harness (`73430c7` + smoke `cdca9e6`) · **(2)**
golden-recall **v2 hardening** measured to convergence (`f2c1dd3`) · **(3)** set **RATIFIED**
(operator delegation, thresholds pre-registered) + MiniLM provisioned byte-exact + the three
decision legs → **C1 DECIDED OFF · C2 DECIDED floor-default (falsifier trip!) + the flip executed
and lived**. ISSUE-11 RESOLVED. `GOLDEN_RECALL` fully accounted for.

## Session-specific state the standing docs don't carry

- **The embed substrate CHANGED:** the genome default embedder is now **all-MiniLM-L6-v2 f16
  (384-dim, `pooling: mean`)** on `:8090`; Qwen3-Embedding-0.6B stays on disk as the lock
  `fallback`. The Ring-4 sidecars were auto-rebuilt under the new fingerprint
  (`all-minilm-l6-v2:384`, 30 vecs, verified). Any stale Qwen3 embed server must NOT be revived on
  :8090 — the new **dim-guard** drops wrong-dim vectors loudly if one is.
- **C1/C2 decision record:** proposal §9 (`docs/proposals/golden-recall-set.md`) + keel.lock
  annotations + the 2026-07-10 WORKLOG entries. Re-open triggers recorded there (C1: organic
  recall misses / k=1; C2: Qwen3 instruct-prefix experiment / symmetric-hardening pass). Decision
  artifacts: `.keelstate/bench/recall-*.json` (per-query `top_ids` included).
- **Standing local procs** (all self-reviving): llama-server `:8080` (`--jinja`, pid fresh — the
  old one died and the resolver revived it) · embed server `:8090` (**MiniLM now**) · `keel-serve`
  `:7070` (restarted on the fresh build, keys injected). Rerank/MiniLM bench servers (:8091/:8092)
  were killed after their legs. Stop `keel-serve` before any `cargo build/test` that relinks the
  `keel` crate (sibling-bin file lock → os error 5).
- **`git stash@{0}`** = the June-17 anomaly (forensics only — never pop casually).
- **The `nul` junk file** reappears after cargo runs; `cmd /c del "\\?\C:\KEEL\nul"` (a
  path-protection hook may block it — then just leave it; it's untracked and unstaged).
- **NightScribe repo** (`C:\ClaudeCode\photo2deck`) is still LOCAL-ONLY (no remote).
- **A7 honest residual** (unchanged): cold-eyes single-judge recall on adversarial plants is
  stochastic on the 9B; 2-of-3 vote in place; upgrade trigger = a stronger local judge.

## The queue (ROADMAP order; first unblocked `[ ]`/`[?]` wins)

1. **B1 — `svc::amplify`** built clamped-OFF (n=1) + the ISSUE-4 best-of-N-vs-single-pass
   benchmark → decide ON/OFF. (Structure is model-free buildable; the benchmark is a live session.
   The golden-recall bench pattern — pre-registered threshold, decision-grade artifacts — is the
   template.)
2. **B3/ISSUE-5 — flywheel ignition:** out-of-band LoRA (Unsloth) over the exported corpus;
   measure the `escalation_rate` trend (flat is an acceptable *decided* outcome).
3. **A5 — privacy rung-3** (`ort`/ONNX) — operator's explicit LAST. Then **C3**.
4. **D2 — SEXTANT** (the canon's named first cell) + **D3 — ToolHost** (pulled by D2; the last
   conformance-coverage gap).
5. **E2 — the DONE review** (C1/C2/C4/C5 are now decided/prelim-passed; remaining for DONE:
   B1-decided, C3, ISSUE-6 pins, and the falsifier re-checks with fresh data).

**Operator-only ISSUES open:** ISSUE-6 (`sha256:` pins → then `kernel::lock` verify) · the Fable-5
v0.3.0 hindsight ruling (piecemeal, non-blocking) · autonomy re-grant (sessions run SUPERVISED
until re-granted).

## Disciplines (unchanged, load-bearing)

Contracts + goldens frozen (agent read-only; seal `db4377b3`). Layer rule. Zero-contract-edit bar
for cells. Pre-register thresholds before measuring (the C1/C2 template). TTL every live run
(file-redirect + `Start-Process`/`WaitForExit`/`Kill`, never `| Out-String`); verify by artifact.
Use the **PowerShell tool** for cargo/git (never git-bash). One gated commit per slice, push.
Decide-and-document; operator-only acts go to the ISSUES register.
