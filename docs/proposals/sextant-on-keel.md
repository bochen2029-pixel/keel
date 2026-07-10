# SEXTANT on KEEL — the D2 boundary map (scoping slice, 2026-07-10)

*Status: SCOPED — the joint-by-joint map from `C:\SEXTANT\SPEC.md` (v0.1, 2026-06-13, pre-KEEL)
onto as-built KEEL. The build follows in fresh sessions (the D1 pattern). D2's deliverable for
KEEL is the §21/§23 **boundary verdict**: every genome-shaped need comes FROM KEEL unchanged, or
the boundary is wrong and KEEL gets fixed first.*

## 1 · What exists

- **The spec:** `C:\SEXTANT\SPEC.md` — a complete v0.1 design (L0–L10 pipeline: discover → triage
  → research → tailor → gate → stage → dispatch; Canon; Directive; Dossier; Truth Gate; Conductor).
  Written **before** KEEL completed, so it plans its own router (§16), gate (§11), state (§9), and
  conductor (§15) — exactly the pieces the genome now supplies.
- **The repo:** `C:\SEXTANT\` contains only the spec. Greenfield — *the first real build-ON-KEEL
  cell* (canon §17), unlike D1's re-homing.
- **The fleet** (spec §18 reuse targets — ORBIT discovery/auto-fill, APPLY2 precision fill, the
  pre-submit gate, career-ops tailoring logic): locations unverified; resolve via Everything at
  build time. Reuse is periphery-side and does not affect the KEEL boundary.

## 2 · Consumption mode: Python 3.12 **over protocol**

Canon §16 places SEXTANT in the over-protocol family; the fleet it wraps is Python; the spec's
document path (python-docx + Word COM) is Python-native. The cell talks to `keel-serve` (`:7070`)
— every cognition step is one **routed, verified, memory-riding, audited** KEEL turn. The D1
boundary finding generalizes: an over-protocol cell keeps its domain loop and deterministic
periphery client-side; KEEL owns cognition, routing, invariants, and the substrate.

## 3 · The joint map (canon §17 line → as-built mechanism)

| §17 says | As-built mechanism | Verdict |
|---|---|---|
| **Conductor = genome `engine`** | Every stage call runs KEEL's §8 loop inside `keel-serve` (route → chain → verify → checkpoint → Tape). The *pipeline* orchestration (fan-out over Leads, caps, resume, kill-switch) is the cell's client-side loop — domain sequencing, not cognition. | FROM KEEL (per turn) + periphery (sequencing) |
| **Router (Claude/Cerebras/local)** | The spec's §16 table maps onto KEEL's three-tier economy by `kind`: mechanical extraction/scoring → `scaffolding`→local Qwen ($0); research/tailoring writes → `core-wire`→cheap-API; the few highest-value writes → explicit frontier. The spec's Cerebras slot = KEEL's cheap-API tier (tier-interchangeable by design — rented cognition). | FROM KEEL unchanged |
| **Oracle (Truth Gate, `INSUFFICIENT_SOURCE`→human)** | Layered per spec §11: rungs 1–4 (Canon whitelist / derivation / numeric / banned-phrases) are **deterministic client-side periphery** (the D1 rule: deterministic checks live where the data is — the Canon is cell-side, gitignored). Rung 5 (embedding ground) = cosine over KEEL's embed organ (`:8090`, sovereign-local). The `INSUFFICIENT_SOURCE` escape = prompt contract + client detection → `needs_human` (spec §11.6). KEEL-side teeth that ride every turn anyway: **grammar-constrained decode** (every tailored artifact is schema-forced JSON — lived in D1), the I3 redaction rungs as PII backstop, and the audit/Tape record. *Optional future app-layer slice (NOT required for the boundary): `keel-serve --cell-goldens <file>` to let a cell's frozen cases resolve server-side via `critical`+`golden_refs` — legal (L5 config), build only if the cell wants server-side gating.* | FROM KEEL (grammar, I3, audit) + periphery (Canon-side rungs) |
| **Memory (the Canon, factual register)** | Two registers, deliberately distinct: the **Canon** (profile.json / cv.md / narratives — operator-authored ground truth, cell-side, gitignored, immutable during a run) is the *gate's* source of truth; **KEEL's Tape/rings** ride every serve turn automatically (A7 autopilot) and give the run its working/episodic memory for free. No cell-side memory system gets built — that is the point. | FROM KEEL (run memory) + Canon files (operator ground truth) |
| **Store (its SQLite schema)** | The cell's domain DB (spec §9: companies/postings/leads/applications…) is periphery — domain rows, not run-state. KEEL's Spine + audit ledger record every turn KEEL-side. Two append-only records, one per concern. | periphery (domain rows); FROM KEEL (run-state/audit) |
| **ToolHost (Gmail MCP) = D3** | **Deferred to the dispatch phase** (spec P6 — its own LAST phase; fly-before-build P5 + canon §22 anti-speculation). Until then `email` dispatch = `.eml` staging (the spec's own alternative, zero deps, approval-gated anyway). When dispatch lands: `keel-adapters::mcp` implements the frozen `ToolHost` joint (the §3 protocol bet), new dep `rmcp` (the official Rust MCP SDK) vetted then. **D3 builds when the cell pulls it, not before.** | FROM KEEL at P6 (D3) |
| **Vision retina (JD/DOM reading)** | Screenshot/JD-image reading over `serve_openai` `image_url` → native Qwen vision, forced local (I3) — the exact D1 lived path (`KeelBackend`). | FROM KEEL unchanged |
| **I3 / reversibility (PII filter + approval gate)** | `sovereign: true` on every Canon-bearing call (router forces local — the no-cloud invariant ENFORCED by KEEL, as D1 proved); the cell's L1 filtered-view (relevance-filtered Canon per call) stays client-side; KEEL's redaction rungs backstop egress. The **approval gate** (P7: nothing reaches a company without the operator's click) is the reversibility gate made flesh — client-side staging + explicit human dispatch. | FROM KEEL (I3) + periphery (approval UX) |

**The §23 falsifier restated:** if any of the left column forces a `keel-contracts`/golden/kernel
edit, the boundary is wrong — fix KEEL first, in the same change. D1 held with three legal-layer
fixes surfaced; D2 is expected to hold with (at most) app-layer additions (`--cell-goldens` class).

## 4 · Build phases (KEEL-anchored; every phase ends LIVED — spec P5)

- **S0 — the keystone (spec P0):** repo scaffold + Canon schema (template committed; real Canon
  gitignored) + `sextant tailor <jd-file>` — one JD → grammar-constrained tailoring turn over
  serve (sovereign, schema-forced JSON) → client gate rungs 1–4 → DOCX (`python-docx`) → PDF
  (Word COM, Edge-headless fallback; **no Playwright in the document path** — P2). Lived = a
  tailored, gate-clean PDF+DOCX from a real JD in <10 s offline-after-the-turn.
- **S1 — vertical slice (spec P1):** 5 real postings → full Dossiers staged (summary, cv, cover,
  email.md, form_answers, brief, strategy, gate_report, status) + `manifest.md`. Lived = the
  15-minute approval surface exists on real jobs, zero sent. **← the D2 boundary verdict is
  rendered here** (the genome surface fully exercised: route/verify/memory/vision/I3/economics).
- **S2 — discovery breadth (spec P2):** ATS pulls + the off-board company→ATS resolver +
  websearch extraction (extraction turns are `scaffolding`→local).
- **S3 — conductor autonomy (spec P4):** overnight run, caps, resume, kill-switch, digest —
  client-side sequencing over serve; TTL discipline (ISSUE-8 patterns) throughout.
- **S4 — dispatch (spec P6) → pulls D3:** Gmail-draft via KEEL `ToolHost`/MCP (the adapter slice
  lands here), APPLY2/ORBIT bridges, all approval-gated.

S2–S4 are product build; **D2-for-KEEL completes at S1** (the ROADMAP flips D2 on the boundary
verdict, with S2–S4 continuing as the cell's own roadmap).

## 5 · Decisions taken now (decide-and-document)

1. **Repo:** new standalone `C:\SEXTANT\` repo (spec open-question #2 resolved — not folded into
   ORBIT; the fleet is wrapped, not merged). Seeded this slice: git init + README (pointing here)
   + `canon/profile.template.json` + `directives/directive.template.yml`. The real Canon is
   operator-authored and **gitignored** (PII never in a repo; the repo has no remote until the
   operator adds one).
2. **Conductor runtime** (spec open-question #3): the cell's own thin Python loop over serve —
   NOT a Claude-Code-harness dependency (KEEL is the brain; the harness pattern died with the
   spec's pre-KEEL framing). ORBIT's scheduler optional later.
3. **Email dispatch** (open-question #4): `.eml` staging until S4; Gmail MCP via D3 at S4.
4. **Auto-dispatch policy** (open-question #5): ALL tiers approval-gated, permanently (P7 +
   the reversibility gate — an outward-facing send is the least reversible act in the system).
5. **Codename/voice/people-depth** (open-questions #1/6/7): operator's, at his leisure; defaults
   = keep SEXTANT, professional voice, public-sources-only contact discovery.

## 6 · Risks carried forward

Word COM requires an installed Word (verify at S0; Edge fallback is specified) · fleet repo
locations unverified (resolve at S1; worst case the resolver/tailoring build fresh — the spec
already salvages only *logic*, not code) · `rmcp` unvetted (S4 gate) · serve throughput for
overnight fan-out is single-GPU sequential (fine: the spec's wall-clock envelope is hours, and
`scaffolding` turns are ~seconds each; measure at S3).
