# Sub-agent 1 — Top-level KEEL overview (verbatim)

> Original task: survey root docs, workspace manifest, build tooling, directory layout, and the stated mission. READ-ONLY.

---

## What is KEEL — a consolidated summary

### 1. The pitch / mission (in the operator's own words)

KEEL is **"a single-operator substrate — the persistent, sovereign self that perceives (eyes and ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model to punch above its weight, and grounds every critical output in an assertion no model authored."** The model that thinks is interchangeable; KEEL is the self.

The name is a metaphor: *the backbone laid first, running the whole length, that everything is built upon and that keeps the vessel upright in any sea.* (opening line of `C:\KEEL\README.md`).

Two pillars from `C:\KEEL\KEEL_ARCHITECTURE.md` §0:
- *"The API is **rented cognition** — stateless, interchangeable, billed per token. KEEL is **the self** — persistent, user-owned, sovereign, portable across providers and across years."*
- *"KEEL is not an agent and not a loop. It is a **trust-and-cost economy with a tiny immortal core**, inside which a closed verify-and-distill cycle lets the cheapest available brain clear a bar that something no model authored has set."*

The motivating problem (README "Why KEEL exists"): the operator kept re-building the *same* AI/LLM substrate by hand across his own projects (a local vision-LLM, Whisper ears, privacy filtering, cheap-vs-frontier routing). KEEL builds that common core **once, from first principles, as scale-invariant and reusable as possible — and never rewrites it.** You can't embed OpenCode/Claude Code inside a game; KEEL is a *substrate you embed*, not an agent you talk to.

**Three destinies from one frozen core** (differing only by which modules are toggled, never by rewriting the loop):
1. An embeddable, reusable AI bundle for the operator's own apps/games — *"the .NET of his AI apps."*
2. A sovereign personal harness/assistant (OpenClaw-class, but local-first, grounded by a non-model oracle).
3. Aspirationally, the orchestration kernel of an intelligence that can run an entire organization.

The load-bearing property is **scale-invariance and case-agnosticism**: one frozen kernel loop + ten frozen contracts that reduce to a game's minimal AI module and extend to org-scale **purely by which modules are slotted, never by rewriting the loop.** KEEL calls itself the **"genome"** (frozen core) from which every specialization (a **"cell"**) is grown.

**Domain confirmation** (the terms you guessed at, all real and defined in the canon §4 glossary):
- **Tape / Spine / ledger** — the append-only, lossless, human-readable file that is the system of record; "the ledger is the Tape, and the Tape is the Spine." The SQLite **index** is derived, disposable, rebuildable from the ledger.
- **Memory** — ringed, budgeted assembly (Rings 0–4: soul, calibration, working, compressed narrative, retrieved) + lossless ledger + model-authored consolidation. Two registers: **narrative** (lossy, voice-shaped) vs **factual** (lossless, externalized) — never conflated.
- **Embeddings / recall** — a Memory *organ*, not a tier; format-committing (the index carries an embedder fingerprint; mismatch ⇒ rebuild-from-ledger). Default Qwen3-0.6B, with a MiniLM CPU floor.
- **Agents / Driver** — the initiative seam: `poll() → Option<Step>`. Three default drivers: UserTurn, Heartbeat, Watch. This is what turns KEEL from "responds" into "a self that acts."
- **QC / conformance** — the "externality layer" (I5): every critical output carries ≥1 assertion **no model authored**. Operator golden cases are **agent-frozen** (language-neutral conformance layer; the agent may never edit them). A freeze-gate test re-hashes them vs `.frozen.json` and fails the build on any change. `JOINT_WRONG` = code + its own tests agree but a golden disagrees (the most dangerous failure). The borrow checker is explicitly called "a non-model oracle (I5) for the memory/concurrency bug class" — that's the Rust-over-C/C++ deciding argument.

### 2. Language / ecosystem

- **Rust**, edition **2021**, native single-binary core (ADR #5: "native Rust from Stage 0, decided; not earned-into-later"). Target `x86_64-pc-windows-msvc`, rustc **1.96.0**.
- A native **C/C++ port is a contemplated future** — explicitly called "the most reversible decision in the system" because the frozen golden cases are a language-neutral conformance layer, so any reimplementation merely re-passes the same goldens.
- Polyglot *above* the core: Rust embeds in-process; Python/C# consume over HTTP / MCP / OpenAI-egress. One genome, three+ languages.
- A read-only Python reference bench ("Marrow-L1" at `C:\loom\marrow-l1`) is diffed against — never linked or ported.
- License **MIT**, author **Bo Chen** (Dallas–Fort Worth). Workspace version **0.2.0**. Public repo: `https://github.com/bochen2029-pixel/keel.git`, on branch `main`.
- Substrate (resolved at runtime, not embedded): `C:\llama.cpp` (llama-server), `C:\models` (Qwen3.5-9B-Q5_K_M + mmproj-F16, whisper large-v3-turbo, OpenAI privacy-filter), `C:\whisper.cpp`. Cloud keys `DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY` in env (never committed).

### 3. Workspace manifest (`C:\KEEL\Cargo.toml`)

```toml
[workspace]
resolver = "2"
members = ["crates/keel-contracts", "crates/keel-kernel", "crates/keel-middleware",
           "crates/keel-adapters", "crates/keel-store", "crates/keel-services", "crates/keel"]

[workspace.package]
edition = "2021"
version = "0.2.0"
license = "MIT"
authors = ["Bo Chen"]
```

Seven crates, one per architecture layer (L0→L5):

| Crate | Layer | Role (from its `description`) |
|---|---|---|
| `keel-contracts` | L0 | "the frozen joints: core types + the ten contracts (the genome's load-bearing surface)." Imports nothing. |
| `keel-kernel` | L1 | "the spine: manifest · context · registry. Imports only L0." (also `chain`, `lifecycle`+resolver, `engine`, `lock`) |
| `keel-middleware` | L3 | "invariant middleware (I1 audit · I3 privacy · I4 cost) on the kernel chain." |
| `keel-adapters` | L2 | "tier adapters under OpenAI Chat Completions (local_llama; deepseek/anthropic/whisper next)." (+ `mic`/`screen` capture organs, feature-gated) |
| `keel-store` | L2 | "the index (SQLite, bundled, behind a Store seam): backs the Spine (I2 checkpoint/resume)." |
| `keel-services` | L4 | "services (route · amplify · verify · memory · perception · driver). Composes adapters under middleware." |
| `keel` | L5 | "apps: the daily-driver CLI and the OpenAI-compatible server, sharing one wiring lib." (`keel` + `keel-serve` bins; `default-run = keel`) |

**Layer-import rule (enforced as a bug, not style):** `contracts ← kernel ← {adapters, middleware} ← services ← apps`. Dependencies point strictly down; the kernel imports only contracts; middleware may never import a service.

### 4. Build system & tooling

- **Cargo workspace** (above). The gate is `cargo check && cargo test && cargo clippy`, run from **PowerShell / a native MSVC dev shell** (not git-bash — it mangles `$LASTEXITCODE` and false-fails cargo). No `justfile`, no `Makefile`, no `deny.toml`, no `rustfmt.toml`, no `clippy.toml`, no `rust-toolchain.toml` — all confirmed absent (a `rust-toolchain.toml` pin is explicitly deferred per the autonomy charter, since it touches the global toolchain other projects share).
- `.gitignore` (`C:\KEEL\.gitignore`) ignores `/target`, `/.keelstate/` (the runtime ledger — never committed), `/_memories/` and `chunker/*_chunks/` (these hold the genesis transcript with the operator's **plaintext API keys** — flagged as "NEVER commit these"), and OS cruft.
- **`keel.lock`** (`C:\KEEL\keel.lock`) — the reproducible-substrate pin: model ids + (TODO) sha256 hashes, server paths/endpoints, tier prices ($/1M tokens: local $0, cheap-API DeepSeek $0.435/0.003625/0.87, frontier Opus $5/0.5/25), the resolver order, and the ledger/index split. A fresh box is provisioned from it.
- Two non-Rust helper scripts: `tools\keel-autoloop.ps1` (a respawn supervisor that self-perpetuates autonomous build sessions) and `chunker\chunker.py` + `chunk.cmd` (a self-contained token-aware document chunker for reading files bigger than the context window).
- A **golden-freeze gate** test (`crates/keel-contracts/tests/golden_freeze.rs`) re-hashes `tests/golden/golden.json` against `tests/golden/.frozen.json` (seal `db4377b3…`, operator-re-stamped 2026-06-14) and **fails the build on any golden content change**. 21 cases across 6 families: ROUTER, ORACLE, PERCEPTION, MODEL_TIER, RECALL, PRIVACY.

### 5. Directory layout (top 2 levels)

```
C:\KEEL\
├── Cargo.toml              # workspace manifest (7 members)
├── Cargo.lock              # locked deps
├── keel.lock               # the reproducible substrate pin (models, servers, tiers, prices)
├── README.md               # the pitch / "Read this first" / shape-in-one-screen
├── KEEL_ARCHITECTURE.md    # THE CANON (~600 lines: §0–§23, the source of truth)
├── CLAUDE.md               # build constitution / session instructions for the coding agent
├── AUTONOMY_CHARTER.md     # supervised-autonomy rules + reversibility-gate prohibitions
├── .gitignore
├── crates/                 # the 7 workspace crates (one per layer L0–L5)
│   ├── keel-contracts/     # L0 — the ten frozen joints + types + error taxonomy
│   ├── keel-kernel/        # L1 — manifest, context, registry, chain, lifecycle+resolver, engine, lock
│   ├── keel-middleware/    # L3 — audit (I1), privacy (I3), cost (I4)
│   ├── keel-adapters/      # L2 — local_llama, deepseek, anthropic, whisper, openai(shared), mic, screen, wav
│   ├── keel-store/         # L2 — SQLite (bundled) index behind a Store seam; first Spine impl (I2)
│   ├── keel-services/      # L4 — router, verifier, memory, perception, recall, driver, distill, trace_sink
│   └── keel/               # L5 — CLI (`keel` bin) + OpenAI-egress server (`keel-serve` bin) + shared wiring lib
├── tests/
│   └── golden/             # golden.json + .frozen.json — agent-frozen, language-neutral conformance
├── docs/
│   ├── PROJECT-STATE.md    # current build state
│   ├── AUDIT-2026-06-15.md # multi-agent QC audit
│   ├── RUN-2026-06-15.md   # a run report
│   └── proposals/perpetual-memory.md  # the memory-organ enrichment proposal
├── _run_state/             # the agent's own run memory / reconstitution anchors (NOT the KEEL runtime Tape)
│   ├── STATE.md            # live authoritative build-state anchor ("trust STATE.md + git")
│   ├── WORKLOG.md, ROADMAP.md, WAKE_UP.md (+ parts 1–5), GENESIS-ARC.md, INIT_PROMPT.md,
│   ├── OPERATOR_DIRECTIVE.md, AUTOSTART.md, AUTOSTART_SETUP.md, SESSION-ACCOUNT-…, trajectory-account.md,
│   └── handoff/            # forward-arc.md + recent-turns.md (the narrative register for handoff)
├── .keelstate/             # the KEEL runtime ledger + index (gitignored; the actual Tape/Spine at runtime)
│   ├── audit.jsonl, tape/tape.jsonl, tape/tape.narrative.md, traces/, index.db, llama-server.log, …
├── _memories/              # gitignored — the genesis transcripts (contain plaintext API keys; SECRETS)
├── chunker/                # standalone Python token-aware document chunker (chunker.py, chunk.cmd, README.md)
│   └── _keel2_chunks/, _transcript_chunks/, _abc_marrow_chunks/  # gitignored chunk outputs
├── tools/
│   └── keel-autoloop.ps1   # the autonomous-session respawn supervisor
└── target/                 # cargo build artifacts (gitignored)
```

(Note: there is **no root `src/`** — all Rust source lives under `crates/*/src/`.)

### 6. Current status (from `_run_state\STATE.md` + the canon)

The canon (v0.2) is the spec; the binary is the proof. Build status as of the latest commits:
- **Stage 0 (spine): complete** — three-tier economy (local Qwen3.5-9B · cheap-API DeepSeek-V4-Pro · frontier Claude Opus 4.8) through one invariant chain, file ledger + SQLite Spine (I2), self-resolving substrate (probe → launch → cloud → embedded-tiny → fail), consumable **embedded** (CLI) **and over protocol** (`serve_openai`).
- **Stage 1 (amplification & senses): largely landed** — `DifficultyRouter` (GOLDEN_ROUTER green), self-driving `kernel::engine` (the §8 loop over injected joints), perception change-gates (dHash frames / VAD audio), Whisper ears, `xcap` screen eyes feeding the native Qwen vision, the `Driver` seam + daemon select-loop.
- **Stage 2 (correctness & memory): largely complete** — `svc::verifier` (I5, GOLDEN_ORACLE green) wired into the running loop; ringed `svc::memory` (Tape + Rings 0/2/3/4 + narrative register + consolidate/cold-eyes); metrics as an off-loop SQLite reader (`keel metrics`); the golden freeze-gate active. **Still ahead: privacy rung-3** (the ONNX OpenAI Privacy Filter — the operator's stated last item).
- **Stage 3 (flywheel): started** — `FileTraceSink` (secret-scrubbed distill corpus) + `distill-export` landed; out-of-band LoRA training deferred.
- The first real "cell" proof is to re-home **NightClerk/NightScribe**, then build **SEXTANT** on KEEL — if a cell ever forces a kernel/contract edit, the boundary is wrong and KEEL gets fixed first.

The repo is currently under an **autonomous self-perpetuating build loop** (per `OPERATOR_DIRECTIVE.md` and `tools/keel-autoloop.ps1`), with the operator's directive to prioritize real work over meta-work, never ask questions, and round-robin the roadmap. The most recent commits (`6ac319d fix(qc)...`, `f9c031a docs(qc): multi-agent codebase audit...`) show an active multi-agent QC pass addressing I3/I5 guards, Tape exclusion of maintenance turns, and doc-drift.

---

### TL;DR

KEEL is a **single-operator, sovereign, native-Rust AI-harness substrate** ("the genome") — not an agent or a loop, but a *trust-and-cost economy with a tiny immortal core*. It perceives (eyes = native Qwen vision, ears = Whisper), remembers (a ringed memory over a lossless file Tape + disposable SQLite index), routes every step to the cheapest of three tiers (local / cheap-API / frontier) that clears a trust bar, and grounds every critical output in a non-model oracle (frozen golden cases + property/deterministic gates), with a verified-trace distillation flywheel meant to drive `escalation_rate` down. Built as a 7-crate Cargo workspace (contracts → kernel → middleware/adapters/store → services → apps), edition 2021, v0.2.0, MIT, public at `github.com/bochen2029-pixel/keel`. The build is the gate (`cargo check && cargo test && cargo clippy` from PowerShell); the frozen ten contracts + 21 golden cases are the governance. The mission is to build this common AI core *once* and reuse it across the operator's apps/games, his personal harness, and aspirationally an org-scale orchestrator — *purely by which modules are toggled, never by rewriting the loop.*
