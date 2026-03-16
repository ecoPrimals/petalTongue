// SPDX-License-Identifier: AGPL-3.0-only
//! Compatibility Layer - `AudioSystemV2`
//!
//! Provides backward-compatible synchronous API over the new substrate-agnostic
//! `AudioManager`. This allows gradual migration of existing code.

use super::manager::AudioManager;
use crate::audio_pure_rust::{SAMPLE_RATE, Waveform, generate_tone};
use crate::error::{AudioError, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// `AudioSystemV2` - Backward-compatible wrapper around `AudioManager`
///
/// Provides the same synchronous API as the old `AudioSystem`,
/// but uses the new substrate-agnostic `AudioManager` internally.
pub struct AudioSystemV2 {
    manager: Arc<Mutex<AudioManager>>,
    runtime: tokio::runtime::Handle,
    /// Kept alive so the handle remains valid when no external runtime exists.
    _owned_runtime: Option<tokio::runtime::Runtime>,
}

impl AudioSystemV2 {
    /// Create new audio system (blocking initialization)
    ///
    /// This is synchronous for backward compatibility, but internally
    /// uses async `AudioManager`.
    pub fn new() -> Result<Self> {
        info!("🎵 Initializing AudioSystemV2 (substrate-agnostic)...");

        let (runtime, owned) = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            (handle, None)
        } else {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| AudioError::TokioRuntimeCreation(e.to_string()))?;
            let handle = rt.handle().clone();
            (handle, Some(rt))
        };

        let manager = runtime
            .block_on(AudioManager::init())
            .map_err(|e| AudioError::AudioManagerInitFailed(e.to_string()))?;

        info!("✅ AudioSystemV2 initialized");

        Ok(Self {
            manager: Arc::new(Mutex::new(manager)),
            runtime,
            _owned_runtime: owned,
        })
    }

    /// Play a tone with given waveform, frequency, and duration
    ///
    /// Synchronous API for backward compatibility.
    pub fn play_tone(&self, waveform: Waveform, frequency: f32, duration_secs: f32) {
        let manager = Arc::clone(&self.manager);

        self.runtime.block_on(async move {
            // Generate tone samples (frequency, duration, waveform, volume)
            let samples = generate_tone(frequency, duration_secs, waveform, 0.3);

            // Play via AudioManager
            let mut mgr = manager.lock().await;
            if let Err(e) = mgr.play_samples(&samples, SAMPLE_RATE).await {
                error!("Failed to play tone: {}", e);
            }
        });
    }

    /// Play audio file (synchronous API)
    ///
    /// Decodes MP3/WAV via symphonia and plays through `AudioManager`.
    pub fn play_file(&self, path: &Path) -> Result<()> {
        info!("🎵 Playing file: {}", path.display());

        use symphonia::core::audio::{AudioBufferRef, Signal};
        use symphonia::core::codecs::DecoderOptions;
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let mut format = probed.format;
        let track = format.default_track().ok_or(AudioError::NoAudioTrack)?;
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let track_id = track.id;

        let mut all_samples: Vec<f32> = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => return Err(e.into()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            let frame = decoder.decode(&packet)?;

            match frame {
                AudioBufferRef::F32(buf) => {
                    for &s in buf.chan(0) {
                        all_samples.push(s);
                    }
                }
                AudioBufferRef::S16(buf) => {
                    for &s in buf.chan(0) {
                        all_samples.push(f32::from(s) / 32768.0);
                    }
                }
                AudioBufferRef::S32(buf) => {
                    for &s in buf.chan(0) {
                        all_samples.push(s as f32 / 2_147_483_648.0);
                    }
                }
                _ => {}
            }
        }

        self.play_samples(&all_samples, sample_rate);
        Ok(())
    }

    /// Play raw samples (synchronous API)
    pub fn play_samples(&self, samples: &[f32], sample_rate: u32) {
        let manager = Arc::clone(&self.manager);
        let samples = samples.to_vec();

        self.runtime.block_on(async move {
            let mut mgr = manager.lock().await;
            if let Err(e) = mgr.play_samples(&samples, sample_rate).await {
                error!("Failed to play samples: {}", e);
            }
        });
    }

    /// Get active backend name (for display)
    #[must_use]
    pub fn active_backend(&self) -> Option<String> {
        self.runtime.block_on(async {
            let mgr = self.manager.lock().await;
            mgr.active_backend_metadata().map(|meta| meta.name)
        })
    }

    /// Get all available backends (for diagnostics)
    #[must_use]
    pub fn available_backends(&self) -> Vec<String> {
        self.runtime.block_on(async {
            let mgr = self.manager.lock().await;
            mgr.available_backends()
                .into_iter()
                .map(|meta| format!("{} ({:?})", meta.name, meta.backend_type))
                .collect()
        })
    }

    /// Play a sound by name (compatibility method)
    ///
    /// This is a simplified method for backward compatibility.
    /// In the new system, we just play a simple tone.
    pub fn play(&self, _sound_name: &str) {
        // Play a simple notification tone for compatibility
        self.play_tone(Waveform::Sine, 440.0, 0.1);
    }

    /// Play polyphonic tones (compatibility method)
    ///
    /// Simplified version for backward compatibility.
    /// Each tuple: (frequency, volume, waveform)
    pub fn play_polyphonic(&self, tones: &[(f64, f32, Waveform)], duration: f64) -> Result<()> {
        // For compatibility, just play the first tone
        if let Some((freq, _vol, wave)) = tones.first() {
            self.play_tone(*wave, *freq as f32, duration as f32);
        }
        Ok(())
    }
}

impl Default for AudioSystemV2 {
    fn default() -> Self {
        Self::new().expect("AudioSystemV2::default() requires working audio init (test-only)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_system_v2_creation() {
        // Should not panic
        let audio = AudioSystemV2::new().expect("test requires working audio");

        // Should have at least silent backend
        let backends = audio.available_backends();
        assert!(!backends.is_empty(), "Should have at least one backend");

        println!("Available backends:");
        for backend in &backends {
            println!("  - {backend}");
        }
    }

    #[test]
    fn test_audio_system_v2_play_tone() {
        let audio = AudioSystemV2::new().expect("test requires working audio");

        // Should not panic (may be silent)
        audio.play_tone(Waveform::Sine, 440.0, 0.1);
    }

    #[test]
    fn test_play_polyphonic_empty_tones() {
        let audio = AudioSystemV2::new().expect("test requires working audio");
        let result = audio.play_polyphonic(&[], 0.1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_polyphonic_single_tone() {
        let audio = AudioSystemV2::new().expect("test requires working audio");
        let tones = [(440.0, 0.5, Waveform::Sine)];
        let result = audio.play_polyphonic(&tones, 0.1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_sound_by_name() {
        let audio = AudioSystemV2::new().expect("test requires working audio");
        audio.play("notification");
    }

    #[test]
    fn test_available_backends_format() {
        let audio = AudioSystemV2::new().expect("test requires working audio");
        let backends = audio.available_backends();
        for backend in &backends {
            assert!(!backend.is_empty());
            assert!(backend.contains('(') || backend.contains("Silent"));
        }
    }

    #[test]
    fn test_waveform_variants() {
        assert_eq!(Waveform::Sine, Waveform::Sine);
        assert_ne!(Waveform::Sine, Waveform::Square);
    }
}
