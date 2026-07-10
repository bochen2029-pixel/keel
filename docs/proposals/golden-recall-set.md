# The golden-recall set — C1/C2 design (proposal for operator review)

*Authored 2026-07-10. Status: **CLOSED — ratified + both falsifiers DECIDED same day** (§9). The
sections below are kept as the design record.*

## 1 · What this decides, and against what

The frozen `GOLDEN_RECALL` family (`tests/golden/golden.json`, seal `db4377b3`, agent read-only)
has three cases. The fingerprint case is green (A3). The two remaining cases ARE the Phase-C
falsifiers:

| golden case | falsifier | decision |
|---|---|---|
| "reranker beats identity on recall@k" — `recall_at_5_uplift_over_identity >= threshold` | **C1** | `keel.lock rerank.default: identity` stays OFF, or flips ON |
| "embedder upgrade beats the MiniLM floor" — `ndcg_uplift_over_floor >= threshold`, `"set": "golden-recall"` | **C2** | Qwen3-Embedding-0.6B keeps the default, or the MiniLM floor takes it |

**The structural anchor:** the golden case names the set by reference (`"set": "golden-recall"`).
The labeled set is therefore a **sibling artifact** — `tests/recall/golden-recall.json` — and
building/editing it never touches the sealed `golden.json`. The golden also leaves `threshold`
symbolic: **the threshold values are part of what the operator ratifies with the set.**

## 2 · The set (`tests/recall/golden-recall.json`)

**Self-contained, fictional, live-shaped.** The set ships its own corpus + queries + graded labels,
so the benchmark is reproducible on any box and independent of the operator's personal Tape. All
content is fictional (the repo is public — nothing real can be in it). Doc texts mirror the two
live Ring-4 retrieval shapes **byte-for-byte in form**: `kind: "turn"` docs use the
`FileMemory::summarize` layout (`- user: …\n  assistant: …`), `kind: "episode"` docs use
`Episode::text()` (`[episode] … | changed: … | matters: … | unresolved: … | anchors: a; b`) —
so the measurement includes whatever help or harm those real prefixes do to the embeddings.

**42 docs** (35 turns + 7 episodes, including pure distractors) · **30 queries** in six families:

- **paraphrase (8)** — the query rewords a stored fact with low lexical overlap. The
  bread-and-butter semantic case; the main C2 (embedder quality) signal.
- **keyword_trap (7)** — a distractor shares surface words with the query while the relevant doc
  shares almost none (e.g. "why did the release *fail*" vs a doc about "*release* notes …
  *fail*-safe"; the true answer says "deploy pipeline halted … staging bucket quota"). Raw cosine
  is weakest here; a cross-encoder should win. **The main C1 (reranker) signal.**
- **entity (4)** — exact-recall plants (codename PELICAN, repo copper-kettle-7, locker 214,
  SSID teapot-guest) — the BLUEFIN/F-M2 shape.
- **episodic (4)** — queries whose answers live in episode digests, not turns; exercises the
  coarse tier of the two-tier index.
- **multi (4)** — 2–3 relevant docs with mixed grades; the ordering (nDCG) signal.
- **negative (3)** — no relevant doc exists. Recall/nDCG are skipped; the harness records the
  top-1 cosine score → **calibration data for the Ring-4 relevance floor** (currently `cos <= 0`,
  i.e. nearly everything injects; these scores tell us where a real floor should sit).

**Grading:** `2` = directly answers, `1` = partially relevant, absent = irrelevant. Binary
relevance for recall@k is grade ≥ 1; nDCG uses gain `2^grade − 1`.

**Proposed thresholds** (in the set file, ratified with it):

- `c1_recall_at_5_uplift: 0.10` — with 27 scored queries, recall@5 quantizes at ~0.037/query;
  +0.10 ≈ three net queries improved — above noise, honest at this N.
- `c2_ndcg_at_10_uplift: 0.05` — nDCG is continuous; +0.05 mean uplift on 27 queries is a real
  effect. If Qwen3 clears it, the 1024-dim default is earned; if not, the floor takes the default
  (smaller, faster, frees VRAM).
- `base_recall_at_5_floor: 0.70` — a sanity bar on the **baseline** (Qwen3 + identity): below
  this, something is wrong (set, server, or embedder) — investigate before deciding anything.
- `c1_rerank_p95_ms_budget: 1500` — the reranker must not only win, it must be affordable
  per-query on this box; p95 above budget weighs against ON even with uplift.

**Statistical honesty:** 30 queries is smoke-scale, not an IR eval. The thresholds are set at or
above quantization noise, the per-family breakdown catches "wins only on traps" patterns, and the
WORKLOG decision entry must state N. A falsifier this size can justify a *default*; it cannot
rank models finely — it doesn't need to.

**Ratification protocol (operator):** edit docs/queries/labels/thresholds freely → set
`ratified: true` + `ratified_by`/`ratified_date` → commit. A structural lint runs in CI
(`recall::tests`) asserting coherence only (ids resolve, families valid, negatives empty,
non-negatives labeled) — **never content**, so operator edits can't break the gate. Optional
extra ceremony (a `.frozen.json` hash like the main goldens) is available but not proposed:
this is a benchmark artifact, not a runtime gate; git history + the ratified flag suffice.

## 3 · The harness (built this slice, model-free-tested)

- **`svc::recall` (L4):** `Rerank` seam trait (mirrors `Embed`) + `IdentityRerank` (the shipped
  default — preserves cosine order, exactly `keel.lock rerank.default: identity`); `rank_all`
  (scored cosine ranking; `recall_top_k` unchanged); metrics `recall_at_k` / `ndcg_at_k` / `mrr`;
  set loader (`RecallSet::load` + `lint`); `run_recall_bench(&dyn Embed, Option<&dyn Rerank>, …)`
  — the whole pipeline is stub-testable without a model.
- **`keel-adapters::Reranker` (L2):** HTTP → llama-server `POST /v1/rerank`
  (`{model, query, documents, top_n}` → `results[].relevance_score`), sovereign-local like the
  embedder (scores/vectors never egress, I3). Parse unit-tested; live call `#[ignore]`d.
- **`keel recall-bench` (L5):** loads the set (default `tests/recall/golden-recall.json`), lints
  (hard-aborts on incoherence), embeds docs + queries against `--embed-endpoint` (default
  `:8090`, the keel.lock embed server), cosine-ranks, optionally reranks the top `--candidates`
  (default 20) via `--rerank-endpoint`, computes per-family + overall metrics + embed/rerank
  latency p50/p95, prints a summary, and writes a JSON artifact to `.keelstate/bench/`
  (verify-by-artifact). `--baseline <artifact>` prints recall@5/nDCG@10 uplift vs a prior run.
  **While `ratified: false` every output is stamped DRAFT and no decision line is printed.**
- **Not built (deliberately):** lifecycle-managed rerank-server launch and Ring-4 rerank wiring
  in `FileMemory::assemble`. That plumbing is only justified **if C1 decides ON** — building it
  first would be the §22 anti-pattern. For the bench, the rerank server is launched by the
  session script (ISSUE-8 pattern). `keel.lock` gains `rerank.file`/`rerank.port: 8091` now
  (config, not pins) so both the script and a future lifecycle read one source.

## 4 · The measurement plan (the focused live session, after ratification)

Three runs, one variant each, all local, TTL'd, file-redirected (ISSUE-8 discipline):

| run | embed server | rerank | artifact |
|---|---|---|---|
| baseline | Qwen3-Embedding-0.6B `:8090` (`--embeddings --pooling last`) | — | `recall-qwen3-identity.json` |
| C1 leg | same | Qwen3-Reranker-0.6B `:8091` (`--reranking`) | `recall-qwen3-rerank.json` |
| C2 leg | all-MiniLM-L6-v2 `:8090` (restarted) | — | `recall-minilm-identity.json` |

Then: `keel recall-bench --baseline` comparisons → **C1**: ON iff recall@5 uplift ≥ 0.10 AND
rerank p95 ≤ budget; **C2**: Qwen3 keeps the default iff nDCG@10 uplift over MiniLM ≥ 0.05, else
flip `substrate.embedding.id` to the floor. Either way the deliverable is the **decision**,
recorded in WORKLOG + the `keel.lock` flip (config, not a pin). A C2 embedder flip changes the
fingerprint → Ring-4 sidecars rebuild from the ledger automatically (`GOLDEN_RECALL` case 3 —
already green).

**Step 0 smoke (C1):** `qwen3-reranker-0.6b-q8_0.gguf` must expose a rank head to llama-server
(`--reranking` refuses otherwise — Qwen3 rerankers need the sequence-classification conversion).
If the on-disk GGUF is the raw causal variant, that's an ISSUE (re-provision), not a C1 verdict.

## 5 · What needs the operator (the honest list)

1. **Ratify the set** — edit freely, flip `ratified: true`. (The draft is mine; the ground truth
   becomes yours at ratification — same propose→ratify path as the 21 frozen goldens.)
2. **Provision the C2 floor** — `all-MiniLM-L6-v2` GGUF (~25–45 MB, Apache-2.0) into `C:\models`.
   It is already the keel.lock-pinned fallback (`fallback: all-minilm-l6-v2`), so this is
   provisioning the lock, not a new dependency — but downloads have been operator-authorized to
   date (ISSUE-10 precedent), so it's on your list, or say the word and the session does it.
3. **(Contingent)** re-provision the reranker GGUF if the §4 step-0 smoke refuses.

Filed as **ISSUE-11** in the ROADMAP register. C1/C2 stay `[?]` until the live session lands the
decisions.

## 6 · Non-goals / follow-up knobs (recorded so they aren't re-litigated)

- **Query instruction-prefixing.** Qwen3-Embedding scores higher with instruct-prefixed queries;
  Ring-4 today embeds raw text, so the bench measures the **as-built** pipeline. If the baseline
  disappoints, an instruct-prefix experiment is a one-line `Embedder` knob — a separate,
  set-reusing follow-up.
- **Relevance-floor tuning.** The negative-control top-1 scores in every artifact are the data;
  tightening the `cos <= 0` floor is a later, evidence-backed change.
- **sqlite-vec** stays closed (ISSUE-1) unless the bench's own latency numbers re-open it.

## 7 · ADDENDUM (same day, 2026-07-10): the DRAFT smoke ran live — findings that shape ratification

The full harness was smoked against the real organs the same session (DRAFT-stamped; artifacts in
`.keelstate/bench/`). Both legs lived: **baseline** (Qwen3 + identity) and **C1 rerank leg**
(Qwen3-Reranker-0.6B on `:8091`). §4's step-0 contingency is **retired**: the on-disk reranker GGUF
loads under `--reranking` (up in 6 s) and `/v1/rerank` scores correctly (relevant 0.9975 vs
distractor 1.3e-05) — ISSUE-11 item (3) is moot.

**Draft numbers (n=27 scored queries; NOT decision-grade until ratification):**

| leg | recall@5 | nDCG@10 | MRR | latency |
|---|---|---|---|---|
| Qwen3 + identity | 0.975 | 0.869 | 0.917 | embed p50/p95 = 18/30 ms |
| Qwen3 + reranker | 1.000 | 0.930 | 0.957 | rerank p50/p95 = **358/551 ms** |
| uplift | **+0.025** | **+0.061** | +0.040 | well under the 1500 ms budget |

The set discriminates where designed: keyword_trap MRR 0.750 → 0.833 (traps steal top-1 from
cosine; the cross-encoder recovers), multi recall@5 0.833 → 1.000, paraphrase MRR 0.938 → 1.000.
Negative-control top-1 cosines: **0.689 / 0.528 / 0.443** — the live `cos <= 0` floor would inject
all three; any future floor sits near ~0.7 on this embedder, or stays score-relative.

**The load-bearing finding — recall@5 saturates on the draft:** the identity baseline already
recalls 97.5% into the top-5, so the golden-named C1 measure (`recall_at_5_uplift >= 0.10`) has
only 0.025 of headroom — as drafted, C1 would mechanically decide OFF even though the reranker
visibly improves ordering everywhere (nDCG +0.061, MRR +0.040). **Recommendation (Remedy A): harden
the corpus to v2 before ratifying** — grow it ~2–3× with near-topic confusable distractors (same
entities/domains, wrong facts) and add queries engineered so identity's recall@5 lands ~0.6–0.8
(the per-query artifact data shows where cosine ranks each relevant doc — the authoring feedback
loop). That restores recall@5 headroom and keeps both the frozen golden's letter and the proposed
thresholds intact. The alternative (redefining C1's input as ordering uplift) would drift from the
frozen golden's named measure — not worth it when hardening the set achieves the same honestly.

v2 authoring is a session task on the operator's word (or the next unsupervised session's, if
ungated); ratification then applies to v2.

## 8 · ADDENDUM (2026-07-10, operator go): v2 HARDENED + measured — ready to ratify

The v2 pass landed (operator: "go ahead with the v2 hardening pass"). The corpus grew **42 → 108
docs** (near-topic confusable clusters engineered around the hard queries — same entities/domains,
wrong facts, all judge-honest grade-0s) and **30 → 41 queries** (36 scored + 5 negatives). Two v1
queries were re-specified for judge-clarity (`q03` "my medication", `q09` "software release");
six relevance maps were extended where the new corpus made additional docs honestly relevant
(q06/q16/q21/q22/q27/q10/q24/q35/q39). The harness gained per-query `top_ids` (the final top-10
ranking in every artifact) — the authoring/ratification feedback loop that drove the three
measure→reinforce iterations (0.975 → 0.886 → 0.842 → 0.786).

**v2 DRAFT numbers (real organs, DRAFT-stamped; artifacts in `.keelstate/bench/`):**

| leg | recall@5 | nDCG@10 | MRR | latency |
|---|---|---|---|---|
| Qwen3 + identity | **0.786** | 0.718 | 0.772 | embed p50/p95 = 14/56 ms |
| Qwen3 + reranker | 0.856 | 0.828 | 0.859 | rerank p50/p95 = 534/676 ms |
| uplift | **+0.070** | **+0.111** | +0.087 | ≤ budget |

The baseline sits **inside the 0.6–0.8 target window** — the saturation problem is fixed and the
golden-named measure has real headroom (family floor: keyword_trap 0.611, episodic 0.694; paraphrase
deliberately stays high at 0.963 — recall must still work). The reranker leg recovers six queries
with zero regressions (q34 0→1, q06/q39/q35 to full, q10/q27 partial) and leaves four hard cases
down for both (q11 dentist, q14/q32/q33 traps) — honest hard cases, not artifacts.

**The decision input ratification must settle (numbers, not speculation):** as drafted
(`c1_recall_at_5_uplift: 0.10`) the draft reranker uplift **+0.070 would decide C1 = OFF**, even
though ordering gains are large (nDCG **+0.111**, MRR +0.087) and every change is an improvement.
With n=36, +0.070 ≈ 2.5 net queries — above quantization noise (~0.028/query). The operator's
threshold choice at ratification is the real C1 policy call:
- **keep 0.10** → C1 likely OFF (reranker not worth a third server for top-5 *presence*), or
- **ratify 0.05** → C1 likely ON (ordering + presence gains justify it at 534 ms/query), or
- keep 0.10 and note the nDCG evidence in the WORKLOG decision either way.
The agent deliberately does NOT pre-pick this — it is exactly the "operator-authored ground truth"
the design review exists for. (The C2 leg is unchanged: blocked only on the MiniLM download.)

## 9 · DECISION RECORD (2026-07-10, operator delegation: "proceed for all at your recommendation")

The set was **ratified at v2 with the thresholds kept as pre-registered** (0.10 / 0.05 / 0.70 /
1500) — the recommendation explicitly refused to move a bar after seeing a number in the direction
that flips an outcome. MiniLM f16 was provisioned byte-exact (45,949,216 B). Three decision-grade
legs ran on the ratified set (artifacts: `.keelstate/bench/recall-*.json`).

**C1 — reranker vs identity: OFF (identity stays).** Uplift +0.070 recall@5 < 0.10 (nDCG@10
+0.111, MRR +0.087, zero regressions, p95 594 ms recorded). KEEL injects top-k memories wholesale,
so presence beats ordering, and the presence gain doesn't buy a permanent third managed server +
~0.5 s on every assemble. The `Rerank` seam/adapter/bench stay built; a cell can wire them.
Re-open: organic recall misses in lived use, or a `recall_k=1` pattern.

**C2 — embedder vs the MiniLM floor: the FLOOR takes the default (falsifier trip — the expected
direction inverted).** On the as-built pipeline (raw queries, live-shaped texts) MiniLM-L6-v2 f16
beat Qwen3-Embedding-0.6B-Q8 in **every family**: recall@5 0.906 vs 0.786, nDCG@10 0.840 vs 0.718
(Qwen3 uplift over the floor = **−0.122** vs the +0.05 bar), episodic +0.25, traps +0.14, 3× faster
(p50 5 ms), better negative separation (0.29–0.49 vs 0.44–0.69). **Flip executed + lived:**
keel.lock → minilm/384/`pooling: mean` (pooling config-driven through manifest + lifecycle);
fingerprint flip cleared + rebuilt sidecars from the ledger (0→30 vecs, 384-dim, by artifact);
write-side dim-guard added (a stale wrong-model server can never fossilize vectors); embed input
head-capped at 1500 chars (MiniLM's 512-token window; stored text stays full). Caveats recorded:
trap clusters were adversarially selected against Qwen3's rankings (the floor's win is broad-based
across untuned families regardless), and Qwen3 ran without its instruct-prefix query format.
Re-open: the instruct-prefix experiment · a symmetric-hardening set pass. Qwen3 stays on disk as
the lock `fallback`.

Both decisions, rationale, and re-open triggers are also annotated in `keel.lock` and recorded in
the WORKLOG. `GOLDEN_RECALL` is now fully accounted for: the deterministic case is CI-green, the
two uplift cases are measured-and-decided (conformance-coverage.md updated).
