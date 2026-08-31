//! Procedural retro mechanical key click sound synthesizer.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle};

/// Generates a synthesized 15ms mechanical key click waveform in memory.
fn generate_click_waveform() -> (Vec<f32>, u32) {
    let sample_rate = 44100u32;
    let duration_samples = (sample_rate as f32 * 0.015) as usize;
    let mut samples = Vec::with_capacity(duration_samples);

    for i in 0..duration_samples {
        let t = i as f32 / sample_rate as f32;
        let decay = (-t / 0.0035).exp();
        // Sharp transient (2400 Hz) combined with tactile mechanical thud (750 Hz)
        let sample = decay
            * (0.6 * (2.0 * std::f32::consts::PI * 2400.0 * t).sin()
                + 0.4 * (2.0 * std::f32::consts::PI * 750.0 * t).sin());
        samples.push(sample * 0.3); // Safe comfortable volume
    }

    (samples, sample_rate)
}

/// Thread-safe procedural audio engine for button click feedback.
pub struct SoundEngine {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    samples: Arc<Vec<f32>>,
    sample_rate: u32,
    muted: AtomicBool,
}

impl Default for SoundEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundEngine {
    /// Initializes the audio engine. Gracefully falls back if no audio device is available.
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None), // Headless or unsupported audio environment
        };

        let (samples, sample_rate) = generate_click_waveform();

        Self {
            _stream: stream,
            handle,
            samples: Arc::new(samples),
            sample_rate,
            muted: AtomicBool::new(false),
        }
    }

    /// Plays a short tactile key click sound if sound is enabled.
    pub fn play_click(&self) {
        if self.muted.load(Ordering::Relaxed) {
            return;
        }

        if let Some(ref handle) = self.handle {
            let buffer = SamplesBuffer::new(1, self.sample_rate, (*self.samples).clone());
            let _ = handle.play_raw(buffer);
        }
    }

    /// Toggles the mute state and returns the new state (true = muted).
    pub fn toggle_mute(&self) -> bool {
        let new_state = !self.muted.load(Ordering::Relaxed);
        self.muted.store(new_state, Ordering::Relaxed);
        new_state
    }

    /// Returns whether the audio engine is currently muted.
    #[allow(dead_code)]
    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_generation() {
        let (samples, rate) = generate_click_waveform();
        assert_eq!(rate, 44100);
        assert!(!samples.is_empty());
        assert!(samples.len() > 500);
    }

    #[test]
    fn test_sound_engine_mute_toggle() {
        let engine = SoundEngine::new();
        assert!(!engine.is_muted());
        assert!(engine.toggle_mute());
        assert!(engine.is_muted());
        assert!(!engine.toggle_mute());
        assert!(!engine.is_muted());
    }
}
