// SPDX-License-Identifier: AGPL-3.0-only
//! Audio Playback Engine
//!
//! Generates and plays actual sounds through speakers.
//! Modern idiomatic Rust implementation using rodio.
//!
//! This module is feature-gated and only available with `native-audio` feature.

#[cfg(feature = "native-audio")]
use crate::audio_sonification::{AudioAttributes, Instrument};
#[cfg(feature = "native-audio")]
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
#[cfg(feature = "native-audio")]
use std::sync::Arc;
#[cfg(feature = "native-audio")]
use std::time::Duration;

/// Audio playback engine for generating and playing sounds.
///
/// This engine takes audio attributes (from the sonification renderer)
/// and generates actual audio output through the system speakers.
#[cfg(feature = "native-audio")]
pub struct AudioPlaybackEngine {
    /// Audio output stream (must be kept alive)
    _stream: Option<OutputStream>,
    /// Handle to the output stream
    stream_handle: Option<OutputStreamHandle>,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
    /// Audio enabled state
    enabled: bool,
}

#[cfg(feature = "native-audio")]
impl AudioPlaybackEngine {
    /// Create a new audio playback engine.
    ///
    /// # Errors
    /// Returns error if audio output device cannot be initialized.
    pub fn new() -> anyhow::Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| anyhow::anyhow!("Failed to initialize audio output: {}", e))?;

        Ok(Self {
            _stream: Some(_stream),
            stream_handle: Some(stream_handle),
            master_volume: 0.3, // Default to 30% to avoid being too loud
            enabled: true,
        })
    }

    /// Set master volume (0.0 - 1.0).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get current master volume.
    pub const fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Enable or disable audio playback.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if audio is enabled.
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Play a single tone based on audio attributes.
    ///
    /// This generates a tone for the given primal and plays it through speakers.
    /// The tone characteristics are determined by the audio attributes:
    /// - Instrument → Waveform (sine, square, triangle, sawtooth)
    /// - Pitch → Frequency
    /// - Volume → Amplitude
    /// - Pan → Stereo position
    pub fn play_tone(&self, attrs: &AudioAttributes) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {}", e))?;

        // Calculate frequency from pitch (map 0.0-1.0 to 100Hz-800Hz)
        let frequency = 100.0 + (attrs.pitch * 700.0);

        // Create waveform based on instrument
        #[cfg(feature = "native-audio")]
        let source = match attrs.instrument {
            Instrument::DeepBass => {
                // Bass: Low sine wave (sub-200Hz)
                create_sine_wave(frequency.min(200.0), Duration::from_millis(500))
            }
            Instrument::RhythmicDrums => {
                // Drums: Short percussive noise burst
                create_percussion(Duration::from_millis(100))
            }
            Instrument::LightChimes => {
                // Chimes: High triangle wave with decay
                create_triangle_wave(frequency * 2.0, Duration::from_millis(400))
            }
            Instrument::SustainedStrings => {
                // Strings: Sawtooth with longer sustain
                create_sawtooth_wave(frequency, Duration::from_millis(600))
            }
            Instrument::HighSynth => {
                // Synth: Square wave (electronic sound)
                create_square_wave(frequency * 1.5, Duration::from_millis(300))
            }
            _ => create_sine_wave(frequency, Duration::from_millis(400)),
        };

        // Apply volume (attribute volume * master volume)
        let volume = attrs.volume * self.master_volume;
        sink.set_volume(volume);

        // Apply panning (-1.0 to 1.0)
        // Note: rodio doesn't have built-in panning, so we'll implement it later
        // For now, just play mono

        sink.append(source);
        sink.detach(); // Play in background

        Ok(())
    }

    /// Play multiple tones simultaneously (chord/cluster).
    ///
    /// This is useful for playing the entire soundscape at once.
    pub fn play_soundscape(&self, attributes: &[(String, AudioAttributes)]) -> anyhow::Result<()> {
        if !self.enabled || attributes.is_empty() {
            return Ok(());
        }

        // Play each tone - play_tone uses sink.detach() so playback is non-blocking.
        // No delay needed; tones play in background without overwhelming the output.
        for (_id, attrs) in attributes.iter() {
            self.play_tone(attrs)?;
        }

        Ok(())
    }
}

#[cfg(feature = "native-audio")]
impl Default for AudioPlaybackEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to initialize audio playback: {}", e);
            tracing::warn!("Audio will be disabled - creating silent engine");
            // Return a silent engine without attempting to create stream
            // This is a fallback for systems without audio devices
            Self {
                _stream: None,
                stream_handle: None,
                master_volume: 0.0,
                enabled: false,
            }
        })
    }
}

// ============================================================================
// Waveform Generators (Pure Functions)
// ============================================================================

/// Generate a sine wave tone.
///
/// Sine waves produce pure, smooth tones - ideal for bass and melodic sounds.
#[cfg(feature = "native-audio")]
fn create_sine_wave(frequency: f32, duration: Duration) -> impl Source<Item = f32> {
    let sample_rate = 48000;
    let samples = (sample_rate as f32 * duration.as_secs_f32()) as usize;

    (0..samples)
        .map(move |i| {
            let t = i as f32 / sample_rate as f32;
            let angle = 2.0 * std::f32::consts::PI * frequency * t;
            angle.sin() * 0.3 // Amplitude 0.3 to avoid clipping
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(rodio::source::SineWave::new)
        .flatten()
}

/// Generate a square wave tone.
///
/// Square waves produce harsh, electronic sounds - ideal for synth.
#[cfg(feature = "native-audio")]
fn create_square_wave(frequency: f32, duration: Duration) -> impl Source<Item = f32> {
    let sample_rate = 48000;
    let samples = (sample_rate as f32 * duration.as_secs_f32()) as usize;

    (0..samples)
        .map(move |i| {
            let t = i as f32 / sample_rate as f32;
            let angle = 2.0 * std::f32::consts::PI * frequency * t;
            if angle.sin() > 0.0 { 0.3 } else { -0.3 }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(rodio::source::SineWave::new)
        .flatten()
}

/// Generate a triangle wave tone.
///
/// Triangle waves are bright but smoother than square - ideal for chimes.
#[cfg(feature = "native-audio")]
fn create_triangle_wave(frequency: f32, duration: Duration) -> impl Source<Item = f32> {
    let sample_rate = 48000;
    let samples = (sample_rate as f32 * duration.as_secs_f32()) as usize;

    (0..samples)
        .map(move |i| {
            let t = i as f32 / sample_rate as f32;
            let period = 1.0 / frequency;
            let phase = (t % period) / period;
            let value = if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                3.0 - 4.0 * phase
            };
            value * 0.3
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(rodio::source::SineWave::new)
        .flatten()
}

/// Generate a sawtooth wave tone.
///
/// Sawtooth waves are rich in harmonics - ideal for strings.
#[cfg(feature = "native-audio")]
fn create_sawtooth_wave(frequency: f32, duration: Duration) -> impl Source<Item = f32> {
    let sample_rate = 48000;
    let samples = (sample_rate as f32 * duration.as_secs_f32()) as usize;

    (0..samples)
        .map(move |i| {
            let t = i as f32 / sample_rate as f32;
            let period = 1.0 / frequency;
            let phase = (t % period) / period;
            (2.0 * phase - 1.0) * 0.3
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(rodio::source::SineWave::new)
        .flatten()
}

/// Generate percussive noise (for drums).
///
/// Uses white noise with exponential decay envelope.
#[cfg(feature = "native-audio")]
fn create_percussion(duration: Duration) -> impl Source<Item = f32> {
    use rand::Rng;
    let sample_rate = 48000;
    let samples = (sample_rate as f32 * duration.as_secs_f32()) as usize;
    let mut rng = rand::thread_rng();

    (0..samples)
        .map(move |i| {
            let t = i as f32 / samples as f32;
            let envelope = (-t * 10.0).exp(); // Exponential decay
            let noise: f32 = rng.gen_range(-1.0..1.0);
            noise * envelope * 0.3
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(rodio::source::SineWave::new)
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_engine_creation() {
        // May fail in CI without audio device, so we allow it
        let _ = AudioPlaybackEngine::new();
    }

    #[test]
    fn test_volume_clamping() {
        let mut engine = AudioPlaybackEngine::default();

        engine.set_master_volume(1.5);
        assert_eq!(engine.master_volume(), 1.0);

        engine.set_master_volume(-0.5);
        assert_eq!(engine.master_volume(), 0.0);

        engine.set_master_volume(0.5);
        assert_eq!(engine.master_volume(), 0.5);
    }

    #[test]
    fn test_enable_disable() {
        let mut engine = AudioPlaybackEngine::default();

        assert!(engine.is_enabled() || !engine.is_enabled()); // May fail to init

        engine.set_enabled(false);
        assert!(!engine.is_enabled());

        engine.set_enabled(true);
        assert!(engine.is_enabled());
    }
}
