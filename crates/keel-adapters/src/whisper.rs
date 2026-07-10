//! keel-adapters::whisper — the ears (canon §12). A standalone **transcription organ**, NOT a
//! `ModelTier`: audio → text *before* cognition (like the embedder/reranker are Memory organs, the
//! router never routes here). whisper.cpp transcribes **locally and sovereignly** — raw audio never
//! leaves the box (I3). The perception service VAD-gates capture (canon §12, `GOLDEN_PERCEPTION`),
//! hands a voiced clip here, and the transcript becomes a `Percept(Audio, text)` that rides the same
//! route → verify → remember loop. **Speaker attribution is capture topology** (mic = "me", loopback
//! = "them"), set by the caller — never inferred by a model here.
//!
//! Minimal cut: shell out to `whisper-cli` for a clean, timestamp-free transcript. Deferred: the
//! `svc::perception` `hear()` retina that wires the VAD gate + this organ + the `Percept`;
//! `whisper-server` / `spawn_blocking` for off-hot-path async; word/segment timestamps via `-oj`.

use keel_contracts::{KeelError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// One timestamped transcript segment (D1: NightScribe's timeline fusion anchors on millisecond
/// offsets — the `-oj` enrichment the minimal cut deferred). Offsets are relative to the clip start;
/// the caller anchors them to wall-clock via its capture manifest (attribution + time are capture
/// topology, never model output).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// The local Whisper transcription organ (whisper.cpp `whisper-cli`). The binary + GGML model are
/// resolved from `keel.lock`'s substrate (shared system assets), never inlined.
pub struct Whisper {
    /// Path to `whisper-cli` (whisper.cpp), e.g. `C:\whisper.cpp\whisper-cli.exe`.
    cli: PathBuf,
    /// Path to the GGML model, e.g. `C:\models\ggml-large-v3-turbo.bin`.
    model: PathBuf,
}

impl Whisper {
    pub fn new(cli: impl Into<PathBuf>, model: impl Into<PathBuf>) -> Self {
        Self { cli: cli.into(), model: model.into() }
    }

    /// The `whisper-cli` args for a clean, timestamp-free transcript on stdout (factored out for a
    /// model-free unit test — the request shape is asserted without a live transcription).
    fn args(model: &Path, audio: &Path) -> Vec<String> {
        vec![
            "-m".into(),
            model.display().to_string(),
            "-f".into(),
            audio.display().to_string(),
            "--no-timestamps".into(), // stdout becomes the plain transcript
            "--no-prints".into(),     // suppress whisper.cpp's banner/progress on stdout
        ]
    }

    /// Clean `whisper-cli` stdout into a single transcript: trim each line, drop blanks, join with a
    /// space. (Word/segment timestamps are a later `-oj` enrichment.)
    fn parse_transcript(stdout: &str) -> String {
        stdout.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
    }

    /// Transcribe a whisper-compatible (16 kHz mono PCM) WAV to text. **Sovereign + local.** Blocking
    /// (the transcription runs off the perception hot path, not the cognition loop — a future
    /// refinement is `spawn_blocking` / `whisper-server`). The perception capture organ guarantees the
    /// WAV format before handing the clip here.
    pub async fn transcribe(&self, audio: &Path) -> Result<String> {
        let out = Command::new(&self.cli)
            .args(Self::args(&self.model, audio))
            .output()
            .map_err(|e| KeelError::Other(format!("whisper-cli spawn ({}): {e}", self.cli.display())))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(KeelError::Other(format!("whisper-cli failed: {}", err.trim())));
        }
        Ok(Self::parse_transcript(&String::from_utf8_lossy(&out.stdout)))
    }

    /// The `whisper-cli` args for a **timestamped JSON** transcript (`-oj` writes `<base>.json`).
    fn args_json(model: &Path, audio: &Path, out_base: &Path) -> Vec<String> {
        vec![
            "-m".into(),
            model.display().to_string(),
            "-f".into(),
            audio.display().to_string(),
            "-oj".into(), // JSON with per-segment millisecond offsets
            "-of".into(),
            out_base.display().to_string(),
            "--no-prints".into(),
        ]
    }

    /// Parse whisper.cpp's `-oj` JSON into segments (`transcription[].offsets.{from,to}` are
    /// millisecond offsets; empty/blank texts are dropped). Factored for a model-free unit test.
    fn parse_segments_json(raw: &str) -> Result<Vec<Segment>> {
        let v: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| KeelError::Other(format!("whisper json: {e}")))?;
        let Some(arr) = v["transcription"].as_array() else {
            return Err(KeelError::Other("whisper json: no transcription array".into()));
        };
        Ok(arr
            .iter()
            .filter_map(|s| {
                let text = s["text"].as_str()?.trim().to_string();
                if text.is_empty() {
                    return None;
                }
                Some(Segment {
                    start_ms: s["offsets"]["from"].as_u64().unwrap_or(0),
                    end_ms: s["offsets"]["to"].as_u64().unwrap_or(0),
                    text,
                })
            })
            .collect())
    }

    /// Transcribe to **timestamped segments** (D1: the ears over protocol need offsets a cell can
    /// anchor to its capture timeline). Runs `whisper-cli -oj` to a temp JSON, parses, cleans up.
    /// **Sovereign + local** — the audio path never leaves the box; only text comes back (I3).
    pub async fn transcribe_segments(&self, audio: &Path) -> Result<Vec<Segment>> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let base = std::env::temp_dir()
            .join(format!("keel-whisper-{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed)));
        let out = Command::new(&self.cli)
            .args(Self::args_json(&self.model, audio, &base))
            .output()
            .map_err(|e| KeelError::Other(format!("whisper-cli spawn ({}): {e}", self.cli.display())))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(KeelError::Other(format!("whisper-cli failed: {}", err.trim())));
        }
        let json_path = base.with_extension("json");
        let raw = std::fs::read_to_string(&json_path)
            .map_err(|e| KeelError::Other(format!("whisper json read ({}): {e}", json_path.display())))?;
        let _ = std::fs::remove_file(&json_path); // temp artifact; the segments are the product
        Self::parse_segments_json(&raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_transcript_cleans_and_joins() {
        let raw = "\n  And so my fellow Americans  \n   ask not what your country can do for you \n\n";
        assert_eq!(
            Whisper::parse_transcript(raw),
            "And so my fellow Americans ask not what your country can do for you"
        );
        assert_eq!(Whisper::parse_transcript("   \n\n  "), ""); // silence/empty → empty transcript
    }

    #[test]
    fn args_pin_model_and_audio_and_request_a_clean_transcript() {
        let a = Whisper::args(Path::new("M.bin"), Path::new("a.wav"));
        assert!(a.windows(2).any(|w| w[0] == "-m" && w[1] == "M.bin"), "model pinned");
        assert!(a.windows(2).any(|w| w[0] == "-f" && w[1] == "a.wav"), "audio pinned");
        assert!(a.contains(&"--no-timestamps".to_string()), "timestamp-free transcript");
    }

    #[test]
    fn parse_segments_pulls_ms_offsets_and_drops_blanks() {
        let raw = r#"{ "transcription": [
            { "timestamps": {"from":"00:00:00,000","to":"00:00:02,500"}, "offsets": {"from":0,"to":2500}, "text": " Hello there. " },
            { "offsets": {"from":2500,"to":2600}, "text": "   " },
            { "offsets": {"from":2600,"to":5000}, "text": "Second segment." }
        ] }"#;
        let segs = Whisper::parse_segments_json(raw).unwrap();
        assert_eq!(segs.len(), 2, "blank segments dropped");
        assert_eq!(segs[0], Segment { start_ms: 0, end_ms: 2500, text: "Hello there.".into() });
        assert_eq!(segs[1].start_ms, 2600);
        assert!(Whisper::parse_segments_json("{}").is_err(), "no transcription array -> honest error");
    }

    #[test]
    fn args_json_request_the_timestamped_json() {
        let a = Whisper::args_json(Path::new("M.bin"), Path::new("a.wav"), Path::new("out"));
        assert!(a.contains(&"-oj".to_string()), "JSON output");
        assert!(a.windows(2).any(|w| w[0] == "-of" && w[1] == "out"), "output base pinned");
    }

    /// Live: needs `whisper-cli` + a model + a WAV. Ignored by default (like the tier `live_*` tests);
    /// fix the paths/sample and run with `-- --ignored` to verify a real transcription end-to-end.
    #[tokio::test]
    #[ignore]
    async fn live_transcribe() {
        let w = Whisper::new(r"C:\whisper.cpp\whisper-cli.exe", r"C:\models\ggml-large-v3-turbo.bin");
        let text = w.transcribe(Path::new(r"C:\whisper.cpp\samples\jfk.wav")).await.unwrap();
        assert!(!text.is_empty(), "a real WAV transcribes to non-empty text");
    }
}
