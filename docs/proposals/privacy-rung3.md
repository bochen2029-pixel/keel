# Privacy rung-3 — the A5 design (provisioned + scoped 2026-07-10; the ort build is next)

*Status: the model is ON DISK, byte-verified; the seam + thresholds are DECIDED here; the
implementation session builds against this. Polish item #1 (E2 exclusion #1, operator-un-gated
2026-07-10: "go ahead with A5").*

## 1 · The model (verified on disk this session)

**`openai/privacy-filter`** (HF, Apache-2.0, released 2026-04-17, 335K downloads) →
`C:\models\privacy-filter\` — every file byte-exact vs content-length:
`onnx/model_quantized.onnx` (162,239 B graph) + `onnx/model_quantized.onnx_data`
(**1,618,042,064 B** — the int8 weights; CPU-friendly) · `tokenizer.json` (27.9 MB, HF-tokenizers
format) · `config.json` (arch `OpenAIPrivacyFilterForTokenClassification`; **33 BIOES labels = O +
8 PII classes × {B,I,E,S}**: account_number, private_address, private_date, private_email,
private_person, … ; `max_position_embeddings` 131072, hidden 640) · `viterbi_calibration.json`
(named transition biases, all-zero "default" operating point — **the recall bias is ours to set,
exactly the keel.lock `operating_point: recall` design: a NON-model knob**).

The quantized-CPU variant is deliberate: rung-3 runs **in-process on CPU** (like the embedder
runs on its own port) — zero VRAM contention with the LLM/embed servers, no server to supervise.

## 2 · The seam (layer-legal, minimal-core)

- **`keel-middleware` defines the trait** (the AuditSink pattern):
  `pub trait PiiClassifier: Send + Sync { fn spans(&self, text: &str) -> Vec<PiiSpan> }` with
  `PiiSpan { start, end, class }`. `Redactor` gains an optional `Arc<dyn PiiClassifier>` — **rung 3
  is ADDITIVE-ONLY**: model spans are masked *in addition to* rung-1/2 findings, audited as
  `rung3:{class}` labels (never values); the model can never *unmask* or override a deterministic
  finding (canon §5.1 — the guarantee stays on rungs 1–2 forever).
- **The impl lives in `keel-services` behind a `privacy-model` feature** (services→middleware is
  layer-legal; the mic/screen precedent keeps the heavy dep out of the default genome):
  `svc::privacy_model::OnnxPiiClassifier` — `ort` session over the quantized graph + `tokenizers`
  over tokenizer.json + **KEEL-owned Viterbi BIOES decode** (deterministic; biases loaded from
  `viterbi_calibration.json` + a keel.lock-tunable recall bias on `background_to_start`).
- **L5 wires it** when the feature is on AND the manifest resolves the files
  (`substrate.privacy.file/tokenizer` — config, not pins), with **graceful degrade**: load
  failure ⇒ rungs 1–2 only + one stderr note (a turn is never blocked on rung 3).
- **Placement: EGRESS-ONLY by default** (the sovereignty-critical direction — PII leaving the
  box; local turns skip it, so the 1.5 GB model never taxes the $0 path). An output-side flag is
  a later knob if evidence wants it.
- **New deps (the ISSUE-2 decision, taken):** `ort` 2.x (pyke; downloads the onnxruntime CPU
  binary at build — heavy, hence feature-gated) + `tokenizers` (HF's Rust core). Both behind
  `privacy-model`; the default `cargo build` pulls neither.

## 3 · Pre-registered C3 thresholds (BEFORE any measurement — the house template)

C3 = "privacy model vs deterministic-only on `GOLDEN_PRIVACY`". Rung-3 flips keel.lock
`privacy.default: off → on` iff ALL hold:

1. **The frozen golden case passes:** "third-party name in context the model adds over
   deterministic" → `model_adds_span: true` on the Dana-style input (rungs 1–2 provably miss it).
2. **Recall uplift ≥ +0.30 absolute** on a ~30-item labeled fixture (`tests/privacy/rung3-set.json`,
   authored fictional: person names / addresses / dates-in-context that no regex catches, plus
   clean negatives) — model+deterministic vs deterministic-only, on the model-class items.
3. **False-positive rate ≤ 10%** on the clean negatives (over-redaction is the CHOSEN bias —
   leak-uncertain ⇒ redact — but bounded; the fixture records every FP for operator review).
4. **p95 classify latency ≤ 500 ms** per egress call at typical prompt sizes on CPU (an inline
   egress mask must not double a cloud turn's latency; measured by the bench, artifact to
   `.keelstate/bench/`).

Fail any ⇒ rung-3 stays OFF (the machinery remains built + one flag away), decision + re-open
triggers recorded — the C1/B1 pattern.

## 4 · The build plan (one focused session)

1. `cargo add ort tokenizers --features ...` gated under `privacy-model` in keel-services; first
   build is minutes (onnxruntime fetch) — stop keel-serve first (sibling-bin lock).
2. Inspect the ONNX graph I/O once loaded (input_ids/attention_mask → logits [tokens × 33]);
   wire tokenize → run → Viterbi(BIOES, calibrated biases + recall bias) → char-offset spans
   (tokenizer offsets ride tokenizer.json).
3. Trait + Redactor slot + I1 audit labels + tests (stub classifier for the middleware tests —
   model-free; the ort path live-`#[ignore]`d + the golden harness).
4. `GOLDEN_PRIVACY` case green · author the fixture · run the C3 bench · decide per §3.
5. Gate → docs → commit → push. keel.lock `privacy.file/tokenizer` added now (this checkpoint);
   `sha256:` stays TODO (operator pins, ISSUE-6).

## 5 · Risks

ort 2.x API drift vs my knowledge (post-cutoff releases — read the crate docs at build) · the
ONNX graph may expect extra inputs (position ids / MoE routing tensors — inspect, don't assume) ·
26 MB tokenizer load time (lazy-init once per process) · int8 quality vs the fp16 variant (if
recall disappoints, `model_fp16` is on the same repo — a re-download away, threshold unchanged).
