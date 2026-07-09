# Sub-agent 3 — `crates/` workspace map (verbatim)

> Original task: map all 7 Rust crates — purpose, public API, intra-workspace deps, dependency graph, binaries, domain glossary with file paths. READ-ONLY.

---

# KEEL Workspace Crate Map

The workspace at `C:\KEEL\Cargo.toml` declares **7 crates** in a strict 6-layer architecture (L0-L5), all at version `0.2.0`, edition 2021, MIT. Subdirectories under `C:\KEEL\crates\`:

`keel-contracts` · `keel-kernel` · `keel-middleware` · `keel-adapters` · `keel-store` · `keel-services` · `keel`

The layer rule (canon §6) is enforced by import direction: contracts depend on nothing; everything depends on contracts; middleware never imports services; services may import middleware; the `keel` app crate wires concrete impls together.

---

## 1. keel-contracts — L0, the frozen joints (CORE)

`C:\KEEL\crates\keel-contracts\Cargo.toml` · `src\lib.rs`
Purpose: The load-bearing, language-neutral surface — core types + the **ten contracts**. Frozen (canon §7). Runtime-pure (no clock, no side effects, no I/O).

Intra-workspace deps: **none** (this is the foundation).
Notable external deps: `async-trait`, `serde`/`serde_json`, `futures-core`. Dev-only `sha2` for the golden freeze-gate hash test.

Public modules: `errors`, `traits`, `types` (all re-exported via `pub use *`).

- `src\errors.rs` — `KeelError` enum (12 variants with stable `.code()` strings: `ROUTE_NO_TIER`, `TIER_UNAVAILABLE`, `ORACLE_FAIL`, `JOINT_WRONG`, `BUDGET_EXCEEDED`, `ESCALATION_EXHAUSTED`, `REVERSIBILITY_BLOCK`, `INSUFFICIENT_SOURCE`, `PERCEPT_LOW_CONFIDENCE`, `SUBSTRATE_UNRESOLVED`, `GRAMMAR_VIOLATION`, `KEEL_ERROR`) + `Result<T>`.
- `src\types.rs` — the domain data surface: `Json`, `Time`, `RunId`, `State`; `Role`, `Content` (Text/Image/Clip/Audio), `Message`; `Kind` (Scaffolding/CoreWire), `Trust`, `DataClass`, `Effort`; `Usage`, `Price`, `compute_cost()`, `Capabilities`; `ToolCall`/`ToolDef`/`ToolResult`; `GenerateRequest`/`GenerateResult`; `Step` (the atomic routed-work unit), `Decision`; `StepOutput`, `Assertion`, `Verdict`, `GoldenCase`; `Modality`, `Percept`, `SampleSpec`; `TokenBudget`, `AssembledContext` (the 5 memory rings); `Trace`, `VerifiedTrace`; `CostAcc`, `Context` (flows through every call).
- `src\traits.rs` — the **ten contracts**: `ModelTier`, `ToolHost`, `Next`, `Middleware`, `Router`, `Oracle`, `Memory`, `Spine`, `Driver`, `PerceptionSource` (+ `PerceptStream` type alias).

---

## 2. keel-kernel — L1, the spine (CORE — the loop)

`C:\KEEL\crates\keel-kernel\Cargo.toml` · `src\lib.rs`
Purpose: Runs the genome — manifest parsing, context minting, the tier registry, the middleware chain executor, the substrate resolver, and the canonical closed loop. Imports **only** L0.

Intra-workspace deps: `keel-contracts`.
Notable external deps: `serde_yaml_ng` (parses `keel.lock`), `uuid`, `async-trait`.

Public modules: `chain`, `context`, `engine`, `lifecycle`, `manifest`, `registry`. Re-exports: `Chain`, `new_context`/`new_trace_id`/`now_millis`, `Engine`/`Outcome`/`TierSlot`, `default_local_candidates`/`launch`/`probe`/`resolve_endpoint`/`LlamaServer`/`LlamaServerConfig`, `Manifest`/`CostCfg`/`PriceCfg`/`RouterCfg`/`TierCfg`, `Registry`.

- `src\engine.rs` — **the canonical loop** (canon §8): `assemble → route → chain → verify → checkpoint → emit`. `Engine::run()` runs one turn; `Engine::tick()` is the driver select-loop step; `Engine::run_until_idle()` is the bounded burst. Holds one egress-correct `Chain` per tier (I3). Owns the `Context` (folds cost post-call, I4) and `Step` history (escalation ladder). Implements the I5 teeth: unresolved `golden_ref` → fail-closed; a `critical` step with no correctness assertion → config-fault. `Outcome` exposes result + decision + tier_used + substituted + verdict. `select()` and `request_from_step()` are free functions.
- `src\chain.rs` — `Chain`: the onion-shaped middleware executor; `Chain::run(req, ctx, terminal)` threads through `Middleware`s down to the terminal `ModelTier`. Structurally unbypassable (I1/I3/I4 live here).
- `src\manifest.rs` — `Manifest` parsed from `keel.lock` YAML: `tiers` (`TierCfg`: adapter/model/vision/api_key_env/endpoint/price/max_tokens), `router` (`RouterCfg`), `cost` (`CostCfg`), `servers` (`ServersCfg`/`LlamaCppCfg`), `substrate` (`SubstrateCfg`/`LlmVisionCfg`). Helpers `llama_exe()`/`llm_model_path()`/`llm_mmproj_path()` join launch paths.
- `src\lifecycle.rs` — substrate resolver, dependency-free (std only): `(c1)` `probe()`/`resolve_endpoint()` TCP-liveness check over `default_local_candidates()` (LM Studio/Ollama/llama-server); `(c2)` `launch()` spawns llama-server, polls `/health` until ready (distinguishes 200 ready from 503 loading), returns a `LlamaServer` handle (drop does NOT kill — reuse).
- `src\context.rs` — `new_context(manifest)` mints the `Context` (trace_id, seeded budget, clean redaction state). `now_millis()`/`new_trace_id()` are where the kernel reads the clock (contracts stay clock-free).
- `src\registry.rs` — `Registry`: tier-name → `Arc<dyn ModelTier>` map (kernel holds only the L0 trait object, never concrete adapters).

---

## 3. keel-middleware — L3, the invariant chain

`C:\KEEL\crates\keel-middleware\Cargo.toml` · `src\lib.rs`
Purpose: The cross-cutting invariants applied to every call — I1 audit, I3 privacy, I4 cost. Each is a `Middleware` composed in the kernel's `Chain`.

Intra-workspace deps: `keel-contracts`, `keel-kernel`.
Notable external deps: `regex` (PII patterns).

Public modules: `audit`, `cost`, `privacy`. Re-exports: `AuditEvent`/`AuditMiddleware`/`AuditSink`/`FileAuditSink`, `CostMiddleware`, `Finding`/`PrivacyMiddleware`/`Redactor`.

- `src\audit.rs` — I1: `AuditMiddleware` emits one `AuditEvent` per call (success or failure) behind the pluggable `AuditSink` trait. `FileAuditSink` is the append-only JSONL ledger. `AuditEvent::redaction()` records I3 mask decisions (labels only, never values).
- `src\privacy.rs` — I3, the deterministic mask (rungs 1-2). `Redactor::redact()` scrubs operator markers (rung 1, exact strings) + structured regex/Luhn patterns (rung 2: email, SSN, `sk-`/`AKIA` keys, Luhn-valid cards). `PrivacyMiddleware`: egress rung scrubs outbound text only on cloud tiers; **output** rung scrubs the response on **every** tier (so model-authored PII never lands in the Tape/ledger). Returns `Finding`s.
- `src\cost.rs` — I4: `CostMiddleware` is a pre-call **gate** (returns `BudgetExceeded` when remaining budget < hard-stop floor). Post-call accumulation is the engine's job (the chain sees only `&Context`).

---

## 4. keel-adapters — L2, the tiers + senses (ADAPTERS)

`C:\KEEL\crates\keel-adapters\Cargo.toml` · `src\lib.rs`
Purpose: Thin shims under the OpenAI Chat Completions protocol — one shared mapping, each provider adds only compat specifics. Imports only L0.

Intra-workspace deps: `keel-contracts`.
Notable external deps: `reqwest` (rustls-tls); opt-in `cpal` (mic) and `xcap` (screen) behind features.

Features: `mic` → `Microphone`; `screen` → `ScreenCapture` (both OFF by default — "minimal-core thesis").

Public modules: `anthropic`, `deepseek`, `local_llama`, `openai`, `wav`, `whisper`, plus feature-gated `mic`/`screen`. Re-exports: `Anthropic`, `DeepSeek`, `LocalLlama`, `Whisper`, `write_wav_i16`, (gated) `Microphone`/`ScreenCapture`.

- `src\openai.rs` — the shared protocol mapping: `base_body()` shapes the request (multimodal `Content`→OpenAI parts; audio/clip rejected as must-be-pre-processed); `parse_response()` → uniform `GenerateResult` with `compute_cost()`. Pure functions, no live endpoint.
- `src\local_llama.rs` — on-box workhorse (`LocalLlama: ModelTier`): HTTP→llama-server, vision via `image_url`, **constrained decode** (GBNF string→`grammar`, JSON-schema→`json_schema`), thinking toggle via `chat_template_kwargs.enable_thinking`. Cost $0.
- `src\whisper.rs` — `Whisper`: a transcription **organ** (NOT a `ModelTier` — the router never routes here). Shells out to whisper.cpp `whisper-cli`; sovereign + local.
- `src\anthropic.rs`, `src\deepseek.rs` — cloud tiers (frontier / cheap-API) over their HTTPS APIs.
- `src\mic.rs` (gated), `src\screen.rs` (gated) — OS-capture organs for the perception retinas. `src\wav.rs` — WAV writer for the `listen()` retina.

### IMPORTANT — current shape vs. the embed.rs deletion

`git status` shows an **uncommitted working-tree change** (not yet committed):
- `D crates/keel-adapters/src/embed.rs` — the file defining `Embedder` (a llama-server `/v1/embeddings` HTTP client) is **deleted**.
- `M crates/keel-adapters/src/lib.rs` — `pub mod embed;` and `pub use embed::Embedder;` are **removed**.
- `M crates/keel-services/src/lib.rs` — `pub mod recall;` and `pub use recall::{cosine, recall_top_k, should_rebuild, Fingerprint};` are **removed**.

This leaves the working tree in a **non-compiling state**: `crates/keel-services/src/recall.rs` still contains `impl Embed for keel_adapters::Embedder` (line 23) and a doc-reference to `keel_adapters::Embedder` (line 15), and `crates/keel-services/src/memory.rs` still `use crate::recall::{cosine, Embed, Fingerprint}` — but `recall` is no longer a declared module of `keel-services` and `Embedder` no longer exists in `keel-adapters`. The last committed HEAD (`6ac319d`) is consistent; this in-progress refactor appears to be relocating/retiring the embedder organ. `recall.rs` and `memory.rs` files still physically exist on disk with their Ring-4 semantic-recall logic intact.

At HEAD (committed), the recall/embedding story is: `keel-adapters::embed::Embedder` (HTTP embedder organ) is referenced by `keel-services::recall` (the `Embed` trait, `Fingerprint`, `cosine()`, `recall_top_k()`, `should_rebuild()`), which `keel-services::memory::FileMemory` wires via `.with_embedder()` for Ring-4 semantic recall.

---

## 5. keel-store — L2, the index (STORE)

`C:\KEEL\crates\keel-store\Cargo.toml` · `src\lib.rs`
Purpose: SQLite (bundled into the binary, ~1MB) behind the `Store`/`Spine` seam — durable, resumable run-state for crash-resume (I2). The append-only file ledger remains the system of record; this index is derived/rebuildable.

Intra-workspace deps: `keel-contracts`.
Notable external deps: `rusqlite` (bundled feature).

Public API (single file): `SqliteStore` (implements `Spine`: `checkpoint()`/`resume()` over the `runs` table) + `MetricsSummary` and `SqliteStore::metrics()` — an **off-loop, read-only** rollup (the "flywheel instrument"): `turns`, `escalation_rate` (turns above the kind's base tier), `rework_rate` (model/content verify-fails excluding wiring faults), `by_tier`, `total_cost`. Helper fns `tier_rank()`, `base_rank()`, `is_wiring_fault()`.

---

## 6. keel-services — L4, the services (SERVICES)

`C:\KEEL\crates\keel-services\Cargo.toml` · `src\lib.rs`
Purpose: Default (swappable) services composed from the contracts. A service MAY import middleware; middleware may never import a service. Composes adapters under middleware.

Intra-workspace deps: `keel-contracts`, `keel-adapters` (the whisper organ for `hear()`), `keel-middleware` (reuses `Redactor` in the `TraceSink` scrub).
Notable external deps: `regex`, `jsonschema` (default-features OFF — in-memory only, no network egress for the safety oracle).

Features: `mic`, `screen` (forward to `keel-adapters`' features).

Public modules: `distill`, `driver`, `memory`, `perception`, `router`, `trace_sink`, `verifier` (and on disk but currently un-exported at lib.rs due to the in-progress refactor: `recall`). Re-exports: `export_training_jsonl`/`training_pair`, `HeartbeatDriver`/`UserTurnDriver`/`WatchDriver`, `FileMemory`, `ChangeGate`/`FrameGate`, `DifficultyRouter`, `FileTraceSink`, `GoldenDispatchOracle`/`GoldenOracle`/`PropertyOracle`/`SchemaOracle`/`SourceOracle`/`Verifier`.

- `src\router.rs` — `DifficultyRouter` (the `Router` policy): cheapest tier clearing the trust bar. Rules: raw perception/sovereign/PHI → forced local (I3); projected cost over budget → BLOCK (I4); `kind` picks base tier (scaffolding→local, core-wire→cheap-API); repeated oracle failure escalates up the `["local","cheap-API","frontier"]` ladder. `effort_for()` sets best-of-N × thinking per tier economics.
- `src\verifier.rs` — I5 externality layer (canon §10). Oracle kinds: `PropertyOracle` (deterministic property/metamorphic, fail-closed on unknown), `GoldenOracle` (the **joint-wrong detector**: self-tests pass yet a frozen golden violated → `JOINT_WRONG`), `SourceOracle` (trace-to-canon "Truth Gate": ungrounded claim → `INSUFFICIENT_SOURCE`), `SchemaOracle` (JSON Schema, Draft 2020-12 **pinned**, in-memory, reject-never-partially-apply), `GoldenDispatchOracle` (dispatches a resolved `golden_ref` by family: `input.schema`/`input.property`/else-assert-nothing). `Verifier` is the pluggable registry (AND over `passed`, OR over `joint_wrong`) and is itself an `Oracle` composite.
- `src\memory.rs` — `FileMemory` (the first `Memory` impl): a file-backed ringed-context assembler over the **Tape** (append-only JSONL ledger of every `Trace` — the lossless factual register, canon §11). `assemble()` injects Ring-0 (soul, empty for bare genome) → Ring-3 (model-authored narrative) → Ring-2 (recent working turns from the Tape) → Ring-4 (semantic recall). `record()` appends a Trace (skips memory-maintenance turns so they don't pollute the factual Tape). `consolidate()` returns a self-interview maintenance `Step`. `cold_eyes_prompt()` diffs the narrative against the Tape (I5). `with_embedder()` wires Ring-4.
- `src\driver.rs` — initiative (canon §7/§8): `UserTurnDriver` (FIFO queue), `HeartbeatDriver` (perpetual tick, clock injected for testability), `WatchDriver` (poll-based watch-on-change via a comparable `u64` token). Each stamps `Step.source` (`"user"`/`"heartbeat"`/`"watch"`).
- `src\perception.rs` — the afferent change-gate (canon §12): `ChangeGate` (dHash Hamming distance for frames; energy-based VAD for audio — silence is free), `FrameGate` (stateful dedup), retinas `see()`/`hear()`/`listen_from_samples()`/`listen()`(gated)/`see_screen()`(gated), `resample_to_16k()`. Conformance-pinned by `GOLDEN_PERCEPTION`.
- `src\trace_sink.rs` — `FileTraceSink` (`TraceSink`): passed verdicts → append-only distill corpus JSONL, **scrubbed first** via the shared `Redactor` (the reversibility gate §5 — never train on a secret).
- `src\distill.rs` — `training_pair()`/`export_training_jsonl()`: turns the scrubbed corpus into chat-format training pairs for an out-of-band trainer (Unsloth). KEEL §16-refuses the trainer itself.
- `src\recall.rs` (on disk; **not currently re-exported from lib.rs** due to the uncommitted refactor) — Ring-4 retrieval: the `Embed` trait, `Fingerprint` (embedder + dim, format-committing), `cosine()`, `recall_top_k()` (brute-force), `should_rebuild()`. The `impl Embed for keel_adapters::Embedder` here is the broken reference once `embed.rs` is deleted.

---

## 7. keel — L5, the apps / wiring layer (BINARIES)

`C:\KEEL\crates\keel\Cargo.toml` · `src\lib.rs`
Purpose: The **injection layer** that reads `keel.lock`, builds concrete adapters/services, and hands them down to the kernel as L0 trait objects. Provides two assembly paths plus two binaries.

Intra-workspace deps: **all six** — `keel-contracts`, `keel-kernel`, `keel-middleware`, `keel-adapters`, `keel-services`, `keel-store`.
Notable external deps: `tokio` (rt-multi-thread/macros/net), `axum` (the server), `serde`/`serde_json`.

This crate is BOTH a lib and binaries (`[lib]` + two `[[bin]]`):
- `src\lib.rs` — the wiring lib. `Engine` (the self-driving wrapper over `keel_kernel::engine::Engine`): `Engine::assemble(manifest)` wires every available tier (local always; a cloud tier only when its API key is in env) each behind its own `audit→privacy→cost` chain with the correct egress mask, injects the `DifficultyRouter`, `Verifier` (as composite `Oracle`), `SqliteStore` (Spine), `FileMemory` (Memory), `FileTraceSink` (TraceSink), and the golden registry. `Engine::run()`/`run_on()`/`tick()`/`run_until_idle()` delegate to the kernel loop. `assemble()` is the single-tier path (the `--tier` override). Constants: `AUDIT_LEDGER`, `INDEX_DB`, `TAPE_PATH`, `TRACES_PATH`, `GOLDEN_PATH`, `LLAMA_LOG`, endpoints. `watch_token()` and `daemon_perpetual()` are daemon helpers.
- `src\main.rs` — **binary `keel`** (default-run), the daily-driver CLI. Subcommands: plain prompt (self-driving route→run→verify→checkpoint→emit), `keel daemon` (self-driving select-loop with HeartbeatDriver + optional WatchDriver + `--consolidate-every` self-consolidation), `keel metrics` (off-loop rollup), `keel distill-export`, `keel consolidate`, `keel cold-eyes`. Flags: `--manifest`, `--tier`, `--kind core-wire`, `--sovereign`, `--critical`, `--golden-ref`, `--think`. Has the I3/I5 `--tier` guard (refuses `--sovereign`/`--critical`/`--golden-ref` under a manual override that skips the gates).
- `src\bin\keel-serve.rs` — **binary `keel-serve`**, an OpenAI-compatible HTTP server (axum) on `127.0.0.1:7070`: `GET /health`, `GET /v1/models`, `POST /v1/chat/completions`. One `Engine` assembled at startup, shared across requests; every request is routed. Exposes KEEL extensions (`kind`, `sovereign`, `think`, `critical`, `golden_refs`, `grammar`) and returns routing/verdict story under the `keel` field.

---

## Dependency Graph (intra-workspace)

```
                       keel-contracts  (L0 — frozen; depends on nothing)
                              ^
                              |
        +---------------------+---------------------+
        |                     |                     |
   keel-kernel (L1)    keel-adapters (L2)     keel-store (L2)
        ^                     ^                     ^
        |                     |                     |
   keel-middleware (L3)       |                     |
        ^   ^                 |                     |
        |   +---- keel-services (L4) ----------------+
        |             ^
        |             |
        +-------------+-----> keel (L5, app/wiring + binaries)
                              depends on ALL six crates below it
```

Edge detail:
- `keel-kernel` → `keel-contracts`
- `keel-middleware` → `keel-contracts`, `keel-kernel`
- `keel-adapters` → `keel-contracts`
- `keel-store` → `keel-contracts`
- `keel-services` → `keel-contracts`, `keel-adapters`, `keel-middleware`
- `keel` (L5) → `keel-contracts`, `keel-kernel`, `keel-middleware`, `keel-adapters`, `keel-services`, `keel-store`

Classification: **Core** = `keel-contracts` (frozen types/contracts) + `keel-kernel` (the loop/chain/registry/manifest/lifecycle). **Adapters** = `keel-adapters` (provider shims + sense organs) and `keel-store` (SQLite Spine). **Middleware** = `keel-middleware` (invariants I1/I3/I4). **Services** = `keel-services` (router/verifier/memory/driver/perception/trace_sink/distill/recall). **Binaries** = `keel` (`keel` CLI + `keel-serve` server, sharing one wiring lib).

---

## Binaries Summary

| Binary | Path | Role |
|---|---|---|
| `keel` (default-run) | `crates\keel\src\main.rs` | Daily-driver CLI + daemon + memory commands + metrics + distill-export |
| `keel-serve` | `crates\keel\src\bin\keel-serve.rs` | OpenAI-compatible HTTP server (axum, 127.0.0.1:7070) |

No other crates have `[[bin]]`, `src/main.rs`, or `src/bin/`.

---

## Glossary of Central Domain Concepts (with file paths)

- **The ten contracts** — `crates\keel-contracts\src\traits.rs`: `ModelTier`, `ToolHost`, `Next`, `Middleware`, `Router`, `Oracle`, `Memory`, `Spine`, `Driver`, `PerceptionSource`.
- **Tape** — the lossless, append-only JSONL factual register of every `Trace`. Concept lives in canon §11; implemented as `FileMemory`'s `tape` field in `crates\keel-services\src\memory.rs`; path const `TAPE_PATH = ".keelstate/tape/tape.jsonl"` in `crates\keel\src\lib.rs`. "The Tape is the Spine" (§11) — today the SQLite index is the derived checkpoint and the Tape is the system of record.
- **Memory (the ringed self)** — contract `Memory` (`traits.rs`); impl `FileMemory` (`crates\keel-services\src\memory.rs`). The five rings in `AssembledContext`/`TokenBudget` (`types.rs`): Ring-0 soul · Ring-1 exemplars · Ring-2 working turns · Ring-3 compressed narrative · Ring-4 retrieved. Ring-3 narrative register (model-authored, lossy, **separate** from the factual Tape) + `consolidate()`/`cold_eyes_prompt()` live in `memory.rs`.
- **Embedding** — the `Embed` trait + `Fingerprint` + `cosine()`/`recall_top_k()`/`should_rebuild()` in `crates\keel-services\src\recall.rs`; the real HTTP embedder organ was `crates\keel-adapters\src\embed.rs::Embedder` (now **deleted in the uncommitted working tree**, breaking `recall.rs` line 23's `impl Embed for keel_adapters::Embedder`). Ring-4 semantic recall wired via `FileMemory::with_embedder()`.
- **Agent/Daemon** — initiative is the `Driver` contract (`traits.rs`); impls `UserTurnDriver`/`HeartbeatDriver`/`WatchDriver` in `crates\keel-services\src\driver.rs`; the select-loop logic (`select`/`tick`/`run_until_idle`) is in `crates\keel-kernel\src\engine.rs`; the perpetual daemon wrapper (`run_daemon`, idle=sleep) is in `crates\keel\src\main.rs`.
- **QC / conformance ("the golden freeze-gate")** — operator-frozen, language-neutral ground truth at `C:\KEEL\tests\golden\golden.json` (with `.frozen.json` hash gate). In-code shape `GoldenCase` (`types.rs`). Conformance tests live in each service crate, reading `golden.json` by section: `passes_golden_router` (router.rs), `passes_golden_oracle` (verifier.rs), `passes_golden_perception` (perception.rs), `passes_golden_recall_fingerprint` (recall.rs), `passes_golden_model_tier` (openai.rs/verifier.rs). `load_goldens()` in `crates\keel\src\lib.rs` builds the name→case map the engine resolves `step.golden_refs` against.
- **I5 / externality (verify)** — the `Oracle` contract + `Verdict`/`Assertion`/`StepOutput` (`types.rs`); impls in `crates\keel-services\src\verifier.rs`. The engine's I5 teeth (unresolved-ref fail-closed, critical-step config-fault) are in `crates\keel-kernel\src\engine.rs`.
- **Invariants I1/I3/I4** — I1 audit (`crates\keel-middleware\src\audit.rs`), I3 privacy mask (`privacy.rs`), I4 cost gate (`cost.rs`); all `Middleware`s composed in the kernel `Chain` (`crates\keel-kernel\src\chain.rs`).
- **Tier / routing** — `ModelTier` contract + `Decision`/`Kind`/`Trust`/`DataClass`/`Effort` (`types.rs`); ladder `["local","cheap-API","frontier"]`; `DifficultyRouter` (`crates\keel-services\src\router.rs`); adapter impls in `keel-adapters`; registry `crates\keel-kernel\src\registry.rs`.
- **Perception (eyes/ears)** — `PerceptionSource`/`Percept`/`Modality`/`SampleSpec` (`types.rs`); change-gate + retinas in `crates\keel-services\src\perception.rs`; capture organs `Whisper`/`Microphone`/`ScreenCapture` in `keel-adapters`.
- **Spine (run-state, I2)** — `Spine` contract (`traits.rs`); `SqliteStore` impl (`crates\keel-store\src\lib.rs`).
- **TraceSink / distill (the flywheel)** — `TraceSink`/`Trace`/`VerifiedTrace` (`types.rs`); `FileTraceSink` (`crates\keel-services\src\trace_sink.rs`, scrubs before write — reversibility gate §5); `distill.rs` exports chat-format pairs. Flywheel metrics rollup in `SqliteStore::metrics()`.
- **Substrate** — local inference-server resolution/launch: `crates\keel-kernel\src\lifecycle.rs`; keel.lock-driven launch paths via `Manifest` (`manifest.rs`) wired in `crates\keel\src\lib.rs::resolve_local_endpoint`. Error variant `SubstrateUnresolved`.
- **Manifest (keel.lock)** — `Manifest`/`TierCfg`/`RouterCfg`/`CostCfg`/`ServersCfg`/`SubstrateCfg` in `crates\keel-kernel\src\manifest.rs`; the operator's config→behavior surface (canon §14).

### Note on the "embed.rs just deleted" observation
The deletion is **uncommitted** (`D crates/keel-adapters/src/embed.rs` plus the two lib.rs edits). At the committed HEAD `6ac319d` the embedder organ exists and Ring-4 recall compiles. The working tree currently does **not** compile because `recall.rs` and `memory.rs` still reference the removed `keel_adapters::Embedder` / `recall` module — this looks like an in-progress refactor (likely relocating the embedder, or retiring Ring-4 recall from the default build) that has not yet updated its dependents.
