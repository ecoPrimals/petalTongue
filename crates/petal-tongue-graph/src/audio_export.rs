//! Pure Rust Audio File Export
//!
//! Generates WAV files from audio attributes without requiring system audio libraries.
//! This is 100% pure Rust using the `hound` crate.

use crate::audio_sonification::{AudioAttributes, Instrument};
use anyhow::Result;
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
    pub fn with_quality(mut self, quality: AudioQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Set master volume (0.0 - 1.0)
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
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
    ) -> Result<()> {
        let spec = WavSpec {
            channels: self.quality.channels,
            sample_rate: self.quality.sample_rate,
            bits_per_sample: self.quality.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path.as_ref(), spec).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create WAV file: {} - {}",
                path.as_ref().display(),
                e
            )
        })?;

        let num_samples = (self.quality.sample_rate as f32 * duration_secs) as usize;
        let frequency = 100.0 + (attrs.pitch * 700.0);

        // Apply volume
        let volume = attrs.volume * self.master_volume;

        // Generate samples based on instrument type
        for i in 0..num_samples {
            let t = i as f32 / self.quality.sample_rate as f32;
            let sample = self.generate_sample(t, frequency, &attrs.instrument, volume);

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

                let left = (sample * left_gain * i16::MAX as f32) as i16;
                let right = (sample * right_gain * i16::MAX as f32) as i16;

                writer.write_sample(left)?;
                writer.write_sample(right)?;
            } else {
                // Mono
                let mono = (sample * i16::MAX as f32) as i16;
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
    ) -> Result<()> {
        let spec = WavSpec {
            channels: self.quality.channels,
            sample_rate: self.quality.sample_rate,
            bits_per_sample: self.quality.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path.as_ref(), spec).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create WAV file: {} - {}",
                path.as_ref().display(),
                e
            )
        })?;

        let num_samples = (self.quality.sample_rate as f32 * duration_secs) as usize;

        // Mix all tones together
        for i in 0..num_samples {
            let t = i as f32 / self.quality.sample_rate as f32;

            // Mix all primal tones
            let mut mixed_sample = 0.0;
            for (_id, attrs) in soundscape {
                let frequency = 100.0 + (attrs.pitch * 700.0);
                let volume = attrs.volume * self.master_volume;
                mixed_sample += self.generate_sample(t, frequency, &attrs.instrument, volume);
            }

            // Normalize to prevent clipping
            let num_tones = soundscape.len() as f32;
            mixed_sample /= num_tones.sqrt(); // RMS normalization

            // Write stereo samples
            if self.quality.channels == 2 {
                // For soundscape, we could spatialize each source, but for now just center
                let sample_i16 = (mixed_sample * i16::MAX as f32) as i16;
                writer.write_sample(sample_i16)?;
                writer.write_sample(sample_i16)?;
            } else {
                let sample_i16 = (mixed_sample * i16::MAX as f32) as i16;
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
    fn generate_sample(&self, t: f32, frequency: f32, instrument: &Instrument, volume: f32) -> f32 {
        use std::f32::consts::PI;

        let angle = 2.0 * PI * frequency * t;

        let waveform = match instrument {
            Instrument::Bass => {
                // Sine wave (smooth, fundamental)
                angle.sin()
            }
            Instrument::Drums => {
                // White noise with decay
                #[cfg(feature = "native-audio")]
                {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let envelope = (-t * 10.0).exp();
                    rng.gen_range(-1.0..1.0) * envelope
                }
                #[cfg(not(feature = "native-audio"))]
                {
                    // Simple noise approximation without rand
                    let envelope = (-t * 10.0).exp();
                    let noise = (t * 12345.6789).sin() * (t * 98765.4321).cos();
                    noise * envelope
                }
            }
            Instrument::Chimes => {
                // Triangle wave (bright)
                let period = 1.0 / frequency;
                let phase = (t % period) / period;
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            Instrument::Strings => {
                // Sawtooth wave (rich harmonics)
                let period = 1.0 / frequency;
                let phase = (t % period) / period;
                2.0 * phase - 1.0
            }
            Instrument::Synth => {
                // Square wave (electronic)
                if angle.sin() > 0.0 { 1.0 } else { -1.0 }
            }
            _ => angle.sin(),
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
        assert_eq!(generator.quality.sample_rate, 48000);
        assert_eq!(generator.quality.bits_per_sample, 16);
        assert_eq!(generator.quality.channels, 2);
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
}
