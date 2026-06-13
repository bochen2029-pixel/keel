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
- **Golden cases**: ratified + **FROZEN** (`tests/golden/golden.json` + `.frozen.json`, 21 cases / 6 sections). Language-neutral conformance; **agent read-only**.
- Docs: `CLAUDE.md` (build constitution), `AUTONOMY_CHARTER.md`, `README.md`, `keel.lock`.
- **Reference bench**: Marrow-L1 (Python, green, golden-tested) at `C:\loom\marrow-l1` — diff the Rust core against it + the goldens (the ASTRA-textverse pattern). Don't port its code.
- Substrate (resolved, local): `C:\llama.cpp`, `C:\models` (Qwen3.5-9B + `mmproj-F16`; whisper `large-v3-turbo`; openai privacy-filter), `C:\whisper.cpp`.
- Toolchain: rustc **1.96.0** (a stuck 1.95→1.96 rustup update was completed during setup; `rust-std` re-fetched). **Build from PowerShell/MSVC env, not git-bash.**

## Next — Stage 0 (the spine). Do NOT build it all at once; contract-first, golden/bench-gated.
- **kernel**: ~~manifest · context · registry · chain~~ (slices 1–2 ✓) → **lifecycle + substrate-resolver** · engine · lock
- **invariant middleware** (`crates/keel-middleware`, L3): the immediate target is `mw::cost` (I4 gate) to prove the chain carries a real invariant; then audit (I1) + privacy rungs 1–2 (I3)
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
