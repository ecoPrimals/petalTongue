//! Audio Provider System
//!
//! TRUE PRIMAL Multi-tiered audio system (Pure Rust):
//! 1. Pure Rust tones (rodio, always available)
//! 2. User-provided sound files (rodio + symphonia decoder)
//! 3. Toadstool integration (advanced synthesis via primal network)
//!
//! # Sovereignty
//!
//! **EVOLVED**: 100% Pure Rust audio stack!
//! - Tier 1 (Self-Stable): rodio + symphonia
//! - Tier 2 (Network): Toadstool primal (optional)
//! - Tier 3 (Extensions): REMOVED - no external dependencies

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info, warn};

// Forward declaration for StatusReporter (avoid circular dependency)
use crate::status_reporter::{AudioProviderInfo, StatusReporter};

/// Play audio samples using Audio Canvas (direct hardware!)
///
/// EVOLVED: Like WGPU for graphics - direct device access!
fn play_samples(samples: &[f32], _sample_rate: u32) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;

    info!(
        "🎨 Playing {} samples via Audio Canvas (100% pure Rust!)",
        samples.len()
    );

    // Open audio canvas (direct hardware access!)
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {}", e))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(samples)
        .map_err(|e| format!("Failed to write samples: {}", e))?;

    info!("✅ Audio playback complete (Audio Canvas)");

    Ok(())
}

/// Play audio file using Audio Canvas + symphonia (100% pure Rust!)
fn play_file(path: &Path) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;
    use std::fs;

    info!("🎨 Playing audio file: {} (Audio Canvas)", path.display());

    // Read file
    let data = fs::read(path).map_err(|e| format!("Failed to read audio file: {}", e))?;

    // Decode with symphonia (pure Rust!)
    let decoded = crate::startup_audio::decode_audio_symphonia(&data)
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    // Open audio canvas
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {}", e))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(&decoded.samples)
        .map_err(|e| format!("Failed to write samples: {}", e))?;

    info!("✅ Audio file playback complete (Audio Canvas)");

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
            "🔊 Playing user sound: {} from {:?} (pure Rust)",
            sound_name, sound_path
        );

        // EVOLVED: Play with pure Rust (rodio + symphonia)
        play_file(&sound_path)?;

        Ok(())
    }

    fn stop(&self) {
        // TODO: Track Sink handles for proper stop() control
        // With rodio, we can implement proper stop by:
        // 1. Store Arc<Mutex<Vec<Sink>>> in provider
        // 2. Call sink.stop() on all active sinks
        //
        // For now, sounds play to completion (most are <1s)
        info!("UserSoundProvider: stop() called (sounds complete naturally)");
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

    /// Request audio synthesis from Toadstool
    async fn request_synthesis(&self, params: &str) -> Result<Vec<u8>, String> {
        let endpoint = self.endpoint.as_ref().ok_or("Toadstool not configured")?;

        let url = format!("{endpoint}/api/v1/audio/synthesize");
        info!("🔊 Requesting audio synthesis from Toadstool: {}", params);

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        #[derive(serde::Serialize)]
        struct SynthesisRequest {
            params: String,
            format: String,
        }

        let request = SynthesisRequest {
            params: params.to_string(),
            format: "wav".to_string(),
        };

        // Make async HTTP POST request
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Toadstool returned error: {} ({})",
                response.status(),
                url
            ));
        }

        // Get audio bytes
        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read audio data: {}", e))?;

        info!(
            "✅ Received {} bytes of audio from Toadstool",
            audio_bytes.len()
        );
        Ok(audio_bytes.to_vec())
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

        let endpoint = self.endpoint.clone();
        let sound = sound_name.to_string();

        info!("🔊 Requesting Toadstool synthesis: {}", sound_name);

        // Spawn async task to request synthesis
        tokio::spawn(async move {
            if let Some(ep) = endpoint {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build();

                if let Ok(client) = client {
                    let url = format!("{}/api/v1/audio/play", ep);

                    #[derive(serde::Serialize)]
                    struct PlayRequest {
                        sound: String,
                    }

                    let request = PlayRequest { sound };

                    match client
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .json(&request)
                        .send()
                        .await
                    {
                        Ok(response) if response.status().is_success() => {
                            info!("✅ Toadstool playing sound");
                        }
                        Ok(response) => {
                            warn!("⚠️ Toadstool returned error: {}", response.status());
                        }
                        Err(e) => {
                            warn!("❌ Failed to request playback: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn stop(&self) {
        if !self.available {
            return;
        }

        let endpoint = self.endpoint.clone();

        info!("🛑 Sending stop command to Toadstool");

        // Spawn async task to send stop command
        tokio::spawn(async move {
            if let Some(ep) = endpoint {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build();

                if let Ok(client) = client {
                    let url = format!("{}/api/v1/audio/stop", ep);

                    match client.post(&url).send().await {
                        Ok(response) if response.status().is_success() => {
                            info!("✅ Toadstool stopped playback");
                        }
                        Ok(response) => {
                            warn!("⚠️ Toadstool stop returned: {}", response.status());
                        }
                        Err(e) => {
                            warn!("❌ Failed to send stop command: {}", e);
                        }
                    }
                }
            }
        });
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

    /// Detect available system audio players (for diagnostics only)
    ///
    /// NOTE: petalTongue no longer uses external players!
    /// This is kept for diagnostic/informational purposes only.
    fn detect_system_players() -> Vec<String> {
        // Rodio is always available (pure Rust)
        vec!["rodio (pure Rust)".to_string()]
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
