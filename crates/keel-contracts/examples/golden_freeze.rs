//! KEEL-native golden freeze tool — **operator-only**. The freeze is KEEL's own now; it depends on
//! no external (Marrow/Python) script.
//!
//!   cargo run -p keel-contracts --example golden_freeze              # dry run: print the hash
//!   cargo run -p keel-contracts --example golden_freeze -- --update  # (re-)write .frozen.json
//!
//! Re-freezing ratifies ground truth (I5) — it is an **operator action**. The agent prints/inspects
//! but never runs `--update`. The canonicalization here is identical to the gate in
//! `tests/golden_freeze.rs`: drop `_meta`, sorted-key compact JSON (serde_json default), sha256.

use sha2::{Digest, Sha256};
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    let golden = std::fs::read_to_string(dir.join("golden.json")).expect("read golden.json");

    let mut v: serde_json::Value = serde_json::from_str(&golden).expect("parse golden.json");
    let version = v["_meta"]["version"].as_str().unwrap_or("0.2.0").to_string();
    v.as_object_mut().expect("golden.json is an object").remove("_meta");
    let canonical = serde_json::to_string(&v).expect("canonical serialize");
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    println!("canonical sha256 = {hash}");

    if std::env::args().any(|a| a == "--update") {
        let frozen = serde_json::json!({
            "sha256": hash,
            "version": version,
            "note": "sha256 of golden.json `cases` (every section except _meta), canonical-sorted \
                     (KEEL-native: serde_json sorted-key compact JSON). Operator-frozen; agent \
                     read-only. Verified by tests/golden_freeze.rs."
        });
        let out = serde_json::to_string_pretty(&frozen).expect("serialize frozen") + "\n";
        std::fs::write(dir.join(".frozen.json"), out).expect("write .frozen.json");
        println!("WROTE {} — operator re-freeze complete", dir.join(".frozen.json").display());
    } else {
        println!("(dry run — pass `-- --update` to (re-)write .frozen.json; operator-only)");
    }
}
