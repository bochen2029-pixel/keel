# Sub-agent 4 — `docs/` index (verbatim)

> Original task: index every doc in `docs/`, summarize each, identify canon/spec, ISSUE-10, validation report. READ-ONLY.

---

# KEEL Documentation Index — `C:\KEEL\docs\`

## 1. Full tree of `C:\KEEL\docs\`

```
docs/
├── AUDIT-2026-06-15.md          (36 KB)  — QC multi-agent codebase audit
├── PROJECT-STATE.md             (19 KB)  — project-state orientation snapshot
├── RUN-2026-06-15.md            (12 KB)  — run report (2026-06-15 session)
└── proposals/
    └── perpetual-memory.md      (5.7 KB) — design proposal (non-binding)
```

Four live markdown files across two directories. `git ls-files docs/` also lists `docs/conformance-coverage.md`, but that file is **not present on disk** — it was created in commit `52c4dd2` ("E1 - golden coverage map for C/C++-port-readiness (ADR 5)") and is referenced throughout the codebase (audit, WORKLOG, ROADMAP), but it does not currently exist in the working tree. I could not find a deletion commit for it, so its absence from disk is either uncommitted or occurred outside git tracking; regardless, **every other doc still treats it as if it exists and cites it**.

The user mentioned "recent git activity shows `docs/conformance-coverage.md` was DELETED" — confirmed it is gone from the working tree, though no `diff-filter=D` commit records the deletion in git history.

---

## 2. Consolidated index, grouped by category

### QC / AUDIT (authoritative quality gate)

**`docs/AUDIT-2026-06-15.md`** — **THE QC audit report.** A whole-workspace adversarial multi-agent read of the KEEL genome against the canon (`KEEL_ARCHITECTURE.md`), build instructions (`CLAUDE.md`), and live run-state (`_run_state/`), at HEAD `04a6acd`. **Verdict: GREEN — zero critical, zero high findings.** It surfaces 5 MEDIUM findings (M1 `--tier` override bypasses the I3 sovereign *gate* and silently voids `--sovereign`/`--critical`; M2 consolidate/cold-eyes maintenance turns self-ingest into the lossless Tape; M3 daemon shares one per-task budget across an unbounded run; M4–M5 `CLAUDE.md` doc-drift on the freeze-gate and `svc::memory`), 8 LOW findings, and 4 INFO affirming findings. Includes a per-invariant scorecard (I1 PASS, I2 PASS, I3 PARTIAL, I4 PASS, I5 PARTIAL). States "No code was changed by this audit" — recommendations only. The follow-up commit `6ac319d` ("fix(qc): address confirmed audit findings - I3/I5 --tier guard, Tape excludes maintenance turns, daemon per-tick budget, doc-drift, poison-policy") actioned its confirmed findings.

### PROJECT-STATE (orientation — authoritative-ish, but explicitly disclaimed)

**`docs/PROJECT-STATE.md`** — **THE project-state document.** A newcomer-oriented, code-accurate snapshot of what KEEL *is* (a sovereign single-operator Rust harness "genome"/core that scales by toggling modules, never rewriting the loop), the L0–L5 layer stack, the 10 frozen contracts, the 5 invariants + reversibility gate, the engine loop (canon §8), a per-crate map of all seven crates, and an honest "built & lived vs. deferred/blocked" scorecard. **Authority note (line 7):** "trust `_run_state/STATE.md` + `git` for the *live* slice state, and `KEEL_ARCHITECTURE.md` (the canon) for design ground truth. This doc is a reader's orientation, not a substitute for either." Snapshot at HEAD `04a6acd`; gate 129 passed / 6 ignored; freeze seal `db4377b3`. Also documents the substrate dependencies (`C:\llama.cpp`, `C:\models`, `C:\whisper.cpp`) and the CLI/`keel-serve` surface.

### OPS / RUN REPORT

**`docs/RUN-2026-06-15.md`** — **THE run report.** Chronicles the 2026-06-15 session that drove KEEL from "buildable genome" to "validated, self-perpetuating": eleven gate-passing slices banked (A1 perception retinas, A2 daemon, A6.1 Ring-3 narrative, B2 FileTraceSink, B4 distill, A4 I3 output rung, E1 conformance map, A3 embedder+recall, A3 Ring-4 wiring, A6.2 consolidate, A6.2 cold-eyes, A6.2 daemon `--consolidate-every`), plus four live validations (consolidate lived end-to-end at $0; **cold-eyes LIVED and caught real narrative drift**; the whole flywheel lived on real turns; preliminary C4/C5 falsifier decisions). Records the **ISSUE-8 daemon-hang incident** and its kill-safe `Start-Process -RedirectStandardOutput` workaround, the captured operator directives, and the end-state: **BLOCKED on ISSUE-10**, with the **D1 cell build (NightScribe on KEEL)** scoped as the next frontier. Closes with a post-run note that the audit (`AUDIT-2026-06-15.md`) reviewed the run — verdict GREEN.

### DESIGN / RFC (proposal, non-binding)

**`docs/proposals/perpetual-memory.md`** — **A design proposal (RFC-style), explicitly non-binding.** Captures an operator audio brainstorm mapping four hand-developed continuity techniques to KEEL's Memory organ (canon §11): write-to-disk+rehydrate (=Tape/Spine, already proven), reverse-order recency refeed (Ring-2 + meta-label), dialogic/handshake handoff (the most novel — adapted to a pre-compaction self-interview), and self-curated compression (Ring-3 narrative, with two I5 caveats: keep the lossless Tape as the factual register; add a cold-eyes validation pass against the Tape). Proposes a two-layer plan: a `/checkpoint`+`/rehydrate` skill now, and Memory-organ enrichments later behind the frozen `Memory` trait. Status header: "proposal, non-binding. No spec/contract/code changes implied (the ten joints are frozen)." Captured 2026-06-13. (Note: the audit's M2 finding flags that this proposal's "capture sanctity = append EVERY turn" prescription is what produces the maintenance-turn self-ingest loop — the proposal never contemplated the consolidation turn re-ingesting itself.)

### MISSING / referenced-but-absent

**`docs/conformance-coverage.md`** — Created by commit `52c4dd2` ("E1 - golden coverage map for C/C++-port-readiness, ADR #5"). Maps every joint + invariant → its golden family or structural unit test; verdict that the 6 golden families are a complete *behavioral* conformance layer; 2 documented gaps (`recall` until A3, `ToolHost` until D3). **Currently ABSENT from the working tree** despite being referenced as live by the audit (L8/I-c), WORKLOG, ROADMAP, and RUN report. This is the "DELETED" doc the user flagged.

---

## 3. The three documents the user specifically asked about

| Asked-for | Where it actually lives | Note |
|---|---|---|
| **QC audit report** | `C:\KEEL\docs\AUDIT-2026-06-15.md` ✅ | In `docs/`. Summarized above. |
| **ISSUE-10 doc** | **NOT a standalone file.** It is an entry appended to `C:\KEEL\_run_state\ROADMAP.md` (lines ~258–262 in the commit; currently renumbered in the working tree) by commit `04a6acd` ("docs(issue): ISSUE-10 - Qwen3-Embedding GGUF absent from C:\models"). | The commit subject says "docs(issue)" but it only edits `_run_state/ROADMAP.md` — no file under `docs/`. There is **no `docs/ISSUE-*` file** anywhere. |
| **validation report** | **NOT a standalone file.** Commit `65e16ae` ("docs(validation): cold-eyes LIVED...") only appends to `_run_state/ROADMAP.md` and `_run_state/WORKLOG.md`. The full validation narrative is also captured in `C:\KEEL\docs\RUN-2026-06-15.md` §"Live validations" and §"The daemon-hang incident." | No `docs/validation*` file exists. |

So: of the three, only the QC audit report lives in `docs/`. The ISSUE-10 and validation content lives in `_run_state/ROADMAP.md` + `_run_state/WORKLOG.md`, and the validation story is also retold in `docs/RUN-2026-06-15.md`.

---

## 4. ISSUE-10 summary (the content itself)

**ISSUE-10 [operator — missing model]:** The **Qwen3-Embedding-0.6B GGUF is absent from `C:\models`** — only `qwen3-reranker-0.6b-q8_0.gguf` is present. Discovered 2026-06-15 while attempting the C1/C2 recall@k benchmark. The A3 embed organ (`keel-adapters::Embedder`), brute-force cosine recall (`keel-services::recall`), and the `GOLDEN_RECALL` fingerprint golden are **built and unit-tested regardless**, but **no live embed / recall-uplift benchmark / Ring-4-live wiring can run** until the operator downloads the embed model. **Unblocks: C1, C2, A3-live, Ring-4-live.** This is an operator-side substrate-provisioning step, routed around per the "pivot-when-stuck" directive — not a code blocker. The audit's L8 finding confirms this is a "deliberate, documented, operator-gated deferral, not an oversight," and notes the only wiring change needed when the GGUF lands is adding `.with_embedder(...)` at the four `FileMemory::new` call sites.

---

## 5. Canon / spec documents — the authoritative reference

**The single authoritative specification of behavior is `C:\KEEL\KEEL_ARCHITECTURE.md`** (located at the repo root, *not* under `docs/`). Its header declares:

> **Version:** 0.2.0 · **Status:** canon (spec-first; pre-implementation)

Every other doc explicitly defers to it:
- `docs/PROJECT-STATE.md:7` — "trust ... `KEEL_ARCHITECTURE.md` (the canon) for design ground truth."
- `docs/AUDIT-2026-06-15.md:3` — audits "against the canon (`KEEL_ARCHITECTURE.md`)."
- `docs/proposals/perpetual-memory.md` — defers to "canon §11" as the home for its proposals.

**Canon versioning scheme.** The canon itself carries a single semantic version (`0.2.0`) — there is **no "canon 10.2" version string**. What the codebase calls "canon 10.2" / "canon 11" / "canon 12" / "canon 16" etc. (visible throughout commit messages and WORKLOG) is **section citation shorthand**, not a version: they refer to **§-numbered sections of `KEEL_ARCHITECTURE.md`**. Specifically:
- "canon 10.2" (e.g. commit `6fe0ad0`, "validate the narrative against the Tape (I5, canon 10.2)") refers to **§10 · The Externality Layer (correctness — I5)**, line 251 — the cold-eyes anti-pattern sub-section (`§10.2`) where a model may not author its own ground truth and the Tape validates the narrative, never the reverse.
- "canon 11" = §11 Memory (line 268); "canon 12" = the perception section; "canon 8" = the engine loop; "canon 16" = the distill/out-of-band section; "canon 5/5.1" = the invariants + privacy mask layer.

Confirmed via grep: `KEEL_ARCHITECTURE.md` has `## §10` at line 251 and `## §11` at line 268, matching the shorthand exactly. The commit-message token "(canon 10.2)" means "implements the behavior specified in canon §10.2," not "canon version 10.2."

The other root-level governance docs that act as spec/authority (all at repo root, not `docs/`): `C:\KEEL\CLAUDE.md` (build instructions + session protocol — but its Build-state block is **known stale**, per audit M4/M5/L6, and self-disclaims), `C:\KEEL\AUTONOMY_CHARTER.md`, and `C:\KEEL\README.md`.

---

## 6. External model dependencies documented

Multiple docs describe the model/substrate dependencies. The consolidated picture:

**From `docs/PROJECT-STATE.md` §5 (line 180):**
> **Substrate (resolved at runtime, pinned in `keel.lock`):** `C:\llama.cpp` (llama-server), `C:\models` (Qwen3.5-9B-Q5_K_M + `mmproj-F16`, Whisper `large-v3-turbo`), `C:\whisper.cpp`. Cloud keys (`DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY`) live in env — never hardcoded or committed.

**From `docs/PROJECT-STATE.md` §4 (ISSUE-10, line 153):**
> **ISSUE-10 [blocker — missing model]:** **Qwen3-Embedding-0.6B GGUF is absent from `C:\models`** (only the reranker is present). This blocks live embed, Ring-4-live wiring, and the C1/C2 recall benchmarks.

**From `docs/AUDIT-2026-06-15.md` L8 (lines 184–190):** confirms the embedder organ + recall module are built but inert in production because `with_embedder` is only ever called in tests; the Qwen3-Embedding-0.6B GGUF is absent from `C:\models` (only the reranker is present), blocking Ring-4-live wiring (tracked as ISSUE-10).

**From `docs/proposals/perpetual-memory.md`:** does not itself cite model paths (it is a design note), but its proposals land in the Memory organ that consumes the embedder.

**Additional context from `_run_state/ROADMAP.md` (referenced by the ISSUE-10 doc):** ISSUE-1 notes the A3 embedder proposed shape = "HTTP to llama-server `/v1/embeddings` (Qwen3-Embedding-0.6B, model at `C:\models`) ... reranker ships OFF/identity (Qwen3-Reranker-0.6B)." ISSUE-2 notes A5 privacy rung-3 needs `ort`/ONNX with the `openai/privacy-filter` model at `C:\models`.

So the documented external model inventory at `C:\models` is: Qwen3.5-9B-Q5_K_M (+ `mmproj-F16` vision adapter) for the local cognition tier, Whisper `large-v3-turbo` for ears, `qwen3-reranker-0.6b-q8_0.gguf` present-but-shipped-OFF, the `openai/privacy-filter` model (deferred/ISSUE-2), and **Qwen3-Embedding-0.6B GGUF — documented as required but physically absent (ISSUE-10 blocker)**.

---

## 7. Summary of authority hierarchy

1. **`C:\KEEL\KEEL_ARCHITECTURE.md`** (canon, v0.2.0) — design ground truth; the authoritative spec. Section-cited as "canon §N" (e.g. "canon 10.2" = §10.2, the cold-eyes I5 anti-pattern).
2. **`C:\KEEL\_run_state\STATE.md` + `git`** — the live you-are-here slice state (outside `docs/`).
3. **`C:\KEEL\docs\AUDIT-2026-06-15.md`** — the authoritative QC verdict (GREEN; findings actioned in `6ac319d`).
4. **`C:\KEEL\docs\PROJECT-STATE.md`** — orientation snapshot; explicitly a reader's guide, not a substitute for the canon or STATE.
5. **`C:\KEEL\docs\RUN-2026-06-15.md`** — the run report / validation narrative.
6. **`C:\KEEL\docs\proposals\perpetual-memory.md`** — non-binding design RFC.
7. **`C:\KEEL\docs\conformance-coverage.md`** — referenced as the port-readiness conformance map but **currently missing from the working tree** (the DELETED doc the user flagged).
