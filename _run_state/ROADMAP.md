# KEEL — ROADMAP (the single forward plan: NOW → DONE → perpetual polish)

> **What this is.** The durable, machine-followable blueprint from the current state to KEEL's
> completion, then a self-improvement loop that never "stops." **Any session reads this (after
> WAKE_UP + STATE) and knows the exact next action — with zero operator re-explanation.** It is the
> forward complement to `STATE.md` (the live you-are-here cursor) and `WORKLOG.md` (the chronological
> trail). **Trust `git` + `cargo` over this doc for *state*; trust this doc for the *plan + completion
> criteria*.** Author: the autonomous build loop, 2026-06-14; maintained every slice.

---

## 0 · THE AUTONOMY CONTRACT (how a session uses this file — the loop)

You are an autonomous KEEL build session operating under the operator's standing grant (WORKLOG
"AUTONOMOUS RUN" entries + `AUTONOMY_CHARTER.md`). Run this loop without asking:

1. **Reconstitute.** Read `WAKE_UP.md` → `STATE.md` → **this ROADMAP** → the latest `WORKLOG.md`
   entries. **Verify by artifact, never recall:** `git -C C:\KEEL log --oneline -10`, `git status`,
   `cargo test` from PowerShell, the freeze seal. Keep the "lived vs reconstructed" line honest.
2. **Pick the next slice.** The first `[ ]` (todo) entry below that is **unblocked** (its deps are
   `[x]`) and **not `[G]`/`[!]`** (operator-gated / blocked). Respect dependency order.
3. **Build it** against the **frozen** contracts (never bend a joint/golden to ease an impl — if a
   slice seems to need that, the boundary is wrong: flag it in ISSUES and skip). Follow the slice
   protocol (CLAUDE.md §Session-protocol).
4. **Gate, bank, push.** `cargo test && cargo clippy` zero-warning → **secret-scan the staged diff**
   → one commit (one-line intent + trailer `Co-Authored-By: Claude Opus 4.8 (1M context)
   <noreply@anthropic.com>`) → **push**. Feature-gated deps: also build/clippy `--features <f>`.
5. **Record.** Mark the slice `[x]` here, update `STATE.md` (the cursor + banner), append `WORKLOG.md`
   (decision + commit result). Keep WAKE_UP §0/§5.1 current when the plan shifts. Doc edits in the
   **same commit** as the slice. (WAKE_UP.md is CRLF = the LF parts re-stitched — edit the parts +
   `WAKE_UP_part1..5.md`, then regenerate WAKE_UP.md; STATE/WORKLOG/ROADMAP are LF.)
6. **Loop** to step 2 until **~90% context**, then **checkpoint and exit** (the supervisor respawns a
   fresh session that reconstitutes and continues). Never continue through a forced compaction.
7. **On compaction** (mid-session): rehydrate via the ⛑ protocol (`STATE.md` §⛑) — re-read +
   re-verify before any further action.
8. **Decide-and-document on EVERYTHING** (use the §5 operator-calibration: `GENESIS-ARC.md` §5 +
   the `keel-operator-calibration` memory) **EXCEPT the operator-only acts**, which you NEVER
   self-authorize: **edit a frozen contract · ratify/change/re-stamp a golden or the seal · mutate
   the global Rust toolchain.** Those → the ISSUES register; **route around them, never block the
   rest.** A non-operator-only blocker: resolve it, or decide + press forward. **Do not stop the run.**
9. **TTL everything + pivot when stuck (operator standing rule, 2026-06-15).** Put a kill switch on
   anything that can block: every shell call gets an explicit `timeout`; live model/daemon/server runs
   are **bounded** (`--max-ticks`, a deadline) and verified **by artifact** (`.keelstate` ledger / Tape /
   `keel metrics`), never by a capturing pipe that can hang. **Never start something unbounded.** If a
   thing gets stuck or fails ~twice: stop retrying, record what you have, file it in §5 ISSUES, and
   **pivot to the next actionable slice** — leave the stuck branch for later/the operator. One blocker
   must never stall the run. (Memories: `keel-ttl-everything-and-pivot`, `keel-live-run-kill-switch`.)

**Operator-touch is required exactly once** (the env wiring that turns this loop on) — see
`tools/keel-autoloop.ps1` + the SessionStart/PreCompact hooks. After that one-time setup the loop is
self-perpetuating; the operator reviews the ISSUES queue whenever he chooses, never to keep it moving.

**Why this works (not more context — a tighter loop):** the self is the durable record + the
discipline of re-reading it (WAKE_UP §3). This file is that record for the *build*; the gate + I5 are
the guardrails that make unsupervised iteration safe; the supervisor is the temporary external
`Driver` — until KEEL is complete enough to host its own loop (then the scaffold dissolves).

## Status legend
`[ ]` todo · `[x]` done · `[~]` in progress · `[!]` blocked (see ISSUES) · `[G]` operator-gated
(needs an operator-only act / review) · `[?]` unknown (needs a falsifier/benchmark to decide ON/OFF)

---

## 1 · DONE (the foundation — do not redo; verify by `git log`)
Stage 0 (spine: kernel · invariant middleware I1/I3/I4 · 3-tier economy local/DeepSeek/Opus · file
ledger + SQLite Spine · CLI + `serve_openai` · substrate resolver) **✅**. Stage 1 (`DifficultyRouter`
· `kernel::engine` L1 §8 loop · `svc::verifier` I5 + freeze-gate · perception change-gate + whisper
ears + `hear()` · `svc::driver` initiative + the daemon `select`/`tick`/`run_until_idle`) **✅**.
Stage 2 partial (`metrics` · `svc::memory` minimal persistent Tape · config-from-`keel.lock` · ears
**cpal mic** capture + eyes **xcap screen** capture, both feature-gated → the native Qwen vision /
Whisper) **✅**. **112 tests green / 5 ignored; seal `db4377b3`; public.** (Latest commits: see `git`.) Phase A: A1 ✅.

---

## 2 · THE PLAN (NOW → DONE), dependency-ordered

### Phase A — Stage 2 completion
- `[x] A1` · **`listen()` + `see_screen()` retina wrappers** (svc::perception) — DONE 2026-06-14.
  `listen()` (`#[cfg(feature="mic")]`) = mic(cpal)→`voiced_ms` VAD-gate (silence short-circuits)
  →`resample_to_16k` (no-dep linear)→`write_wav`→whisper→Audio `Percept`; `see_screen()`
  (`#[cfg(feature="screen")]`) = screen(xcap)→`FrameGate`→`see()`→Image `Percept`. Factored the
  hardware-free `listen_from_samples` so the silence-gate is unit-tested without a mic; live paths
  `#[ignore]`'d. `mic`/`screen` features forwarded keel-adapters→keel-services. **No new dep.** +2
  unit + 2 feature-gated live. Gate: 112/5 green, clippy clean (default + both features).
- `[x] A2` · **the Driver daemon (L5)** — DONE 2026-06-15. `keel daemon [--max-ticks N] [--interval MS]
  [--watch PATH] [--prompt …]` wires a `HeartbeatDriver` (+ optional file `WatchDriver`) and runs the §8
  loop (tick = select→run→verify→checkpoint→Tape) → idle = sleep `--interval` → re-poll. **Bounded by
  default** (`--max-ticks 1`, terminates); `--max-ticks 0` / `--watch` w/o a bound = perpetual. Exposed
  `keel::Engine::tick`/`run_until_idle` + pure helpers `watch_token`/`daemon_perpetual` (CI-tested).
  **Lived BY ARTIFACT:** a 2-tick run self-drove end-to-end — distinct traces `…-0`/`…-2`, local, $0,
  ok:true, checkpointed+Taped+audited (`keel metrics` saw the turns). **No new dep.** +4 tests.
  *(NB: capturing the daemon's output in the same shell can hang on Windows cold-start — a shell/handle
  artifact, NOT a daemon defect; the daemon exits fine. → ISSUE-8; verify-by-artifact instead.)*
- `[x] A4` · **no-SSN baseline → I3 output rung** — DONE 2026-06-15. `mw::privacy` gains an OUTPUT rung:
  scrubs PII from the response on EVERY tier (audited I1), so the model's own PII never lands in the
  persistent Tape/ledger/egress/display. The no-SSN I5 stopgap is retired (engine `baseline` wired
  `None`; the slot stays a generic always-on extra-oracle seam, excluded from #3). **Decided
  mask-all-output** (genome state-hygiene default; a cell can swap a no-op redactor). +1 test; 123/5 green.
- `[~] A3` · **embedder + `GOLDEN_RECALL`** — FIRST PASS DONE 2026-06-15 (ISSUE-1 decided autonomously:
  **brute-force cosine, NO `sqlite-vec`** — an L1 memory is small; sqlite-vec is a deferred scale opt).
  Built `keel-adapters::Embedder` (HTTP `/v1/embeddings`, sovereign-local — vectors never egress, I3;
  live `#[ignore]`'d) + `keel-services::recall` (`cosine` · `recall_top_k` brute force · `Fingerprint{id,dim}`
  + `should_rebuild`). **`GOLDEN_RECALL` deterministic case GREEN** (`passes_golden_recall_fingerprint`:
  mismatch → no-serve + rebuild-from-ledger). **No new dep.** +3 tests. **Remaining:** the recall@k/ndcg
  **uplift** cases = the C1/C2 falsifiers (real embeddings, later). **Ring-4 WIRED 2026-06-15:**
  `FileMemory::with_embedder` (the `Embed` seam — stub-testable) + `assemble` embeds the query → cosine
  top-k from the `.vec.jsonl` sidecar → injects "Relevant earlier" (dedup vs Ring-2); embed-on-record;
  fingerprint-mismatch clears the stale sidecar. **Opt-in** (genome default off — no live embed dep).
  +3 tests. Remaining: the C1/C2 uplift falsifiers (model) + a live/default embed wiring.
- `[G] A5` · **privacy rung-3** — OpenAI Privacy Filter via `ort`/ONNX (in-process, NOT GGUF), behind
  `GOLDEN_PRIVACY`; additive recall, **never the guarantee** (rungs 1-2 carry it). **Operator: least
  urgent / LAST.** **New dep:** `ort` (heavy native). → ISSUE-2.
- `[~] A6` · **memory narrative register + consolidation** — operator-directed 2026-06-15 (was `[G]`
  ISSUE-3; the operator steered me onto it, so the **safe structural cut** proceeded; the riskier
  model-dependent generation stays for a careful pass).
  **A6.1 DONE** (model-free, additive, behind the frozen `Memory` trait — no contract change): the Ring-3
  **narrative register** (a sibling of the Tape; `FileMemory::narrative()`/`set_narrative()`), **separate
  from the lossless factual Tape** (I5 — a model may not author its own ground truth); `assemble` now
  layers Ring-0 soul → Ring-3 narrative → Ring-2 recent; `consolidate()` returns a real **self-interview
  / forward-narrative** prompt over the prior narrative + recent turns (per `perpetual-memory.md`). +3
  tests; gate 117/5 green.
  **A6.2 partial** (2026-06-15): the L5 consolidation trigger **`keel consolidate`** landed — builds the
  self-interview prompt → routes it (**sovereign → local**) so the model authors the Ring-3 narrative →
  stores via `set_narrative` (closes the loop). Live generation = model-dependent, deferred (operator/
  bounded verify). Ring-4 = DONE (A3). **Remaining:** the **cold-eyes validation** Step (diff narrative
  vs Tape, I5) — **DONE 2026-06-15** (`keel cold-eyes` + `FileMemory::cold_eyes_prompt`: a fresh pass flags
  narrative claims the Tape doesn't support, replies CONSISTENT else lists drift; sovereign→local) — a
  swappable consolidation policy + Ring-1 remain. (**Daemon auto-trigger DONE 2026-06-15:**
  `keel daemon --consolidate-every N` self-consolidates every N ticks — a self that acts AND compresses.)

- `[x] A7` · **Memory Autopilot — "solve memory"** — **DONE + LIVED 2026-07-09** (all six slices,
  same session, zero contract/golden edits; **F-M1/F-M2/F-M3 PASS, F-M4 machinery proven** with one
  honest residual — single-judge recall on adversarial plants is stochastic on the 9B; 2-of-3 vote
  implemented; upgrade trigger = a stronger local judge. Results + six hardened lived lessons:
  `docs/proposals/memory-autopilot.md` §6.5). (operator-directed 2026-07-09: functionally
  perpetual, self-managing, contextually-relevant memory with ZERO operator intervention; **priority
  before D1** — D1 then consumes it as the lived field validation). Full design + acceptance falsifiers:
  `docs/proposals/memory-autopilot.md` (extends `perpetual-memory.md`; REEL-v1.0 Tier-2 fidelity with
  the work-first register correction). Everything lands in L4/L5 behind the frozen `Memory` trait —
  **zero contract/golden edits**. Sub-slices, dependency-ordered:
  - `[x] A7.1` **budgeted assembly** — `MemoryBudget` (char-proxy, keel.lock-derived ratios) enforced
    in `assemble`: Ring-0 verbatim always; narrative/Ring-2/Ring-4 capped → O(1) context per turn
    regardless of Tape size (the "functionally forever" precondition).
  - `[x] A7.2` **episodes register** — consolidation also APPENDS a five-field digest
    (`<tape>.episodes.jsonl`, append-only, never re-compressed) — the durable mid-resolution layer
    between the rolling narrative and the raw Tape; retrieval targets for A7.3, diff substrate for A7.5.
  - `[x] A7.3` **Ring-4 ON by default** — lifecycle resolves/launches the embed server (keel.lock
    `embedded_tiny` rung; ISSUE-8 handle discipline); L5 wires `with_embedder` at all four call sites
    when the substrate resolves (graceful degrade to Ring-0/2/3 otherwise); **two-tier index** (all
    episodes + a bounded recent-turn window) keeps ISSUE-1 brute-force fast forever; cold-start
    backfill = rebuild-from-Tape. *Latency falsifier re-opens `sqlite-vec`.*
  - `[x] A7.4` **the autopilot policy** — L4 `MaintenancePolicy` (turns-since · Ring-2 pressure ·
    session-end · cold-eyes cadence) drives maintenance with NO flags: daemon generalizes
    `--consolidate-every`; CLI consolidates on session-end when due; serve checks post-turn.
    Thresholds in keel.lock (config, not pins).
  - `[x] A7.5` **self-correcting narrative** — parse the cold-eyes verdict; drift → ONE bounded
    regenerate-from-ground-truth consolidation (REEL §10.2 fix) + an I1 `MEMORY_DRIFT_CORRECTED`
    event; persistent drift → metrics/ISSUES, never a retry loop.
  - `[x] A7.6` **persona-grade polish (optional)** — Ring-1 exemplar pool (anchor + rotation, never
    auto-deleted) + Ring-2 as real conversation messages.
  **Acceptance (proposal §6):** F-M1 O(1)-context at a 5k-turn Tape · F-M2 plant-and-recall across
  consolidations · F-M3 multi-session zero-intervention lived run (by artifact) · F-M4
  drift→auto-correct→CONSISTENT. Exclusions (each with a re-open trigger): flywheel · full L1–L4
  cascade/pruning · sqlite-vec · tokenizer dep · Tier-3 model-native.

### Phase B — Stage 3 (the flywheel; size to the base case, ignition is upside)
- `[x] B2` · **`TraceSink` file impl** — DONE 2026-06-15. `keel-services::FileTraceSink` appends each
  passed `VerifiedTrace` to an append-only JSONL distill corpus (`.keelstate/traces/corpus.jsonl`),
  **scrubbing secrets/PII first** (the reversibility gate §5 — never train on a secret) via the **same
  `Redactor`** the I3 egress mask uses (one definition of "secret"; services→middleware, layer-legal).
  Scrubs the (prompt, completion) pair = `step.content` + `result.content`/`reasoning_content`. Wired
  into the engine's emit-on-pass (L5 `trace_sink: Some(...)`). **No new external dep.** +2 tests (scrub
  secret/ssn/email before write · append-one-line-per-trace + clean text verbatim); 119/5 green.
- `[?] B1` · **`svc::amplify` (best-of-N + verifier-select)** — build the structure **clamped OFF**
  (n=1). The §23 falsifier: does verified best-of-N beat single-pass on a fixed benchmark? → ISSUE-4
  (run the benchmark; decide ON/OFF). **No new dep** (uses local tier + the verifier).
- `[~] B3` · **flywheel metric** — **PRELIM 2026-06-15:** `escalation_rate` = **0.000 over 18 live turns**
  (base case — no oracle-failure escalations; the canon base case, "ignition is upside"). The
  **trend-down** needs the Stage-3 flywheel (distillation) running over cycles — deferred (the out-of-band
  trainer isn't run). Base case measured ✓; trend pending the flywheel. → ISSUE-5.
- `[x] B4` · **`svc::distill` (out-of-band)** — DONE 2026-06-15. `keel-services::distill`
  (`training_pair`/`export_training_jsonl`) flattens the scrubbed corpus → chat-format
  `{messages:[user,assistant]}` JSONL; `keel distill-export [--in][--out]` writes the training file.
  Corpus scrubbed at write (B2) ⇒ export carries no secret. LoRA training stays external (Unsloth). +3 tests.

### Phase C — the §23 falsifiers (check + DECIDE each; a decision is the deliverable)
- `[?] C1` reranker vs identity on `GOLDEN_RECALL` → keep OFF or turn ON. (after A3) — **STATUS
  2026-06-15:** the embed organ + brute-force cosine recall + fingerprint golden are built (A3); the
  recall@k **uplift** benchmark needs the embed model served + a labeled set. ~~HARD BLOCKER: the
  Qwen3-Embedding-0.6B GGUF is NOT in `C:\models`~~ → ISSUE-10 **RESOLVED 2026-07-09** (downloaded +
  smoke-tested). **STATUS 2026-07-10: design + machinery LANDED** — proposal
  `docs/proposals/golden-recall-set.md` · DRAFT labeled set `tests/recall/golden-recall.json`
  (42 docs / 30 queries / 6 families, fictional, live-shaped, `ratified:false`, thresholds proposed
  in-file) · `Rerank` seam + `IdentityRerank` (L4) · `keel-adapters::Reranker` (`/v1/rerank`, L2) ·
  IR metrics + stub-tested `run_recall_bench` pipeline · `keel recall-bench` CLI (DRAFT-stamped
  until ratified; `--baseline` uplift; artifact → `.keelstate/bench/`) · keel.lock/manifest
  `rerank.file/port:8091`. **Remaining:** ISSUE-11 (operator ratifies the set) + the step-0 rerank-GGUF
  smoke (`--reranking` accepts it?) + the focused live run → DECIDE ON/OFF (threshold: recall@5
  uplift ≥ 0.10 AND p95 ≤ budget). Ring-4 rerank wiring + lifecycle launch are built ONLY if ON.
- `[?] C2` embedder vs the MiniLM floor → keep floor or upgrade. (after A3) — same design + harness as
  C1 (one more `recall-bench` run against a MiniLM-served `:8090`; nDCG@10 uplift ≥ 0.05 keeps Qwen3
  as default, else the floor takes it — a keel.lock config flip either way; the fingerprint guard
  auto-rebuilds sidecars). **Blocker: the `all-MiniLM-L6-v2` GGUF is not in `C:\models`** (it IS the
  keel.lock-pinned fallback) — provisioning is on ISSUE-11's operator list (~25–45 MB, Apache-2.0).
- `[?] C3` privacy model vs deterministic-only on `GOLDEN_PRIVACY`. (after A5)
- `[~] C4` `rework_rate` < 10% with oracles on — **PRELIM PASS 2026-06-15:** rework_rate **0.056 (5.6%)**
  over 18 live turns, oracles on → under 10%. ✓ (Small N; revisit with more daemon data.)
- `[~] C5` economic: KEEL vs cheap-API-for-everything — **PRELIM KEEL-FAVORABLE 2026-06-15:** routed
  **17/18 turns to FREE local**, 1 to cheap-API (total $0.0004) vs ~$0.0018 for cheap-API-everything
  (~78% saved). KEEL's routing pays. ✓ (Small N.)
- *(Each falsifier trip says "revise, don't extend" — blast radius one adapter. Record the decision +
  rationale in WORKLOG; flip the relevant default in `keel.lock` if warranted [config, not a pin].)*

### Phase D — the first real cell (the §17/§21 proof that the genome is at the right altitude)
- `[x] D1` · **re-home NightScribe on KEEL — DONE + LIVED 2026-07-09.** **The boundary held: zero
  frozen-contract/golden edits were forced by the cell.** Built: `KeelBackend` (vision+synthesis over
  `serve_openai`; `sovereign` + `grammar` extensions — the lab's no-cloud invariant now ENFORCED by
  KEEL's router) + `KeelTranscriber` (ears over the new sidecar-local `/v1/audio/transcriptions` —
  an L5 protocol addition the cell pulled into existence, whisper segments w/ ms offsets) — both now
  the cell's DEFAULT backends (legacy one flag away); its managed-llama-server subsystem is bypassed
  in keel mode; every synthesis lands on KEEL's Tape/audit + rides A7 recall automatically. **LIVED:**
  `nightscribe snap` → screenshot → KEEL → schema-valid verdict, exit 0; transcription endpoint lived
  (TTS WAV → 2 ms-offset segments). 101/101 C# tests (incl. 4 new seam tests); NightScribe commit
  `32c4f9a`. **Three genome bugs surfaced + fixed (adapter/app layers only — the falsifier working):**
  lifecycle lacked `--jinja` (thinking toggle silently ignored) · grammar+thinking mutually exclusive
  (adapter now forces thinking off under constrained decode) · the engine stacked a second system
  message over a caller's own (now MERGES — protocol-composition correctness for every OpenAI client).
  **Boundary finding (recorded):** for an over-protocol cell, afferent capture + the dHash/VAD gate
  stay CLIENT-side (capture is inherently where the devices are); the from-KEEL change-gate applies to
  embedded cells. Cell periphery kept as mapped: capture topology, attribution, timeline fusion,
  minutes schemas, eval harness, intent-memory db, GUI. *(Original scope below.)*
  (controlled experiment) re-home NightScribe on KEEL — **SCOPED 2026-06-15** (boundary
  clean — confirms the genome is at the right altitude; the build is the next major effort, C#-app→KEEL
  over `serve_openai`). NightScribe (`C:\ClaudeCode\photo2deck\labs\nightscribe`, C#/.NET, Phases 0-3
  done) **independently rebuilt KEEL's exact pieces by hand** — the .NET-of-AI-apps case in the flesh.
  **FROM KEEL (unchanged):** eyes (native Qwen vision, `local_llama` image_url) · ears (`whisper`) · the
  perception change-gate (dHash `FrameGate` + VAD + `see()`/`hear()`/`listen()`/`see_screen()`) · route
  (`DifficultyRouter`, local-first) · I5 oracle (its "deterministic token match, no LLM judges itself"
  = a `PropertyOracle`/`SourceOracle`) · memory (Tape + Ring-4 = its intent-memory routing db) ·
  constrained decode (its schema-constrained frame descriptions = `local_llama` json_schema). **CELL
  PERIPHERY (write):** meeting capture topology (dual-track mic=me/loopback=them → KEEL's `source` field)
  · MP4 import (ffmpeg) · minutes synthesis (map-reduce at slide boundaries — a domain prompt chain) ·
  GUI/tray · the golden-meeting eval. **Done =** those KEEL pieces come unchanged; only the periphery is
  written. If the cell forces a kernel/contract edit → KEEL's boundary is wrong, fix KEEL first.
- `[ ] D2` · **SEXTANT on KEEL** (the canon first cell) — done = Conductor (`engine`) / Router /
  Gate (Truth Gate `Oracle`, `INSUFFICIENT_SOURCE`→human) / Canon (factual `Memory`) / State (`Store`)
  / ToolHost (Gmail MCP) / vision retina — **all from KEEL unchanged**; only job-domain periphery.
  **If a cell forces a kernel/contract edit → KEEL's boundary is wrong: FIX KEEL FIRST** (a §23 trip).
- `[ ] D3` · **`ToolHost` (MCP) adapter** — a §3 protocol bet, unbuilt; **pulled by D2** (SEXTANT's
  Gmail MCP). Build when the cell needs it. **New dep:** an MCP client crate (vet at the time).
- *(The Backrooms Director at `C:\backrooms` is the parallel dogfood **consumer** over `serve_openai`
  — NOT a cell; it can start anytime and does not block D1/D2.)*

### Phase E — completion gates
- `[x] E1` · **C++-port-readiness** — DONE 2026-06-15. `docs/conformance-coverage.md` maps every joint +
  invariant → its golden family or structural unit test, with the two documented gaps (`recall`
  conformance-ahead until A3; `ToolHost` unbuilt until D3). Verdict: the 6 golden families are a complete
  *behavioral* conformance layer; structural joints carry no golden by design (a port re-passes their unit tests).
- `[ ] E2` · **the DONE review** — all phases done/decided, ISSUES resolved-or-accepted, the §4.2
  invariant scorecard all-green; write the completion account; flip `keel.lock` `stage:` to `stage3`/done.

---

## 3 · DONE definition
KEEL is **complete** when **all** hold: Stage 0–3 functionally done (amplify/reranker/privacy-model/
embedder each **ON or OFF per its falsifier — decided, not skipped**) · the first cell (D2, or at least
D1) is built on KEEL **with zero kernel/contract edits** · every Phase-C falsifier is checked-and-decided
· the operator-only ISSUES are resolved or explicitly accepted · E1 + E2 pass. **Then the loop does not
stop — it enters perpetual-polish mode (§4).** Write `.keelstate/DONE` only when E2 passes (the supervisor
reads it to wind down to polish cadence, not to halt).

## 4 · Perpetual-polish mode (post-DONE; the self-improvement loop, until quota/power)
When §2 is exhausted, shift to continuous improvement: (1) run `/code-review` on the tree → fix
findings; (2) raise test/golden coverage where thin; (3) re-check the §23 falsifiers with fresh data;
(4) reconcile any doc drift; (5) a **completeness-critic** pass — "what is unverified, missing, or
stale?" → new polish slices; (6) harden + simplify (smaller, never larger). Each polish item is a
gated/banked/pushed slice like any other. Honest about diminishing returns — bounded by the gate + I5,
not a promise of literal perfection. *(Also slot a completeness-critic pass every ~N build slices, not
just post-DONE, to catch drift early.)*

## 5 · ISSUES / BLOCKERS register (the operator-only + unknown queue — route AROUND; never block the rest)
- **ISSUE-1 [operator design-review]** — A3 embedder is format-committing (ADR #13). A session must
  PROPOSE the fingerprint / `sqlite-vec` / embed-adapter design (append it here) for the operator's OK
  **before** committing the index format. Until OK'd → skip A3; do A1/A2/A4/B2/etc. Models +
  HF links: Qwen3-Embedding-0.6B, Qwen3-Reranker-0.6B (operator's note; at `C:\models`).
  **RESOLVED 2026-06-15** (per "decide, never ask"): chose **brute-force cosine, NO `sqlite-vec`**;
  fingerprint `(embedder_id, dim)` + rebuild-from-ledger on mismatch. A3 first pass landed; sqlite-vec
  revisited only if memory size makes brute force slow.
- **ISSUE-2 [operator · least-urgent/LAST]** — A5 privacy rung-3 needs `ort` (heavy native) + is the
  operator's explicit last item. Defer to the end; `openai/privacy-filter` model at `C:\models`.
- **ISSUE-3 [operator-review]** — A6 memory narrative register = the highest-risk seam-cut; reserved
  for the operator. Propose the design here first (vs `perpetual-memory.md`).
- **ISSUE-4 [unknown/benchmark]** — B1 amplify ON/OFF needs a verified-best-of-N-vs-single-pass
  benchmark on a fixed set. Build OFF; run it; decide + record.
- **ISSUE-5 [unknown/data]** — B3/C4 `escalation_rate` + `rework_rate` trends need the A2 daemon
  producing multi-turn data over time (and ideally the flywheel running).
- **ISSUE-6 [operator-only]** — `kernel::lock` (substrate-hash verify) is a no-op until the operator
  pins the `sha256: TODO` fields in `keel.lock`. Build the verify-logic; it stays dormant until pinned.
- **ISSUE-7 [deferred — no trigger yet]** — `mw::cache` (cache-prefix discipline) waits until
  cache-hit-rate matters (scale + the daemon running). §22 anti-pattern to build it speculatively.
- **ISSUE-8 [deferred — tooling, not a KEEL defect]** — capturing a live `keel`/`keel daemon`/`keel-serve`
  run's stdout/stderr **in the same shell** can hang on Windows when it cold-starts llama-server: the
  detached server inherits keel's std-handle pipe, so a capturing consumer (`… 2>&1 | Out-String`) blocks
  on an EOF that never comes. **The daemon/CLI itself exits fine** (proven by artifact). *Workaround now:*
  verify by artifact (`.keelstate/audit.jsonl`, `tape`, `keel metrics`) + always TTL the run. *Real fix
  (deferred):* spawn llama-server fully detached (all 3 std handles explicit + `DETACHED_PROCESS`) in
  `kernel::lifecycle::launch` — a tried patch was gate-green but did NOT resolve the live hang, so the
  root cause needs more investigation; reverted to keep the checkpoint honest. **WORKAROUND WORKS
  (used live 2026-06-15):** run live model commands via `Start-Process -RedirectStandardOutput <file>
  -PassThru` + `$p.WaitForExit(ms)` → `$p.Kill($true)` (a reliable self-kill) and a **file redirect, NOT
  `| Out-String`** — `keel consolidate` then cold-started + ran + exited clean, no hang. The lifecycle
  detach root-fix stays nice-to-have (low priority now the workaround is proven).
- **ISSUE-9 [operator policy — privacy]** — A4's I3 output rung needs a policy decision (operator's
  flagged forward-design area): does the genome default **mask output PII on all tiers** (keeps PII out
  of the persistent Tape/ledger/egress, but masks a local sovereign answer's own PII) **or egress-only +
  audit-local** (sovereign local answers intact; PII can sit in the local Tape)? The middleware can't
  see a turn's `sovereign` class (it only sees request/response), so one default must be chosen. A6.1
  made this sharper (the Tape now persists outputs). **RESOLVED 2026-06-15:** per the operator's "decide
with common sense, never ask" directive, chose **mask-all-output** (state-hygiene default) and built A4.
- **ISSUE-10 [operator — missing model]** — the **Qwen3-Embedding-0.6B GGUF is absent from `C:\models`**
  (only `qwen3-reranker-0.6b-q8_0.gguf` is there); discovered 2026-06-15 attempting the C1/C2 recall
  benchmark. The A3 embed organ + recall + fingerprint golden are built/tested, but no live embed /
  recall-uplift benchmark / Ring-4 live wiring can run until the operator downloads the embed model.
  Unblocks: C1, C2, A3-live, Ring-4-live. **RESOLVED 2026-07-09 (operator-authorized):** downloaded
  `qwen3-embedding-0.6b-q8_0.gguf` (official `Qwen/Qwen3-Embedding-0.6B-GGUF` Q8_0; 639,150,592 B,
  byte-exact vs content-length) to `C:\models` + **smoke-tested LIVE** — llama-server `--embeddings
  --pooling last` on :8090 (bounded, file-redirected, killed — the ISSUE-8 pattern) served
  `/v1/embeddings` → a **1024-dim** vector, matching the keel.lock `dim: 1024` pin. C1/C2/A3-live/
  Ring-4-live are now unblocked (each still needs its focused live session).
- **ISSUE-11 [operator-review] — ratify the golden-recall set + provision the C2 floor.** The C1/C2
  benchmark set `tests/recall/golden-recall.json` is DRAFTED (`ratified:false`) + the harness is
  built + gated (2026-07-10; design: `docs/proposals/golden-recall-set.md`). Operator: (1) edit
  docs/queries/labels/**thresholds** freely, flip `ratified:true` + `ratified_by`/`date` (a
  structural-only lint guards coherence in CI — content edits can never break the gate);
  (2) authorize/download `all-MiniLM-L6-v2` GGUF (~25–45 MB, Apache-2.0, already the keel.lock
  `embedding.fallback`) into `C:\models` for the C2 leg — or say the word and a session does it;
  (3) contingent: if the step-0 smoke shows `qwen3-reranker-0.6b-q8_0.gguf` lacks the rank head
  (llama-server `--reranking` refuses), re-provision the sequence-classification conversion.
  Unblocks: the focused C1/C2 live run → the two DECISIONS. Until then `keel recall-bench` runs
  DRAFT-stamped (usable for smoke, never for the decision).
- *(Append new issues as discovered, each: `ISSUE-N [type] — description · what unblocks it`. If the
  loop STALLS — only `[G]`/`[!]`/`[?]` slices remain and none can advance — write `.keelstate/STALLED`
  with the reason so the supervisor stops respawning, and the operator resolves the queue on next look.)*

## 6 · The cursor
`STATE.md` is the live you-are-here (per-slice banner + the ⛑ protocol). **This ROADMAP is the map;
STATE is the pin.** A session: reconstitute → find the next actionable `[ ]` here → go.
