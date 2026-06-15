//! keel-adapters::mic — the microphone **capture organ** (canon §12 ears), behind the `mic` feature.
//!
//! cpal records a clip from the default input device; the perception layer VAD-gates it
//! (`ChangeGate::voiced_ms`) and the whisper organ transcribes it. **Sovereign + local** — raw audio
//! never leaves the box (I3). Feature-gated so the default genome core pulls **no** audio dependency
//! (a consumer that never hears pays nothing — the minimal-core thesis). Downmixes to mono `i16` at
//! the device's native rate; the caller resamples to 16 kHz for whisper if the rate differs.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use keel_contracts::{KeelError, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The default-input-device microphone (cpal). **Not a tier** — a sovereign capture organ (the router
/// never routes here), like whisper.
pub struct Microphone;

impl Microphone {
    /// Record ~`seconds` of audio from the default input device, downmixed to mono `i16`. Returns the
    /// samples + the device's sample rate (the caller resamples to 16 kHz mono for whisper if it
    /// differs). **Blocking** (records for the duration on cpal's callback thread, then stops),
    /// sovereign + local. Errors honestly when no input device / config is available.
    pub fn capture(seconds: u32) -> Result<(Vec<i16>, u32)> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| KeelError::Other("mic: no default input device".into()))?;
        let supported = device
            .default_input_config()
            .map_err(|e| KeelError::Other(format!("mic: no default input config: {e}")))?;
        let sample_rate = supported.sample_rate();
        let channels = (supported.channels() as usize).max(1);
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();

        let buf = Arc::new(Mutex::new(Vec::<i16>::new()));
        let sink = buf.clone();
        let err_fn = |e| eprintln!("[keel] mic stream error: {e}");

        // downmix each frame to its first channel; convert f32 -> i16 when needed.
        let stream = match format {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    sink.lock().unwrap_or_else(|p| p.into_inner()).extend(data.chunks(channels).map(|f| f[0]));
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    sink.lock()
                        .expect("mic buffer")
                        .extend(data.chunks(channels).map(|f| (f[0] * i16::MAX as f32) as i16));
                },
                err_fn,
                None,
            ),
            other => return Err(KeelError::Other(format!("mic: unsupported sample format {other:?}"))),
        }
        .map_err(|e| KeelError::Other(format!("mic: build input stream: {e}")))?;

        stream.play().map_err(|e| KeelError::Other(format!("mic: play: {e}")))?;
        std::thread::sleep(Duration::from_secs(seconds as u64));
        drop(stream); // stop capturing

        let samples = buf.lock().unwrap_or_else(|p| p.into_inner()).clone();
        Ok((samples, sample_rate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live: needs a real input device. Ignored by default (like the tier/whisper `live_*` tests);
    /// run with `--features mic -- --ignored` on a box with a mic to verify real capture.
    #[test]
    #[ignore]
    fn live_capture() {
        let (samples, rate) = Microphone::capture(1).expect("capture from the default mic");
        assert!(rate > 0);
        assert!(!samples.is_empty(), "1s of capture yields samples");
    }
}
