//! keel-adapters::screen — the screen capture organ (canon §12 eyes), behind the `screen` feature.
//!
//! Grabs the primary monitor's current frame as raw RGBA8. The perception layer dHashes + `FrameGate`s
//! it (the model-free change-gate) and the **native Qwen3.5-9B vision** (`local_llama` image_url)
//! captions it — there is **no separate VLM**; the brain *is* the eye (canon §2.8/§3). Sovereign +
//! local: the frame is gated/resized locally and raw frames never egress (I3). Feature-gated so the
//! default genome core pulls no capture dep — toggle it on per consumer (the minimal-core thesis, §3.5).

use keel_contracts::{KeelError, Result};
use xcap::Monitor;

/// The primary-monitor screen grabber (xcap). **Not a tier** — a sovereign capture organ feeding the
/// perception gate; the router never routes here (like whisper / the mic).
pub struct ScreenCapture;

impl ScreenCapture {
    /// Grab the primary monitor's current frame as raw **RGBA8** — returns `(pixels, width, height)`,
    /// ready for [`ChangeGate::dhash_rgba`](../../keel_services/perception) + the `see()` retina.
    /// Prefers the primary monitor, else the first; errors honestly if none is available.
    /// Sovereign + local (the frame stays on the box).
    pub fn grab() -> Result<(Vec<u8>, usize, usize)> {
        let monitors = Monitor::all().map_err(|e| KeelError::Other(format!("screen: enumerate monitors: {e}")))?;
        let monitor = monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .ok_or_else(|| KeelError::Other("screen: no monitor found".into()))?;
        let img = monitor.capture_image().map_err(|e| KeelError::Other(format!("screen: capture: {e}")))?;
        let (w, h) = (img.width() as usize, img.height() as usize);
        Ok((img.into_raw(), w, h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live: needs a real display. Ignored by default (like the cpal/whisper `live_*` tests); run with
    /// `--features screen -- --ignored` on a box with a screen to verify a real grab.
    #[test]
    #[ignore]
    fn live_grab() {
        let (px, w, h) = ScreenCapture::grab().expect("grab the primary monitor");
        assert!(w > 0 && h > 0);
        assert_eq!(px.len(), w * h * 4, "RGBA8 = 4 bytes/pixel");
    }
}
