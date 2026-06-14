# KEEL — Build State & Reconstitution Anchor
*updated 2026-06-13*

## ⛑ RECONSTITUTION PROTOCOL — read this FIRST after any compaction/resume, before ANY other action
The conversation summary is **lossy**. The committed repo + the canon are the truth. Before resuming work or answering a new turn:
1. **Read this file (STATE.md) fully.**
2. **Read the canon** `C:\KEEL\KEEL_ARCHITECTURE.md` **+ the constitution** `C:\KEEL\CLAUDE.md`.
   ⚠ **The session cwd is `C:\loom`, so the AUTO-LOADED CLAUDE.md is Marrow-L1's, NOT KEEL's.** The active project is **KEEL at `C:\KEEL`**. Do not act on the Marrow-L1 constitution.
3. **Verify real state, never recall:** `git -C C:\KEEL log --oneline -8` + `git -C C:\KEEL status`; then from **PowerShell** `cargo check -p keel-contracts` (must be green).
4. Confirm goldens frozen: `C:\KEEL\tests\golden\.frozen.json` sha256 `63d5ba7cee610e92…`.
5. **Only then** resume from "Next" below. **Trust files over summary; verify by artifact, never by memory.**

## Where we are (2026-06-13)
- **KEEL** = the sovereign genome harness. Canon **v0.2 adopted**. **Native Rust core** (ADR #5). Consumed **embedded or over protocol**. L1 personal tool, not a product.
- **Committed `d83d6ac` on `main`; pushed PUBLIC to `github.com/bochen2029-pixel/keel`.**
- **L0 contracts** (`crates/keel-contracts`): the ten joints + types + §18 error taxonomy. **`cargo check` + `cargo clippy` GREEN** on rustc **1.96.0**.
- **L1 kernel spine — slices 1–2 landed** (`crates/keel-kernel`): `manifest` (YAML; parses the real `keel.lock` → typed tiers/router/cost, reuses L0 `Price`/`Effort`/`Capabilities`) · `context` (trace-id + clock + per-task budget; L0 stays clock-free) · `registry` (tier→`Arc<dyn ModelTier>` container the **wiring layer** fills, so the kernel imports no L2 — a deliberate fix over the bench) · `chain` (the middleware onion; I1/I3/I4 ride here and become unbypassable; middleware observes/gates/transforms-request, the **engine owns `Context`** for accumulation). `check` + `clippy` clean, **12 tests green**. Manifest format = YAML behind serde (swappable); `rust-toolchain.toml` pin deferred (charter §5: a global-toolchain touch).
- **L3 invariant middleware — the deterministic trio landed** (`crates/keel-middleware`): `mw::audit` (I1 — structured `AuditEvent` per call behind a pluggable `AuditSink`; fires even on a blocked call) · `mw::privacy` (I3 — rung-1/2 deterministic mask: operator markers + regex/checksums incl. Luhn cards; redacts on egress, passes through locally; the operator's marker list stays operator-authored) · `mw::cost` (I4 — pre-call budget hard-stop gate). All compose on one chain → invariants proven unbypassable end-to-end. **12 middleware tests green**.
- **L2 first real tier landed** (`crates/keel-adapters`): `local_llama` — HTTP→llama-server (OpenAI chat-completions): multimodal `Content`→`image_url`, GBNF/JSON constrained-decode hook, `Effort.thinking`→Qwen `enable_thinking` toggle, $0 local cost via `compute_cost`. Shared `openai` mapping module (deepseek/anthropic reuse it next). **Live-validated** against the running b9627 server (`-- --ignored` green).
- **L5 Stage-0 capstone — a runnable `keel` binary** (`crates/keel`): assembles the spine from `keel.lock` (manifest → registry(+`local_llama`) → chain(audit·privacy·cost)), runs a prompt end-to-end to the live tier, prints the answer, and `mw::audit` writes JSONL to the file ledger `.keelstate/audit.jsonl` (I1/I2 to disk). **Validated live**: `keel "…"` → correct Qwen3.5-9B answer, cost $0, trace + audit on disk. Added `FileAuditSink` + optional `TierCfg.endpoint`. **The canonical Stage-0 outcome is met: resolve substrate · talk to a tier · log every call.** **32 tests green** (the binary is validated by running).
- **L2 cheap-API tier landed** (`keel-adapters::deepseek`): DeepSeek over HTTPS (`/chat/completions` no `/v1`; `thinking`+`reasoning_effort`), reusing the shared `openai` mapping (the factor-out pays off). **Live-validated** on the real API — correct answer, `reasoning_content` parsed, **real cost** computed; the CLI is now `--tier`-aware (local | cheap-API) with the **I3 egress mask on for cloud tiers** (local stays sovereign-safe). The ledger records both tiers' real costs (local $0 · cheap-API $0.000126). Key in `DEEPSEEK_API_KEY` (env), never a file. **36 unit tests + 2 live.** (`deepseek` later fixed to honor `Effort.thinking` explicitly — v4-pro defaults thinking ON; lean ⇒ `disabled`.)
- **L1 substrate resolver — FULL (c1 + c2) landed** (`kernel::lifecycle`): `keel "…"` **discovers** a running server (probe :1234/:11434/:8080) OR **cold-starts** one when none is up (`launch` spawns llama-server from `keel.lock` paths → polls a hand-rolled HTTP `/health` until the model loads, 200=ready vs 503=loading → returns a detached `LlamaServer` handle, reused on the next call). `SUBSTRATE_UNRESOLVED` only if launch fails. Dependency-free (kernel stays tiny). **Live-proven**: killed the server → `keel "…"` cold-started a fresh one (pid 36008, answered `Atlantic`) → a second call reused it (`Mars`). **KEEL is now self-sufficient.** 5 lifecycle tests (incl. a `/health` 200-vs-503 parse).
- **L5 `serve_openai` landed — KEEL over protocol** (`keel-serve` bin + shared `keel` wiring lib): an axum OpenAI-compatible server (`POST /v1/chat/completions` · `/v1/models` · `/health`) that assembles the spine **once** (resolving/launching the substrate) and runs every request through the same registry + invariant chain. The CLI and server now share one `assemble()` — no duplicated wiring. **Live-validated**: POST → correct Qwen answer, with a `keel` extension exposing `tier`+`cost`. Default thinking = lean (content-forward; `think:true` opts into reasoning). **KEEL is consumable both ways the canon promises — embedded (lib/CLI) AND over protocol (the fleet points at `:7070`).**
- **Golden cases**: ratified + **FROZEN** (`tests/golden/golden.json` + `.frozen.json`, 21 cases / 6 sections). Language-neutral conformance; **agent read-only**.
- Docs: `CLAUDE.md` (build constitution), `AUTONOMY_CHARTER.md`, `README.md`, `keel.lock`.
- **Reference bench**: Marrow-L1 (Python, green, golden-tested) at `C:\loom\marrow-l1` — diff the Rust core against it + the goldens (the ASTRA-textverse pattern). Don't port its code.
- Substrate (resolved, local): `C:\llama.cpp`, `C:\models` (Qwen3.5-9B + `mmproj-F16`; whisper `large-v3-turbo`; openai privacy-filter), `C:\whisper.cpp`.
- Toolchain: rustc **1.96.0** (a stuck 1.95→1.96 rustup update was completed during setup; `rust-std` re-fetched). **Build from PowerShell/MSVC env, not git-bash.**

## Next — Stage 0 (the spine). Do NOT build it all at once; contract-first, golden/bench-gated.
- **kernel**: ~~manifest · context · registry · chain~~ (slices 1–2 ✓) → **lifecycle + substrate-resolver** · engine · lock
- **invariant middleware** (`crates/keel-middleware`, L3): ~~cost (I4) · audit (I1) · privacy rungs 1–2 (I3)~~ ✓ — deterministic trio complete; privacy **rung 3** (OpenAI Privacy Filter, a verification pass) is Stage 2 behind `GOLDEN_PRIVACY`
- ~~**a (deepseek)** · **c (resolver: probe + launch/supervise)** · **b (serve_openai)**~~ ✓ — the a→c→b trio is complete; KEEL is consumable embedded + over protocol. **Remaining Stage-0:** kernel `engine` (the canonical loop) · `lock` (reproducibility) · `store::sqlite` (the index) · the auto-restart supervision loop (primitives exist; the loop lands with a long-lived host). Then **Stage 1** (router · amplify · perception).
- **one local adapter** (HTTP → llama-server)
- **invariant middleware**: audit (I1) · privacy rungs 1–2 deterministic (I3) · cost (I4)
- **file ledger** (I2) + **SQLite store** (the index)
- **CLI** + **`serve_openai`** (OpenAI-compatible egress)
- Outcome: a binary that resolves the substrate, talks to a tier, logs every call, consumable embedded or over protocol. **Falsifier: > ~2 weeks → rethink the native-core thesis.**
- Then **first cell**: re-home **NightClerk or NightScribe** (controlled experiment). Then SEXTANT.

## Disciplines (don't drift)
- Contracts + goldens are **frozen** (agent read-only). The contract-freeze IS the governance.
- Layer rule: `contracts ← kernel ← {adapters, middleware} ← services ← apps`.
- Five invariants + reversibility gate on every call; **memory-safety = I5 on the source**.
- Size to the **flywheel base case** (worth it even if `escalation_rate` stays flat).
- **Do not mutate the global Rust toolchain** without asking. No sovereign/vector egress. No secret into a LoRA.
- Build sessions follow `CLAUDE.md` §Session-protocol; end with layer-check → budget → golden-freeze (verify unchanged) → `cargo test` green → one commit, one-line intent.
