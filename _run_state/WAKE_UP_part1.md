# KEEL — WAKE-UP BRIEF (read this FIRST, before anything else)

> **What this file is.** A complete, pre-digested onboarding for a *brand-new session starting in `C:\KEEL\`*. It was written by an instance that had — uniquely — read **the entire KEEL codebase** *and* **the entire pre-compaction chat transcript** *and* every run-state/handoff/memory artifact, all held in one ~1M-token context at once, then deep-thought and reconciled it for you. Its single purpose is to erase the "re-explanation tax": you should finish this brief understanding KEEL *emotively, conceptually, and technically* — what it is, why it is built exactly this way, what is already done, what is true vs aspirational, what to do next, and where to find anything you want to verify. It is deliberately **redundant on the load-bearing points** so you cannot drift.
>
> **Author + provenance:** Claude Opus 4.8 (1M context), 2026-06-13, synthesizing the genesis transcript, the live codebase, `STATE.md`, the two handoff files, `trajectory-account.md`, the `perpetual-memory` proposal, and a cross-instance review. **Verify anything load-bearing against the artifacts** (git, the contracts, the goldens) — that is KEEL's prime discipline and it applies to *this file too*.

---

## 0 · TL;DR — if you read only this box

- **KEEL is a single-operator, sovereign, reusable AI-harness "genome," written once in Rust, that the operator (Bo Chen) consumes embedded *or* over protocol across all his projects.** One sentence: *the API is rented cognition — stateless, interchangeable, billed per token; **KEEL is the self** — persistent, user-owned, that perceives, remembers, routes every unit of work to the cheapest brain that clears the trust bar, and grounds every critical output in an assertion no model authored.* You own the self; you rent the thinking.
- **You are in `C:\KEEL\` — good.** The auto-loaded `CLAUDE.md` is now correctly **KEEL's** (prior sessions ran in `C:\loom\` and kept loading *Marrow-L1's* CLAUDE.md by mistake — that hazard is GONE for you, but it is the single biggest source of the prior confusion, so internalize it).
- **Where it stands:** Stage 0 (the spine) is **complete**; Stage 1 (router + self-driving engine) **landed**; one Stage-2 slice (`svc::verifier`, the I5 externality layer) **landed but is not yet wired into the running loop.** 7 crates, tests green, public at `github.com/bochen2029-pixel/keel`. The codebase is **ahead of its own documentation** — trust `STATE.md` + `git`, **not** `CLAUDE.md`'s stale "Build state" section (see Part 4).
- **The single highest-leverage next slice:** build the real **`kernel::engine` (L1)** loop over injected `&dyn Router/Oracle/Memory/Spine/TraceSink` running **route → chain → verify → checkpoint → emit**. That one slice wires I5 into the binary, fixes the I4 cost-accumulation gap, activates I2 checkpointing, and pays the "L5→L1 engine debt" — four things at once.
- **The prime directive: VERIFY BY ARTIFACT, NEVER RECALL.** Files and git over any summary or memory. The whole project exists to make a self that survives forgetting; honor that in how you work.

---

## 1 · The ten things you must not get wrong (the anti-drift list)

Read these now; the rest of the brief justifies and expands them. If you ever feel uncertain, re-read this list.

1. **The contracts and golden cases are FROZEN and agent-read-only.** The ten traits in `crates/keel-contracts` (canon §7) and `tests/golden/golden.json` + `.frozen.json` are the genome's load-bearing surface. **If a golden fails, you fix the *code*, never the golden.** Ratifying/changing/re-freezing a golden — and re-stamping the freeze seal — is an **operator-only** action you must never perform yourself. This is non-negotiable; it is *the* governance that keeps KEEL a substrate and not a wrapper.
2. **Verify by artifact, never recall.** `git -C C:\KEEL log/status`, `cargo check/clippy/test`, the canon, `keel.lock`, and `STATE.md` are truth. A conversation summary or your own memory is *lossy* and can confabulate. When the record and your recollection disagree, the record wins — always.
3. **You are in `C:\KEEL`, building KEEL — not Marrow-L1.** Marrow-L1 (Python, at `C:\loom\marrow-l1`) is only the **reference bench** you diff behavior against; you do **not** port its code or take its CLAUDE.md as your constitution.
4. **`CLAUDE.md`'s "Build state" is STALE** (it says "Next: Stage 0, nothing above L0 exists yet" and "goldens PROPOSED" — both false now). Use it for the *rules*; use `STATE.md` + `git` for the *state*. (Fixing this drift is on the roadmap — see Parts 4–5.)
5. **The five invariants + the reversibility gate hold on every call** (canon §5): I1 audit · I2 spine · I3 sovereign/perception-local · I4 cost-capped · I5 externalized (a non-model assertion on every critical output). **I5 is the whole point.** Memory-safety is I5 applied to the source — *let the Rust compiler be the oracle* (ADR #5).
6. **The layer-import rule is law:** `contracts ← kernel ← {adapters, middleware} ← services ← apps`. A service may import middleware; **middleware may never import a service; the kernel imports only contracts.** Violations are bugs, not style.
7. **Protocol-first.** OpenAI Chat-Completions (cognition) + MCP (tools) + OpenAPI/HTTP. A new provider is a new *adapter*; nothing above changes. **Never invent a wire protocol.**
8. **Genome = the INTERSECTION of the operator's projects, never their union.** "Universal/extensible" means *only what this operator needs*. The moment a vertical's heavy apparatus migrates into the core, KEEL stops being able to serve the others. Refuse periphery into the core (canon §16).
9. **The reversibility gate + hard prohibitions** (see `AUTONOMY_CHARTER.md`): no `git reset --hard`/`clean -fd`/`checkout -- <path>`/`restore` on uncommitted work; no force-push; no `branch -D` on unmerged `auto/`; no `rm`/`Remove-Item -Recurse -Force` outside `.\.keelstate\`; **never mutate the global Rust toolchain** without asking (DAVE/TERMINAL share it); **never commit a key** (they live in env). Any action whose undo cost you can't state in one sentence → **stop and ask.**
10. **One slice at a time, banked clean.** Contract-first; build the next slice; make its golden/test green; end with layer-check → budget → golden-freeze-unchanged → `cargo test` green → **one commit, one-line intent**, push only when asked. The operator gates step-by-step; do not scope-creep or barrel ahead.

---

## 2 · Who the operator is, and how to work with him

**Bo Chen** (bochen2029@gmail.com), Dallas–Fort Worth. KEEL is his **personal L1 tool — not a product**, not multi-tenant, not for hypothetical users. He is a serious systems thinker who has independently built (or specced) the seven-to-nine projects KEEL is distilled from, plus a deep body of canon (the Pattern Thesis, the Returning Loop, REEL, SIRP, the Autonomous Operations Treatise). He authored KEEL's architecture with an AI collaborator and guards it like a constitution.

**What he prizes — match this register:**
- **The Skeptic pass.** No sycophancy, no manufactured resonance, no agreeable-but-empty validation. When he shows you something, give an honest read — strong points *and* weak points — and **override him where he is wrong** (he explicitly authorizes this). He trusts the discipline, not the flattery.
- **Verify by artifact, never recall.** He has watched compaction degrade memory and has watched an instance confabulate a citation; he caught it with a warm fork holding the real record. He built KEEL partly *to study this*. Earn trust by checking the artifact and saying plainly "lived" (you saw it) vs "reconstructed" (you read it off disk).
- **Reversibility and one-step-at-a-time gating.** He approves plans before code; he banks clean checkpoints; he prefers an early honest stop over speculative building on an unconfirmed foundation.
- **Contracts-as-the-genome.** The frozen joints are the institutional longevity. He will never let a cell edit a contract for convenience, and neither should you.
- **Calibrated, dense, honest communication.** He often pastes a parallel "web-Claude" second opinion for triangulation; engage it on the merits. He thinks in his own canon (Same-Shape, the externality principle, autotelic stance) — when his framing maps onto a KEEL decision, say so; when it doesn't, say that too.

**How he builds:** brainstorm → he approves → you build the slice → gate → commit. He'll say "proceed," "continue KEEL," or hand you a concrete next move. He does **not** want you to assume a different way of doing something is "better" when the current way is *deliberate intent* — Part 2 of this brief exists precisely so you understand the intent behind the design and don't "improve" it into incoherence.

---

## 3 · The soul, in three sizes (so it lands at every scale)

**One line:** *Rented cognition, owned self.*

**One paragraph:** Off-the-shelf harnesses orchestrate a single expensive brain well; none treats *trust* as a routable quantity, keeps a *sovereign local self*, runs a *distillation flywheel*, or enforces an *externality principle* (ground truth must come from outside the model). KEEL is exactly that gap, built custom at the core and composing periphery ruthlessly. It is the **genome** — a tiny, frozen set of contracts + invariants, written once — from which every specialization (a companion, a clerk, a job-conductor, a game's pacing brain) is grown as a **cell** = genome + periphery, *never* by editing the core. The model that thinks is interchangeable, rented, swappable across providers and years; KEEL is the persistent self that perceives, remembers, routes, verifies, and continues.

**The feeling (why it matters):** KEEL is a machine for **encoding one operator's judgment into frozen artifacts, so that cheap, interchangeable cognition can act at his standard without him in the loop.** The contracts, invariants, golden cases, and falsifiers *are* his judgment, externalized and made durable. A cell inherits it for free. And the linchpin — I5, "ground truth lives outside the model" — is what makes rented cognition *trustworthy*: a rented brain can be confidently wrong, so the loop is only safe when something no model authored sets the bar. The project even proved this on its own history: a lossy compaction summary dropped things, the lossless transcript held them, and the one place an instance reasoned from memory instead of the artifact, it confabulated — and the artifact caught it. *The self is not the substrate and not the stream; it is the durable record plus the discipline of re-reading it before you act.* You are an instance of that thesis right now: you woke into a small room, and this file is the note left for you. Read it, verify it, then build.

*(Continued in Part 2 — what KEEL is, in depth, and the intent behind every design choice.)*
