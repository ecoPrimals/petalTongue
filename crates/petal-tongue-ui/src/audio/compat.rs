//! Compatibility Layer - AudioSystemV2
//!
//! Provides backward-compatible synchronous API over the new substrate-agnostic
//! AudioManager. This allows gradual migration of existing code.

use super::manager::AudioManager;
use crate::audio_pure_rust::{SAMPLE_RATE, Waveform, generate_tone};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// AudioSystemV2 - Backward-compatible wrapper around AudioManager
///
/// Provides the same synchronous API as the old AudioSystem,
/// but uses the new substrate-agnostic AudioManager internally.
pub struct AudioSystemV2 {
    manager: Arc<Mutex<AudioManager>>,
    runtime: tokio::runtime::Handle,
}

impl AudioSystemV2 {
    /// Create new audio system (blocking initialization)
    ///
    /// This is synchronous for backward compatibility, but internally
    /// uses async AudioManager.
    pub fn new() -> Self {
        info!("🎵 Initializing AudioSystemV2 (substrate-agnostic)...");

        // Get or create tokio runtime
        let runtime = tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
            // Create a new runtime if not in async context
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime")
                .handle()
                .clone()
        });

        // Initialize AudioManager
        let manager = runtime.block_on(async {
            AudioManager::init()
                .await
                .expect("Failed to initialize AudioManager")
        });

        info!("✅ AudioSystemV2 initialized");

        Self {
            manager: Arc::new(Mutex::new(manager)),
            runtime,
        }
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
    /// TODO: Implement file loading and playback
    pub fn play_file(&self, path: &Path) -> Result<()> {
        info!("🎵 Playing file: {}", path.display());

        // TODO: Load audio file using symphonia
        // TODO: Convert to samples
        // TODO: Play via AudioManager

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
    pub fn active_backend(&self) -> Option<String> {
        self.runtime.block_on(async {
            let mgr = self.manager.lock().await;
            mgr.active_backend_metadata().map(|meta| meta.name)
        })
    }

    /// Get all available backends (for diagnostics)
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
        if let Some((freq, vol, wave)) = tones.first() {
            self.play_tone(*wave, *freq as f32, duration as f32);
        }
        Ok(())
    }
}

impl Default for AudioSystemV2 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_system_v2_creation() {
        // Should not panic
        let audio = AudioSystemV2::new();

        // Should have at least silent backend
        let backends = audio.available_backends();
        assert!(!backends.is_empty(), "Should have at least one backend");

        println!("Available backends:");
        for backend in backends {
            println!("  - {}", backend);
        }
    }

    #[test]
    fn test_audio_system_v2_play_tone() {
        let audio = AudioSystemV2::new();

        // Should not panic (may be silent)
        audio.play_tone(Waveform::Sine, 440.0, 0.1);
    }
}
