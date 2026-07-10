# Flywheel ignition — the turnkey runbook (B3 DECIDED 2026-07-10: deferred on evidence)

## The decision (ISSUE-5 closed)

**B3 = the base case HOLDS; ignition is DEFERRED ON EVIDENCE.** Measured 2026-07-10 over the
lifetime index (73 turns, including D2/SEXTANT's real cell traffic): `escalation_rate` **0.000**
— not one turn has ever escalated above its kind's base tier — and `rework_rate` **0.014** (one
verify-fail in 73). The corpus holds **59 verified, secret-scrubbed pairs** (4 → 59 since June:
the B2/B4 accumulation machinery works). Distillation exists to reduce escalations; there are
none to reduce, and 59 pairs is ~10× below the floor where a LoRA demonstrates anything (it
mostly risks degrading a model failing at 1.4%). The canon pre-authorized exactly this outcome
(§21: *"size everything to the base case where `escalation_rate` stays flat — ignition is
upside"*; ROADMAP: *"flat is an acceptable decided outcome"*). Flat-at-zero also satisfies the
keel.lock `escalation_rate: downward_trend` threshold vacuously — the live watch signal is
therefore **"does not RISE"**, checked by any `keel metrics` read.

## Pre-registered ignition triggers (any ONE re-opens B3 as a live training session)

1. **Corpus ≥ 500 verified pairs** (check: `keel distill-export` prints the count), AND at least
   one of:
2. **`escalation_rate` > 0.02 sustained** across two `keel metrics` reads a week apart (the local
   model has real failures for distillation to target), or
3. **`rework_rate` > 0.05** (verify-fails climbing — capability slipping), or
4. **A cell-identified capability gap with its own eval set** (e.g. SEXTANT tailoring quality
   graded against the SPEC §20 golden set, or the amplify-set logic family, where pass@8 showed
   sampling cannot fix a wrong mode — a task-specific LoRA has a measurable target).

## The turnkey pipeline (when triggered — one focused session, out-of-band per canon §21)

0. **Preflight:** GPU free (stop `keel-serve` + llama-servers for the training window); disk ≥
   60 GB; a fresh venv (`python -m venv .venv-train`) — never the system Python.
1. **Data:** `keel distill-export --out training.jsonl` (already chat-format
   `{messages:[user,assistant]}`; scrubbed at corpus-write, B2). Split 90/10 train/eval by hash.
2. **Train (Unsloth, 4-bit QLoRA):** base = the keel.lock local model's HF repo (Qwen3.5-9B —
   ~18 GB download, `disposition: shared` under `C:\models\hf\`). r=16, lr 2e-4, 1–3 epochs,
   early-stop on the eval split. Artifact: the LoRA adapter dir.
3. **Merge + convert:** merge the adapter (unsloth `save_pretrained_merged`) → llama.cpp
   `convert_hf_to_gguf.py` → `llama-quantize` to Q5_K_M — same quant as the incumbent, so the
   comparison is quant-fair.
4. **Swap behind the fingerprint disciplines:** the new GGUF lands beside the incumbent in
   `C:\models`; keel.lock `substrate.llm_vision.file` flips (config, not a pin; the incumbent
   stays on disk — one-line rollback). NOTE: vision/mmproj compatibility must be re-validated
   (`nightscribe snap` is the lived check).
5. **Measure (pre-registered, the C1/C2/B1 template):** (a) re-run `keel amplify-bench --n 8` —
   the fixed 25-task set is the capability regression guard (pass@1 must not drop >0.05); (b) the
   triggering metric itself (escalation/rework trend over the next N live turns, or the cell's
   eval set); (c) golden gate untouched (`cargo test` — the frozen goldens are model-free and
   must stay green). **Decide KEEP or ROLLBACK; record in WORKLOG + keel.lock.**

## Standing watch (no action, zero cost)

Every session that runs `keel metrics` sees the two numbers. The corpus grows by itself (B2
emit-on-pass). When trigger 1 + any of 2–4 hold, this file is the session brief.
