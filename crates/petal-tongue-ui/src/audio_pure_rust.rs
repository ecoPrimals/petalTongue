// SPDX-License-Identifier: AGPL-3.0-only
//! Pure Rust Audio Generation
//!
//! Provides simple audio generation without external dependencies.
//! Generates tones, beeps, and basic sounds using mathematical waveforms.

use std::f32::consts::PI;

/// Audio sample rate (44.1 kHz - CD quality)
pub const SAMPLE_RATE: u32 = 44100;

/// Simple audio waveforms that can be generated without dependencies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    /// Sine wave (smooth, pure tone)
    Sine,
    /// Square wave (8-bit style, harsh)
    Square,
    /// Sawtooth wave (bright, buzzy)
    Sawtooth,
    /// Triangle wave (mellow)
    Triangle,
    /// White noise
    WhiteNoise,
}

/// Generate audio samples for a tone
#[must_use]
pub fn generate_tone(
    frequency: f32,
    duration_secs: f32,
    waveform: Waveform,
    volume: f32,
) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample = match waveform {
            Waveform::Sine => generate_sine(frequency, t),
            Waveform::Square => generate_square(frequency, t),
            Waveform::Sawtooth => generate_sawtooth(frequency, t),
            Waveform::Triangle => generate_triangle(frequency, t),
            Waveform::WhiteNoise => generate_noise(),
        };

        // Apply volume and envelope
        let envelope = apply_envelope(i, num_samples);
        samples.push(sample * volume * envelope);
    }

    samples
}

/// Generate a sine wave sample
fn generate_sine(frequency: f32, t: f32) -> f32 {
    (2.0 * PI * frequency * t).sin()
}

/// Generate a square wave sample
fn generate_square(frequency: f32, t: f32) -> f32 {
    if (2.0 * PI * frequency * t).sin() >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// Generate a sawtooth wave sample
fn generate_sawtooth(frequency: f32, t: f32) -> f32 {
    2.0 * (frequency * t - (frequency * t + 0.5).floor())
}

/// Generate a triangle wave sample
fn generate_triangle(frequency: f32, t: f32) -> f32 {
    4.0 * (frequency * t - (frequency * t + 0.5).floor()).abs() - 1.0
}

/// Generate white noise sample
fn generate_noise() -> f32 {
    // Simple pseudo-random noise
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f32)
        .sin()
        * 2.0
        - 1.0
}

/// Apply ADSR envelope (Attack, Decay, Sustain, Release)
fn apply_envelope(sample_index: usize, total_samples: usize) -> f32 {
    let t = sample_index as f32 / total_samples as f32;

    // Simple fade in/out envelope
    if t < 0.1 {
        // Attack (10%)
        t / 0.1
    } else if t > 0.9 {
        // Release (10%)
        (1.0 - t) / 0.1
    } else {
        // Sustain
        1.0
    }
}

/// Predefined UI sound effects (pure Rust, no dependencies)
pub struct UISounds;

impl UISounds {
    /// Success chime (pleasant ascending tone)
    #[must_use]
    pub fn success() -> Vec<f32> {
        let mut samples = Vec::new();
        // C major chord: C (261.63 Hz), E (329.63 Hz), G (392.00 Hz)
        samples.extend(generate_tone(261.63, 0.15, Waveform::Sine, 0.3));
        samples.extend(generate_tone(329.63, 0.15, Waveform::Sine, 0.3));
        samples.extend(generate_tone(392.00, 0.3, Waveform::Sine, 0.4));
        samples
    }

    /// Error beep (attention-grabbing)
    #[must_use]
    pub fn error() -> Vec<f32> {
        let mut samples = Vec::new();
        // Two low beeps
        samples.extend(generate_tone(200.0, 0.1, Waveform::Square, 0.4));
        samples.extend(vec![0.0; (SAMPLE_RATE as f32 * 0.05) as usize]); // Silence
        samples.extend(generate_tone(200.0, 0.1, Waveform::Square, 0.4));
        samples
    }

    /// Click sound (UI feedback)
    #[must_use]
    pub fn click() -> Vec<f32> {
        generate_tone(800.0, 0.05, Waveform::Sine, 0.2)
    }

    /// Notification ping (gentle)
    #[must_use]
    pub fn notification() -> Vec<f32> {
        let mut samples = Vec::new();
        samples.extend(generate_tone(880.0, 0.1, Waveform::Sine, 0.3));
        samples.extend(generate_tone(1108.73, 0.2, Waveform::Sine, 0.25));
        samples
    }

    /// Primal discovered (two-tone sequence)
    #[must_use]
    pub fn primal_discovered() -> Vec<f32> {
        let mut samples = Vec::new();
        samples.extend(generate_tone(440.0, 0.1, Waveform::Sine, 0.3));
        samples.extend(generate_tone(554.37, 0.15, Waveform::Sine, 0.35));
        samples
    }

    /// Data refresh (quick blip)
    #[must_use]
    pub fn data_refresh() -> Vec<f32> {
        generate_tone(1000.0, 0.03, Waveform::Triangle, 0.15)
    }

    /// Warning alert
    #[must_use]
    pub fn warning() -> Vec<f32> {
        let mut samples = Vec::new();
        // Alternating tones
        for _ in 0..3 {
            samples.extend(generate_tone(400.0, 0.08, Waveform::Square, 0.3));
            samples.extend(generate_tone(300.0, 0.08, Waveform::Square, 0.3));
        }
        samples
    }

    /// Connection established
    #[must_use]
    pub fn connected() -> Vec<f32> {
        let mut samples = Vec::new();
        // Ascending scale
        let freqs = [261.63, 293.66, 329.63, 349.23];
        for freq in &freqs {
            samples.extend(generate_tone(*freq, 0.08, Waveform::Sine, 0.25));
        }
        samples
    }

    /// Startup chime (welcoming, bright, uplifting)
    /// Inspired by morning awakening sounds
    #[must_use]
    pub fn startup() -> Vec<f32> {
        let mut samples = Vec::new();

        // Bright, ascending melody (major scale feel)
        // Start gentle and build
        samples.extend(generate_tone(440.0, 0.15, Waveform::Sine, 0.25)); // A4
        samples.extend(vec![0.0; (SAMPLE_RATE as f32 * 0.05) as usize]); // Short pause
        samples.extend(generate_tone(554.37, 0.15, Waveform::Sine, 0.30)); // C#5
        samples.extend(vec![0.0; (SAMPLE_RATE as f32 * 0.05) as usize]); // Short pause
        samples.extend(generate_tone(659.25, 0.2, Waveform::Sine, 0.35)); // E5
        samples.extend(vec![0.0; (SAMPLE_RATE as f32 * 0.05) as usize]); // Short pause
        samples.extend(generate_tone(880.0, 0.3, Waveform::Sine, 0.40)); // A5 - triumphant finish!

        samples
    }
}

/// Export samples as WAV file bytes
#[must_use]
pub fn export_wav(samples: &[f32]) -> Vec<u8> {
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = SAMPLE_RATE * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
    let block_align = num_channels * bits_per_sample / 8;

    let data_size = (samples.len() * 2) as u32; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    let mut wav = Vec::new();

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // Audio format (1 = PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Convert f32 samples to i16
    for &sample in samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }

    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tone() {
        let samples = generate_tone(440.0, 0.1, Waveform::Sine, 1.0);
        assert_eq!(samples.len(), 4410); // 0.1s at 44.1kHz
        assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0));
    }

    #[test]
    fn test_ui_sounds() {
        // Test that all sounds generate valid samples
        assert!(!UISounds::success().is_empty());
        assert!(!UISounds::error().is_empty());
        assert!(!UISounds::click().is_empty());
        assert!(!UISounds::notification().is_empty());
    }

    #[test]
    fn test_waveforms() {
        let sine = generate_tone(440.0, 0.01, Waveform::Sine, 1.0);
        let square = generate_tone(440.0, 0.01, Waveform::Square, 1.0);
        let saw = generate_tone(440.0, 0.01, Waveform::Sawtooth, 1.0);
        let tri = generate_tone(440.0, 0.01, Waveform::Triangle, 1.0);

        // All should produce valid samples
        assert!(sine.iter().all(|&s| s.abs() <= 1.0));
        assert!(square.iter().all(|&s| s.abs() <= 1.0));
        assert!(saw.iter().all(|&s| s.abs() <= 1.0));
        assert!(tri.iter().all(|&s| s.abs() <= 1.0));
    }

    #[test]
    fn test_wav_export() {
        let samples = generate_tone(440.0, 0.1, Waveform::Sine, 1.0);
        let wav = export_wav(&samples);

        // Check WAV header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
    }
}
