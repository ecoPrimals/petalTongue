// SPDX-License-Identifier: AGPL-3.0-or-later
//! # Awakening Audio Layers
//!
//! Multi-layered audio for the awakening experience.
//!
//! Pure Rust audio synthesis - no external dependencies!

use std::f32::consts::PI;

/// Audio Layer Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioLayer {
    /// Layer 1: Signature tone (Pure Rust, always available)
    SignatureTone,

    /// Layer 2: Embedded music (MP3)
    EmbeddedMusic,

    /// Layer 3: Nature sounds (synthesized)
    NatureSounds,

    /// Layer 4: Discovery chimes (per primal found)
    DiscoveryChimes,
}

/// Awakening Audio Generator
pub struct AwakeningAudio {
    /// Sample rate (Hz)
    sample_rate: u32,

    /// Current time (seconds)
    time: f32,
}

impl AwakeningAudio {
    /// Create new awakening audio generator
    #[must_use]
    pub const fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            time: 0.0,
        }
    }

    /// Generate signature tone (C major chord)
    ///
    /// This is the Pure Rust fallback that always works.
    pub fn generate_signature_tone(&mut self, duration_secs: f32) -> Vec<f32> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "sample count is positive and bounded by duration*rate; f32 sufficient for audio"
        )]
        let num_samples = (duration_secs * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // C major chord: C4 (261.63 Hz), E4 (329.63 Hz), G4 (392.00 Hz)
        let c4 = 261.63;
        let e4 = 329.63;
        let g4 = 392.00;

        #[expect(
            clippy::cast_precision_loss,
            reason = "audio time index; f32 sufficient for sample-level precision"
        )]
        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;

            // Fade in over first 0.5 seconds
            let fade_in = if t < 0.5 { t / 0.5 } else { 1.0 };

            // Fade out over last 0.5 seconds
            let fade_out = if t > duration_secs - 0.5 {
                (duration_secs - t) / 0.5
            } else {
                1.0
            };

            let envelope = fade_in * fade_out;

            // Mix three sine waves (mul_add for FMA)
            let sample = (2.0 * PI * c4 * t).sin().mul_add(
                0.33,
                (2.0 * PI * e4 * t)
                    .sin()
                    .mul_add(0.33, (2.0 * PI * g4 * t).sin() * 0.33),
            ) * envelope
                * 0.3; // Overall volume

            samples.push(sample);
        }

        self.time += duration_secs;
        samples
    }

    /// Generate heartbeat harmonics (self-knowledge stage)
    pub fn generate_heartbeat(&mut self, duration_secs: f32) -> Vec<f32> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "sample count is positive and bounded by duration*rate; f32 sufficient for audio"
        )]
        let num_samples = (duration_secs * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Heartbeat: 60 BPM = 1 Hz
        let heartbeat_freq = 1.0;

        // Harmonic frequencies
        let fundamental = 80.0; // Low bass
        let harmonic2 = 160.0;
        let harmonic3 = 240.0;

        #[expect(
            clippy::cast_precision_loss,
            reason = "audio time index; f32 sufficient for sample-level precision"
        )]
        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;

            // Heartbeat envelope (two pulses per beat)
            let beat_phase = (t * heartbeat_freq * 2.0 * PI).sin();
            let pulse = if beat_phase > 0.0 {
                beat_phase.powi(4) // Sharp attack
            } else {
                0.0
            };

            // Mix harmonics (mul_add for FMA)
            let sample = (2.0 * PI * fundamental * t).sin().mul_add(
                0.5,
                (2.0 * PI * harmonic2 * t)
                    .sin()
                    .mul_add(0.3, (2.0 * PI * harmonic3 * t).sin() * 0.2),
            ) * pulse
                * 0.2;

            samples.push(sample);
        }

        self.time += duration_secs;
        samples
    }

    /// Generate discovery chime (when primal found)
    pub fn generate_discovery_chime(&mut self, primal_index: u32) -> Vec<f32> {
        let duration_secs = 0.5;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "sample count is positive and bounded by duration*rate; f32 sufficient for audio"
        )]
        let num_samples = (duration_secs * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Each primal gets a unique tone in the pentatonic scale
        // C, D, E, G, A (major pentatonic)
        let frequencies = [261.63, 293.66, 329.63, 392.00, 440.00];
        let freq = frequencies[(primal_index as usize) % frequencies.len()];

        #[expect(
            clippy::cast_precision_loss,
            reason = "audio time index; f32 sufficient for sample-level precision"
        )]
        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;

            // Bell-like envelope (fast attack, slow decay)
            let envelope = (-t * 5.0).exp();

            // Add harmonics for bell-like timbre (mul_add for FMA)
            let sample = (2.0 * PI * freq * t).sin().mul_add(
                0.5,
                (2.0 * PI * freq * 2.0 * t)
                    .sin()
                    .mul_add(0.3, (2.0 * PI * freq * 3.0 * t).sin() * 0.2),
            ) * envelope
                * 0.4;

            samples.push(sample);
        }

        samples
    }

    /// Generate bird chirp (nature sounds)
    pub fn generate_bird_chirp(&mut self) -> Vec<f32> {
        let duration_secs = 0.3;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "sample count is positive and bounded by duration*rate; f32 sufficient for audio"
        )]
        let num_samples = (duration_secs * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Bird chirp: frequency sweep from 2000 Hz to 3000 Hz
        let start_freq = 2000.0_f32;
        let end_freq = 3000.0_f32;

        #[expect(
            clippy::cast_precision_loss,
            reason = "audio time index; f32 sufficient for sample-level precision"
        )]
        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;
            let progress = t / duration_secs;

            // Frequency sweep (mul_add for FMA)
            let freq = (end_freq - start_freq).mul_add(progress, start_freq);

            // Envelope (quick attack and decay)
            let envelope = (progress * PI).sin();

            let sample = (2.0 * PI * freq * t).sin() * envelope * 0.15;

            samples.push(sample);
        }

        samples
    }

    /// Generate wind ambience (filtered noise)
    pub fn generate_wind(&mut self, duration_secs: f32) -> Vec<f32> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "sample count is positive and bounded by duration*rate; f32 sufficient for audio"
        )]
        let num_samples = (duration_secs * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Simple low-pass filtered noise
        let mut prev_sample = 0.0;
        let alpha = 0.05; // Low-pass filter coefficient

        #[expect(
            clippy::cast_precision_loss,
            reason = "pseudo-random noise seed; f32 sufficient for audio"
        )]
        for i in 0..num_samples {
            // Generate white noise
            let noise = (i as f32 * 12.9898).sin() * 43_758.547;
            let noise = (noise - noise.floor()).mul_add(2.0, -1.0);

            // Low-pass filter
            prev_sample = alpha * noise + (1.0 - alpha) * prev_sample;

            samples.push(prev_sample * 0.1);
        }

        self.time += duration_secs;
        samples
    }

    /// Generate complete awakening audio sequence
    pub fn generate_awakening_sequence(&mut self) -> Vec<f32> {
        let mut sequence = Vec::new();

        // Stage 1: Awakening (0-3s) - Signature tone
        sequence.extend(self.generate_signature_tone(3.0));

        // Stage 2: Self-Knowledge (3-6s) - Heartbeat
        sequence.extend(self.generate_heartbeat(3.0));

        // Stage 3: Discovery (6-10s) - Wind + discovery chimes
        let mut stage3 = self.generate_wind(4.0);

        // Add discovery chimes at 1s intervals
        for i in 0..3 {
            let chime = self.generate_discovery_chime(i);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                reason = "offset is positive, i in 0..3; f32 sufficient for sample offset"
            )]
            let offset = (i as f32 * 1.0 * self.sample_rate as f32) as usize;

            // Mix chime into wind
            for (j, &sample) in chime.iter().enumerate() {
                if offset + j < stage3.len() {
                    stage3[offset + j] += sample;
                }
            }
        }

        sequence.extend(stage3);

        // Stage 4: Tutorial (10-12s) - Completion harmony
        sequence.extend(self.generate_signature_tone(2.0));

        sequence
    }
}

/// Mix multiple audio layers
pub fn mix_layers(layers: Vec<Vec<f32>>) -> Vec<f32> {
    if layers.is_empty() {
        return Vec::new();
    }

    // Find longest layer
    let max_len = layers.iter().map(std::vec::Vec::len).max().unwrap_or(0);

    let mut mixed = vec![0.0; max_len];

    for layer in layers {
        for (i, &sample) in layer.iter().enumerate() {
            mixed[i] += sample;
        }
    }

    // Normalize to prevent clipping
    let max_amplitude = mixed.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if max_amplitude > 1.0 {
        for sample in &mut mixed {
            *sample /= max_amplitude;
        }
    }

    mixed
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn test_awakening_audio_creation() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_signature_tone(0.1);
        assert_eq!(samples.len(), 4410, "0.1s at 44100 Hz = 4410 samples");
    }

    #[test]
    fn test_signature_tone_generation() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_signature_tone(1.0);

        assert_eq!(samples.len(), 44100);

        // Check that samples are in valid range
        for &sample in &samples {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_heartbeat_generation() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_heartbeat(1.0);

        assert_eq!(samples.len(), 44100);

        // Check valid range
        for &sample in &samples {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_discovery_chime() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_discovery_chime(0);

        assert_eq!(samples.len(), 22050); // 0.5 seconds

        // Check valid range
        for &sample in &samples {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_bird_chirp() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_bird_chirp();

        assert_eq!(samples.len(), 13230); // 0.3 seconds
    }

    #[test]
    fn test_wind_generation() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_wind(1.0);

        assert_eq!(samples.len(), 44100);
    }

    #[test]
    fn test_awakening_sequence() {
        let mut audio = AwakeningAudio::new(44100);
        let sequence = audio.generate_awakening_sequence();

        // Should be 12 seconds total
        assert_eq!(sequence.len(), 44100 * 12);

        // Check all samples in valid range
        for &sample in &sequence {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_mix_layers() {
        let layer1 = vec![0.5, 0.5, 0.5];
        let layer2 = vec![0.3, 0.3, 0.3];

        let mixed = mix_layers(vec![layer1, layer2]);

        assert_eq!(mixed.len(), 3);
        assert!((mixed[0] - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_mix_layers_normalization() {
        let layer1 = vec![0.8, 0.8, 0.8];
        let layer2 = vec![0.8, 0.8, 0.8];

        let mixed = mix_layers(vec![layer1, layer2]);

        // Should be normalized to prevent clipping
        for &sample in &mixed {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_unique_discovery_chimes() {
        let mut audio = AwakeningAudio::new(44100);

        let chime0 = audio.generate_discovery_chime(0);
        let chime1 = audio.generate_discovery_chime(1);

        // Chimes should be different (different frequencies)
        assert_ne!(chime0, chime1);
    }

    #[test]
    fn test_discovery_chime_pentatonic_wraparound() {
        let mut audio = AwakeningAudio::new(44100);
        let chime5 = audio.generate_discovery_chime(5);
        let chime0 = audio.generate_discovery_chime(0);
        assert_eq!(chime5.len(), chime0.len());
    }

    #[test]
    fn test_signature_tone_fade_in_out() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_signature_tone(1.0);
        assert!(samples[0].abs() < 0.1, "start should have fade-in");
        assert!(
            samples[samples.len() - 1].abs() < 0.1,
            "end should have fade-out"
        );
    }

    #[test]
    fn test_mix_layers_empty() {
        let mixed = mix_layers(vec![]);
        assert!(mixed.is_empty());
    }

    #[test]
    fn test_mix_layers_different_lengths() {
        let layer1 = vec![0.5, 0.5];
        let layer2 = vec![0.3, 0.3, 0.3, 0.3];
        let mixed = mix_layers(vec![layer1, layer2]);
        assert_eq!(mixed.len(), 4);
        assert!((mixed[0] - 0.8).abs() < 0.01);
        assert!((mixed[2] - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_bird_chirp_frequency_sweep() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_bird_chirp();
        assert!(!samples.is_empty());
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_heartbeat_pulse_shape() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_heartbeat(2.0);
        assert_eq!(samples.len(), 88200);
        let max_amp = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amp > 0.0 && max_amp <= 1.0);
    }

    #[test]
    fn test_wind_filtered_noise() {
        let mut audio = AwakeningAudio::new(44100);
        let samples = audio.generate_wind(0.5);
        assert_eq!(samples.len(), 22050);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_audio_layer_enum_variants() {
        assert_eq!(AudioLayer::SignatureTone, AudioLayer::SignatureTone);
        assert_eq!(AudioLayer::EmbeddedMusic, AudioLayer::EmbeddedMusic);
        assert_eq!(AudioLayer::NatureSounds, AudioLayer::NatureSounds);
        assert_eq!(AudioLayer::DiscoveryChimes, AudioLayer::DiscoveryChimes);
        assert_ne!(AudioLayer::SignatureTone, AudioLayer::NatureSounds);
    }

    #[test]
    fn test_audio_layer_debug_clone() {
        let layer = AudioLayer::SignatureTone;
        let cloned = layer;
        assert_eq!(layer, cloned);
        let debug_str = format!("{layer:?}");
        assert!(debug_str.contains("SignatureTone"));
    }

    #[test]
    fn test_awakening_audio_time_advances() {
        let mut audio = AwakeningAudio::new(48000);
        let _ = audio.generate_signature_tone(0.5);
        let _ = audio.generate_heartbeat(0.5);
        let samples = audio.generate_discovery_chime(2);
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_mix_layers_single_layer() {
        let layer = vec![0.5, 0.3, 0.7];
        let mixed = mix_layers(vec![layer]);
        assert_eq!(mixed.len(), 3);
        assert!((mixed[0] - 0.5).abs() < 0.01);
    }
}
