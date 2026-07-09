# _brainstorm\GLM\ — KEEL codebase reconnaissance dump

**Produced:** 2026-06-17
**Mode at time of analysis:** READ-ONLY (no changes were made to KEEL during the survey; this folder is the only write)
**Method:** A fan-out of 6 parallel Explore sub-agents, each covering a distinct partition of `C:\KEEL`, reporting upstream for consolidation.

## What this is

A full "biggest picture" reconnaissance of `C:\KEEL` — what it is, where it's at, and its current status — assembled by delegating six sub-agents to read the whole codebase in parallel, then consolidating their findings into one report.

## Files in this folder

| File | Contents |
|---|---|
| `README.md` | This index. |
| `00_BIG_PICTURE.md` | **The consolidated final report** — the single best starting point. Synthesizes all six agents into the full picture. |
| `01_agent_top-level.md` | Sub-agent 1 verbatim: top-level overview (mission, language, workspace manifest, build tooling, directory layout, current status). |
| `02_agent_run-state.md` | Sub-agent 2 verbatim: `_run_state/` deep read (ROADMAP, STATE, WORKLOG, WAKE_UP, ISSUES, canon refs, working-tree caveat). |
| `03_agent_crates.md` | Sub-agent 3 verbatim: `crates/` map — all 7 crates, public API, dependency graph, binaries, domain glossary with file paths. |
| `04_agent_docs.md` | Sub-agent 4 verbatim: `docs/` index — every doc summarized, authority hierarchy, ISSUE-10, canon versioning. |
| `05_agent_test-build-ci.md` | Sub-agent 5 verbatim: test/build/CI/scripts/daemon/lints/conformance surface. |
| `06_agent_git-history.md` | Sub-agent 6 verbatim: recent commit through-line, current uncommitted diff semantics, the `6ac319d` QC-fix decode. |

## Headline findings (read `00_BIG_PICTURE.md` for the full version)

- **KEEL** = Bo Chen's sovereign, native-Rust AI-harness "genome" — rented cognition, owned self. 7-crate Cargo workspace (Rust 2021, v0.2.0, MIT), strict L0→L5 layer architecture.
- **Committed HEAD `6ac319d` is GREEN** — ~129–130 tests pass / 6 ignored, clippy clean, golden freeze-seal `db4377b3` green, 48-agent QC audit returned GREEN (zero critical/high).
- **⚠️ The working tree does NOT match HEAD and does NOT compile** — an uncommitted, incomplete revert on top of `6ac319d` (deletes `embed.rs` + `conformance-coverage.md`, drops `pub mod recall`, rewrites run-state docs backward) leaves `recall.rs` and `memory.rs`'s Ring-4 references dangling. Plus a stray junk file `nul`. Trust committed HEAD as the real state.
- **One open code-path blocker:** ISSUE-10 — Qwen3-Embedding-0.6B GGUF absent from `C:\models` (blocks live embed/recall benchmarks; routed around).
- **Next major effort:** D1 — re-home NightScribe (C#/.NET) on KEEL as the first real "cell."

## Authority hierarchy (where truth lives)

1. `KEEL_ARCHITECTURE.md` (canon, v0.2.0) — design ground truth.
2. `_run_state/STATE.md` + `git` — live slice state.
3. `docs/AUDIT-2026-06-15.md` — QC verdict.
4. `docs/PROJECT-STATE.md` — orientation (disclaimed).
5. `docs/RUN-2026-06-15.md` — run report.
6. `docs/proposals/*` — non-binding RFCs.

(`CLAUDE.md` = build constitution/rules, but its build-state block is known stale — trust STATE+git for state, CLAUDE.md for rules.)
