# Conformance coverage (E1) — is the golden set a complete language-neutral layer?

**Purpose (ADR #5 / canon §7).** The frozen golden cases are KEEL's *language-neutral conformance
layer*: a future C/C++ port is "done" when it **re-passes the same goldens**. This doc maps every joint
+ invariant to its conformance signal, so the port (and any refactor) knows exactly what must hold and
where the gaps are. **Map, not gospel — verify against `tests/golden/golden.json` + the code.**

## Golden families (frozen — `tests/golden/golden.json`, 21 cases / 6 families, seal `db4377b3`)
| family | joint/invariant | green test | notes |
|---|---|---|---|
| `router` | Router (§9) | `router::tests::passes_golden_router` | the fusion point — trust×difficulty, force-local, BLOCK |
| `model_tier` | ModelTier (§7) | `verifier::tests::passes_golden_model_tier` | cost · `json_schema` constrained decode · reasoning replay |
| `oracle` | Oracle / I5 (§10) | `verifier` + `SchemaOracle`/`PropertyOracle`/`GoldenDispatchOracle` | the joint-wrong detector |
| `perception` | PerceptionSource (§12) | `perception::tests::passes_golden_perception` | dHash + VAD change-gate |
| `privacy` | I3 (§5.1) | `privacy::tests` (rung-2 mirrors GOLDEN_PRIVACY) | deterministic redactor; rung-3 model = A5 (behind GOLDEN_PRIVACY) |
| `recall` | Memory recall / Ring-4 (§11) | **conformance-ahead — green when A3 lands** | embedder + `sqlite-vec` index; format-committing (ISSUE-1) |

The freeze-gate (`keel-contracts/tests/golden_freeze.rs::goldens_match_the_frozen_hash`) makes the set
itself a non-model assertion — any change fails the build → fix the code, never the golden.

## Joints golden-covered behaviorally vs structurally code-tested
The goldens cover **behavioral** contracts (the families above). The remaining joints are **structural**
seams whose behavior is pinned by **unit tests** a port re-implements (no golden is needed — there is no
language-neutral *I/O* to freeze, only a shape):
| joint | conformance | port re-verifies via |
|---|---|---|
| Middleware (I1/I3/I4) | code: `mw::{audit,privacy,cost}` tests | re-pass the middleware unit tests (chain order, unbypassable) |
| Spine (I2) | code: `store::sqlite` roundtrip/upsert tests | checkpoint→resume roundtrip |
| Memory (§11) | code: `memory` Tape/assemble/narrative tests + the `recall` golden (A3) | Tape append + ring assembly + Ring-4 golden |
| Driver (§7/§8) | code: `driver` + `engine` select-loop tests | poll/select/tick semantics |
| TraceSink (§8/§5) | code: `trace_sink` scrub+append tests | secrets-scrubbed-before-feedstock |
| ToolHost (MCP, §3) | **unbuilt — D3** (pulled by SEXTANT) | (gap by design; build with the first cell) |

## Invariant coverage (the five + reversibility)
- **I1 audit** — `mw::audit` tests; every call (and redaction) emits an event.
- **I2 durable** — `store::sqlite` + `engine` checkpoint tests; the loop checkpoints each turn.
- **I3 sovereign** — `mw::privacy` request rung (egress) + **output rung (A4, every tier)**; force-local is the Router's (`router` golden).
- **I4 governed** — `mw::cost` hard-stop + `engine` cost-fold tests.
- **I5 externalized** — the `oracle`/`model_tier` goldens + the **freeze-gate** (the governance itself).
- **reversibility** — policy (AUTONOMY_CHARTER) + the TraceSink secret-scrub (no secret → LoRA).

## Verdict
The golden set is a **complete behavioral conformance layer** for the joints that have language-neutral
I/O (router, tier, oracle, perception, privacy, and recall once A3 lands). Structural joints (Spine,
Driver, TraceSink, Memory-mechanics, Middleware) carry no golden by design — a port re-passes their unit
tests. **Only documented gap: `recall` is conformance-ahead until A3, and `ToolHost` is unbuilt until the
first cell (D3).** A C/C++ port that re-passes the 6 golden families + re-implements the structural unit
tests is conformance-complete. *(Revisit when A3 turns `recall` green and when D3 lands ToolHost.)*
