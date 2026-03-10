// SPDX-License-Identifier: AGPL-3.0-only
//! Pure Rust audio provider — mathematical waveforms, no external dependencies.

use super::AudioProvider;
use super::playback;
use tracing::{info, warn};

/// Pure Rust audio provider (always available, no dependencies)
pub struct PureRustAudioProvider {
    enabled: bool,
}

impl PureRustAudioProvider {
    /// Create new pure Rust audio provider
    pub fn new() -> Self {
        info!("🔊 Pure Rust audio provider initialized (always available)");
        Self { enabled: true }
    }
}

impl Default for PureRustAudioProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProvider for PureRustAudioProvider {
    fn name(&self) -> &'static str {
        "Pure Rust Tones"
    }

    fn is_available(&self) -> bool {
        self.enabled
    }

    fn play(&self, sound_name: &str) -> Result<(), String> {
        use crate::audio_pure_rust::{SAMPLE_RATE, UISounds};

        info!("🔊 Playing pure Rust sound: {}", sound_name);

        // Generate samples based on sound name
        let samples = match sound_name {
            "success" => UISounds::success(),
            "error" => UISounds::error(),
            "click" => UISounds::click(),
            "notification" => UISounds::notification(),
            "primal_discovered" => UISounds::primal_discovered(),
            "data_refresh" => UISounds::data_refresh(),
            "warning" => UISounds::warning(),
            "connected" => UISounds::connected(),
            "startup" => UISounds::startup(),
            _ => {
                return Err(format!("Unknown sound: {sound_name}"));
            }
        };

        info!("✓ Generated {} samples for '{}'", samples.len(), sound_name);

        // Actually play the audio using rodio
        if let Err(e) = playback::play_samples(&samples, SAMPLE_RATE) {
            warn!("Failed to play audio: {}", e);
            // Still return Ok - audio generation worked even if playback failed
        }

        Ok(())
    }

    fn stop(&self) {
        // No-op for pure Rust provider
    }

    fn available_sounds(&self) -> Vec<String> {
        vec![
            "success".to_string(),
            "error".to_string(),
            "click".to_string(),
            "notification".to_string(),
            "primal_discovered".to_string(),
            "data_refresh".to_string(),
            "warning".to_string(),
            "connected".to_string(),
            "startup".to_string(),
        ]
    }

    fn description(&self) -> &'static str {
        "Pure Rust mathematical waveforms. No external dependencies. Always available."
    }
}
