# NEXT SESSION — handoff brief (written 2026-07-10, session end)

> **How to use:** read this FIRST, then run the ⛑ reconstitution protocol in `_run_state/STATE.md`
> (canon + CLAUDE.md + ROADMAP + the last FOUR WORKLOG entries — 2026-07-10 was a four-slice day).
> This file carries only the deltas the standing docs don't; on any conflict, trust git + the gate
> over this snapshot. Supersede or delete this file at the next session's end.

## Where 2026-07-10 ended (verify, never recall)

`git -C C:\KEEL log --oneline -6` → HEAD = the B1-decision commit, pushed, tree clean.
`cargo test` (PowerShell, from `C:\KEEL`) → **178 passed / 7 ignored / 0 failed**, clippy
**0 warnings whole-tree**, seal `db4377b3` green.

Five slices in one day: **(1)** C1/C2 design + harness (`73430c7` + smoke `cdca9e6`) · **(2)**
golden-recall **v2 hardening** measured to convergence (`f2c1dd3`) · **(3)** set **RATIFIED** +
MiniLM provisioned + the decision legs → **C1 OFF · C2 floor-default (falsifier trip!) + the flip
lived** (`76845dd`, ISSUE-11 resolved) · **(4)** **B1 amplify BUILT clamped-OFF + DECIDED OFF**
(`72a74ef`, ISSUE-4 resolved: uplift +0.115 < the pre-registered 0.15 bar; the §8 `amplify?` loop
is real in `kernel::engine` behind `router.amplify_n: 1`) · **(5)** **D2 SEXTANT SCOPED + SEEDED**
(the D1 pattern: boundary map `docs/proposals/sextant-on-keel.md`; repo `C:\SEXTANT` git-init'd
`ea5b9ed`, LOCAL-ONLY; D3/ToolHost timing decided = S4; S0 keystone is the next build).

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
- **Standing local procs** (all self-reviving): llama-server `:8080` (`--jinja`) · embed server
  `:8090` (**MiniLM now**) · `keel-serve` `:7070` (restarted on the final build, keys injected).
  The llama-servers get **reaped when their spawning tool-shell tears down** (observed twice
  2026-07-10) — not a defect; any `keel` call re-resolves the whole substrate. Rerank/MiniLM bench
  servers (:8091/:8092) were killed after their legs. **Two tooling rules:** stop `keel-serve`
  before any `cargo build/test` that relinks the `keel` crate (sibling-bin lock → os error 5), and
  **`cargo test` does NOT relink `target\debug\*.exe`** — run `cargo build -p keel` before
  executing a just-edited bin (a stale bin routes the subcommand as a plain prompt → junk Tape turn).
- **`git stash@{0}`** = the June-17 anomaly (forensics only — never pop casually).
- **The `nul` junk file** reappears after cargo runs; `cmd /c del "\\?\C:\KEEL\nul"` (a
  path-protection hook may block it — then just leave it; it's untracked and unstaged).
- **NightScribe repo** (`C:\ClaudeCode\photo2deck`) is still LOCAL-ONLY (no remote).
- **SEXTANT repo** (`C:\SEXTANT`, `ea5b9ed`) is LOCAL-ONLY (no remote — operator adds one if
  wanted; the gitignore keeps Canon/PII out regardless).
- **A7 honest residual** (unchanged): cold-eyes single-judge recall on adversarial plants is
  stochastic on the 9B; 2-of-3 vote in place; upgrade trigger = a stronger local judge.

## The queue (ROADMAP order; first unblocked `[ ]`/`[?]` wins)

1. **D2 is DONE — VERDICT: PASS** (S0+S1 lived on real postings, cell `df7c7c5`, 20 tests; zero
   KEEL-side changes of any kind; two cells now stand on KEEL). SEXTANT continues as PRODUCT work
   (S2 discovery breadth/research · S3 conductor · S4 dispatch → **D3/ToolHost lands there**) —
   sessions build it on the operator's ask, not as KEEL falsifier work. **Operator: author the
   real Canon** (`C:\SEXTANT\canon\profile.json` + `cv.md`) and re-run
   `python -m sextant batch postings --limit 5` — the machinery is deterministic.
2. **B3/ISSUE-5 — flywheel ignition:** out-of-band LoRA (Unsloth) over the exported corpus;
   measure the `escalation_rate` trend (flat is an acceptable *decided* outcome).
2. **B3/ISSUE-5 — flywheel ignition:** out-of-band LoRA (Unsloth) over the exported corpus;
   measure the `escalation_rate` trend (flat is an acceptable *decided* outcome).
3. **A5 — privacy rung-3** (`ort`/ONNX) — operator's explicit LAST. Then **C3**.
4. **E2 — the DONE review** — scorecard: C1 ✓ C2 ✓ B1 ✓ (pre-registered + decided) · C4/C5
   prelim-passed · C3 pending A5 · B3 pending the LoRA run · D2 verdict renders at S1.
   **D3/ToolHost lands at SEXTANT S4** (decided — `.eml` staging until then; vet `rmcp` there).

**Operator-only ISSUES open:** ISSUE-6 (`sha256:` pins → then `kernel::lock` verify) · the Fable-5
v0.3.0 hindsight ruling (piecemeal, non-blocking) · autonomy re-grant (sessions run SUPERVISED
until re-granted).

## Disciplines (unchanged, load-bearing)

Contracts + goldens frozen (agent read-only; seal `db4377b3`). Layer rule. Zero-contract-edit bar
for cells. Pre-register thresholds before measuring (the C1/C2 template). TTL every live run
(file-redirect + `Start-Process`/`WaitForExit`/`Kill`, never `| Out-String`); verify by artifact.
Use the **PowerShell tool** for cargo/git (never git-bash). One gated commit per slice, push.
Decide-and-document; operator-only acts go to the ISSUES register.
