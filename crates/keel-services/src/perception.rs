//! keel-services::perception — the afferent senses' change-gate (canon §12).
//!
//! The cost control without which perception bankrupts the budget: the expensive cognition model is
//! consulted **only on change**. Frames are deduped by **dHash** Hamming distance (a static screen is
//! GPU-free); audio is gated by **VAD** (silence is never transcribed). This is the genome-level gate
//! that ships **with** the senses (canon §12), conformance-pinned by `GOLDEN_PERCEPTION`. The capture
//! organs themselves — `PerceptionSource` impls over a real screen/camera/mic + the `see()`/`hear()`
//! retinas that turn pixels/audio → compact text locally — ride on top of this gate (a later slice,
//! once the substrate + an image/audio decoder are wired); this is the model-free gate they all share.

/// The perceptual-hash (dHash) Hamming distance above which a frame counts as "changed" (canon §12,
/// calibrated ~4). At or below it, consecutive frames are treated as identical and the model is NOT
/// consulted — a static screen costs zero inference (NFR §19 "perception thrift").
pub const FRAME_DHASH_THRESHOLD: u32 = 4;

/// The afferent change-gate (canon §12): pure, deterministic, model-free. A frame or audio sample
/// is allowed to emit a `Percept` (→ routed/verified/remembered) only when it carries new
/// information. Swappable threshold for cells that want a tighter/looser frame sensitivity.
#[derive(Clone, Copy, Debug)]
pub struct ChangeGate {
    /// dHash Hamming-distance threshold for "frame changed" (default [`FRAME_DHASH_THRESHOLD`]).
    pub frame_threshold: u32,
}

impl Default for ChangeGate {
    fn default() -> Self {
        Self { frame_threshold: FRAME_DHASH_THRESHOLD }
    }
}

impl ChangeGate {
    /// A gate with an explicit frame threshold (e.g. a cell tuning frame sensitivity).
    pub fn new(frame_threshold: u32) -> Self {
        Self { frame_threshold }
    }

    /// Hamming distance between two dHashes (the count of differing bits). `0` = identical frames.
    /// The capture organ computes each frame's dHash; this is the model-free comparison the gate runs.
    pub fn dhash_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    /// A frame emits a percept only if it changed enough vs the previous one (`distance > threshold`).
    /// A static screen (`distance <= threshold`) is GPU-free — the cognition model is not consulted.
    pub fn frame_changed(&self, dhash_distance: u32) -> bool {
        dhash_distance > self.frame_threshold
    }

    /// Audio emits a percept (→ Whisper transcription) only when voiced speech is present
    /// (`voiced_ms > 0`). Silence (VAD = 0) is free — never transcribed.
    pub fn audio_voiced(voiced_ms: u32) -> bool {
        voiced_ms > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// The conformance test (I5): the change-gate MUST satisfy every frozen `GOLDEN_PERCEPTION` case.
    /// The goldens are language-neutral — `frame_delta` is the dHash distance; `voiced_ms` is the VAD
    /// reading — so the gate is asserted against operator-frozen ground truth, no model.
    #[test]
    fn passes_golden_perception() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/golden.json");
        let raw = std::fs::read_to_string(path).expect("read golden.json");
        let golden: Value = serde_json::from_str(&raw).expect("parse golden.json");
        let cases = golden["perception"].as_array().expect("perception cases");
        assert!(!cases.is_empty());
        let gate = ChangeGate::default();

        for case in cases {
            let name = case["name"].as_str().unwrap_or("?");
            let input = &case["input"];
            let expect_emitted = case["expect"]["emitted"].as_bool().expect("expect.emitted");

            // dispatch by input shape: a frame (`frame_delta`) → dHash gate; audio (`voiced_ms`) → VAD.
            let emitted = if let Some(delta) = input["frame_delta"].as_u64() {
                gate.frame_changed(delta as u32)
            } else if let Some(voiced) = input["voiced_ms"].as_u64() {
                ChangeGate::audio_voiced(voiced as u32)
            } else {
                panic!("case '{name}': unrecognized perception input shape");
            };
            assert_eq!(emitted, expect_emitted, "case '{name}': emitted");
        }
    }

    #[test]
    fn dhash_distance_is_hamming_and_gates() {
        assert_eq!(ChangeGate::dhash_distance(0, 0), 0);
        assert_eq!(ChangeGate::dhash_distance(0b1011, 0b0000), 3); // three set bits differ
        let gate = ChangeGate::default();
        // identical frames are gated (free); a full 64-bit flip emits.
        assert!(!gate.frame_changed(ChangeGate::dhash_distance(42, 42)));
        assert!(gate.frame_changed(ChangeGate::dhash_distance(0, u64::MAX)));
        // VAD: silence is free, voiced emits.
        assert!(!ChangeGate::audio_voiced(0));
        assert!(ChangeGate::audio_voiced(120));
    }
}
