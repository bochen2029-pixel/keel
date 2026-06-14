# KEEL

*The backbone laid first, running the whole length, that everything is built upon and that keeps the vessel upright in any sea.*

KEEL is a single-operator substrate — the persistent, sovereign **self** that perceives (eyes and ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model to punch above its weight, and grounds every critical output in an assertion no model authored. The model that does the thinking is interchangeable, rented cognition plugged into a tiny stable core.

> The API is rented cognition — stateless, interchangeable, billed per token.
> **KEEL is the self** — persistent, user-owned, sovereign, portable across providers and across years.

**Status:** in active build — the spine, the three-tier router, and the I5 externality layer are real and running, consumed embedded (CLI) *and* over protocol (`serve_openai`). Stage 2 (correctness & memory) in progress. The canon is finished the moment it stops being a substitute for a running binary; trust `_run_state/STATE.md` + `git` for live state.

---

## Why KEEL exists — the intent

Across project after project (NightClerk, NightScribe, photo2deck, companions, games), its operator kept re-building the **same AI/LLM substrate by hand** — the same local vision-LLM, the same Whisper ears, the same privacy filtering, the same cheap-vs-frontier routing. KEEL builds that common core **once, from first principles, as scale-invariant and reusable as possible** — and never rewrites it. You can't embed OpenCode or Claude Code inside a game or an app; those are agents you *talk to*, not a substrate you *embed*. KEEL is the substrate: **sovereign, custom-coded, local-first, online-API-capable, intelligently routing between them.**

One frozen core, three destinies — differing only by which modules are toggled on:

1. **An embeddable, reusable AI bundle** for the operator's own apps and games — *the .NET of his AI apps*: build an LLM-infused app or game without re-coding the AI piece each time.
2. **A sovereign personal harness/assistant** — an OpenClaw-class daily-driver, but local-first and grounded by a non-model oracle, not a YOLO cloud agent.
3. **Aspirationally, the orchestration kernel of an intelligence that can run an entire organization** — the same loop, scaled by slotting in more modules.

The unifying property is **scale-invariance and case-agnosticism**: one deliberately simple, frozen kernel loop + ten frozen contracts that reduce to a game's minimal AI module and extend to an org-scale orchestrator **purely by which modules are slotted — never by rewriting the loop**. That is the genome / stem-cell, and that is the name KEEL.

**Concrete defaults** (chosen, not arbitrary; all swappable via the resolver + `keel.lock`): Qwen3.5-9B (native early-fusion vision) local · Whisper ears · OpenAI Privacy Filter + regex · DeepSeek V4 Pro (cheap-API) · Claude Opus 4.8 (frontier) · Qwen3-0.6B embedder/reranker · SQLite index · MCP first-class. *(Rust today; a native C/C++ port is a contemplated future — the language-neutral frozen golden cases make it a re-pass-the-same-goldens exercise, so the language is the most reversible decision in the system.)*

---

## Read this first

- **[`KEEL_ARCHITECTURE.md`](./KEEL_ARCHITECTURE.md)** — the canon. Intent, the three protocol bets, the five invariants + reversibility gate, the layered architecture, **the ten boundary contracts (the joints)**, the closed loop, the router, the externality layer, memory, perception (eyes & ears), the substrate, the module inventory, the refusal list, the staged build plan, and the falsifiers. Everything is here.
- **[`keel.lock`](./keel.lock)** — the reproducible substrate KEEL expects (models, servers, tiers, the resolver order, the ledger/index split).

## The shape, in one screen

```
L5  apps          consume services; expose KEEL (CLI · OpenAI egress · MCP server · embed lib)
L4  services      route · amplify · verify · memory · perception · driver
L3  middleware    audit (I1) · privacy (I3) · cost (I4) · cache        ← invariants, unbypassable
L2  adapters      local_llama(+vision) · whisper(ears) · deepseek · anthropic · mcp · store(sqlite)
L1  kernel        manifest · context · registry · chain · lifecycle(+substrate resolver) · engine · lock
L0  contracts     the ten joints + agent-frozen golden cases
```

**The closed loop:** `route → amplify → verify (externally) → record → recalibrate / distill`. Each verified trace makes the next run cheaper and safer. The proof it ignited: `escalation_rate` trending down.

## Genome, not framework

KEEL is the **genome** — a frozen set of contracts + invariants, written once. Every use is a **cell**: *genome + periphery*, composed, never a core edit. The genome is the *intersection* of seven systems that each rebuilt this skeleton by hand (Tenancy, Terminal, TARS, REEL, the In-Home Companion, SEXTANT, photo2deck/NightScribe/NightClerk) — never their union.

## How it's consumed

- **Embedded** (in-process, native) — Rust/Tauri apps link the core.
- **Over protocol** (any language) — Python/C# apps talk to a KEEL binary via OpenAI-compatible egress, MCP, or HTTP.

One genome; Rust, Python, and C# all consume it.

## Substrate (resolved, not embedded)

KEEL sits *above* inference — it discovers / launches / supervises / routes to whatever the box has (its own `llama-server`/`whisper-server`, an existing LM Studio/Ollama, or cloud), and falls back gracefully. The canonical primitives: **Qwen3.x** (brain + eyes), **Whisper** (ears), an **embedding model** (recall), and **SQLite** (the index). SQLite is baked in; the models are shared system assets pinned in `keel.lock`. The ledger is append-only files (the system of record); the index is disposable SQLite (rebuildable from the ledger).

## Build order (each stage independently useful; falsifier-gated)

0. **Spine** — contracts + kernel + two tiers + invariant middleware + ledger + store + CLI + egress.
1. **Amplification & senses** — constrained decoding, router, best-of-N (ships off), perception (eyes + ears), cache.
2. **Correctness & memory** — golden registry + freeze, verifier + joint-wrong, ringed memory, metrics.
3. **Flywheel** — verified-trace distillation; `escalation_rate` trends down.

**First real cell:** build **SEXTANT** on KEEL. Done = its Conductor/Router/Gate/Canon/State come *from* KEEL unchanged; only job-domain periphery is written. If it forces a kernel edit, KEEL's boundary is wrong — fix KEEL first.

---

*Build Stage 0 first.*
