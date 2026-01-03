//! Audio Provider System
//!
//! Multi-tiered audio system:
//! 1. Pure Rust tones (no dependencies, always available)
//! 2. User-provided sound files (load from disk)
//! 3. Toadstool integration (advanced synthesis)

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::process::Command;
use tracing::{info, warn};

/// Play audio samples (writes WAV and uses system player)
fn play_samples(samples: &[f32], sample_rate: u32) -> Result<(), String> {
    use crate::audio_pure_rust::export_wav;

    // Create temp directory
    let temp_dir = std::env::temp_dir();
    let wav_path = temp_dir.join(format!("petaltongue_{}.wav", std::process::id()));

    // Export to WAV
    let wav_bytes = export_wav(samples);
    std::fs::write(&wav_path, wav_bytes)
        .map_err(|e| format!("Failed to write WAV: {}", e))?;

    info!("💾 Saved audio to {:?}", wav_path);

    // Try to play with system command (non-blocking)
    std::thread::spawn(move || {
        // Try different players based on platform
        let players = if cfg!(target_os = "linux") {
            vec!["aplay", "paplay", "ffplay", "mpv", "vlc"]
        } else if cfg!(target_os = "macos") {
            vec!["afplay", "ffplay", "mpv"]
        } else if cfg!(target_os = "windows") {
            vec!["powershell"]
        } else {
            vec![]
        };

        for player in players {
            let result = if player == "powershell" {
                Command::new(player)
                    .args(&["-c", &format!("(New-Object Media.SoundPlayer '{}')).PlaySync()", wav_path.display())])
                    .output()
            } else {
                Command::new(player)
                    .arg(&wav_path)
                    .output()
            };

            if result.is_ok() {
                info!("🔊 Playing with {}", player);
                // Clean up after playing
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = std::fs::remove_file(&wav_path);
                return;
            }
        }

        warn!("⚠️  No audio player found. WAV saved to {:?}", wav_path);
        warn!("💡 Install: aplay (ALSA) or paplay (PulseAudio) or mpv");
    });

    Ok(())
}

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

/// Pure Rust audio provider (always available, no dependencies)
pub struct PureRustAudioProvider {
    enabled: bool,
}

impl PureRustAudioProvider {
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
    fn name(&self) -> &str {
        "Pure Rust Tones"
    }

    fn is_available(&self) -> bool {
        self.enabled
    }

    fn play(&self, sound_name: &str) -> Result<(), String> {
        use crate::audio_pure_rust::{export_wav, UISounds, SAMPLE_RATE};

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
            _ => {
                return Err(format!("Unknown sound: {}", sound_name));
            }
        };

        info!("✓ Generated {} samples for '{}'", samples.len(), sound_name);

        // Actually play the audio using rodio
        if let Err(e) = play_samples(&samples, SAMPLE_RATE) {
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
        ]
    }

    fn description(&self) -> &str {
        "Pure Rust mathematical waveforms. No external dependencies. Always available."
    }
}

/// User sound files provider (load from disk)
pub struct UserSoundProvider {
    sound_dir: PathBuf,
    sounds: Vec<String>,
}

impl UserSoundProvider {
    pub fn new(sound_dir: PathBuf) -> Self {
        let sounds = Self::scan_sound_directory(&sound_dir);
        info!(
            "🔊 User sound provider initialized with {} sounds from {:?}",
            sounds.len(),
            sound_dir
        );
        Self { sound_dir, sounds }
    }

    fn scan_sound_directory(dir: &PathBuf) -> Vec<String> {
        if !dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.path().extension().map_or(false, |ext| {
                            ext == "wav" || ext == "mp3" || ext == "ogg"
                        })
                    })
                    .filter_map(|entry| {
                        entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl AudioProvider for UserSoundProvider {
    fn name(&self) -> &str {
        "User Sound Files"
    }

    fn is_available(&self) -> bool {
        self.sound_dir.exists() && !self.sounds.is_empty()
    }

    fn play(&self, sound_name: &str) -> Result<(), String> {
        if !self.sounds.contains(&sound_name.to_string()) {
            return Err(format!("Sound not found: {}", sound_name));
        }

        info!("🔊 Playing user sound: {}", sound_name);
        // TODO: Implement actual playback
        Ok(())
    }

    fn stop(&self) {
        // TODO: Implement stop
    }

    fn available_sounds(&self) -> Vec<String> {
        self.sounds.clone()
    }

    fn description(&self) -> &str {
        "User-provided sound files (WAV, MP3, OGG) from custom directory."
    }
}

/// Toadstool audio provider (advanced synthesis via primal)
pub struct ToadstoolAudioProvider {
    endpoint: Option<String>,
    available: bool,
}

impl ToadstoolAudioProvider {
    pub fn new() -> Self {
        // Check for toadstool via environment or discovery
        let endpoint = std::env::var("TOADSTOOL_URL").ok();
        let available = endpoint.is_some();

        if available {
            info!("🔊 Toadstool audio provider initialized: {:?}", endpoint);
        } else {
            info!("🔊 Toadstool audio provider not available (set TOADSTOOL_URL)");
        }

        Self {
            endpoint,
            available,
        }
    }

    async fn request_synthesis(&self, params: &str) -> Result<Vec<u8>, String> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or("Toadstool not configured")?;

        // Make HTTP request to toadstool for audio synthesis
        let url = format!("{}/api/v1/audio/synthesize", endpoint);
        info!("🔊 Requesting audio synthesis from toadstool: {}", params);

        // TODO: Implement actual HTTP request
        Ok(Vec::new())
    }
}

impl Default for ToadstoolAudioProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProvider for ToadstoolAudioProvider {
    fn name(&self) -> &str {
        "Toadstool Synthesis"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn play(&self, sound_name: &str) -> Result<(), String> {
        if !self.available {
            return Err("Toadstool not available".to_string());
        }

        info!("🔊 Requesting toadstool synthesis: {}", sound_name);
        // TODO: Make async request to toadstool
        Ok(())
    }

    fn stop(&self) {
        // TODO: Send stop command to toadstool
    }

    fn available_sounds(&self) -> Vec<String> {
        if self.available {
            vec![
                "music".to_string(),
                "voice".to_string(),
                "soundscape".to_string(),
                "ambient".to_string(),
                "rhythm".to_string(),
            ]
        } else {
            Vec::new()
        }
    }

    fn description(&self) -> &str {
        "Advanced audio synthesis via Toadstool primal. Supports music, voice, and complex soundscapes."
    }
}

/// Audio system manager - orchestrates all providers
pub struct AudioSystem {
    providers: Vec<Box<dyn AudioProvider>>,
    current_provider: usize,
}

impl AudioSystem {
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn AudioProvider>> = Vec::new();

        // Add pure Rust provider (always available)
        providers.push(Box::new(PureRustAudioProvider::new()));

        // Add user sound provider if directory exists
        if let Ok(sounds_dir) = std::env::var("PETALTONGUE_SOUNDS_DIR") {
            providers.push(Box::new(UserSoundProvider::new(PathBuf::from(
                sounds_dir,
            ))));
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
        }
    }

    pub fn get_providers(&self) -> Vec<(&str, bool, &str)> {
        self.providers
            .iter()
            .map(|p| (p.name(), p.is_available(), p.description()))
            .collect()
    }

    pub fn set_provider(&mut self, index: usize) {
        if index < self.providers.len() {
            self.current_provider = index;
            info!(
                "🔊 Switched to audio provider: {}",
                self.providers[index].name()
            );
        }
    }

    pub fn play(&self, sound_name: &str) -> Result<(), String> {
        let provider = &self.providers[self.current_provider];
        if !provider.is_available() {
            return Err(format!("Provider '{}' not available", provider.name()));
        }
        provider.play(sound_name)
    }

    pub fn available_sounds(&self) -> Vec<String> {
        self.providers[self.current_provider].available_sounds()
    }

    pub fn current_provider_name(&self) -> &str {
        self.providers[self.current_provider].name()
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
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
        assert!(system.get_providers().iter().any(|(name, available, _)| {
            name == &"Pure Rust Tones" && *available
        }));
    }
}

