# KEEL — Project State

> **Audience:** a newcomer to the codebase. This is the honest, code-accurate snapshot — what KEEL *is*, how it's layered, what's actually built and lived vs. deferred, and how to run it.
>
> **Snapshot point:** HEAD `04a6acd`. Verified gate at this commit: `cargo test --workspace` → **129 passed / 6 ignored**; `cargo clippy` clean. Golden freeze seal `db4377b3` (the `goldens_match_the_frozen_hash` gate is **green and active**).
>
> **Authority note:** trust `_run_state/STATE.md` + `git` for the *live* slice state, and `KEEL_ARCHITECTURE.md` (the canon) for design ground truth. This doc is a reader's orientation, not a substitute for either.

---

## 1. What KEEL is

KEEL is Bo Chen's personal, sovereign, reusable **harness core** — the *genome* from which every specialization is grown as a *cell*. It is the persistent **self** that perceives (eyes + ears), remembers, routes every unit of work to the cheapest brain that clears the trust bar, amplifies a small local model, and grounds every critical output in an assertion **no model authored**. The model that thinks is interchangeable — *rented cognition*; KEEL is the self that owns it. It is a native Rust core consumed **embedded** (in-process) *or* **over protocol** (HTTP + MCP + OpenAI-egress) — the *.NET-of-my-AI-apps* reuse model. The whole thing is **one frozen kernel loop + ten frozen contracts** that scale from a game's minimal embeddable AI module up to an org-scale orchestrator **purely by which modules are toggled, never by rewriting the loop** — scale-invariant by design, case-agnostic, an L1 personal tool (no multi-tenancy, no hypothetical users).

---

## 2. Architecture

### The L0..L5 stack (layer rule, canon §6)

Dependencies point **down**; layer *N* never imports *N+1*. Static-analysis enforced — violations are bugs, not style. Every crate below has been audited and the rule **holds**.

```
L0  contracts   keel-contracts     pure types + 10 traits, zero runtime logic
L1  kernel      keel-kernel        the loop runner; imports only contracts
L2  adapters    keel-adapters      tier/organ shims (OpenAI-wire + Anthropic)   ┐ same layer,
L2  middleware  keel-middleware    I1/I3/I4 invariant chain                      ┘ neither imports the other
L2  store       keel-store         SQLite Spine (I2) + off-loop metrics
L4  services    keel-services      router, verifier, perception, memory, driver, trace, distill, recall
L5  apps        keel               wiring lib + `keel` CLI + `keel-serve` HTTP
```

Rule of thumb: `contracts ← kernel ← {adapters, middleware} ← services ← apps`. A service may import middleware; middleware may **never** import a service; the kernel imports only contracts; contracts import nothing.

### The 10 contracts (canon §7 — frozen, all async, language-neutral)

The genome's sacred joints. Never rewrite a joint to ease an impl.

| Trait | Role |
|---|---|
| `ModelTier` | generate cognition (incl. multimodal); reports `caps` |
| `ToolHost` | MCP client: list / call |
| `Middleware` (+ `Next`) | handle-with-next-chain; I1 / I3 / I4 live here |
| `Router` | route step → `Decision` (sync, rules-only — not a model call) |
| `Oracle` | verify output vs golden cases → `Verdict` |
| `Memory` | assemble ringed context / record → ledger / consolidate → a maintenance `Step` |
| `Spine` | checkpoint / resume — I2 |
| `Driver` | poll → optional `Step` (initiative / heartbeat) |
| `TraceSink` | emit verified trace — the flywheel feed |
| `PerceptionSource` | percepts stream — afferent eyes + ears |

Core types include `Step` (kind `Scaffolding|CoreWire`, `trust_required`, `data_class Normal|Sovereign|Phi`, `critical`, multi-part `content`, `golden_refs`), `Decision{tier|"BLOCK", effort, reason}`, `Verdict{passed, failures, joint_wrong, evidence}`, `Percept`, `Context`.

### The 5 invariants + reversibility gate (canon §5 — enforced every call)

- **I1 Observable** — one audit event per call, in the middleware chain, unbypassable.
- **I2 Durable** — state persists to Spine + ledger; resume from checkpoint.
- **I3 Filtered / Sovereign** — a *gate* (force `local` for Sovereign/PHI/raw perception incl. embeddings; in the router) + a *mask* (scrub PII before egress; in the chain).
- **I4 Governed** — per-task cost tracked + hard-stopped, in the chain.
- **I5 Externalized** — every critical output carries ≥1 assertion **no model authored** (the Oracle loop stage). Memory safety is I5 applied to the source — the borrow checker is the non-model oracle (ADR #5).
- **Reversibility gate** — undo cost unstatable in one sentence → stop/ask. Fails toward: irreversible = blocked; leak-uncertain = local.

### The engine loop (canon §8)

One canonical cycle, lived in `keel-kernel::engine`:

```
poll driver → memory.assemble → router.route (→ tier or BLOCK)
  → [amplify? OFF] chain.run (adapter.generate terminal — I1/I3/I4 happen here)
  → verifier.verify (I5; flags JOINT_WRONG) → spine.checkpoint (I2)
  → if passed: trace_sink.emit
```

Invariants live in the **chain + spine**, not the engine, so exotic cells can compose their own loop and still inherit them. One egress-correct chain *per tier*, resolved by `Decision.tier`. `amplify` (best-of-N) ships **OFF** (ADR #11 — a hypothesis, not an assumption).

---

## 3. Crate map

Seven crates. Each was audited; all respect the layer rule; clippy is clean across the workspace.

### `keel-contracts` (L0) — the frozen genome surface
- **Purpose:** pure type/trait crate, no runtime logic except two helpers.
- **Key modules:** `errors` (`KeelError` 12-variant taxonomy + `code()`/`message()`), `traits` (the 10 traits + `Next` + `PerceptStream` alias), `types` (all enums/structs + the only logic: `compute_cost`, `CostAcc::add`, `Context::budget_remaining`).
- **Public API:** the 10 traits; types `Message, Effort, Usage, Price, Capabilities, ToolCall/Def/Result, GenerateRequest/Result, Step, Decision, StepOutput, Assertion, Verdict, GoldenCase, Percept, SampleSpec, TokenBudget, AssembledContext, Trace, VerifiedTrace, CostAcc, Context`.
- **Deps:** no internal `keel-*` deps — compiles standalone. External: `async-trait`, `serde`, `serde_json`, `futures-core`; dev-only `sha2` (keeps L0 runtime-pure).
- **Tests:** 1 integration test — `goldens_match_the_frozen_hash` (the §3/§10 freeze gate, **live and green**, seal `db4377b3`). 0 ignored.
- **Notes:** `compute_cost` is overflow-safe (`saturating_sub`). One doc-worthy ambiguity: `Percept.confidence: f32` defaults to `0.0`, so "no score given" is indistinguishable from "lowest confidence."

### `keel-kernel` (L1) — the loop runner
- **Purpose:** the genome's spine; constructs none of the trait objects, only runs them.
- **Key modules:** `manifest` (YAML `keel.lock` → typed `Manifest`/`TierCfg`/`RouterCfg`/`CostCfg`), `context` (`new_context`/`new_trace_id`/`now_millis` — the kernel stamps clock + trace so L0 stays clock-free), `registry` (tier-name → `Arc<dyn ModelTier>`), `chain` (the middleware onion, terminating at `ModelTier::generate`), `lifecycle` (substrate resolver: `probe`/`resolve_endpoint`/`launch`/`LlamaServer`), `engine` (`Engine`/`EngineConfig`/`Outcome`/`TierSlot` + `select`/`request_from_step` + the `tick`/`run_until_idle` driver loop).
- **Deps:** only `keel-contracts`. External: serde, serde_json, serde_yaml_ng, uuid (v4), async-trait.
- **Tests:** 39, 0 ignored (manifest 3, context 3, registry 2, chain 4, lifecycle 5, engine 22).
- **Notes:** `now_millis` maps a pre-epoch clock failure to `0` (low-risk); `launch` is intentionally synchronous (`thread::sleep`) — call it pre-runtime, not on an executor thread.

### `keel-adapters` (L2) — tier & organ shims
- **Purpose:** the cognition/sense shims, all speaking OpenAI Chat Completions except Anthropic.
- **Key modules:** `openai` (shared request/response mapping: `base_body`, `parse_response`, `OaiResponse`), `LocalLlama` / `DeepSeek` / `Anthropic` (`ModelTier` impls), `Embedder` (Memory organ, `/v1/embeddings`), `Whisper` (shells `whisper-cli`), `wav` (`write_wav_i16`, no-dep RIFF writer), `Microphone::capture` (`mic` feat, cpal), `ScreenCapture::grab` (`screen` feat, xcap).
- **Deps:** only `keel-contracts`. External: `async-trait`, `serde`, `serde_json`, `reqwest` (rustls-tls); optional `cpal`→`mic`, `xcap`→`screen`.
- **Tests:** ~14 unit + 1 golden runner, all pass without network. **6 `#[ignore]`** live tests (need real endpoint/key/device): the three tier `live_generate`s, `whisper::live_transcribe`, `embed::live_embed`, `mic::live_capture`, plus `screen::live_grab`.
- **Notes (known smells):** `Whisper::transcribe` runs a blocking `Command::output()` inside an `async fn` (no `spawn_blocking` yet — documented refinement); the cpal callback uses `.lock().expect(...)` (panic-on-poison in a real-time path); `embed.rs` silently drops non-numeric array elements.

### `keel-middleware` (L2) — the invariant chain
- **Purpose:** the structurally-unbypassable invariant middleware on the kernel `Chain`.
- **Key modules:** `audit` (I1 — `AuditMiddleware`, `AuditEvent`, `AuditSink`, `FileAuditSink` append-only JSONL), `privacy` (I3 — `Redactor` rung-1 operator markers + rung-2 regex for email/ssn/`sk-`/`AKIA`/Luhn cards; `PrivacyMiddleware` scrubs request text on egress and response text on every tier; emits `REDACTION` audit events with labels, never values), `cost` (I4 — `CostMiddleware` pre-call gate, returns `BudgetExceeded` below the floor; ungated when `task_budget` is `None`).
- **Deps:** `keel-contracts`, `keel-kernel`. External: `async-trait`, `serde`, `serde_json`, `regex`.
- **Tests:** 15 unit (audit 2, cost 3, privacy 10), 0 ignored.
- **Notes:** the only prod `expect` is `Regex::new(...).expect("static pattern is valid")` on compile-time constants. The `GOLDEN_PRIVACY` rung-3 referenced in docs is **not yet present** here.

### `keel-store` (L2) — SQLite Spine + metrics
- **Purpose:** the I2 durable/resumable run-state store + a derived off-loop metric rollup. Single table `runs(run_id PK, state, updated_at)`.
- **Public API:** `SqliteStore::open(path)` / `::in_memory()`; `impl Spine` (`checkpoint` upsert, `resume` → `Option<State>`); `SqliteStore::metrics()` → `MetricsSummary{turns, rework_rate, escalation_rate, by_tier, total_cost}`.
- **Deps:** only `keel-contracts`. External: `async-trait`, `serde_json`, `rusqlite` (feature `bundled`, ~1MB).
- **Tests:** 6 unit, 0 ignored (roundtrip, upsert, unknown-run, metrics rollup, empty-store, skip-non-Trace).
- **Notes:** three `self.conn.lock().unwrap()` sites (panic-on-poison, standard rusqlite-sync pattern, no lock-across-await). Two acknowledged watch-items: metrics collapse one row per `run_id` (correct only while each turn is its own run), and `is_wiring_fault` is substring-matched on engine alarm prose (fragile coupling, flagged in-code).

### `keel-services` (L4) — the default services
- **Purpose:** the default services composed from contracts, run under middleware.
- **Key modules / API:**
  - `router::DifficultyRouter` (`Router::route`) — rules-only tier pick (perception/sovereign→local, cost→BLOCK, kind+escalation climb the ladder).
  - `verifier` — `Verifier` (pluggable `Oracle` registry, folds AND/OR), `PropertyOracle`, `GoldenOracle` (joint-wrong detector), `SourceOracle`, `SchemaOracle` (Draft-2020-12 pinned, fail-closed), `GoldenDispatchOracle`.
  - `perception` — `ChangeGate`/`FrameGate` (dHash dedup, VAD), `see`/`hear`/`listen`/`listen_from_samples`/`resample_to_16k`/`percept_from_transcript`; `listen` (`mic`), `see_screen` (`screen`).
  - `memory::FileMemory` (`Memory`) — Tape JSONL ledger, Ring-0/2/3/4 assembly, narrative + cold-eyes/consolidate prompts, optional embedder recall.
  - `driver` — `UserTurnDriver`/`HeartbeatDriver`/`WatchDriver` (`Driver::poll`).
  - `trace_sink::FileTraceSink` (`TraceSink`) — scrub-then-append distill corpus.
  - `distill::{training_pair, export_training_jsonl}`; `recall::{Embed, Fingerprint, cosine, recall_top_k, should_rebuild}`.
- **Deps:** `keel-contracts`, `keel-adapters`, `keel-middleware`. External: `async-trait`, `regex`, `serde_json`, `jsonschema` (no network/$ref egress). Features: `mic`, `screen`.
- **Tests:** ~33, **3 ignored** (all live perception: `live_hear_transcribes_voiced_audio`, `live_listen_captures_and_transcribes` (`mic`), `live_see_screen_grabs_and_emits` (`screen`)).
- **Notes:** inconsistent poison policy — `driver` uses `.lock().expect("...poisoned")` (panics) while `memory`/`trace_sink` use `unwrap_or_else(|p| p.into_inner())` (recover); low severity (no await held), worth unifying.

### `keel` (L5) — wiring lib + apps
- **Purpose:** the injection layer: reads `keel.lock`, builds concrete adapters/services as L0 trait objects, hands them to the kernel loop.
- **Targets:** `src/lib.rs` (shared wiring), `src/main.rs` (the `keel` CLI), `src/bin/keel-serve.rs` (the OpenAI HTTP server).
- **Public API (lib):** `Engine` (wraps the kernel `Engine`; `assemble`/`available`/`run`/`run_on`/`tick`/`run_until_idle`), `Assembled` + `assemble()` (single-tier `--tier` path), `Outcome` re-export, path consts (`AUDIT_LEDGER`, `INDEX_DB`, `TAPE_PATH`, `TRACES_PATH`, `GOLDEN_PATH`), helpers `watch_token`, `daemon_perpetual`.
- **Deps:** all six `keel-*` crates. External: `tokio` (rt-multi-thread, macros, net), `axum`, `serde`, `serde_json`.
- **Tests:** 4 total, none ignored (lib 2: `watch_token...`, `daemon_perpetual_rules`; serve 2: `parses_i5_and_grammar_extensions`, `extensions_default_when_absent`).
- **Notes:** `keel-serve.rs` ends server-run with `.await.unwrap()` (panic on a fallible path vs. the clean `fail()` used in `main`); `--kind` only recognizes `core-wire`, any other value silently sets `core_wire=false` (consistent but silent across CLI/serve/daemon).

---

## 4. Built & validated vs. deferred/blocked (honest scorecard)

### Built — and lived end-to-end on real turns
- **Spine (Stage 0):** kernel (manifest · context · registry · chain · lifecycle/substrate-resolver · engine · config from `keel.lock`); invariant middleware **I1 audit / I3 privacy rungs 1–2 + an I3 mask-all-output rung / I4 cost**; three-tier economy (local Qwen $0 · DeepSeek cheap-API · Opus-4.8 frontier) through one invariant chain; file ledger + SQLite Spine (I2); CLI + `serve_openai` (consumable embedded **and** over protocol).
- **Routing & verification (Stage 1):** `DifficultyRouter` (GOLDEN_ROUTER ✓); the kernel §8 engine loop; `verifier` **I5** (GOLDEN_ORACLE ✓ — both reject and accept directions lived in-binary); the **golden freeze-gate active** (seal `db4377b3`).
- **Perception:** the change-gate ships *with* the senses (dHash + VAD, GOLDEN_PERCEPTION ✓); Whisper ears; `hear()`/`see()`/`listen()`/`see_screen()` retinas; cpal-mic + xcap-screen capture devices (feature-gated → native Qwen vision).
- **Daemon & self-direction:** `driver` (`UserTurn`/`Heartbeat`/`Watch`) + daemon select-loop / `tick` / `run_until_idle`; **self-consolidation** auto-triggered.
- **Memory (Stage 2):** persistent **Tape** across processes; **Ring-3 narrative** register; **Ring-4 semantic recall**; `consolidate` / **cold-eyes** (caught real drift live); off-loop `metrics` reader; `SchemaOracle` / `GoldenDispatchOracle`; GOLDEN_MODEL_TIER ✓; the **A3 embedder** + GOLDEN_RECALL fingerprint case ✓.
- **Flywheel (Stage 3):** `FileTraceSink` (secrets scrubbed) + `distill` / `distill-export` (chat training pairs). The whole flywheel **lived end-to-end** on real turns. LoRA training stays external/out-of-band.

### Deferred / blocked
- **ISSUE-10 [blocker — missing model]:** **Qwen3-Embedding-0.6B GGUF is absent from `C:\models`** (only the reranker is present). This blocks live embed, Ring-4-live wiring, and the C1/C2 recall benchmarks. The embed organ, recall, and the fingerprint golden are built + tested regardless.
- **A5 — privacy rung-3** (the OpenAI Privacy Filter / `ort`+ONNX local verification-pass): operator-gated, heaviest, scheduled **last**. By canon §14 it is a verification-pass, never the sole oracle. `GOLDEN_PRIVACY` is not yet in `keel-middleware`.
- **`amplify` (best-of-N), canon §22 / ADR #11:** ships **OFF** — needs a best-of-N benchmark to justify turning on; built OFF by design.
- **D1 — the first cell** (re-home NightScribe on KEEL over `serve_openai`): the major remaining effort; scoped, not yet built.
- Lower-priority deferrals with no current trigger: `kernel::lock` substrate-hash (no-op until the operator pins a `sha256:`), `mw::cache`, multi-turn escalation/rework trend analysis (needs daemon + flywheel data).

---

## 5. Build / run / test

Build, test, and run from a **native MSVC dev shell (PowerShell)** — not git-bash. Rust stable `x86_64-pc-windows-msvc`, edition 2021.

```powershell
# the gate (run before and after any slice)
cargo check
cargo test --workspace      # expect: 129 passed / 6 ignored
cargo clippy

# one-time toolchain repair, if rustc can't find std/core (E0463) — OPERATOR-GATED
rustup toolchain install stable-x86_64-pc-windows-msvc --force

# local inference substrate (resolved, not embedded)
powershell -ExecutionPolicy Bypass -File C:\loom\marrow-l1\scripts\serve_local.ps1   # llama-server
```

The 6 ignored tests are **live** (real endpoint / API key / mic / screen): the three tier `live_generate`s, `whisper::live_transcribe`, `embed::live_embed`, and one capture-device test. The `screen` and `mic` features gate their own additional live tests.

**Substrate (resolved at runtime, pinned in `keel.lock`):** `C:\llama.cpp` (llama-server), `C:\models` (Qwen3.5-9B-Q5_K_M + `mmproj-F16`, Whisper `large-v3-turbo`), `C:\whisper.cpp`. Cloud keys (`DEEPSEEK_API_KEY` / `ANTHROPIC_API_KEY`) live in env — never hardcoded or committed.

---

## 6. CLI surface

### `keel` (the binary — `src/main.rs`)

| Command | What it does |
|---|---|
| `keel "<prompt>"` (default turn) | Run one turn through the full loop (assemble → route → chain → verify → checkpoint → emit). |
| `keel metrics` | Print the off-loop `MetricsSummary` (turns, rework/escalation rate, by-tier, total cost) from the SQLite store. |
| `keel daemon` | Run the perpetual self-driving select-loop (`tick` / `run_until_idle`), with auto-consolidation (`--consolidate-every N`). |
| `keel consolidate` | Run a memory consolidation maintenance Step (lossy Ring-3 narrative compression; sovereign → local). |
| `keel cold-eyes` | Run a cold-eyes re-read pass over memory to catch narrative drift vs. the Tape. |
| `keel distill-export` | Export the verified-trace corpus as chat training pairs (JSONL). |

Routing flags: `--tier <name>` takes the single-tier assemble path; `--kind core-wire` marks the step as core-wire (any other value silently leaves it scaffolding — same in serve and daemon). The daemon path also reads a `watch_token` (changes on edit; `None` when absent).

### `keel-serve` (`src/bin/keel-serve.rs`)

The OpenAI-egress HTTP server (axum) — consume KEEL **over protocol**, the `serve_openai` surface. Accepts Chat Completions requests; parses the I5 and grammar extensions (defaulting when absent). Routing content is built from **all** messages (vs. the CLI, which routes on just the user prompt) — a benign asymmetry; image-detection still works.
