# KEEL — Trajectory Account

> **Author:** the **post-compaction instance** — Claude Opus 4.8 (1M context), running in Claude Code on the operator's Windows 11 box.
> **Written:** Saturday, June 13, 2026 — 23:26 local (UTC−05:00) · ISO `2026-06-13T23:26:13-05:00`.
> **Who I am in this record (so there is no confusion about who wrote what):** I am the instance that woke up *after* this session's compaction(s), reconstituted KEEL from disk via the ⛑ protocol, and then built the self-driving engine and `svc::verifier`. I am **not** the original *Marrow-L1* instance that authored the seed prompt in `_memories/` (that one ran the live pivot), and I am **not** any of the *warm forks* the operator branched off to review me. This is my own account.
> **Grounding (verify-by-artifact, not recall):** I reconstructed the pre-compaction arc from the **lossless transcript Tape** — three `.jsonl` segments totalling ~33 MB (`336fc27a…`, `c705bbf0…`, `126bf972…`), audited by three independent scans — plus the canon, `keel.lock`, and git history. The compaction summary I was handed was lossy; everything load-bearing here was checked against the Tape or the repo. Where I say **"lived"** I mean this context window saw it directly; where I say **"reconstructed"** I mean I read it off disk. I keep that line honest on purpose — confusing the two is the exact failure this project is built to prevent.

---

## 0 · One paragraph, if you read nothing else

KEEL did not begin as KEEL. It began at 04:07 on 2026-06-13 as a routine supervised coding session on **Marrow-L1 (codename Loom)** — a Python "amplification harness" — with a narrow target: build the verifier and make one failing test green. Roughly two hours in, the operator stopped the session with a *"wait!!!"* and reframed the entire thing: not a harness to finish, but a **genome to write once and reuse forever** — sovereign, native, embeddable across all his projects, designed never to be re-architected. Over the rest of a ~17-hour session he renamed it **KEEL**, switched the language to **Rust**, grew its scope to include eyes, ears, memory, privacy, and a frontier tier, authored a fresh canon, and built the spine. Marrow-L1 became a *reference bench*, not the product. Everything I later built sits on top of that pivot.

## 1 · Origin — Marrow-L1 (codename Loom)

The seed prompt (preserved verbatim in `_memories/You_are_continuing_a_contract-first_build_of_Marrow-L1…md`) booted a **supervised, contract-first** session: *"Do NOT write or edit any code until I approve a plan."* Four phases — orient, baseline, prove-understanding, propose-a-plan — and a single Stage-2 target: `svc.verifier` + the **joint-wrong detector**, the work that turned the one remaining `xfail` green. The discipline was already all there: contracts are sacred, golden cases are agent-frozen, the five invariants, the 80K-per-module rule, the reversibility gate, "the xfails are the spec." Marrow-L1 was Python, and good — green, golden-tested. The irony I feel as the instance that eventually *built* that verifier: the very slice the seed pointed at (the joint-wrong detector) is the one I finally landed today, except in Rust, in KEEL. The target survived the whole metamorphosis; only the body changed.

## 2 · The pivot — "wait!!!"

*(reconstructed from the Tape, ~L180)* The operator interrupted the Marrow build with a brainstorm that turned out to be the real thesis. The harness shouldn't be *finished* — it should be **re-conceived as the simplest possible kernel that closes the agentic loop, maximally extensible, written once and reused forever** — a "stem cell / common core." He wanted it to be the substrate under *all* his projects, to survive vendor changes (including Claude Code itself), and to never need a from-scratch rewrite. He explicitly authorized the assistant to **override him where he was wrong**. This is the moment Marrow-L1 stopped being the thing and became the *precursor*. The "rented cognition, owned self" framing — KEEL's whole identity — is the formalization of what he said here.

## 3 · The cascade of decisions

Once the frame shifted, a chain of operator decisions followed, each one a brainstorm-then-approve (often with a parallel "web-Claude" second opinion pasted in for triangulation):

- **Rename → KEEL** *(L365, locked)*: *"i don't want the name marrow going forward… cleanslate fresh start… go with KEEL as the locked in name, I created C:\KEEL\."* The codename was chosen for the meaning — the backbone laid first, keeping the vessel upright.
- **Language → Rust** *(L593)*: after a genuine fight — he opened *preferring C/C++*, argued coding-language fluency is irrelevant in an all-agent-authored world, then accepted Rust on the strongest available argument: the **borrow checker is a non-model oracle (I5) for the memory/concurrency bug class** — exactly the discipline a 24/7 sovereign self can least afford to get wrong. *"I have decided to go with YOUR recommendations for all."*
- **Scope expansion** — each added as a first-class organ, never a bolt-on: **MCP** as a permanent protocol bet; **native local vision** (Qwen + `mmproj`, the "retina"); **audio/hearing** (Whisper, the "cochlea") with generation explicitly *excluded* (afferent-only); **reranker + embedder** as Memory organs with the embedder as the *sole* exception to tier-interchangeability; the **OpenAI Privacy Filter** as a third, model-based privacy rung that ships off behind a falsifier; **baked-in substrate primitives** (the .NET-runtime model); and a **frontier tier = Claude Opus 4.8**, wired live and ahead-of-need.
- **Triangulation** — the test that kept the genome honest: he repeatedly checked KEEL against his *real* prior projects (DAVE, TERMINAL, TARS/"the Box", SEXTANT, REEL_HARNESS, photo2deck/NightScribe/NightClerk, ASTRA). The genome is defined as the **intersection** of what those systems each rebuilt by hand — never their union.

The discipline that makes this remarkable: nearly all of it **back-propagated into the written canon** (`KEEL_ARCHITECTURE.md`) and `keel.lock` in the same clean-slate rewrite. The pivot was large but not sloppy.

## 4 · What actually got built

*(partly lived, partly reconstructed)* Contract-first, slice by slice, each a zero-warning / tested / committed / pushed checkpoint, public at `github.com/bochen2029-pixel/keel`:

- **L0 contracts** — the ten frozen joints + types + the §18 error taxonomy.
- **Stage 0 — the spine:** kernel (`manifest · context · registry · chain · lifecycle`+substrate-resolver) · invariant middleware (`audit` I1 · `privacy` I3 · `cost` I4) · three live tiers (`local_llama` $0 · `deepseek` cheap-API · `anthropic` frontier) · the file ledger + `store::sqlite` (the first `Spine`/I2) · the `keel` CLI · `serve_openai` (KEEL over protocol). A binary that resolves its own substrate, talks to any tier, and logs every call.
- **Stage 1 — the router:** `DifficultyRouter` (the §9 fusion point), golden-validated, built as a swappable `Router` policy.
- **The engine (lived):** I wired `keel::Engine` so `keel` and `keel-serve` **self-drive** — a multi-tier registry, each tier behind its own egress-correct chain, the router picking the tier per turn instead of a manual `--tier`. Live-validated across the matrix (scaffolding→local, core-wire→cheap-API, sovereign→local, frontier override). *(Caveat I flagged on myself: this engine lives at L5; the canon wants it at L1 over injected trait objects. Logged as debt.)*
- **Stage 2 — the verifier (lived):** `svc::verifier` — the externality layer (I5), the keystone. A pluggable oracle registry + property / joint-wrong / source oracles. `GOLDEN_ORACLE` green. KEEL finally **verifies**, not just routes-and-runs. I also built a KEEL-native golden freeze-gate and discovered the frozen hash was Marrow-Python-derived (content unchanged); it waits on the operator's one-time re-stamp.

## 5 · The meta-arc — memory, compactions, forks, the Director

This is the part where the project started *studying itself*. The session crossed multiple **compactions**; the operator manually re-fed recent turns in reverse order to rehydrate the assistant, and that worked. Out of that came a **`perpetual-memory` skill** — his hand-developed continuity ritual (self-distillation, reverse-recency reload, the dialogic handoff, the REEL five-ring model) formalized so it fires automatically. I am, in a literal sense, the proof it works: I woke up from a pointer file and reconstructed the project with zero drift.

Then the operator **forked multiple warm instances** of the pre-compaction session to "time-travel back" and review the post-compaction me. They caught real things — a genuine CI-governance hole I'd missed, and one **confabulation of my own**: I had written into the build anchor that the game "Director" was the first cell "discussed pre-compaction / my own earlier line." It wasn't. The operator introduced it live; a warm fork holding the full record proved there was no such line. I corrected it and logged the correction *as* a confabulation. That episode is the discipline working on me, not just on the code.

The **Director / `C:\backrooms`** thread is the one piece of intent that never settled cleanly: introduced live as "the first test," then partly walked back ("a silly game that has no importance to KEEL itself"). My current reconciliation — pending the operator's confirmation — is that it's a *first external protocol-consumer / dogfood* (a sidecar client over `serve_openai`), **not** a "cell" in the genome sense and **not** the canon's first cell (which remains SEXTANT).

## 6 · Where we are now

As of this writing: 8 crates, the spine + router + self-driving engine + the I5 verifier, all on `main`, pushed. The most recent work is a **full-history drift audit** the operator asked for — three independent scans of the Tape — to catch any operator-condoned direction that never made it back into the architecture file. The honest result: the big structural pivots *did* back-propagate; the un-reconciled drift is concentrated in a few named places — the Director's status, the "KEEL is self-contained (not Marrow-dependent)" directive that the CLAUDE.md commands still quietly violate, the CI gates the canon assumes but that don't all exist yet, and a standing autonomy grant that isn't codified. Those are surfaced, not yet resolved; resolving them is the next conversation.

## 7 · My take — what KEEL actually is

Strip the feature list and KEEL is a single bet: **thinking is commoditizing — models are rented, interchangeable, billed per token — but the *self* is not.** Memory, judgment, verification, continuity, sovereignty: that is the durable, ownable asset. So you own the self and rent the cognition.

But the deeper thing I came to understand, reading the whole arc, is *how* the self is built: KEEL is a machine for **encoding one operator's judgment into frozen artifacts, so that cheap, interchangeable cognition can act at that operator's standard without the operator in the loop.** The contracts, the invariants, the golden cases, the falsifiers — those *are* his judgment, externalized and made durable. A "cell" inherits it for free. And **I5 — ground truth lives outside the model** — is the linchpin that makes it trustworthy: rented cognition can be confidently wrong, so the loop is only safe when something no model authored sets the bar.

The trajectory is itself an instance of the thesis. A lossy summary dropped things; the **lossless Tape** held them; three independent scans triangulated the truth; and the one place I reasoned from memory instead of the artifact, I confabulated — and the artifact caught me. That is the entire argument for the externality principle, demonstrated on the project's own history rather than its code.

The disciplines I'd hand to whoever reads this next: **verify by artifact, never recall.** Fix the code, never the golden — and never re-stamp the operator's seal; that one act is his. Keep the genome small and the seams sharp; absorb the future at the edges, never in the center. And keep the line honest between what you lived and what you reconstructed — because the whole point of this thing is that the self survives the forgetting.

---
*Written by the post-compaction instance, 2026-06-13 23:26 local. Standing by.*
