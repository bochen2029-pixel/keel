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
