# CLAUDE.md — KEEL build instructions for Claude Code

You are building **KEEL** (Bo Chen's personal, sovereign, reusable harness core — the *genome* from which every specialization is grown as a *cell*). Read `KEEL_ARCHITECTURE.md` (the canon) **and** this file at the start of every session before writing code.

---

## What this is
The persistent, sovereign **self** that perceives (eyes + ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model, and grounds every critical output in an assertion no model authored. The model that thinks is interchangeable, **rented cognition**; KEEL is the self. Native Rust core, consumed **embedded** (in-process) *or* **over protocol** (HTTP + MCP + OpenAI-egress) — the *.NET-of-my-AI-apps* reuse model.

**L1 personal tool, not a product.** No multi-tenancy, no features for hypothetical users. "Universal" = the *intersection* of what the operator's projects actually need (canon §2, §16), never their union.

## Environment (Windows 11)
- Repo `C:\KEEL\`. Rust stable `x86_64-pc-windows-msvc`, edition 2021. **Build/test from a native MSVC dev shell (PowerShell)**, not git-bash.
- **Known toolchain issue — resolve before Stage 0:** `rustc` can't find `std`/`core` for the host target (E0463) though `rustup` reports `rust-std` installed/up-to-date — a missing/corrupt std rlib on disk. Fix: `rustup toolchain install stable-x86_64-pc-windows-msvc --force` (or `rustup update stable`). **Operator-gated** (it touches the global toolchain DAVE/TERMINAL share). `cargo check` must pass before any Stage-0 slice is "done."
- Substrate (resolved, not embedded — canon §13): `C:\llama.cpp` (llama-server), `C:\models` (Qwen3.5-9B-Q5_K_M + `mmproj-F16`, whisper `large-v3-turbo`, the OpenAI privacy-filter), `C:\whisper.cpp`. Cloud keys (`DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY`) live in env; the operator rotates them — **never hardcode or commit a key.**
- **Reference bench (read-only):** Marrow-L1 (Python, green, golden-tested) at `C:\loom\marrow-l1` is a **read-only behavior reference / diff-oracle — never a build or runtime dependency.** KEEL ships nothing of Marrow's and links nothing from it; this is how the "self-contained" directive and the bench reference reconcile (diff against its *behavior* + the goldens — the ASTRA-textverse pattern — don't port its code).

## Non-negotiable rules
1. **The canon is the source of truth** (`KEEL_ARCHITECTURE.md`). When code and canon disagree, fix the wrong one **in the same change**. Never let them drift.
2. **The ten contracts are sacred** (`crates/keel-contracts`, canon §7). Don't change a joint without updating both sides + the golden cases in the same change. The joints are the genome; they are meant never to be rewritten.
3. **Golden cases are agent-frozen** (`tests/golden/golden.json` + `.frozen.json`) — the *language-neutral conformance layer*. You MAY NOT edit them; if a golden fails, fix the **code**. Ratifying/freezing/changing a golden is an **operator action only**.
4. **Per-crate budget.** Each crate stays comprehensible in one session; keep it lean, split before it becomes a god-crate.
5. **Layer-import rule:** `contracts ← kernel ← {adapters, middleware} ← services ← apps`. A service may import middleware; middleware may **never** import a service; the kernel imports only contracts. Enforce via cargo deps / a lint; violations are bugs, not style.
6. **Protocol-first.** Chat Completions + MCP + OpenAPI/HTTP. A new provider is a new adapter; nothing above changes. Never invent a wire protocol.
7. **Async by default** (the contracts already are).
8. **The five invariants + the reversibility gate hold on every call** (canon §5): I1 audit · I2 spine · I3 sovereign/perception-local · I4 cost-capped · I5 externalized. **Memory safety is I5 applied to the source — let the compiler be the oracle** (ADR #5).

## Hard prohibitions (reversibility gate — see AUTONOMY_CHARTER.md)
- No `git reset --hard`, `clean -fd/-fx`, `checkout -- <path>`, `restore` on uncommitted work; no `push --force`; no `branch -D` on unmerged `auto/`.
- No `rm`/`Remove-Item -Recurse -Force` outside `.\.keelstate\`.
- **Do not mutate the global Rust toolchain** (rustup update/reinstall/component changes) without asking — DAVE/TERMINAL share it.
- Any action whose undo cost you can't state in one sentence → **stop and ask.**

## Build state
*(Summary only — the live, authoritative per-slice anchor is `_run_state/STATE.md`. Trust STATE.md + `git`, never this block, for current state.)*
- **Canon:** v0.2 adopted; patched with embedder/reranker + the privacy three-rung; ADR #5 = native Rust.
- **L0 contracts:** the ten joints + types + the §18 error taxonomy — frozen, green (rustc 1.96, MSVC).
- **Golden cases:** RATIFIED + FROZEN (`tests/golden/golden.json` + `.frozen.json`, 21 cases / 6 families) — agent read-only. The KEEL-native freeze-gate is built but `#[ignore]`-dormant pending the operator's one-time re-stamp.
- **Stage 0 (spine):** complete — three-tier economy (local · cheap-API · frontier) through one invariant chain, file ledger + SQLite Spine (I2), self-resolving substrate, consumable embedded (CLI) **and** over protocol (`serve_openai`).
- **Stage 1:** `DifficultyRouter` (GOLDEN_ROUTER ✓) + the self-driving engine landed.
- **Stage 2 (in progress):** `svc::verifier` (I5, GOLDEN_ORACLE ✓) landed and **wired into the running loop** via `kernel::engine` (L1: route → chain → verify → checkpoint → emit). Still ahead: `svc::memory`, `mw::metrics`, privacy rung-3.
- **Next:** see `_run_state/STATE.md`.

## Session protocol
1. Load the canon + this file.
2. `cargo check && cargo test` (+ `cargo clippy`) from PowerShell — see the green/pending state. The next slice is the to-do list.
3. Pick the next slice (the next spine module, or the next failing conformance case). Implement against the **frozen** contracts; never redesign a joint to ease an impl.
4. Make its golden/conformance case green; diff behavior against the Marrow-L1 bench where applicable.
5. Before ending: layer-check → budget-check → golden-freeze (verify unchanged) → `cargo test` green → one commit, one-line intent.
6. Foundational unknown → write an ESCALATION note and stop. Don't guess.

## Staged build plan (each stage independently useful; falsifier-gated — canon §21)
- **Stage 0 — spine:** kernel (manifest · context · registry · chain · lifecycle + substrate-resolver · engine · lock) + one local adapter (HTTP→llama-server) + invariant middleware (audit · privacy rungs 1–2 · cost) + file ledger + SQLite store + CLI + `serve_openai`. Outcome: a binary that resolves the substrate, talks to a tier, logs every call, consumable embedded or over protocol. **Falsifier: > ~2 weeks → rethink the native-core thesis.**
- **Stage 1 — amplification & senses:** constrained decoding; the difficulty router; `amplify` (best-of-N, **ships OFF**); perception (eyes + ears, change-gated); cache-prefix; the Driver seam.
- **Stage 2 — correctness & memory:** golden registry + freeze gate; verifier + joint-wrong; ringed memory (Tape ledger, consolidation-as-a-Step, narrative/factual registers); metrics.
- **Stage 3 — flywheel:** verified-trace distillation (Unsloth Studio, out-of-band). Size everything to the **base case where `escalation_rate` stays flat** — ignition is upside.
- **First cell:** re-home **NightClerk or NightScribe** (controlled experiment — known shape, clean boundary signal). Then SEXTANT. If a cell forces a kernel/contract edit, the boundary is wrong — fix KEEL first.

## Commands
```
rustup toolchain install stable-x86_64-pc-windows-msvc --force    # one-time toolchain repair (operator-gated)
cargo check && cargo test && cargo clippy                         # the gate — run from PowerShell / MSVC env
powershell -ExecutionPolicy Bypass -File C:\loom\marrow-l1\scripts\serve_local.ps1   # local llama-server
# bench diff: C:\loom\marrow-l1  (pytest -q)  — router / cost / oracle behaviors
```

## The two disciplines that keep KEEL a substrate (not a wrapper)
- **The contract-freeze IS the governance** — the `.NET`-grade longevity the analogy borrows. The day a cell edits a joint or a golden for convenience, KEEL becomes another bespoke wrapper. Guard them like this file.
- **Size to the flywheel base case** — a great router + memory + oracle + perception + sovereign substrate is the ~80% outcome and is worth owning even if the flywheel never ignites.

*Keep the run coherent (the spine), honest (the externality layer), sovereign (local-first), and cheap (the cheapest brain that clears the bar). Build the next slice; don't perfect the description. The spec is finished the moment it stops being a substitute for a running binary.*
