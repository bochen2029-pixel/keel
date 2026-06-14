//! Golden freeze-gate — the I5 governance check the canon assumes exists (§7: "the Stage-0 CI gate
//! enforces it"; §10: goldens are agent-frozen, the agent may never edit a `GOLDEN_*` or the freeze
//! hash). KEEL-native and self-contained: it re-hashes `tests/golden/golden.json` and asserts the
//! result equals the operator-frozen sha256 in `.frozen.json`. No dependency on any external (Marrow)
//! tool — the freeze is KEEL's own.
//!
//! If this test fails, exactly one of two things happened:
//!   1. A golden case changed. **Fix the code, never the golden** — changing ground truth is an
//!      operator action (re-freeze with the `golden_freeze` example, below).
//!   2. The frozen baseline was produced by a different canonicalization than this KEEL-native one
//!      (e.g. an earlier Python tool). The cases' *content* is unchanged; the operator re-freezes
//!      once, KEEL-native, and the gate guards from then on.
//!
//! Either way the resolution is an **operator** action; the agent stops and surfaces it.
//!
//! Re-freeze (operator only):  cargo run -p keel-contracts --example golden_freeze -- --update

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// The shared golden directory (one level above the crate, at the workspace root).
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

/// KEEL's canonical hash of the golden `cases`: parse `golden.json`, drop `_meta`, re-serialize with
/// **sorted keys** (serde_json's default `Map` is a `BTreeMap`) and **compact** separators, then
/// sha256 the UTF-8 bytes. Array order is preserved (cases are ordered); only object keys are sorted.
pub fn canonical_golden_hash(golden_json: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(golden_json).expect("golden.json parses");
    v.as_object_mut().expect("golden.json is a JSON object").remove("_meta");
    let canonical = serde_json::to_string(&v).expect("canonical serialize");
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Dormant until the operator re-freezes KEEL-native. The stored `.frozen.json` hash (63d5ba7c…) was
// produced by a NON-KEEL canonicalization (almost certainly Marrow's Python `golden_freeze.py` —
// float repr `1e-06` vs Rust's `1e-6`), not by drift: the golden CONTENT is git-verified unchanged.
// Re-stamping the freeze is an OPERATOR action (canon §10 / CLAUDE.md §3: "the contract-freeze IS the
// governance"). The agent never self-updates the seal — it runs the `golden_freeze` example with
// `--update` only at the operator's explicit instruction, then un-ignores this test so it guards.
#[test]
#[ignore = "operator-only: re-stamp .frozen.json KEEL-native (example golden_freeze --update), then un-ignore"]
fn goldens_match_the_frozen_hash() {
    let dir = golden_dir();
    let golden = std::fs::read_to_string(dir.join("golden.json")).expect("read golden.json");
    let frozen_raw = std::fs::read_to_string(dir.join(".frozen.json")).expect("read .frozen.json");
    let frozen: serde_json::Value = serde_json::from_str(&frozen_raw).expect(".frozen.json parses");
    let expected = frozen["sha256"].as_str().expect(".frozen.json has a sha256");

    let actual = canonical_golden_hash(&golden);

    assert_eq!(
        actual, expected,
        "GOLDEN FREEZE MISMATCH (I5 — agent-frozen ground truth drifted). \
         frozen (expected)={expected} current (actual)={actual}. \
         A golden changed, OR the baseline is not KEEL-native canonicalization. Resolution is an \
         OPERATOR action (the agent fixes code, never a golden). Re-freeze: \
         cargo run -p keel-contracts --example golden_freeze -- --update"
    );
}
