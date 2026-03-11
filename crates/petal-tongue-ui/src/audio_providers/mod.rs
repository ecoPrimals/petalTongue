// SPDX-License-Identifier: AGPL-3.0-only
//! DEPRECATED: Legacy Audio Provider System
//!
//! **STATUS**: This module is DEPRECATED as of January 13, 2026
//! **USE INSTEAD**: `crate::audio::AudioSystemV2` (substrate-agnostic)
//!
//! # Migration Path
//!
//! Old (this module):
//! ```ignore
//! use crate::audio_providers::AudioSystem;
//! let audio = AudioSystem::new();
//! ```
//!
//! New (recommended):
//! ```ignore
//! use crate::audio::AudioSystemV2;
//! let audio = AudioSystemV2::new();
//! ```
//!
//! # Why Deprecated
//!
//! This system still had hardcoded assumptions about audio backends.
//! The new `AudioSystemV2` provides TRUE substrate agnosticism:
//! - Runtime discovery of all audio backends
//! - Works on Linux, macOS, Windows, FreeBSD, embedded
//! - Graceful degradation (silent mode)
//! - Zero hardcoding of OS APIs
//!
//! This module will be removed in a future version.
//!
//! # Module Structure
//!
//! - `playback` — shared sample/file playback via Audio Canvas
//! - `pure_rust` — mathematical waveforms (always available)
//! - `user_sound` — WAV/MP3/OGG from user directory
//! - `toadstool` — remote synthesis via Toadstool HTTP API

mod playback;
mod pure_rust;
mod toadstool;
mod user_sound;

use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::status_reporter::{AudioProviderInfo, StatusReporter};

pub use pure_rust::PureRustAudioProvider;
pub use toadstool::ToadstoolAudioProvider;
pub use user_sound::UserSoundProvider;

/// Audio provider interface
pub trait AudioProvider: Send + Sync {
    /// Get name of this provider
    fn name(&self) -> &str;

    /// Check if this provider is available
    fn is_available(&self) -> bool;

    /// Play a sound by name
    fn play(&self, sound_name: &str) -> Result<(), String>;

    /// Stop all sounds
    fn stop(&self);

    /// Get list of available sounds
    fn available_sounds(&self) -> Vec<String>;

    /// Get provider description
    fn description(&self) -> &str;
}

/// Audio system manager - orchestrates all providers
pub struct AudioSystem {
    providers: Vec<Box<dyn AudioProvider>>,
    current_provider: usize,
    status_reporter: Option<Arc<StatusReporter>>,
}

impl AudioSystem {
    /// Create new audio system with all available providers
    ///
    /// **DEPRECATED**: Use `crate::audio::AudioSystemV2` instead
    ///
    /// This method is kept for backward compatibility but will be removed.
    /// The new `AudioSystemV2` provides true substrate agnosticism.
    #[deprecated(
        since = "1.4.0",
        note = "Use `crate::audio::AudioSystemV2` for substrate-agnostic audio"
    )]
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn AudioProvider>> = Vec::new();

        // Add pure Rust provider (always available)
        providers.push(Box::new(PureRustAudioProvider::new()));

        // Add user sound provider if directory exists
        if let Ok(sounds_dir) = std::env::var("PETALTONGUE_SOUNDS_DIR") {
            providers.push(Box::new(UserSoundProvider::new(PathBuf::from(sounds_dir))));
        }

        // Add toadstool provider if available
        providers.push(Box::new(ToadstoolAudioProvider::new()));

        info!(
            "🔊 Audio system initialized with {} providers",
            providers.len()
        );

        Self {
            providers,
            current_provider: 0,
            status_reporter: None,
        }
    }

    /// Set status reporter for AI observability
    pub fn set_status_reporter(&mut self, reporter: Arc<StatusReporter>) {
        info!("🔊 AudioSystem: StatusReporter connected for AI observability");

        // Report initial audio system status
        let provider_info: Vec<AudioProviderInfo> = self
            .providers
            .iter()
            .map(|p| AudioProviderInfo {
                name: p.name().to_string(),
                available: p.is_available(),
                sounds_count: p.available_sounds().len(),
                description: p.description().to_string(),
            })
            .collect();

        let system_players = Self::detect_system_players();

        reporter.update_audio_system(
            self.providers[self.current_provider].name().to_string(),
            provider_info,
            system_players,
        );

        self.status_reporter = Some(reporter);
    }

    /// Detect available system audio players (for diagnostics only)
    ///
    /// NOTE: petalTongue no longer uses external players!
    /// This is kept for diagnostic/informational purposes only.
    fn detect_system_players() -> Vec<String> {
        // Rodio is always available (pure Rust)
        vec!["rodio (pure Rust)".to_string()]
    }

    /// Get list of all providers with their status
    #[must_use]
    pub fn get_providers(&self) -> Vec<(&str, bool, &str)> {
        self.providers
            .iter()
            .map(|p| (p.name(), p.is_available(), p.description()))
            .collect()
    }

    /// Switch to a different audio provider by index
    pub fn set_provider(&mut self, index: usize) {
        if index < self.providers.len() {
            self.current_provider = index;
            info!(
                "🔊 Switched to audio provider: {}",
                self.providers[index].name()
            );
        }
    }

    /// Play audio by name using current provider
    pub fn play(&self, sound_name: &str) -> Result<(), String> {
        info!("🎵 AudioSystem::play('{}') called", sound_name);
        info!(
            "📊 Current provider: {} (index {})",
            self.providers[self.current_provider].name(),
            self.current_provider
        );

        // Try current provider first
        let provider = &self.providers[self.current_provider];
        let provider_name = provider.name().to_string();

        if !provider.is_available() {
            warn!("❌ Current provider '{}' is NOT available!", provider_name);

            // Try to find an available provider with this sound
            for (idx, prov) in self.providers.iter().enumerate() {
                if prov.is_available() && prov.available_sounds().contains(&sound_name.to_string())
                {
                    info!(
                        "✅ Found alternative provider: {} (index {})",
                        prov.name(),
                        idx
                    );
                    let alt_provider_name = prov.name().to_string();
                    match prov.play(sound_name) {
                        Ok(()) => {
                            info!(
                                "✅ Successfully played '{}' with {}",
                                sound_name, alt_provider_name
                            );

                            // Report success to status reporter
                            if let Some(reporter) = &self.status_reporter {
                                reporter.report_audio_event(
                                    sound_name,
                                    &alt_provider_name,
                                    true,
                                    None,
                                );
                            }

                            return Ok(());
                        }
                        Err(e) => {
                            warn!("❌ Failed to play with {}: {}", alt_provider_name, e);
                            // Try next provider (continue is implicit at end of loop)
                        }
                    }
                }
            }

            let err_msg =
                format!("❌ NO PROVIDER CAN PLAY '{sound_name}' - Sound will NOT be heard!");
            error!("{}", err_msg);

            // Report failure to status reporter
            if let Some(reporter) = &self.status_reporter {
                reporter.report_audio_event(
                    sound_name,
                    &provider_name,
                    false,
                    Some(err_msg.clone()),
                );
            }

            return Err(err_msg);
        }

        // Try to play with current provider
        match provider.play(sound_name) {
            Ok(()) => {
                info!(
                    "✅ play() returned Ok for '{}' with {}",
                    sound_name, provider_name
                );

                // Report success to status reporter
                if let Some(reporter) = &self.status_reporter {
                    reporter.report_audio_event(sound_name, &provider_name, true, None);
                }

                Ok(())
            }
            Err(e) => {
                error!("❌ play() returned Err for '{}': {}", sound_name, e);

                // Report failure to status reporter
                if let Some(reporter) = &self.status_reporter {
                    reporter.report_audio_event(sound_name, &provider_name, false, Some(e.clone()));
                }

                Err(e)
            }
        }
    }

    /// Get list of available sounds from current provider
    #[must_use]
    pub fn available_sounds(&self) -> Vec<String> {
        self.providers[self.current_provider].available_sounds()
    }

    /// Get name of current audio provider
    #[must_use]
    pub fn current_provider_name(&self) -> &str {
        self.providers[self.current_provider].name()
    }

    /// Play a continuous tone (for data stream sonification)
    /// frequency: Hz (e.g., 440.0 for A4)
    /// duration: seconds
    /// volume: 0.0-1.0
    /// waveform: sine, square, etc.
    pub fn play_tone(
        &self,
        frequency: f64,
        duration: f64,
        volume: f32,
        waveform: crate::audio_pure_rust::Waveform,
    ) -> Result<(), String> {
        use crate::audio_pure_rust::generate_tone;

        // Generate tone samples (convert f64 to f32)
        let samples = generate_tone(duration as f32, frequency as f32, waveform, volume);

        // Play via system player
        playback::play_samples(&samples, 44100)
    }

    /// Play multiple tones simultaneously (polyphonic)
    /// Each tuple: (frequency, volume, waveform)
    pub fn play_polyphonic(
        &self,
        tones: &[(f64, f32, crate::audio_pure_rust::Waveform)],
        duration: f64,
    ) -> Result<(), String> {
        use crate::audio_pure_rust::generate_tone;

        if tones.is_empty() {
            return Ok(());
        }

        // Generate first tone (convert f64 to f32)
        let mut mixed = generate_tone(duration as f32, tones[0].0 as f32, tones[0].2, tones[0].1);

        // Mix in additional tones
        for &(frequency, volume, waveform) in &tones[1..] {
            let tone = generate_tone(duration as f32, frequency as f32, waveform, volume);

            // Mix by averaging (simple approach)
            for (i, sample) in tone.iter().enumerate() {
                if i < mixed.len() {
                    mixed[i] = (mixed[i] + sample) / 2.0;
                }
            }
        }

        // Play mixed audio
        playback::play_samples(&mixed, 44100)
    }
}

impl Default for AudioSystem {
    #[expect(deprecated)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_rust_provider() {
        let provider = PureRustAudioProvider::new();
        assert!(provider.is_available());
        assert_eq!(provider.name(), "Pure Rust Tones");
        assert!(!provider.available_sounds().is_empty());
    }

    #[test]
    fn test_audio_system() {
        let system = AudioSystem::new();
        assert!(!system.get_providers().is_empty());
        // Pure Rust provider should always be available
        assert!(
            system
                .get_providers()
                .iter()
                .any(|(name, available, _)| { name == &"Pure Rust Tones" && *available })
        );
    }

    #[test]
    fn test_pure_rust_provider_available_sounds() {
        let provider = PureRustAudioProvider::new();
        let sounds = provider.available_sounds();
        assert!(sounds.contains(&"success".to_string()));
        assert!(sounds.contains(&"error".to_string()));
        assert!(sounds.contains(&"click".to_string()));
        assert!(sounds.contains(&"startup".to_string()));
    }

    #[test]
    fn test_audio_system_provider_switching() {
        let mut system = AudioSystem::new();
        let initial_name = system.current_provider_name().to_string();

        system.set_provider(0);
        assert_eq!(system.current_provider_name(), initial_name);

        if system.get_providers().len() > 1 {
            system.set_provider(1);
            assert_ne!(system.current_provider_name(), initial_name);
        }
    }

    #[test]
    fn test_audio_system_available_sounds() {
        let system = AudioSystem::new();
        let sounds = system.available_sounds();
        assert!(!sounds.is_empty());
    }

    #[test]
    fn test_toadstool_provider_availability() {
        // Toadstool is available only when TOADSTOOL_URL is set
        let provider = ToadstoolAudioProvider::new();
        // When env is not set, provider reports not available
        if !provider.is_available() {
            assert!(provider.available_sounds().is_empty());
        }
    }

    #[test]
    fn test_user_sound_provider_scan_empty_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let provider = UserSoundProvider::new(temp.path().to_path_buf());
        assert!(!provider.is_available());
        assert!(provider.available_sounds().is_empty());
    }

    #[test]
    fn test_audio_system_set_provider_out_of_bounds() {
        let mut system = AudioSystem::new();
        let name_before = system.current_provider_name().to_string();
        system.set_provider(999);
        assert_eq!(system.current_provider_name(), name_before);
    }

    #[test]
    fn test_audio_system_set_status_reporter() {
        let mut system = AudioSystem::new();
        let reporter = Arc::new(crate::status_reporter::StatusReporter::new());
        system.set_status_reporter(reporter);
        let providers = system.get_providers();
        assert!(!providers.is_empty());
    }

    #[test]
    fn test_audio_system_play_success_with_pure_rust() {
        let system = AudioSystem::new();
        let result = system.play("success");
        assert!(result.is_ok(), "Pure Rust should play success: {result:?}");
    }

    #[test]
    fn test_audio_system_play_unknown_sound_fails() {
        let system = AudioSystem::new();
        let result = system.play("nonexistent_sound_xyz");
        assert!(result.is_err());
    }
}
