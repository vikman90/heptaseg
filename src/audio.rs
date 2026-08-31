//! In-memory zero-asset PCM mechanical click sound synthesizer and player.

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Synthesizes a crisp, damped mechanical tactile key click waveform in memory.
fn synthesize_click_samples() -> Vec<f32> {
    const SAMPLE_RATE: usize = 44_100;
    const DURATION_SECS: f32 = 0.009; // 9ms tactile click
    let sample_count = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;

    let mut samples = Vec::with_capacity(sample_count);
    for i in 0..sample_count {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = t / DURATION_SECS;

        // Pitch drops rapidly from 1100 Hz down to 280 Hz to simulate keycap bottom-out
        let freq = 1100.0 - (820.0 * progress);
        let phase = 2.0 * PI * freq * t;

        // Exponential decay envelope with sharp attack
        let attack = (t / 0.001).min(1.0);
        let decay = (-t * 450.0).exp();
        let amp = attack * decay * 0.28;

        let val = phase.sin() * amp;
        samples.push(val);
    }
    samples
}

/// Sound manager responsible for low-latency tactile audio feedback.
pub struct SoundManager {
    muted: Arc<AtomicBool>,
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    click_samples: Arc<Vec<f32>>,
}

impl Default for SoundManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundManager {
    /// Creates a new sound manager, gracefully falling back if audio device is unavailable.
    pub fn new() -> Self {
        let samples = Arc::new(synthesize_click_samples());
        let muted = Arc::new(AtomicBool::new(false));

        // Try initializing system audio output stream
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok((stream, handle)) => (Some(stream), Some(handle)),
            Err(_) => (None, None), // Headless environment or no audio device
        };

        Self {
            muted,
            _stream,
            stream_handle,
            click_samples: samples,
        }
    }

    /// Returns true if audio feedback is muted.
    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    /// Toggles audio feedback between muted and unmuted.
    pub fn toggle_mute(&self) -> bool {
        let current = self.muted.load(Ordering::Relaxed);
        self.muted.store(!current, Ordering::Relaxed);
        !current
    }

    /// Plays a non-blocking synthesized mechanical click.
    pub fn play_click(&self) {
        if self.is_muted() {
            return;
        }

        if let Some(handle) = &self.stream_handle {
            if let Ok(sink) = Sink::try_new(handle) {
                let buffer = SamplesBuffer::new(1, 44_100, (*self.click_samples).clone());
                sink.append(buffer);
                sink.detach();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesized_samples_generated() {
        let samples = synthesize_click_samples();
        assert!(!samples.is_empty());
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s), "Sample {} out of range", s);
        }
    }

    #[test]
    fn test_sound_manager_mute_toggle() {
        let mgr = SoundManager::new();
        assert!(!mgr.is_muted());
        assert!(mgr.toggle_mute());
        assert!(mgr.is_muted());
        assert!(!mgr.toggle_mute());
        assert!(!mgr.is_muted());
    }
}
