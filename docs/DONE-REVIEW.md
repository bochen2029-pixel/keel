# E2 — the DONE review (2026-07-10)

*The §3 completion audit, every line verified by artifact this session (gate run, seal read,
index queried, decisions cross-referenced), not by recall. Verdict at the end.*

## 1 · The gate (this session's run)

**178 passed / 7 ignored / 0 failed · clippy 0 warnings whole-tree · seal `db4377b3…` recorded in
`.frozen.json` and re-verified by the green `goldens_match_the_frozen_hash`** (the KEEL-native
freeze gate — a golden content change fails the build). 21 frozen cases / 6 families; the
conformance map (`docs/conformance-coverage.md`, E1 ✓) shows every behavioral joint golden-covered
and every structural joint unit-tested; the only port gap is ToolHost (unbuilt by decision — D3
lands at SEXTANT S4).

## 2 · §3 condition: stages functionally done, every knob DECIDED (not skipped)

| Knob | Decision | Evidence |
|---|---|---|
| **amplify (B1)** | **OFF** — uplift +0.115 < the pre-registered 0.15 | `amplify-n8.json` artifact; kernel loop + bench stay built; keel.lock annotated |
| **reranker (C1)** | **OFF (identity)** — +0.070 recall@5 < 0.10 | ratified golden-recall v2; artifacts in `.keelstate/bench/`; seam+adapter stay built |
| **embedder (C2)** | **the MiniLM floor took the default** (falsifier trip: −0.122 nDCG for the "upgrade") | flip executed + lived (fingerprint rebuild 0→30 vecs); Qwen3 = lock fallback |
| **privacy model (A5/C3)** | **ON — decided 2026-07-10, same day** (exclusion #1 CLOSED): golden `model_adds_span` lived; uplift +0.95 (bar 0.30); 0/10 FP; p95 161 ms | `.keelstate/bench/privacy-c3.json`; keel.lock `privacy.default: on` (egress-only, feature-carrying builds; rungs 1–2 stay the guarantee) |
| **flywheel (B3)** | **base case HOLDS; ignition deferred on evidence** | escalation 0.000 over the 73-turn lifetime; 59-pair corpus; triggers + turnkey pipeline in `docs/flywheel-ignition.md` |
| **rework (C4)** | **PASS** — 0.014 < 0.10, trend improving | lifetime `keel metrics` |
| **economics (C5)** | **KEEL-favorable, re-confirmed at 4× the prelim N**: 72/73 turns routed FREE local, lifetime $0.0004 vs ~$0.01–0.04 for cheap-API-everything (≥ ~96% saved) | lifetime `keel metrics` `by_tier` |
| **recall fingerprint (A3)** | GREEN (CI) + the rebuild-from-ledger mechanic **lived twice** (embed-server cold-start; the C2 flip) | `passes_golden_recall_fingerprint`; the flip artifacts |
| **memory (A7)** | F-M1/2/3 PASS, F-M4 machinery proven (judge-stochasticity residual recorded + mitigated 2-of-3) | `docs/proposals/memory-autopilot.md` §6.5 |

## 3 · §3 condition: the first cell with zero kernel/contract edits

**Exceeded — two cells stand.**
- **D1 NightScribe** (re-homed, C#, over protocol): zero contract/golden edits; three genome bugs
  surfaced and fixed in legal layers (`--jinja` · grammar⇒thinking-off · system-merge).
- **D2 SEXTANT** (greenfield, Python, over protocol): **zero KEEL-side changes of any kind** —
  not contract, kernel, adapter, or app-layer. S1 lived on 5 real postings (full dossiers +
  manifest, 4.0 min, $0, nothing sent); the Truth Gate lived P8 on real data (one REJECT held).

Two languages, both consumption modes' protocol path proven, zero joint edits: **the genome is at
the right altitude** (the §21/§23 boundary falsifier, passed twice).

## 4 · The invariant scorecard (the five + the gate, each live in the running binary)

| Invariant | Status | Lived evidence |
|---|---|---|
| **I1 audit** | GREEN | every call → `.keelstate/audit.jsonl` (fires even on blocked calls); redactions I1-audited (labels, never values); amplify candidates each audited |
| **I2 spine** | GREEN | **73/73 lifetime turns checkpointed** in the SQLite index (queried this session); Tape append-only beside it |
| **I3 sovereign** | GREEN | router **forces** local on sovereign/perception (lived: D1's no-cloud invariant, D2's Canon-bearing turns); egress mask rungs 1–2 + the A4 output rung; vectors/audio/frames never egress; cell PII-filtered views on top |
| **I4 cost** | GREEN | pre-call gate + engine post-call folds (incl. per-amplify-candidate) + daemon per-tick budget; lifetime spend $0.0004 |
| **I5 externalized** | GREEN | verifier in-loop, **both directions lived in-binary**; fail-closed unresolved refs; critical-step config-fault guard; the freeze gate; memory cold-eyes + 2-of-3 vote; the cells' gates are non-model by construction |
| **Reversibility** | GREEN | BLOCK on unstatable undo; scrub-before-distill (no secret into a LoRA); approval-gated dispatch (P7, permanent); one-line rollbacks on every default flip (C2's incumbent kept on disk) |

Layer rule: `contracts ← kernel ← {adapters, middleware} ← services ← apps` — held across every
slice (violations would fail the per-slice layer-check; none recorded since genesis).

## 5 · ISSUES disposition (all eleven)

1 resolved (brute-force decided) · **2 OPEN-ACCEPTED (A5, operator's LAST — see §6)** · 3 resolved
(A6 built) · 4 resolved (B1 decided) · 5 resolved (B3 decided) · **6 OPEN-ACCEPTED (operator
`sha256:` pins; `kernel::lock` verify stays dormant-by-design until pinned — no correctness
exposure: the freeze gate covers the goldens, and substrate files are operator-provisioned)** ·
7 deferred-by-design (no cache trigger) · 8 workaround-proven (file-redirect pattern; root-fix
nice-to-have) · 9 resolved (mask-all-output) · 10 resolved (embed GGUF) · 11 resolved (set
ratified + MiniLM provisioned).

## 6 · The exclusions (explicit, auditable, reversible)

1. ~~**A5 privacy rung-3 + its C3 falsifier.**~~ **CLOSED 2026-07-10, the same day** (operator:
   "go ahead with A5" → "go ahead with the ort build"): built, lived, and **C3 decided ON** under
   the pre-registered thresholds — see §2's row and `docs/proposals/privacy-rung3.md`. The
   guarantee never moved: rungs 1–2 remain the deterministic oracle; rung-3 is additive recall on
   egress, in feature-carrying builds.
2. **ISSUE-6 `sha256:` pins** (operator-only). Until pinned, `kernel::lock` verify stays unbuilt/
   dormant by design. Re-open = the operator pins; the verify slice follows.

## 7 · Verdict (re-stamped 2026-07-10, same day)

**E2: PASS — KEEL is DONE, with ONE remaining exclusion (ISSUE-6, operator-only pins).** Every
stage knob including the privacy model is now **decided, none skipped** — the §3 letter is met in
full on the agent-buildable surface. *(Original verdict below, superseded only in its exclusion
count.)*

**E2: PASS — KEEL is DONE, with the two §6 exclusions explicitly accepted.** Stage 0–3 machinery
built and every decidable falsifier **measured-and-decided under pre-registered thresholds** (C1,
C2, C4, C5, B1, B3, D1, D2 — none skipped); two cells stand on the genome with zero joint edits;
the five invariants + the reversibility gate are live in the running binary at lifetime scale;
the contracts and goldens are exactly as frozen at genesis (seal unmoved through ~90 slices).

Per ROADMAP §3: `keel.lock` `stage:` flips to **stage3**, `.keelstate/DONE` is written (the
supervisor's wind-down-to-polish signal — not a halt), and **the loop enters perpetual-polish
mode (§4)**: A5+C3 first, then code-review/coverage/falsifier-refresh/doc-reconcile cycles.

*The spec stopped being a substitute for a running binary long ago; today the running binary
stopped being a substitute for the finished thing.*
