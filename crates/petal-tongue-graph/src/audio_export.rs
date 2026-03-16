// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure Rust Audio File Export
//!
//! Generates WAV files from audio attributes without requiring system audio libraries.
//! This is 100% pure Rust using the `hound` crate.

use crate::audio_export_error::AudioExportError;
use crate::audio_sonification::{AudioAttributes, Instrument};
use hound::{WavSpec, WavWriter};
use std::path::Path;

/// Audio file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// WAV format (uncompressed)
    Wav,
}

/// Audio quality settings
#[derive(Debug, Clone, Copy)]
pub struct AudioQuality {
    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Bits per sample (16 or 24)
    pub bits_per_sample: u16,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
}

impl Default for AudioQuality {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            bits_per_sample: 16,
            channels: 2, // Stereo for panning
        }
    }
}

/// Pure Rust audio file generator
pub struct AudioFileGenerator {
    quality: AudioQuality,
    master_volume: f32,
}

impl AudioFileGenerator {
    /// Create a new audio file generator
    #[must_use]
    pub fn new() -> Self {
        Self {
            quality: AudioQuality::default(),
            master_volume: 0.3,
        }
    }

    /// Set audio quality
    #[must_use]
    pub const fn with_quality(mut self, quality: AudioQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Set master volume (0.0 - 1.0)
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.master_volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Export a single tone to WAV file
    ///
    /// # Errors
    /// Returns error if file cannot be created or written
    pub fn export_tone<P: AsRef<Path>>(
        &self,
        path: P,
        attrs: &AudioAttributes,
        duration_secs: f32,
    ) -> Result<(), AudioExportError> {
        let spec = WavSpec {
            channels: self.quality.channels,
            sample_rate: self.quality.sample_rate,
            bits_per_sample: self.quality.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let path_buf = path.as_ref().to_path_buf();
        let mut writer =
            WavWriter::create(path.as_ref(), spec).map_err(|e| AudioExportError::CreateFile {
                path: path_buf,
                source: e,
            })?;

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let num_samples = (self.quality.sample_rate as f32 * duration_secs) as usize;
        let frequency = attrs.pitch.mul_add(700.0, 100.0);

        // Apply volume
        let volume = attrs.volume * self.master_volume;

        // Constants for audio conversion
        let max_amplitude = f32::from(i16::MAX);

        // Generate samples based on instrument type
        for i in 0..num_samples {
            #[expect(clippy::cast_precision_loss)]
            let t = i as f32 / self.quality.sample_rate as f32;
            let sample = self.generate_sample(t, frequency, attrs.instrument, volume);

            // Write stereo samples with panning
            if self.quality.channels == 2 {
                let left_gain = if attrs.pan < 0.0 {
                    1.0
                } else {
                    1.0 - attrs.pan
                };
                let right_gain = if attrs.pan > 0.0 {
                    1.0
                } else {
                    1.0 + attrs.pan
                };

                #[expect(clippy::cast_possible_truncation)]
                let left = (sample * left_gain * max_amplitude) as i16;
                #[expect(clippy::cast_possible_truncation)]
                let right = (sample * right_gain * max_amplitude) as i16;

                writer.write_sample(left)?;
                writer.write_sample(right)?;
            } else {
                // Mono
                #[expect(clippy::cast_possible_truncation)]
                let mono = (sample * max_amplitude) as i16;
                writer.write_sample(mono)?;
            }
        }

        writer.finalize()?;
        tracing::info!(
            "Exported audio to: {} ({:.2}s, {} Hz)",
            path.as_ref().display(),
            duration_secs,
            self.quality.sample_rate
        );

        Ok(())
    }

    /// Export a soundscape (multiple tones) to WAV file
    ///
    /// # Errors
    /// Returns error if file cannot be created or written
    pub fn export_soundscape<P: AsRef<Path>>(
        &self,
        path: P,
        soundscape: &[(String, AudioAttributes)],
        duration_secs: f32,
    ) -> Result<(), AudioExportError> {
        let spec = WavSpec {
            channels: self.quality.channels,
            sample_rate: self.quality.sample_rate,
            bits_per_sample: self.quality.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let path_buf = path.as_ref().to_path_buf();
        let mut writer =
            WavWriter::create(path.as_ref(), spec).map_err(|e| AudioExportError::CreateFile {
                path: path_buf,
                source: e,
            })?;

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let num_samples = (self.quality.sample_rate as f32 * duration_secs) as usize;

        // Constants for audio conversion
        let max_amplitude = f32::from(i16::MAX);

        // Mix all tones together
        for i in 0..num_samples {
            #[expect(clippy::cast_precision_loss)]
            let t = i as f32 / self.quality.sample_rate as f32;

            // Mix all primal tones
            let mut mixed_sample = 0.0;
            for (_id, attrs) in soundscape {
                let frequency = attrs.pitch.mul_add(700.0, 100.0);
                let volume = attrs.volume * self.master_volume;
                mixed_sample += self.generate_sample(t, frequency, attrs.instrument, volume);
            }

            // Normalize to prevent clipping
            #[expect(
                clippy::cast_precision_loss,
                reason = "RMS normalization, count fits f32 for audio"
            )]
            let num_tones = soundscape.len() as f32;
            mixed_sample /= num_tones.sqrt(); // RMS normalization

            // Write stereo samples
            #[expect(clippy::cast_possible_truncation)]
            let sample_i16 = (mixed_sample * max_amplitude) as i16;
            writer.write_sample(sample_i16)?;
            if self.quality.channels == 2 {
                // For soundscape, we could spatialize each source, but for now just center
                writer.write_sample(sample_i16)?;
            }
        }

        writer.finalize()?;
        tracing::info!(
            "Exported soundscape ({} tones) to: {} ({:.2}s, {} Hz)",
            soundscape.len(),
            path.as_ref().display(),
            duration_secs,
            self.quality.sample_rate
        );

        Ok(())
    }

    /// Generate a single sample value for a given waveform
    #[expect(clippy::unused_self, reason = "trait-style method for consistency")]
    fn generate_sample(&self, t: f32, frequency: f32, instrument: Instrument, volume: f32) -> f32 {
        use std::f32::consts::PI;

        let angle = 2.0 * PI * frequency * t;

        let waveform = match instrument {
            Instrument::Bass => {
                // Sine wave (smooth, fundamental)
                angle.sin()
            }
            Instrument::Drums => {
                // White noise with decay (deterministic approximation - no rand dependency)
                let envelope = (-t * 10.0).exp();
                let noise = (t * 12_345.679).sin() * (t * 98_765.43).cos();
                noise * envelope
            }
            Instrument::Chimes => {
                // Triangle wave (bright)
                let period = 1.0 / frequency;
                let phase = (t % period) / period;
                if phase < 0.5 {
                    4.0f32.mul_add(phase, -1.0)
                } else {
                    4.0f32.mul_add(-phase, 3.0)
                }
            }
            Instrument::Strings => {
                // Sawtooth wave (rich harmonics)
                let period = 1.0 / frequency;
                let phase = (t % period) / period;
                2.0f32.mul_add(phase, -1.0)
            }
            Instrument::Synth => {
                // Square wave (electronic)
                if angle.sin() > 0.0 { 1.0 } else { -1.0 }
            }
            Instrument::Default => angle.sin(),
        };

        waveform * volume
    }
}

impl Default for AudioFileGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_sonification::Instrument;

    #[test]
    fn test_generator_creation() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_creation.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.7,
            pan: 0.0,
        };

        let temp_path = std::env::temp_dir().join("test_tone.wav");
        let result = generator.export_tone(&temp_path, &attrs, 1.0);
        assert!(result.is_ok());

        // Verify file exists
        assert!(temp_path.exists());

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_export_soundscape() {
        let generator = AudioFileGenerator::new();
        let soundscape = vec![
            (
                "primal1".to_string(),
                AudioAttributes {
                    instrument: Instrument::Bass,
                    pitch: 0.3,
                    volume: 0.5,
                    pan: -0.5,
                },
            ),
            (
                "primal2".to_string(),
                AudioAttributes {
                    instrument: Instrument::Chimes,
                    pitch: 0.8,
                    volume: 0.6,
                    pan: 0.5,
                },
            ),
        ];

        let temp_path = std::env::temp_dir().join("test_soundscape.wav");
        let result = generator.export_soundscape(&temp_path, &soundscape, 2.0);
        assert!(result.is_ok());

        // Verify file exists
        assert!(temp_path.exists());

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_audio_quality_default() {
        let q = AudioQuality::default();
        assert_eq!(q.sample_rate, 48000);
        assert_eq!(q.bits_per_sample, 16);
        assert_eq!(q.channels, 2);
    }

    #[test]
    fn test_with_quality() {
        let q = AudioQuality {
            sample_rate: 22050,
            bits_per_sample: 24,
            channels: 1,
        };
        let generator = AudioFileGenerator::new().with_quality(q);
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_quality.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_with_volume() {
        let generator = AudioFileGenerator::new().with_volume(0.5);
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 1.0,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_vol.wav");
        let _ = generator.export_tone(&temp, &attrs, 0.1);
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_drums() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Drums,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_drums.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_strings() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Strings,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_strings.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_synth() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Synth,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_synth.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_default_instrument() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Default,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_default.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_mono() {
        let q = AudioQuality {
            sample_rate: 44100,
            bits_per_sample: 16,
            channels: 1,
        };
        let generator = AudioFileGenerator::new().with_quality(q);
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_mono.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_pan_left() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.5,
            pan: -1.0,
        };
        let temp = std::env::temp_dir().join("test_pan_left.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_export_tone_pan_right() {
        let generator = AudioFileGenerator::new();
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.5,
            pan: 1.0,
        };
        let temp = std::env::temp_dir().join("test_pan_right.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_audio_file_generator_default() {
        let generator = AudioFileGenerator::default();
        let attrs = AudioAttributes {
            instrument: Instrument::Bass,
            pitch: 0.5,
            volume: 0.5,
            pan: 0.0,
        };
        let temp = std::env::temp_dir().join("test_default_gen.wav");
        assert!(generator.export_tone(&temp, &attrs, 0.1).is_ok());
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn test_frequency_formula() {
        let freq = |pitch: f32| pitch.mul_add(700.0, 100.0);
        assert!((freq(0.0) - 100.0).abs() < f32::EPSILON);
        assert!((freq(1.0) - 800.0).abs() < f32::EPSILON);
    }
}
