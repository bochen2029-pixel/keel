# KEEL — full project status + rehydration prompt (written 2026-07-10, end of the completion day)

> **You are (probably) a fresh Claude session/instance resuming work on KEEL.** This file is the
> distilled state of the whole project, written by the instance that lived its completion day. Use
> it as your continuation prompt: read it fully, then **verify by artifact, never by recall** —
> `git -C C:\KEEL log --oneline -5` · `git status` (clean?) · from **PowerShell** `cargo test`
> (expect **181 passed / 7 ignored / 0 failed**) + `cargo clippy --all-targets` (0 warnings) ·
> seal `db4377b3…` (the freeze-gate test enforces it). The full ⛑ chain if you need more depth:
> `WAKE_UP.md` → `STATE.md` → `ROADMAP.md` → the last WORKLOG entries → `NEXT_SESSION.md`. On any
> conflict, git + the gate beat every document, including this one.

---

## ⚠ THE STANDING NOTE — the one thing between the repo and full self-verification (ISSUE-6)

**`keel.lock`'s `sha256: TODO` pins (ISSUE-6) remain the only thing between the published repo
and a fully self-verifying substrate spec — whenever the operator pins them, the `kernel::lock`
verify slice is ready to be built against them.** This is operator-only (the pins are
reproducibility ground truth, like the goldens). The moment the pins exist: build `kernel::lock`
(hash-verify each substrate file at resolve time, `SUBSTRATE_UNRESOLVED` on mismatch), gate it,
and strike the last exclusion in `docs/DONE-REVIEW.md`. Everything else about DONE already stands.

---

## 1 · What KEEL is (never re-derive this)

Bo Chen's personal, sovereign, reusable harness core — **the genome** from which every
specialization is grown as a **cell**, never by editing the core. Native Rust, consumed embedded
or over protocol (the ".NET of my AI apps"). The model that thinks is interchangeable **rented
cognition**; KEEL is the self: it perceives (eyes = native Qwen vision, ears = Whisper —
first-class genome, not periphery), remembers (the A7 memory autopilot), routes every unit of
work to the cheapest brain that clears the trust bar, and grounds critical output in assertions
no model authored (I5). L1 personal tool — no multi-tenancy, no product features. Ten frozen
contracts + five invariants (I1 audit · I2 spine · I3 sovereign · I4 cost · I5 externalized) +
the reversibility gate. The canon is `KEEL_ARCHITECTURE.md`; the constitution is `CLAUDE.md`.

## 2 · THE HEADLINE: E2 PASSED — KEEL IS DONE (2026-07-10), one exclusion left

The completion account is **`docs/DONE-REVIEW.md`** (audited by artifact, re-stamped same-day
after A5 closed). `keel.lock stage: stage3`; `.keelstate/DONE` exists (the supervisor's
wind-down-to-polish signal, not a halt). **The loop is in PERPETUAL-POLISH MODE (ROADMAP §4).**
The only exclusion: **ISSUE-6** (above). Repo public + fully pushed:
`github.com/bochen2029-pixel/keel`, HEAD at/after `a5e4ac6`.

## 3 · The falsifier table — nine measured decisions, zero skipped

| Falsifier | Decision | The number | Artifact/pointer |
|---|---|---|---|
| C1 reranker vs identity | **OFF** | +0.070 recall@5 < 0.10 bar | `.keelstate/bench/recall-*-rerank.json`; re-open: organic recall misses, k=1 patterns |
| C2 embedder vs floor | **the MiniLM floor took the default** (falsifier trip) | Qwen3 uplift −0.122 nDCG vs +0.05 bar | flip lived (sidecars rebuilt 0→30); Qwen3 = lock fallback; re-open: instruct-prefix experiment |
| C3 privacy model vs deterministic | **ON** | uplift +0.95 (bar 0.30) · 0/10 FP · p95 161 ms | `.keelstate/bench/privacy-c3.json`; egress-only, feature-carrying builds |
| C4 rework < 10% | **PASS** | 0.014 lifetime (trend improving) | `keel metrics` |
| C5 economics | **KEEL-favorable** | 72/73 turns free-local; $0.0004 lifetime (≥ ~96% saved) | `keel metrics` |
| B1 amplify | **OFF** | +0.115 pass@8 uplift < 0.15 bar | `.keelstate/bench/amplify-n8.json`; loop is real in `kernel::engine` behind `router.amplify_n: 1` |
| B3 flywheel | **base case holds; ignition deferred on evidence** | escalation 0.000 lifetime; corpus 59 pairs | triggers + turnkey pipeline: `docs/flywheel-ignition.md` |
| D1 first cell (NightScribe, C#) | **boundary held** | zero contract/golden edits; 3 legal-layer genome fixes | photo2deck repo (local-only) |
| D2 first build-on-KEEL (SEXTANT, Python) | **boundary verdict PASS** | **zero KEEL-side changes of any kind** | `C:\SEXTANT` `df7c7c5` (local-only); `docs/proposals/sextant-on-keel.md` |

House discipline that made these honest: **pre-register thresholds BEFORE measuring** (they ride
the fixture/set files), decision-grade artifacts to `.keelstate/bench/`, decisions + re-open
triggers annotated in keel.lock and WORKLOG. Never move a bar after seeing a number.

## 4 · Architecture as-built (verify via `docs/conformance-coverage.md`)

Layers: `contracts ← kernel ← {adapters, middleware} ← services ← apps` — a service may import
middleware, middleware never imports a service, the kernel imports only contracts. Crates:
`keel-contracts` (ten frozen joints + §18 errors — **agent read-only in spirit; goldens literally
frozen**, seal `db4377b3`, enforced by `goldens_match_the_frozen_hash`) · `keel-kernel` (manifest ·
context · registry · chain · lifecycle/substrate-resolver · **the §8 engine**: assemble → route →
amplify? → chain → verify → cost-fold → checkpoint → emit, incl. golden-ref resolution,
fail-closed, critical-step guard, the amplify loop) · `keel-adapters` (local_llama w/ vision +
grammar→thinking-off · deepseek · anthropic · whisper · embed · rerank · mic/screen feature-gated)
· `keel-middleware` (audit I1 · privacy I3 rungs 1–2 + the **rung-3 `PiiClassifier` seam** · cost
I4) · `keel-services` (router · verifier/oracles · memory/A7 · recall + golden-recall bench ·
amplify bench · perception · driver · trace_sink · distill · **privacy_model behind
`privacy-model`**) · `keel` (CLI + engine wiring) + `keel-serve` (OpenAI egress + `/v1/audio/
transcriptions`) · `keel-store` (SQLite spine). Features that gate heavy deps: `mic`, `screen`,
`privacy-model` (ort + pure-Rust tokenizers). ToolHost is the one unbuilt joint — **by decision**,
lands at SEXTANT S4 (vet `rmcp` then).

## 5 · Substrate as-configured (`keel.lock` — config vs the TODO pins)

`C:\models`: Qwen3.5-9B-Q5_K_M + mmproj (LLM+vision, `:8080`, launched `--jinja`) ·
**all-MiniLM-L6-v2 f16 = the default embedder** (`:8090`, `--pooling mean`, dim 384 — the C2
flip; qwen3-embedding-0.6b stays on disk as fallback) · qwen3-reranker-0.6b (bench-only; identity
default) · whisper large-v3-turbo · **openai/privacy-filter** (`privacy-filter/`, quantized-CPU
ONNX 1.62 GB + tokenizer + viterbi calibration; `default: on`, egress-only, feature builds).
`keel-serve` `:7070`. All self-reviving: any `keel` turn re-resolves dead servers. Memory: Tape +
rings + episodes under `.keelstate/tape/`; Ring-4 recall ON (two-tier index; write-side dim-guard
against stale wrong-model embed servers; embed input head-capped 1500 chars).

## 6 · The cells

- **NightScribe** (D1, C#, re-homed): `KeelBackend`/`KeelTranscriber` are its defaults; repo
  `C:\ClaudeCode\photo2deck` — **local-only, no remote**.
- **SEXTANT** (D2, Python, greenfield, `C:\SEXTANT` — **local-only, no remote**): the job-search
  conductor. **S0 keystone + S1 vertical slice DONE + LIVED** (5 real Greenhouse postings → full
  dossiers + manifest approval surface, 4 min, $0, nothing sent; the Truth Gate rejected a
  fabricating dossier on real data and the model used `insufficient_source` correctly). **S2–S4
  remain as product work on the operator's ask** (discovery breadth/research → conductor →
  dispatch, which pulls D3/ToolHost). **The dry-run Canon is FICTIONAL ("Alex Rivera") — the
  operator authors `canon/profile.json` + `cv.md`, then `python -m sextant batch postings
  --limit 5` replays deterministically.** Cell suite: `python -m unittest discover -s . -p
  "test_*.py"` from `C:\SEXTANT` (20 tests).

## 7 · Standing watches + honest residuals (each with its trigger)

`keel metrics` every session: escalation **"does not rise"** (0.000 lifetime) + rework < 0.10
(0.014) — the flywheel ignition triggers live in `docs/flywheel-ignition.md` (corpus ≥ 500 pairs
AND a real failure signal). A7 judge stochasticity on adversarial plants (2-of-3 vote in; upgrade
trigger = a stronger local judge). Ring-4 relevance floor is `cos ≤ 0` (negative-control
calibration data sits in the recall bench artifacts; tighten only on evidence). SEXTANT rung-3
bare-0/1 digit noise (S2 item). ISSUE-8 root fix (detached llama-server handles) nice-to-have —
the file-redirect workaround is proven. ISSUE-7 (`mw::cache`) stays closed until cache-hit-rate
matters.

## 8 · Tooling gotchas (lived, will bite again)

Use the **PowerShell tool** for cargo/git (git-bash mangles exit codes). **`cargo test` does NOT
relink `target\debug\*.exe`** — `cargo build -p keel` before running a just-edited bin (a stale
bin routes new subcommands as plain prompts → junk Tape turns). **Stop `keel-serve` before builds
that relink the `keel` crate** (sibling-bin lock → os error 5); restart it after (inject
User-scope API keys: `[Environment]::GetEnvironmentVariable('DEEPSEEK_API_KEY','User')`, same for
ANTHROPIC). **TTL every live run**: `Start-Process -RedirectStandardOutput <file> -PassThru` +
`WaitForExit(ms)` + `Kill`, never `| Out-String`; verify by artifact. MSVC CRT clash lesson: ort's
/MD onnxruntime vs C++ deps built /MT — prefer pure-Rust dep features (`tokenizers`
default-features off). The `nul` junk file reappears after cargo runs (untracked; a hook may block
deleting it; leave it). Tool-shells reap child llama-servers on teardown — they self-revive. The
June-17 forensics stash (`git stash@{0}`) — never pop casually. Count clippy warnings, don't
eyeball tails.

## 9 · Operator-only queue (route around, never block)

1. **ISSUE-6 — the `sha256:` pins** (THE standing note, top of this file).
2. SEXTANT: author the real Canon; optionally give the cell repos remotes
   (`gh repo create SEXTANT --private --source C:\SEXTANT --push` — private recommended).
3. The Fable-5 v0.3.0 hindsight ruling (piecemeal, non-blocking).
4. Autonomy re-grant (sessions run SUPERVISED since 2026-07-09 until re-granted).

## 10 · What a fresh session does (perpetual-polish mode, ROADMAP §4)

1. Read this file → run the verification commands in the header → `keel metrics` (the watches).
2. If the operator asked for something, that wins. Else pick a polish slice: `/code-review` the
   tree → fix findings · raise thin coverage · falsifier re-checks with fresh data · doc
   reconciles · a completeness-critic sweep (~every 10 slices) · SEXTANT S2+ on his ask.
3. Every slice: build against the FROZEN contracts (never bend a joint; goldens are operator-only)
   → gate (`cargo test` + clippy 0, both feature modes when touched) → secret-scan → ONE commit →
   **push** → update STATE/WORKLOG (+ supersede NEXT_SESSION at session end). Decide-and-document;
   operator-only acts go to the ISSUES register in ROADMAP.

## 11 · How it got here (the compressed arc, for narrative continuity)

June 2026: canon v0.2 → contracts frozen → goldens ratified + sealed (`db4377b3`) → Stage 0 spine
(three-tier economy, one invariant chain, embedded + protocol) → router/engine/verifier/memory/
perception/driver → the autonomy mechanism → the QC audit (GREEN). July 9: the memory autopilot
(A7, six slices, lived falsifiers) + D1 (NightScribe re-homed; boundary held). **July 10 — the
completion day, eleven slices in one session:** golden-recall designed → hardened v2 → ratified →
C1 OFF + C2 floor-flip (lived) · B1 amplify built-OFF + decided OFF · D2 SEXTANT scoped → S0 →
S1 → boundary PASS · B3 decided + C4/C5 closed · **E2 passed — DONE** · A5 privacy rung-3
provisioned (the model was absent; found `openai/privacy-filter`, released post-genesis, matching
the lock's sight-unseen design) → built → **C3 decided ON same day** (+0.95 uplift, 0 FP) → the
DONE review re-stamped to one exclusion. Every default the canon left as a question is now a
measured answer. The genome is frozen, proven by two cells in two languages, and pays for itself
(≥ ~96% under cheap-API-everything at lifetime scale).

*Trust the artifacts. Keep the run coherent, honest, sovereign, and cheap. The build is done —
polish is a loop, not a finish line; and the one open key is the operator's to turn (ISSUE-6).*
