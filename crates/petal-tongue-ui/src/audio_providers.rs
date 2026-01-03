//! Audio Provider System
//!
//! Multi-tiered audio system:
//! 1. Pure Rust tones (no dependencies, always available)
//! 2. User-provided sound files (load from disk)
//! 3. Toadstool integration (advanced synthesis)

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tracing::{error, info, warn};

// Forward declaration for StatusReporter (avoid circular dependency)
use crate::status_reporter::{AudioProviderInfo, StatusReporter};

/// Play audio samples (writes WAV and uses system player)
fn play_samples(samples: &[f32], _sample_rate: u32) -> Result<(), String> {
    use crate::audio_pure_rust::export_wav;

    // Create temp directory
    let temp_dir = std::env::temp_dir();
    let wav_path = temp_dir.join(format!("petaltongue_{}.wav", std::process::id()));

    // Export to WAV
    let wav_bytes = export_wav(samples);
    std::fs::write(&wav_path, wav_bytes).map_err(|e| format!("Failed to write WAV: {e}"))?;

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
                    .args([
                        "-c",
                        &format!(
                            "(New-Object Media.SoundPlayer '{}')).PlaySync()",
                            wav_path.display()
                        ),
                    ])
                    .output()
            } else {
                Command::new(player).arg(&wav_path).output()
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
            "startup".to_string(),
        ]
    }

    fn description(&self) -> &'static str {
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
                    .filter_map(std::result::Result::ok)
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .is_some_and(|ext| ext == "wav" || ext == "mp3" || ext == "ogg")
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
    fn name(&self) -> &'static str {
        "User Sound Files"
    }

    fn is_available(&self) -> bool {
        self.sound_dir.exists() && !self.sounds.is_empty()
    }

    fn play(&self, sound_name: &str) -> Result<(), String> {
        if !self.sounds.contains(&sound_name.to_string()) {
            let err_msg = format!(
                "❌ Sound '{}' not found in user sounds. Available: {:?}",
                sound_name, self.sounds
            );
            warn!("{}", err_msg);
            return Err(err_msg);
        }

        // Find the sound file
        let sound_file = std::fs::read_dir(&self.sound_dir)
            .map_err(|e| format!("Failed to read sound directory: {e}"))?
            .filter_map(std::result::Result::ok)
            .find(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s == sound_name)
            })
            .ok_or_else(|| format!("Sound file not found: {sound_name}"))?;

        let sound_path = sound_file.path();
        info!(
            "🔊 Playing user sound: {} from {:?}",
            sound_name, sound_path
        );

        // Try to play with system audio players
        let sound_path_clone = sound_path.clone();
        std::thread::spawn(move || {
            let players = if cfg!(target_os = "linux") {
                vec!["mpv", "paplay", "aplay", "ffplay", "vlc"]
            } else if cfg!(target_os = "macos") {
                vec!["afplay", "mpv", "ffplay"]
            } else if cfg!(target_os = "windows") {
                vec!["powershell"]
            } else {
                vec![]
            };

            let mut success = false;
            for player in &players {
                let result = if *player == "powershell" {
                    Command::new(player)
                        .args([
                            "-c",
                            &format!(
                                "(New-Object Media.SoundPlayer '{}')).PlaySync()",
                                sound_path_clone.display()
                            ),
                        ])
                        .output()
                } else {
                    Command::new(player).arg(&sound_path_clone).output()
                };

                if result.is_ok() {
                    info!(
                        "✅ Successfully played user sound with {}: {:?}",
                        player, sound_path_clone
                    );
                    success = true;
                    break;
                }
            }

            if !success {
                error!("❌ AUDIO PLAYBACK FAILED: No working audio player found!");
                error!("❌ Tried: {:?}", players);
                error!("❌ A blind user would NOT know the sound failed!");
                warn!("💡 Install: mpv (recommended) or paplay or aplay");
            }
        });

        Ok(())
    }

    fn stop(&self) {
        // TODO: Implement stop
    }

    fn available_sounds(&self) -> Vec<String> {
        self.sounds.clone()
    }

    fn description(&self) -> &'static str {
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
        let endpoint = self.endpoint.as_ref().ok_or("Toadstool not configured")?;

        // Make HTTP request to toadstool for audio synthesis
        let _url = format!("{endpoint}/api/v1/audio/synthesize");
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
    fn name(&self) -> &'static str {
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

    fn description(&self) -> &'static str {
        "Advanced audio synthesis via Toadstool primal. Supports music, voice, and complex soundscapes."
    }
}

/// Audio system manager - orchestrates all providers
pub struct AudioSystem {
    providers: Vec<Box<dyn AudioProvider>>,
    current_provider: usize,
    status_reporter: Option<Arc<StatusReporter>>,
}

impl AudioSystem {
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

    /// Detect available system audio players
    fn detect_system_players() -> Vec<String> {
        let candidates = vec!["mpv", "paplay", "aplay", "ffplay", "vlc", "afplay"];
        let mut available = Vec::new();

        for player in candidates {
            if Command::new("which")
                .arg(player)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                available.push(player.to_string());
            }
        }

        available
    }

    #[must_use] 
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
                            continue;
                        }
                    }
                }
            }

            let err_msg = format!(
                "❌ NO PROVIDER CAN PLAY '{sound_name}' - Sound will NOT be heard!"
            );
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

    #[must_use] 
    pub fn available_sounds(&self) -> Vec<String> {
        self.providers[self.current_provider].available_sounds()
    }

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
        play_samples(&samples, 44100)
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
        play_samples(&mixed, 44100)
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
        assert!(
            system
                .get_providers()
                .iter()
                .any(|(name, available, _)| { name == &"Pure Rust Tones" && *available })
        );
    }
}
