// SPDX-License-Identifier: AGPL-3.0-or-later
//! Layered soundscape synthesis for ambient and game audio.
//!
//! Extends the basic sine synthesis in `audio_synthesis.rs` with richer
//! waveforms, layered mixing, and spatial positioning for immersive
//! soundscapes driven by springs (ludoSpring game audio, wetSpring
//! ecology ambience, etc.).
//!
//! All synthesis is deterministic and pure Rust — same parameters always
//! produce the same samples.

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

/// Waveform type for oscillator-based synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    WhiteNoise,
}

impl Waveform {
    /// Generate a single sample at phase `t` (0.0 to 1.0 = one cycle).
    #[must_use]
    pub fn sample(self, phase: f64, noise_seed: u64) -> f64 {
        match self {
            Self::Sine => (2.0 * PI * phase).sin(),
            Self::Square => {
                if phase.fract() < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Sawtooth => 2.0f64.mul_add(phase.fract(), -1.0),
            Self::Triangle => {
                let f = phase.fract();
                if f < 0.5 {
                    4.0f64.mul_add(f, -1.0)
                } else {
                    4.0f64.mul_add(-f, 3.0)
                }
            }
            Self::WhiteNoise => {
                let hash = noise_seed
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "noise seed: f64 sufficient for [-1,1] range"
                )]
                let norm = hash as f64 / u64::MAX as f64;
                norm.mul_add(2.0, -1.0)
            }
        }
    }
}

/// A single sound layer in a soundscape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundLayer {
    /// Layer identifier.
    pub id: String,
    /// Waveform type.
    pub waveform: Waveform,
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Amplitude (0.0 to 1.0).
    pub amplitude: f64,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Stereo pan (-1.0 = full left, 0.0 = center, 1.0 = full right).
    #[serde(default)]
    pub pan: f64,
    /// Fade-in duration in seconds.
    #[serde(default)]
    pub fade_in_secs: f64,
    /// Fade-out duration in seconds.
    #[serde(default)]
    pub fade_out_secs: f64,
    /// Start offset within the soundscape (seconds from beginning).
    #[serde(default)]
    pub offset_secs: f64,
}

/// A complete soundscape definition: multiple layers mixed together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soundscape {
    /// Human-readable name.
    pub name: String,
    /// Total duration in seconds (layers beyond this are truncated).
    pub duration_secs: f64,
    /// Sound layers to mix.
    pub layers: Vec<SoundLayer>,
    /// Master amplitude (0.0 to 1.0).
    #[serde(default = "default_master")]
    pub master_amplitude: f64,
    /// Sample rate (default 44100).
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
}

const fn default_master() -> f64 {
    0.8
}

const fn default_sample_rate() -> u32 {
    44100
}

/// Stereo PCM output from soundscape synthesis.
#[derive(Debug, Clone)]
pub struct StereoSamples {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub sample_rate: u32,
}

impl StereoSamples {
    /// Interleave left/right for standard stereo PCM output.
    #[must_use]
    pub fn interleaved(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.left.len() * 2);
        for (l, r) in self.left.iter().zip(&self.right) {
            out.push(*l);
            out.push(*r);
        }
        out
    }

    /// Mono mixdown (average of left and right).
    #[must_use]
    pub fn mono(&self) -> Vec<f32> {
        self.left
            .iter()
            .zip(&self.right)
            .map(|(l, r)| (l + r) * 0.5)
            .collect()
    }

    /// Duration in seconds.
    #[must_use]
    pub fn duration_secs(&self) -> f64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "duration: f64 sufficient for audio"
        )]
        let len_f64 = self.left.len() as f64;
        len_f64 / f64::from(self.sample_rate)
    }
}

/// Synthesize a soundscape into stereo PCM samples.
///
/// Deterministic: same `Soundscape` always produces the same output.
#[must_use]
pub fn synthesize_soundscape(scape: &Soundscape) -> StereoSamples {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "duration * sample_rate is non-negative; truncation acceptable for sample count"
    )]
    let total_samples = (scape.duration_secs * f64::from(scape.sample_rate)) as usize;
    let mut left = vec![0.0f64; total_samples];
    let mut right = vec![0.0f64; total_samples];

    for layer in &scape.layers {
        render_layer(layer, scape.sample_rate, &mut left, &mut right);
    }

    let master = scape.master_amplitude;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "PCM output clamped to [-1,1]"
    )]
    let left_f32: Vec<f32> = left
        .iter()
        .map(|s| (s * master).clamp(-1.0, 1.0) as f32)
        .collect();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "PCM output clamped to [-1,1]"
    )]
    let right_f32: Vec<f32> = right
        .iter()
        .map(|s| (s * master).clamp(-1.0, 1.0) as f32)
        .collect();

    StereoSamples {
        left: left_f32,
        right: right_f32,
        sample_rate: scape.sample_rate,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "sample index for envelope: f64 sufficient"
)]
fn render_layer(layer: &SoundLayer, sample_rate: u32, left: &mut [f64], right: &mut [f64]) {
    let sr = f64::from(sample_rate);

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "offsets are non-negative; truncation acceptable for sample index"
    )]
    let start_sample = (layer.offset_secs * sr) as usize;

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "duration is non-negative; truncation acceptable for sample count"
    )]
    let layer_samples = (layer.duration_secs * sr) as usize;

    let end_sample = (start_sample + layer_samples).min(left.len());

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "fade durations are non-negative; truncation acceptable"
    )]
    let fade_in_samples = (layer.fade_in_secs * sr) as usize;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "fade durations are non-negative; truncation acceptable"
    )]
    let fade_out_samples = (layer.fade_out_secs * sr) as usize;

    let left_gain = ((1.0 - layer.pan) * 0.5).clamp(0.0, 1.0);
    let right_gain = ((1.0 + layer.pan) * 0.5).clamp(0.0, 1.0);

    for i in start_sample..end_sample {
        let local_i = i - start_sample;
        let t = local_i as f64 / sr;
        let phase = layer.frequency * t;

        let noise_seed = (i as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
        let raw = layer.waveform.sample(phase, noise_seed);

        let mut envelope = layer.amplitude;

        if fade_in_samples > 0 && local_i < fade_in_samples {
            envelope *= local_i as f64 / fade_in_samples as f64;
        }

        let remaining = end_sample - start_sample - local_i;
        if fade_out_samples > 0 && remaining < fade_out_samples {
            envelope *= remaining as f64 / fade_out_samples as f64;
        }

        let sample = raw * envelope;
        left[i] += sample * left_gain;
        right[i] += sample * right_gain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waveform_sine_zero_at_start() {
        let s = Waveform::Sine.sample(0.0, 0);
        assert!(s.abs() < 1e-10);
    }

    #[test]
    fn waveform_square_bipolar() {
        assert!((Waveform::Square.sample(0.1, 0) - 1.0).abs() < f64::EPSILON);
        assert!((Waveform::Square.sample(0.7, 0) - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn waveform_sawtooth_range() {
        for i in 0..100 {
            let phase = f64::from(i) / 100.0;
            let s = Waveform::Sawtooth.sample(phase, 0);
            assert!(
                (-1.0..=1.0).contains(&s),
                "sawtooth out of range at phase {phase}: {s}"
            );
        }
    }

    #[test]
    fn waveform_triangle_range() {
        for i in 0..100 {
            let phase = f64::from(i) / 100.0;
            let s = Waveform::Triangle.sample(phase, 0);
            assert!(
                (-1.0..=1.0).contains(&s),
                "triangle out of range at phase {phase}: {s}"
            );
        }
    }

    #[test]
    fn waveform_noise_range() {
        for seed in 0..1000_u64 {
            let s = Waveform::WhiteNoise.sample(0.0, seed);
            assert!((-1.0..=1.0).contains(&s), "noise out of range: {s}");
        }
    }

    #[test]
    fn synthesize_empty_soundscape() {
        let scape = Soundscape {
            name: "silence".to_string(),
            duration_secs: 0.1,
            layers: vec![],
            master_amplitude: 0.8,
            sample_rate: 44100,
        };
        let out = synthesize_soundscape(&scape);
        assert_eq!(out.left.len(), 4410);
        assert!(out.left.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn synthesize_single_layer() {
        let scape = Soundscape {
            name: "tone".to_string(),
            duration_secs: 0.01,
            layers: vec![SoundLayer {
                id: "tone".to_string(),
                waveform: Waveform::Sine,
                frequency: 440.0,
                amplitude: 0.5,
                duration_secs: 0.01,
                pan: 0.0,
                fade_in_secs: 0.0,
                fade_out_secs: 0.0,
                offset_secs: 0.0,
            }],
            master_amplitude: 1.0,
            sample_rate: 44100,
        };
        let out = synthesize_soundscape(&scape);
        assert_eq!(out.left.len(), out.right.len());
        let has_nonzero = out.left.iter().any(|s| s.abs() > 0.001);
        assert!(has_nonzero, "should produce audible samples");
    }

    #[test]
    fn stereo_panning() {
        let scape = Soundscape {
            name: "left".to_string(),
            duration_secs: 0.01,
            layers: vec![SoundLayer {
                id: "l".to_string(),
                waveform: Waveform::Sine,
                frequency: 440.0,
                amplitude: 1.0,
                duration_secs: 0.01,
                pan: -1.0,
                fade_in_secs: 0.0,
                fade_out_secs: 0.0,
                offset_secs: 0.0,
            }],
            master_amplitude: 1.0,
            sample_rate: 44100,
        };
        let out = synthesize_soundscape(&scape);
        let left_energy: f64 = out.left.iter().map(|s| f64::from(*s).powi(2)).sum();
        let right_energy: f64 = out.right.iter().map(|s| f64::from(*s).powi(2)).sum();
        assert!(
            left_energy > right_energy * 10.0,
            "left should be much louder: L={left_energy:.4} R={right_energy:.4}"
        );
    }

    #[test]
    fn fade_in_out() {
        let scape = Soundscape {
            name: "fade".to_string(),
            duration_secs: 0.1,
            layers: vec![SoundLayer {
                id: "f".to_string(),
                waveform: Waveform::Square,
                frequency: 100.0,
                amplitude: 1.0,
                duration_secs: 0.1,
                pan: 0.0,
                fade_in_secs: 0.02,
                fade_out_secs: 0.02,
                offset_secs: 0.0,
            }],
            master_amplitude: 1.0,
            sample_rate: 44100,
        };
        let out = synthesize_soundscape(&scape);
        assert!(
            out.left[0].abs() < 0.01,
            "first sample should be near zero (fade in)"
        );
        let last = out.left.last().unwrap();
        assert!(
            last.abs() < 0.01,
            "last sample should be near zero (fade out)"
        );
    }

    #[test]
    fn interleaved_output() {
        let samples = StereoSamples {
            left: vec![1.0, 2.0],
            right: vec![3.0, 4.0],
            sample_rate: 44100,
        };
        assert_eq!(samples.interleaved(), vec![1.0, 3.0, 2.0, 4.0]);
    }

    #[test]
    fn mono_mixdown() {
        let samples = StereoSamples {
            left: vec![1.0, 0.0],
            right: vec![0.0, 1.0],
            sample_rate: 44100,
        };
        let mono = samples.mono();
        assert!((mono[0] - 0.5).abs() < f64::EPSILON as f32);
        assert!((mono[1] - 0.5).abs() < f64::EPSILON as f32);
    }

    #[test]
    fn deterministic_output() {
        let scape = Soundscape {
            name: "det".to_string(),
            duration_secs: 0.01,
            layers: vec![SoundLayer {
                id: "a".to_string(),
                waveform: Waveform::Sawtooth,
                frequency: 220.0,
                amplitude: 0.5,
                duration_secs: 0.01,
                pan: 0.3,
                fade_in_secs: 0.001,
                fade_out_secs: 0.001,
                offset_secs: 0.0,
            }],
            master_amplitude: 0.8,
            sample_rate: 44100,
        };
        let a = synthesize_soundscape(&scape);
        let b = synthesize_soundscape(&scape);
        assert_eq!(a.left, b.left);
        assert_eq!(a.right, b.right);
    }

    #[test]
    fn soundscape_serde_round_trip() {
        let scape = Soundscape {
            name: "test".to_string(),
            duration_secs: 1.0,
            layers: vec![SoundLayer {
                id: "bg".to_string(),
                waveform: Waveform::WhiteNoise,
                frequency: 0.0,
                amplitude: 0.1,
                duration_secs: 1.0,
                pan: 0.0,
                fade_in_secs: 0.5,
                fade_out_secs: 0.5,
                offset_secs: 0.0,
            }],
            master_amplitude: 0.8,
            sample_rate: 44100,
        };
        let json = serde_json::to_string(&scape).unwrap();
        let back: Soundscape = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
        assert_eq!(back.layers.len(), 1);
        assert_eq!(back.layers[0].waveform, Waveform::WhiteNoise);
    }

    #[test]
    fn duration_secs_calculation() {
        let samples = StereoSamples {
            left: vec![0.0; 44100],
            right: vec![0.0; 44100],
            sample_rate: 44100,
        };
        assert!((samples.duration_secs() - 1.0).abs() < 0.001);
    }
}
