//! keel-services::perception — the afferent senses' change-gate (canon §12).
//!
//! The cost control without which perception bankrupts the budget: the expensive cognition model is
//! consulted **only on change**. Frames are deduped by **dHash** Hamming distance (a static screen is
//! GPU-free); audio is gated by **VAD** (silence is never transcribed). This is the genome-level gate
//! that ships **with** the senses (canon §12), conformance-pinned by `GOLDEN_PERCEPTION`. The capture
//! organs themselves — `PerceptionSource` impls over a real screen/camera/mic + the `see()`/`hear()`
//! retinas that turn pixels/audio → compact text locally — ride on top of this gate (a later slice,
//! once the substrate + an image/audio decoder are wired); this is the model-free gate they all share.
//!
//! **This slice** adds the no-dep model-free pieces — dHash compute from raw pixels
//! ([`ChangeGate::dhash`] / [`ChangeGate::dhash_rgba`]) + a stateful [`FrameGate`] — so the change-gate
//! has real inputs (a raw captured frame is already pixels; no image-decode crate needed). The
//! OS-capture `PerceptionSource` over a real screen/camera/mic is a dependency- + substrate-gated
//! follow-on; the `see()` captioning retina rides the cognition protocol (a frame is multi-part image
//! content, §12) and lands with it.

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

    /// Compute the perceptual **dHash** of a grayscale (luma) frame — the model-free hash the gate
    /// dedups on. Average-pools the `w`x`h` buffer to 9x8, then sets one bit per adjacent-column
    /// comparison (8 per row x 8 rows = 64 bits). Pure arithmetic, **no image-decode dependency** (a
    /// raw captured frame is already pixels). A uniform frame hashes to 0; `w==0`/`h==0`/short
    /// buffers return 0 (never a panic).
    pub fn dhash(luma: &[u8], w: usize, h: usize) -> u64 {
        const TW: usize = 9;
        const TH: usize = 8;
        if w == 0 || h == 0 || luma.len() < w * h {
            return 0;
        }
        let mut hash = 0u64;
        let mut bit = 0u32;
        for ty in 0..TH {
            let y0 = ty * h / TH;
            let y1 = ((ty + 1) * h / TH).clamp(y0 + 1, h);
            let mut row = [0u32; TW];
            for (tx, cell) in row.iter_mut().enumerate() {
                let x0 = tx * w / TW;
                let x1 = ((tx + 1) * w / TW).clamp(x0 + 1, w);
                let (mut sum, mut cnt) = (0u32, 0u32);
                for y in y0..y1 {
                    for x in x0..x1 {
                        sum += luma[y * w + x] as u32;
                        cnt += 1;
                    }
                }
                *cell = sum.checked_div(cnt).unwrap_or(0);
            }
            for pair in row.windows(2) {
                if pair[0] < pair[1] {
                    hash |= 1u64 << bit;
                }
                bit += 1;
            }
        }
        hash
    }

    /// `dhash` over an RGBA frame: convert to luma (Rec.601: 0.299R + 0.587G + 0.114B) then hash. No
    /// dep. `w*h==0` / short buffers return 0.
    pub fn dhash_rgba(rgba: &[u8], w: usize, h: usize) -> u64 {
        let n = w * h;
        if n == 0 || rgba.len() < n * 4 {
            return 0;
        }
        let mut luma = vec![0u8; n];
        for (i, px) in luma.iter_mut().enumerate() {
            let (r, g, b) = (rgba[i * 4] as u32, rgba[i * 4 + 1] as u32, rgba[i * 4 + 2] as u32);
            *px = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
        }
        Self::dhash(&luma, w, h)
    }

    /// Energy-based **VAD**: the voiced duration (ms) of a mono PCM buffer — the model-free signal the
    /// audio gate ([`ChangeGate::audio_voiced`]) consumes (silence → 0 → never transcribed). Splits the
    /// buffer into ~20 ms windows and sums the duration of those whose RMS amplitude exceeds
    /// `rms_threshold` (i16 scale; ~300-1000 separates speech from a quiet room). No dep — the capture
    /// organ feeds raw samples in.
    pub fn voiced_ms(samples: &[i16], sample_rate: u32, rms_threshold: f64) -> u32 {
        if sample_rate == 0 || samples.is_empty() {
            return 0;
        }
        let window = (sample_rate / 50).max(1) as usize; // ~20 ms
        let mut voiced = 0u32;
        for chunk in samples.chunks(window) {
            let sum_sq: f64 = chunk.iter().map(|&s| (s as f64) * (s as f64)).sum();
            let rms = (sum_sq / chunk.len() as f64).sqrt();
            if rms > rms_threshold {
                voiced += (chunk.len() as u32 * 1000) / sample_rate;
            }
        }
        voiced
    }
}

/// A **stateful** frame change-gate — the capture organ's model-free dedup loop. It remembers the
/// last *emitted* frame's dHash and reports a new frame as changed when its Hamming distance exceeds
/// the threshold (the first frame is always new — the baseline). Comparing to the last *emitted*
/// frame (not the last *seen*) lets slow drift accumulate until it genuinely crosses the bar; a static
/// screen never re-emits, so the cognition model is never re-consulted (NFR §19 perception thrift).
#[derive(Clone, Debug, Default)]
pub struct FrameGate {
    gate: ChangeGate,
    last: Option<u64>,
}

impl FrameGate {
    /// A frame gate with an explicit dHash threshold (else [`FrameGate::default`] uses [`FRAME_DHASH_THRESHOLD`]).
    pub fn new(threshold: u32) -> Self {
        Self { gate: ChangeGate::new(threshold), last: None }
    }

    /// Report whether `dhash` is a new frame (the first frame, or a Hamming distance over the
    /// threshold), updating the remembered hash only when it is — so the next comparison is against
    /// the last *emitted* frame.
    pub fn changed(&mut self, dhash: u64) -> bool {
        let changed = match self.last {
            None => true,
            Some(prev) => self.gate.frame_changed(ChangeGate::dhash_distance(prev, dhash)),
        };
        if changed {
            self.last = Some(dhash);
        }
        changed
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

/// The **`see()` retina** (canon §12): dHash change-gate → an Image [`Percept`] on visual change. A
/// static frame (the gate reports unchanged) returns `None` — GPU-free, the cognition model is never
/// consulted (NFR §19 perception thrift). A changed frame emits an Image `Percept` referencing the
/// frame; `source` is the **capture topology** ("screen"/"camera"), never a model. The frame then
/// rides the cognition protocol as image content (§12) — the local vision tier captions/reasons over
/// it downstream (image content forces `local`, I3). `gate` holds the last-emitted hash; an OS-capture
/// `PerceptionSource` over a real screen feeds the `rgba` frames in (a later, dependency-gated slice).
pub fn see(
    gate: &mut FrameGate,
    frame_ref: impl Into<String>,
    rgba: &[u8],
    w: usize,
    h: usize,
    source: impl Into<String>,
    t_utc: Time,
) -> Option<Percept> {
    if !gate.changed(ChangeGate::dhash_rgba(rgba, w, h)) {
        return None; // unchanged frame: gated, never re-captioned
    }
    Some(Percept {
        content: serde_json::json!({ "frame": frame_ref.into() }),
        t_utc,
        modality: Modality::Image,
        source: source.into(),
        confidence: 1.0,
    })
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

    // ── dHash from raw pixels + the stateful frame gate (the capture organ's model-free core) ──

    #[test]
    fn dhash_of_a_uniform_frame_is_zero() {
        // a flat frame has no adjacent-column differences -> hash 0 -> distance 0 -> gated.
        let flat = vec![128u8; 100 * 80];
        assert_eq!(ChangeGate::dhash(&flat, 100, 80), 0);
        // guards: zero dims / short buffers -> 0, never a panic.
        assert_eq!(ChangeGate::dhash(&flat, 0, 80), 0);
        assert_eq!(ChangeGate::dhash(&[], 100, 80), 0);
    }

    #[test]
    fn dhash_distinguishes_a_gradient_from_flat() {
        // a left->right brightness gradient differs from a flat frame by many bits.
        let (w, h) = (90usize, 80usize);
        let mut grad = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                grad[y * w + x] = (x * 255 / (w - 1)) as u8;
            }
        }
        let gh = ChangeGate::dhash(&grad, w, h);
        let fh = ChangeGate::dhash(&vec![128u8; w * h], w, h);
        assert_ne!(gh, 0, "a gradient sets bits");
        assert!(ChangeGate::dhash_distance(gh, fh) > 4, "gradient vs flat is a real change");
    }

    #[test]
    fn dhash_rgba_matches_its_luma() {
        // a gray RGBA frame (R=G=B=v, A=255) hashes identically to the luma frame of value v.
        let (w, h) = (90usize, 80usize);
        let mut luma = vec![0u8; w * h];
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let v = (x * 255 / (w - 1)) as u8;
                luma[y * w + x] = v;
                let i = (y * w + x) * 4;
                rgba[i] = v;
                rgba[i + 1] = v;
                rgba[i + 2] = v;
                rgba[i + 3] = 255;
            }
        }
        assert_eq!(ChangeGate::dhash_rgba(&rgba, w, h), ChangeGate::dhash(&luma, w, h));
    }

    #[test]
    fn frame_gate_emits_on_change_skips_static() {
        let (w, h) = (90usize, 80usize);
        let flat = ChangeGate::dhash(&vec![100u8; w * h], w, h);
        let mut grad = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                grad[y * w + x] = (x * 255 / (w - 1)) as u8;
            }
        }
        let grad_h = ChangeGate::dhash(&grad, w, h);

        let mut g = FrameGate::default();
        assert!(g.changed(flat), "first frame is the baseline -> emits");
        assert!(!g.changed(flat), "identical frame -> gated (free)");
        assert!(g.changed(grad_h), "a big change -> emits");
        assert!(!g.changed(grad_h), "same again -> gated");
    }

    #[test]
    fn see_gates_static_frames_and_emits_an_image_percept_on_change() {
        let (w, h) = (90usize, 80usize);
        let flat = vec![100u8; w * h * 4];
        let mut grad = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let (i, v) = ((y * w + x) * 4, (x * 255 / (w - 1)) as u8);
                grad[i] = v;
                grad[i + 1] = v;
                grad[i + 2] = v;
                grad[i + 3] = 255;
            }
        }
        let mut g = FrameGate::default();
        let p = see(&mut g, "frame-0", &flat, w, h, "screen", 1).expect("first frame emits a Percept");
        assert_eq!(p.modality, Modality::Image);
        assert_eq!(p.source, "screen"); // capture topology, not a model
        assert_eq!(p.content["frame"].as_str(), Some("frame-0"));
        assert!(see(&mut g, "frame-1", &flat, w, h, "screen", 2).is_none(), "static frame -> gated");
        assert!(see(&mut g, "frame-2", &grad, w, h, "screen", 3).is_some(), "visual change -> emits");
    }

    #[test]
    fn voiced_ms_gates_silence_and_measures_speech() {
        let rate = 16000u32;
        // silence -> 0 voiced ms (never transcribed).
        assert_eq!(ChangeGate::voiced_ms(&vec![0i16; rate as usize], rate, 300.0), 0);
        // a loud 1s mono buffer -> ~1000 ms voiced.
        let loud = vec![6000i16; rate as usize];
        let v = ChangeGate::voiced_ms(&loud, rate, 300.0);
        assert!((900..=1000).contains(&v), "loud 1s -> ~1000ms, got {v}");
        // half silence + half loud -> ~500 ms.
        let mut mixed = vec![0i16; (rate / 2) as usize];
        mixed.extend(vec![6000i16; (rate / 2) as usize]);
        let vm = ChangeGate::voiced_ms(&mixed, rate, 300.0);
        assert!((400..=600).contains(&vm), "half-loud -> ~500ms, got {vm}");
        // guards: empty / zero-rate -> 0, never a panic.
        assert_eq!(ChangeGate::voiced_ms(&[], rate, 300.0), 0);
        assert_eq!(ChangeGate::voiced_ms(&loud, 0, 300.0), 0);
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
