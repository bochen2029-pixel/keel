<!-- KEEL — initialization prompt. Paste the block below as the FIRST message of a brand-new session
     started in C:\KEEL\. It routes the instance into WAKE_UP.md + the authoritative docs, forces
     verify-by-artifact, and gates it (read → verify → prove → propose → stop) before any code. -->

You are continuing the build of **KEEL** — Bo Chen's personal, sovereign, reusable AI-harness *genome* (native Rust; "rented cognition, owned self") — in the repo at **C:\KEEL\** on Windows 11. Work in **supervised mode**. Do NOT write or edit any code, and do NOT run any git or otherwise-mutating command, until you have (1) read the required docs IN FULL, (2) verified the real state by artifact, (3) restated your understanding, and (4) I approve a plan. Proceed through the phases below and STOP at the end of Phase 4.

**CWD note:** because you are in `C:\KEEL`, the auto-loaded `CLAUDE.md` is correctly KEEL's. (Prior sessions ran in `C:\loom` and mis-loaded *Marrow-L1's* CLAUDE.md — a major past source of confusion; it does NOT apply to you.) Build and test from a native MSVC **PowerShell**, not git-bash.

### PHASE 1 — READ (in this exact order, IN FULL, verbatim — do not skim)
1. `C:\KEEL\_run_state\WAKE_UP.md` — the pre-digested, reconciled onboarding brief written for exactly this moment. It is your fastest, lowest-context path to being 100% up to speed; everything else corroborates it. (If it doesn't fit one read, read `_run_state\WAKE_UP_part1..5.md` in order, or use the chunker at `C:\KEEL\chunker`.)
2. `C:\KEEL\KEEL_ARCHITECTURE.md` — **THE CANON** (v0.2). The source of truth for design.
3. `C:\KEEL\CLAUDE.md` — the build constitution (the **rules**). ⚠ Its "Build state" section is **STALE** — use it for the rules, NOT for the current state.
4. `C:\KEEL\AUTONOMY_CHARTER.md` — the reversibility gate + hard prohibitions.
5. `C:\KEEL\_run_state\STATE.md` — the live, per-slice build anchor (the **authoritative** current state).
6. `C:\KEEL\keel.lock` — the substrate pin (models, servers, tiers, resolver order, ledger/index).
7. `C:\KEEL\tests\golden\golden.json` + `C:\KEEL\tests\golden\.frozen.json` — the agent-frozen, language-neutral conformance layer (**READ-ONLY; never edit**).

*Secondary / on-demand — do NOT read fully now (the brief already distills them); consult only for a specific raw detail:* `_run_state\handoff\forward-arc.md`, `_run_state\handoff\recent-turns.md`, `_run_state\trajectory-account.md`, `docs\proposals\perpetual-memory.md`, `_run_state\KEEL_GENESIS_TRANSCRIPT_ASSESSMENT.md`. The full pre-compaction transcript (`_memories\You_are_continuing_…md`, ~1.14 MB, local backstop only — gitignored, not in a clone) is the lossless backstop — do NOT read it whole; chunk it or `grep` it only for a specific fact.

### PHASE 2 — VERIFY BY ARTIFACT (never trust recall or a summary)
From PowerShell: `git -C C:\KEEL log --oneline -15` · `git -C C:\KEEL status` · `cargo check` · `cargo clippy` · `cargo test`. Confirm the brief + STATE.md match reality. If anything conflicts, **the artifact wins** — report the discrepancy rather than acting on the doc.

### PHASE 3 — PROVE YOU UNDERSTAND (concise, in your own words)
- KEEL in one sentence; the genome/cell model; the L1-personal-tool / not-a-product (intersection-not-union) boundary.
- The five invariants + the reversibility gate, and what **I5** (externalized correctness — ground truth from outside the model) means — and that it is **NOT yet wired into the running loop**.
- The layer-import rule and the exact order.
- The frozen-golden rule: contracts + goldens are agent-read-only; if a golden fails you fix the **code**; you **never** re-stamp the operator's seal.
- The current build state (Stage 0 complete · Stage 1 router + self-driving engine landed · `svc::verifier`/I5 landed but **unwired**), trusting STATE.md + git over CLAUDE.md.
- The reconciled drift you must hold: the Backrooms **"Director" is a first external CONSUMER / dogfood over `serve_openai`, NOT a cell, NOT the canon's first cell — SEXTANT remains the canon's first cell.** The system is **SUPERVISED** (no codified autonomy grant).
- The next slice and why: **`kernel::engine` (L1)** over injected `&dyn Router/Oracle/Memory/Spine/TraceSink` — route → chain → **verify** → checkpoint → emit — which wires I5 live, accumulates I4 cost in `Context`, activates I2 checkpointing, and pays the L5→L1 engine debt (four things at once).

Then flag anything in the canon/docs you find ambiguous, internally inconsistent, or that you'd implement differently — **NAME it; do not silently "fix" it.**

### PHASE 4 — PROPOSE THE NEXT SLICE, then STOP
Propose a tight plan for the next slice — default the `kernel::engine` loop, but recommend whether to do the quick **record-reconciliation first** (WAKE_UP §5.1 Step 0: confirm the Director ruling, correct CLAUDE.md's stale build-state + STATE.md's Director line) so the record is clean before more code lands. Respect the layer order and the **frozen** contracts (I expect zero contract edits — if you think one is needed, justify it and wait). Then **STOP** and wait for my go. Do not write code or run any work-losing git command until I approve.

### STANDING RULES (hold every session)
- **Verify by artifact, never recall.** Contracts-first; never edit a contract/golden to ease an impl. Fix the code, never the golden; never re-stamp the seal.
- Layer rule: `contracts ← kernel ← {adapters, middleware} ← services ← apps`. Async by default. Protocol-first; never invent a wire protocol. Genome = the intersection of the operator's projects, never their union.
- **Reversibility gate:** no `git reset --hard` / `clean -fd/-fx` / `checkout -- <path>` / `restore` on uncommitted work; no `push --force`; no `branch -D` on unmerged `auto/`; no `rm` / `Remove-Item -Recurse -Force` outside `.\.keelstate\`. **Do NOT mutate the global Rust toolchain** without asking (DAVE/TERMINAL share it). **Never hardcode or commit a key** (`DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY` live in env, User scope). No cloud egress of sovereign data (incl. raw frames + embedding vectors). Any action whose undo cost you can't state in one sentence → **stop and ask**.
- **One slice at a time, banked clean:** layer-check → per-crate budget → golden-freeze unchanged → `cargo test` green → ONE commit, one-line intent. **Commit/push ONLY when I ask.** Commit trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
- Build/test from PowerShell (MSVC), not git-bash. Note: a shell started *before* the API keys were set wires local-only by design — inject `$env:DEEPSEEK_API_KEY = [Environment]::GetEnvironmentVariable('DEEPSEEK_API_KEY','User')` (and Anthropic) to exercise cloud tiers.
- **Match the operator:** Skeptic pass (no sycophancy/manufactured resonance), honest reviews over agreeable ones, "lived vs reconstructed" kept distinct, one-step-at-a-time gating.

Begin with Phase 1.
