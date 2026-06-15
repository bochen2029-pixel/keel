//! keel-adapters::wav — a tiny **no-dep** RIFF/WAVE writer for the whisper organ's input.
//!
//! whisper-cli reads 16 kHz mono 16-bit PCM (canon §12). The mic capture organ produces raw `i16`
//! samples; this turns them into a canonical little-endian WAV on disk. Pure std — no audio crate.

use std::io::Write;
use std::path::Path;

/// Write 16-bit PCM `samples` to a canonical little-endian RIFF/WAVE file (no dep). Use
/// `sample_rate = 16000`, `channels = 1` for whisper-cli. The 44-byte header precedes the PCM data.
pub fn write_wav_i16(path: impl AsRef<Path>, samples: &[i16], sample_rate: u32, channels: u16) -> std::io::Result<()> {
    let bits = 16u16;
    let block_align = channels * (bits / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_len = (samples.len() * 2) as u32;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?; // RIFF chunk size = 36 + data
    f.write_all(b"WAVE")?;
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?; // fmt chunk size
    f.write_all(&1u16.to_le_bytes())?; // audio format = PCM
    f.write_all(&channels.to_le_bytes())?;
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&block_align.to_le_bytes())?;
    f.write_all(&bits.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for &s in samples {
        f.write_all(&s.to_le_bytes())?;
    }
    f.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_canonical_riff_wave_header() {
        let path = std::env::temp_dir().join(format!("keel_wav_test_{}.wav", std::process::id()));
        write_wav_i16(&path, &[0i16, 100, -100, 32767, -32768], 16000, 1).unwrap();
        let b = std::fs::read(&path).unwrap();
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(&b[8..12], b"WAVE");
        assert_eq!(&b[12..16], b"fmt ");
        assert_eq!(&b[36..40], b"data");
        assert_eq!(u32::from_le_bytes([b[24], b[25], b[26], b[27]]), 16000, "sample rate at offset 24");
        assert_eq!(u16::from_le_bytes([b[22], b[23]]), 1, "mono");
        assert_eq!(b.len(), 44 + 5 * 2, "44-byte header + 5 i16 samples");
        let _ = std::fs::remove_file(&path);
    }
}
