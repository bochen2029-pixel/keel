# KEEL

*The backbone laid first, running the whole length, that everything is built upon and that keeps the vessel upright in any sea.*

KEEL is a single-operator substrate — the persistent, sovereign **self** that perceives (eyes and ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model to punch above its weight, and grounds every critical output in an assertion no model authored. The model that does the thinking is interchangeable, rented cognition plugged into a tiny stable core.

> The API is rented cognition — stateless, interchangeable, billed per token.
> **KEEL is the self** — persistent, user-owned, sovereign, portable across providers and across years.

**Status:** spec-first, pre-implementation. The canon is finished the moment it stops being a substitute for a running binary.

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
