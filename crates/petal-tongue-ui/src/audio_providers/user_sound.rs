// SPDX-License-Identifier: AGPL-3.0-only
//! User sound files provider — loads WAV, MP3, OGG from a directory.

use std::path::PathBuf;

use super::AudioProvider;
use super::playback;
use tracing::{info, warn};

/// User sound files provider (load from disk)
pub struct UserSoundProvider {
    sound_dir: PathBuf,
    sounds: Vec<String>,
}

impl UserSoundProvider {
    /// Create new user sound provider from directory
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
        playback::play_file(&sound_path)?;

        Ok(())
    }

    fn stop(&self) {
        // NOTE: Legacy module - sounds play to completion (most are <1s).
        // Stop control would require storing Sink handles; not implemented in archived code.
        info!("UserSoundProvider: stop() called (sounds complete naturally)");
    }

    fn available_sounds(&self) -> Vec<String> {
        self.sounds.clone()
    }

    fn description(&self) -> &'static str {
        "User-provided sound files (WAV, MP3, OGG) from custom directory."
    }
}
