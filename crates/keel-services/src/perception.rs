//! keel-services::perception — the afferent senses' change-gate (canon §12).
//!
//! The cost control without which perception bankrupts the budget: the expensive cognition model is
//! consulted **only on change**. Frames are deduped by **dHash** Hamming distance (a static screen is
//! GPU-free); audio is gated by **VAD** (silence is never transcribed). This is the genome-level gate
//! that ships **with** the senses (canon §12), conformance-pinned by `GOLDEN_PERCEPTION`. The capture
//! organs themselves — `PerceptionSource` impls over a real screen/camera/mic + the `see()`/`hear()`
//! retinas that turn pixels/audio → compact text locally — ride on top of this gate (a later slice,
//! once the substrate + an image/audio decoder are wired); this is the model-free gate they all share.

use keel_adapters::Whisper;
use keel_contracts::{Modality, Percept, Result, Time};
use std::path::Path;

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

/// Build an Audio [`Percept`] from a transcript (canon §12). Model-free assembly: the transcript came
/// from the local Whisper organ; `source` is the **capture topology** ("mic" = me, "loopback" = them),
/// never inferred by a model. `t_utc` is stamped by the caller (the kernel owns the clock).
pub fn percept_from_transcript(transcript: impl Into<String>, source: impl Into<String>, t_utc: Time) -> Percept {
    Percept {
        content: serde_json::json!({ "text": transcript.into() }),
        t_utc,
        modality: Modality::Audio,
        source: source.into(),
        confidence: 1.0,
    }
}

/// The **`hear()` retina** (canon §12): VAD-gate → transcribe (the local Whisper organ) → `Percept`.
/// Silence (`voiced_ms == 0`) is **free** — no transcription, returns `None` (the gate short-circuits
/// before the organ is touched). Voiced audio is transcribed **locally and sovereignly** and emitted
/// as an Audio `Percept` for the route → verify → remember loop. `source` = capture topology; `t_utc`
/// is stamped by the caller. The capture organ (a `PerceptionSource` over a real mic) feeds this.
pub async fn hear(
    whisper: &Whisper,
    audio: &Path,
    voiced_ms: u32,
    source: &str,
    t_utc: Time,
) -> Result<Option<Percept>> {
    if !ChangeGate::audio_voiced(voiced_ms) {
        return Ok(None); // silence is VAD-gated: never transcribed (NFR §19 perception thrift)
    }
    let transcript = whisper.transcribe(audio).await?;
    Ok(Some(percept_from_transcript(transcript, source, t_utc)))
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

    // ── the hear() retina (canon §12): VAD-gate → whisper → Percept ──

    #[tokio::test]
    async fn hear_gates_silence_without_transcribing() {
        // silence (voiced_ms == 0) → None, and the whisper organ is never touched (dummy paths fine).
        let w = keel_adapters::Whisper::new("no-such-cli", "no-such-model");
        let p = hear(&w, std::path::Path::new("no-such.wav"), 0, "mic", 1234).await.unwrap();
        assert!(p.is_none(), "silence is gated -> no Percept, no transcription");
    }

    #[test]
    fn percept_from_transcript_is_a_sourced_audio_percept() {
        let p = percept_from_transcript("hello world", "mic", 42);
        assert_eq!(p.modality, keel_contracts::Modality::Audio);
        assert_eq!(p.source, "mic"); // capture topology, not a model
        assert_eq!(p.t_utc, 42);
        assert_eq!(p.content["text"].as_str(), Some("hello world"));
    }

    /// Live: VAD-voiced audio is transcribed by the real whisper organ. Ignored by default (needs the
    /// model + a WAV); fix the paths and run with `-- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn live_hear_transcribes_voiced_audio() {
        let w = keel_adapters::Whisper::new(r"C:\whisper.cpp\whisper-cli.exe", r"C:\models\ggml-large-v3-turbo.bin");
        let p = hear(&w, std::path::Path::new(r"C:\whisper.cpp\samples\jfk.wav"), 1000, "mic", 0).await.unwrap();
        assert!(p.is_some(), "voiced audio transcribes to a Percept");
    }
}
