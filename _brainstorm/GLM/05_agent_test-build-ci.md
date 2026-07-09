# Sub-agent 5 — Test / Build / CI / Ops surface (verbatim)

> Original task: survey tests, build tooling, CI, scripts, daemon ops, lints/conformance. READ-ONLY.

---

# KEEL — TEST, BUILD, CI, and OPS surface

## 1. Tests

### Layout & approach
Tests are overwhelmingly **inline `#[cfg(test)]` modules** appended to each source file. There are ~135 test functions total across 30 source files. Only one file lives in a crate `tests/` directory (an integration test). Coverage is "model-free by design": stub tiers/routers/oracles/spines stand in for real model + substrate calls, and the *wiring* (the engine loop, the invariants, the golden dispatch) is what's asserted. Live model calls exist only as `#[ignore]` tests.

The single integration test file (per `git ls-files`):
- `C:\KEEL\crates\keel-contracts\tests\golden_freeze.rs` — 1 test, the freeze-gate (see §6).

Test-function counts per file (top contributors):
- 22 — `crates\keel-kernel\src\engine.rs` (the canonical loop: cost fold, I2 checkpoint, flywheel emit, ladder fallback, escalation, the I5 "teeth" — fail-closed on unresolved golden_ref, config-fault on critical-with-no-oracle, baseline-exclusion, ASCII-alarm)
- 15 — `crates\keel-services\src\perception.rs`
- 10 each — `crates\keel-services\src\memory.rs`, `crates\keel-middleware\src\privacy.rs`
- 8 — `crates\keel-services\src\verifier.rs`; 6 — `crates\keel-store\src\lib.rs`, `crates\keel-adapters\src\local_llama.rs`
- 5 — `crates\keel-services\src\driver.rs` (HeartbeatDriver/WatchDriver), `crates\keel-kernel\src\lifecycle.rs` (substrate resolver)
- plus smaller modules in every adapter and middleware crate.

dev-dependencies are uniformly `tokio = { features = ["rt","macros"] }`; `keel-contracts` additionally pulls `sha2` for the freeze-gate. No `#[ignore]`-by-default gating beyond the local-LLM live model test in `local_llama.rs:180`.

### Test fixtures / golden files
Golden fixtures live at the workspace root (not per-crate) and are shared by `CARGO_MANIFEST_DIR/../../tests/golden`:
- `C:\KEEL\tests\golden\golden.json` — the RATIFIED+FROZEN conformance matrix. Six sections: `router`, `model_tier`, `oracle`, `perception`, `recall`, `privacy` (plus `_meta`). Each is an array of `{name, input, expect}` cases.
- `C:\KEEL\tests\golden\.frozen.json` — the operator-frozen sha256 seal (`db4377b3…`, version 0.2.0).

Per-crate conformance harnesses each read this same `golden.json` and assert their slice passes — the "language-neutral conformance layer" pattern:
- `crates\keel-services\src\router.rs:167` `passes_golden_router` (the `router` section)
- `crates\keel-services\src\verifier.rs:312` `passes_golden_oracle` (the `oracle` section)
- `crates\keel-adapters\src\local_llama.rs:184` `passes_golden_model_tier` (the `model_tier` section, model-free cost/decode side; live model is `#[ignore]`)
- `crates\keel-services\src\perception.rs:336` and `crates\keel-services\src\recall.rs` reference `GOLDEN_PERCEPTION` / `GOLDEN_RECALL` conformance.

There is also `C:\KEEL\crates\keel-contracts\examples\golden_freeze.rs` — an **operator-only** tool (not a test) to (re-)write `.frozen.json` via `cargo run -p keel-contracts --example golden_freeze -- --update`.

There are **no separate `*_test.rs`/`test_*.rs` files** and **no per-crate integration `tests/` dirs** other than `keel-contracts`.

---

## 2. Build / tooling

### Workspace
`C:\KEEL\Cargo.toml` — minimal:
```
[workspace]
resolver = "2"
members = ["crates/keel-contracts","crates/keel-kernel","crates/keel-middleware",
           "crates/keel-adapters","crates/keel-store","crates/keel-services","crates/keel"]
[workspace.package]
edition = "2021"
version = "0.2.0"
license = "MIT"
authors = ["Bo Chen"]
```
No `[profile.*]` overrides, no `[workspace.dependencies]`, no metadata tables.

### Missing standard build tooling
Per `git ls-files` filtered for build config: **none present**. There is no:
- `justfile`/`Justfile`/`Makefile`
- `.cargo/config.toml`
- `build.rs` anywhere
- `rust-toolchain.toml` / `rust-toolchain`
- `rustfmt.toml` / `.rustfmt.toml`
- `clippy.toml` / `.clippy.toml`
- `deny.toml` (cargo-deny)

So the build entrypoint is plain cargo. The default-run is `keel` (`crates\keel\Cargo.toml`). Two binaries:
- `keel` → `crates\keel\src\main.rs` (the daily-driver CLI + daemon host)
- `keel-serve` → `crates\keel\src\bin\keel-serve.rs` (OpenAI-compatible HTTP server on `127.0.0.1:7070`)

Typical commands (inferred, no wrapper): `cargo build`, `cargo test --workspace`, `cargo run -p keel -- daemon …`, `cargo run --bin keel-serve`.

### Feature flags
- `keel-adapters`: `mic` (cpal), `screen` (xcap) — both off by default (minimal-core thesis).
- `keel-services`: `mic`/`screen` facade the above.
- `keel-store`: `rusqlite` with `bundled` (SQLite compiled in).
- `jsonschema` in keel-services is `default-features = false` (no network egress in the oracle).

---

## 3. CI

**There are no CI pipelines.** `.github/` does not exist; `git ls-files` returns zero `.yml`/`.yaml`/`.gitlab-ci.yml`/`azure-pipelines.yml`/`.circleci`/`.travis.yml`. The `golden.json` `_meta.freeze` comment says "the Stage-0 CI gate enforces it" and CLAUDE.md references "the Stage-0 CI gate," but **no such pipeline is committed** — the freeze-gate is currently enforced only by the local `cargo test` run of `golden_freeze.rs`. No fmt/clippy/audit/test automation is wired into any provider.

---

## 4. Scripts

Three script locations (all operator/aux tooling, not part of the Rust build):

- `C:\KEEL\tools\keel-autoloop.ps1` — the external "build supervisor" Driver. Respawns headless `claude -p` sessions that self-bootstrap from `_run_state\AUTOSTART.md`, execute ROADMAP slices, commit+push, exit, respawn. Params: `-MaxSessions 50`, `-StallLimit 2`, `-ClaudeFlag` (default `--dangerously-skip-permissions`). Stops on `.keelstate\DONE`/`STALLED` sentinels, a HEAD-unchanged stall, or the session cap. Logs to `.keelstate\autoloop.log`. This is the temporary external loop the docs say the in-process `keel daemon` (ROADMAP A2) is meant to replace.
- `C:\KEEL\chunker\chunker.py` (17 KB) + `chunk.cmd` + `estimate_tokens.py` — a text-chunking utility (produces the `_*_chunks` dirs) for token estimation / transcript splitting. Aux, not part of the build.

No `scripts/` directory exists; the only ops script is the autoloop PowerShell.

---

## 5. Daemon / ops

### Where the daemon is defined
The canonical loop logic lives in the kernel (L1), and the continuously-running daemon is a thin L5 wrapper:
- `C:\KEEL\crates\keel-kernel\src\engine.rs` — `Engine::tick` (select a driver Step, run the full loop), `Engine::run_until_idle` (bounded burst form, the *testable* daemon), `select()` (priority-order driver poll). The docstring is explicit: "The continuously-running daemon (idle = sleep, run forever) is a thin L5 wrapper, deliberately not started here."
- `C:\KEEL\crates\keel\src\main.rs:39-41` dispatches `keel daemon` to `run_daemon()` (`main.rs:170-278`).
- `C:\KEEL\crates\keel\src\lib.rs` — `Engine::tick`/`run_until_idle` delegations, `watch_token`, `daemon_perpetual` (the bounded-vs-perpetual decision rule), and 2 unit tests.

### How it's run / deployed
Invocation: `keel daemon [--manifest PATH] [--max-ticks N] [--interval MS] [--watch PATH] [--prompt …] [--kind core-wire] [--sovereign] [--consolidate-every N]`.
- **Bounded by default** (`--max-ticks 1`, terminates). `--max-ticks 0`, or `--watch` without an explicit bound → perpetual until interrupted (`daemon_perpetual`, `lib.rs:331`).
- Wires a `HeartbeatDriver` (+ optional `WatchDriver` over `--watch PATH`); priority order heartbeat → watch (`main.rs:221`).
- Each tick gets a distinct `trace_id` (`{base}-{attempt}`) so each turn checkpoints as its own run.
- Per-turn report goes to stderr: `tier`, turn + run cost, I5 verdict, answer first line (`report_daemon`, `main.rs:282`).

### Per-tick budget (the recent commit `6ac319d`)
This is the "daemon per-tick budget" fix — at `C:\KEEL\crates\keel\src\main.rs:240-244`, each tick re-seeds the per-task budget headroom so a perpetual paid daemon never climbs one shared budget into a permanent `BudgetExceeded`:
```rust
// M3 (audit): each daemon tick is its own task — re-seed the per-task budget headroom …
ctx.task_budget = Some(ctx.cost.total + manifest.cost.budget_per_task);
```
`cost.total` stays cumulative (final run-cost report); only the remaining headroom resets. The budget primitives: `Context.task_budget` (`keel-contracts\src\types.rs:366`), `budget_remaining()` (`types.rs:377`), seeded in `new_context` (`keel-kernel\src\context.rs:30`), defaults `budget_per_task=5.0`, `hard_stop_at=1.0` (`keel-kernel\src\manifest.rs:152-170`). The pre-call cost gate that consumes it is `keel-middleware\src\cost.rs`.

### Daemon config
There is **no separate daemon TOML/YAML**. All daemon runtime config comes from:
- `C:\KEEL\keel.lock` (the substrate/manifest): `tiers`, `router` (default_tier, sovereign_forces, escalate_after_oracle_failures), `cost` budget knobs, `servers.llama_cpp` launch paths, `resolver_order`. Loaded via `Manifest::load`.
- CLI flags (above) for tick bounds/interval/watch.

Daemon operational state is written under `C:\KEEL\.keelstate\` (audit.jsonl, index.db, tape/, traces/, llama-server.log) — runtime artifacts, not config. Related subcommands in the same binary: `keel consolidate`, `keel cold-eyes`, `keel distill-export`, `keel metrics`.

---

## 6. Lints / conformance

### Lints
There is **no clippy config** (`clippy.toml`/`.clippy.toml`) and **no `deny.toml`** (cargo-deny). The only "lint-like" discipline encoded in tests is an ASCII-only check on operator-facing strings (route reasons and I5 alarms must render on any codepage) — e.g. `router.rs:221 route_reasons_are_ascii`, `engine.rs:863 the_failure_alarm_is_ascii`. No `#![deny(warnings)]` or workspace-level lint attributes were observed in the manifests.

### Conformance = the golden freeze-gate + per-crate golden slices
"Conformance" is KEEL's term for the **language-neutral contract layer** (canon §7, ADR #5: the implementation language is the most reversible decision). It is enforced at two levels:

1. **Freeze-gate (the seal)** — `C:\KEEL\crates\keel-contracts\tests\golden_freeze.rs:44` `goldens_match_the_frozen_hash`. Re-hashes `tests/golden/golden.json` (drop `_meta`, sorted-key compact JSON, sha256) and asserts equality to the operator-frozen hash in `.frozen.json`. Failure message: "GOLDEN FREEZE MISMATCH … Resolution is an OPERATOR action (the agent fixes code, never a golden)." Re-freeze is operator-only via the `golden_freeze` example. This is the "Stage-0 CI gate" the docs reference — though no CI runs it automatically today (see §3).

2. **Per-crate conformance slices** — each `passes_golden_*` test (cited in §1) drives its frozen `golden.json` section through the real implementation and asserts the expected `tier`/`passed`/`joint_wrong`/`reason_contains`. These are the runtime gates; the freeze-gate guarantees the ground truth itself doesn't drift.

### The "tier guard" (commit `6ac319d` "--tier guard")
This is **not** a tier *system* in the routing sense — it's the I3/I5 safety guard in the CLI at `C:\KEEL\crates\keel\src\main.rs:99-110`. The manual `--tier` override skips the router (the I3 force-local gate) **and** the engine (the I5 verifier), so flags that rely on them must be refused rather than silently voided:
- `--sovereign` + non-local `--tier` → exit 2 ("cannot be honored … skips the I3 force-local gate").
- `--critical` or `--golden-ref` + any `--tier` → exit 2 ("require the self-driving path … the manual override skips verification").

The actual **tier system** (the routing economy) is the three-rung ladder `["local","cheap-API","frontier"]` (`engine.rs:44`), configured in `keel.lock` `tiers:`/`router:`, routed by `DifficultyRouter` (`keel-services\src\router.rs`), with cross-turn escalation after `escalate_after_oracle_failures` (default 2) and down-ladder fallback when a tier is unplugged (`Engine::resolve_down`). The I5 "teeth" that gate it are enforced inside `Engine::run` (`engine.rs:205-222`): unresolved `golden_ref` → fail-closed; `critical` step with no correctness-oracle evidence → config-fault; baseline oracle excluded from the critical guard.

---

## Summary

- **Tests:** ~135 inline `#[cfg(test)]` tests across all 7 crates + 1 integration freeze-gate. Model-free stub-based; golden-driven conformance. Fixtures at workspace `tests/golden/`. No per-crate `tests/` dirs besides keel-contracts.
- **Build:** Cargo workspace, resolver 2, no profiles. No justfile/Makefile/build.rs/.cargo/rust-toolchain. Entry: `keel` + `keel-serve` bins. Optional `mic`/`screen` features.
- **CI:** **None committed.** The freeze-gate that docs call "the Stage-0 CI gate" exists only as a local `cargo test`.
- **Scripts:** `tools/keel-autoloop.ps1` (external respawn supervisor) + `chunker/*.py` (text chunking). No `scripts/` dir.
- **Daemon:** Logic in `keel-kernel::engine` (`tick`/`run_until_idle`/`select`); perpetual wrapper in `keel/src/main.rs::run_daemon`. Config via `keel.lock` + CLI flags. Per-tick budget re-seed at `main.rs:244`. State under `.keelstate/`.
- **Lints/conformance:** No clippy/deny config. Conformance = golden freeze-gate (`golden_freeze.rs`) + per-crate `passes_golden_*` slices. "Tier guard" = the `--tier` override refusal in `main.rs:99-110`; the tier *system* is the local/cheap-API/frontier ladder in `engine.rs:44` + `DifficultyRouter`.

One notable gap worth flagging to whoever asked: there is **no committed CI** despite the architecture docs repeatedly referencing a "Stage-0 CI gate" that enforces the golden freeze — today that gate only fires if a human runs `cargo test --workspace` locally.
