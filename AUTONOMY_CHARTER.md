# AUTONOMY_CHARTER — KEEL (Stage 0, supervised)

**System:** Bo Chen's personal, sovereign harness core. Single operator, single tenant.

## §1 — Authority
Stage 0 runs **supervised**. No overnight/unattended autonomy until the run-state spine (I2) + the oracle layer (I5, Stage 2) exist and the operator explicitly authorizes it.

## §5 — Hard prohibitions (the reversibility gate)
- No `git reset --hard`, `git clean -fd/-fx`, `git checkout -- <path>`, `git restore` on uncommitted/unmerged work.
- No `git push --force`; no `branch -D` on unmerged `auto/` branches.
- No `rm` / `Remove-Item -Recurse -Force` outside `.\.keelstate\` and explicitly ephemeral dirs.
- **No mutation of the global Rust toolchain** (rustup update / reinstall / component remove/add) without operator approval — DAVE, TERMINAL, and other projects share it.
- **No cloud egress of sovereign data** — including raw perception (screen / webcam / microphone frames) and embedding **vectors** (I3; vectors are invertible).
- **A secret must never be baked into a distilled LoRA** — scrub before a verified trace becomes distillation feedstock (the flywheel is the one irreversible sink).
- Any action whose undo cost cannot be stated in one sentence → **stop and ask the operator.** Reversibility-uncertain ⇒ treat as irreversible.

## §7 — Termination
- Foundational-blocker fan-out ≥ 3 → prefer an early clean stop over speculative dependents; write an `ESCALATION` note and halt.

## Frozen ground truth
The operator treats the **ten contracts** (`crates/keel-contracts`), the **golden cases** (`tests/golden/golden.json`), and `.frozen.json` like this charter: read-only, never self-edited. Authoring/ratifying/changing any of them is an **operator action**. The agent may not author its own ground truth (I5).
