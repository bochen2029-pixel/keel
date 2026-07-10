# NEXT SESSION — handoff brief (written 2026-07-09, session end)

> **How to use:** read this FIRST, then run the ⛑ reconstitution protocol in `_run_state/STATE.md`
> (canon + CLAUDE.md + ROADMAP + the last two WORKLOG entries). This file carries only the deltas
> the standing docs don't; on any conflict, trust git + the gate over this snapshot. Supersede or
> delete this file at the next session's end.

## Where 2026-07-09 ended (verify, never recall)

`git -C C:\KEEL log --oneline -8` → HEAD `cb8b6a4`, pushed, tree clean.
`cargo test` (PowerShell, from `C:\KEEL`) → **159 passed / 6 ignored**, clippy clean, seal `db4377b3` green.

One session delivered, in order: the June-17 tree anomaly resolved (stashed) · ISSUE-10 closed
(embed GGUF downloaded + smoke-tested) · **A7 Memory Autopilot DONE + LIVED** (memory is now
zero-intervention: budgets · episodes · Ring-4 recall on-by-default · maintenance policy ·
cold-eyes self-correction; falsifiers F-M1/2/3 PASS, F-M4 machinery proven) · **D1 DONE + LIVED**
(NightScribe re-homed on KEEL over `serve_openai`; **the boundary held — zero contract/golden
edits**; three genome bugs surfaced + fixed: `--jinja`, grammar⇒thinking-off, system-message merge).

## Session-specific state the standing docs don't carry

- **Standing local procs** (all self-reviving — any `keel`/`keel-serve` call cold-starts what's
  missing): llama-server `:8080` (now launched **with `--jinja`**) · embed server `:8090` ·
  `keel-serve` `:7070`. After rebuilding the `keel` crate, restart `keel-serve` to pick it up.
- **`git stash@{0}`** in C:\KEEL = the June-17 anomaly (forensics only — never pop casually).
- **The `nul` junk file** reappears in `C:\KEEL` after cargo runs (creator not in repo sources —
  suspected dependency build script). Delete: `cmd /c del "\\?\C:\KEEL\nul"`. A spawned side-task
  chip exists to hunt the creator; don't chase it inline.
- **NightScribe commits `32c4f9a` + `5fbcbcd` live in `C:\ClaudeCode\photo2deck` — a LOCAL-ONLY
  repo (no remote).** If durability matters, the operator should back it up/add a remote.
- **A7 honest residual:** cold-eyes single-judge recall on *adversarial* plants is stochastic on
  the 9B (organic drift catches fine; 2-of-3 vote is in). Upgrade trigger = a stronger local judge
  model. Don't prompt-tweak further without new evidence.
- **D1 follow-on validation is free:** the next real meeting through NightScribe (`--vision-backend
  keel` is now the default) exercises ears + eyes + synthesis + KEEL memory on real data.

## The queue (ROADMAP order; first unblocked `[ ]`/`[?]` wins)

1. **C1/C2 — recall-uplift benchmarks** (unblocked: embed GGUF on disk, Ring-4 live). Needs an
   operator-labeled golden-recall set + a focused live session. **Design-review first** (the set
   format is operator-authored ground truth — propose, then measure). Decides reranker ON/OFF and
   embedder-vs-MiniLM-floor (note: the MiniLM floor impl itself is unbuilt).
2. **B1 — `svc::amplify`** built clamped-OFF + the ISSUE-4 best-of-N benchmark → decide ON/OFF.
3. **B3/ISSUE-5 — flywheel ignition:** run the out-of-band LoRA (Unsloth) over the exported corpus;
   measure `escalation_rate` trend (flat is an acceptable *decided* outcome).
4. **A5 — privacy rung-3** (`ort`/ONNX) — operator's explicit LAST. Then **C3**.
5. **D2 — SEXTANT** (the canon's named first cell) + **D3 — ToolHost** (pulled by D2).
6. **E2 — the DONE review** (all falsifiers decided → completion account → `keel.lock` stage flip).

**Operator-only ISSUES open:** ISSUE-6 (`sha256:` pins → then build `kernel::lock` verify) · the
Fable-5 v0.3.0 hindsight ruling (piecemeal, non-blocking) · autonomy re-grant (sessions run
SUPERVISED until re-granted).

## Disciplines (unchanged, load-bearing)

Contracts + goldens frozen (agent read-only; seal `db4377b3`). Layer rule. Zero-contract-edit bar
for cells. TTL every live run (file-redirect + `Start-Process`/`WaitForExit`/`Kill`, never
`| Out-String`); verify by artifact. Use the **PowerShell tool** for cargo/git (never git-bash).
One gated commit per slice (test + clippy green), push. Decide-and-document; operator-only acts
(contract/golden/seal/toolchain) go to the ISSUES register.
