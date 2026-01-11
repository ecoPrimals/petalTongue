//! Startup Audio System
//!
//! Plays signature audio tone followed by startup music when petalTongue launches.
//!
//! Architecture (TRUE PRIMAL - Pure Rust):
//! 1. Signature Tone: Pure Rust generation (rodio, always works)
//! 2. Startup Music: Embedded MP3 decoded with pure Rust (symphonia)
//! 3. Fallback: External file if embedded not available
//!
//! # Sovereignty
//!
//! **EVOLVED**: Now uses 100% pure Rust audio stack!
//! - rodio: Cross-platform audio playback
//! - symphonia: Pure Rust MP3/WAV decoder
//! - NO external dependencies (mpv, ffplay, aplay, etc.)
//! - Self-stable operation guaranteed

use crate::audio_providers::AudioSystem;
use crate::audio_pure_rust::{SAMPLE_RATE, Waveform, generate_tone};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Play audio samples using pure Rust (rodio)
///
/// This is the TRUE PRIMAL audio playback - no external dependencies!
fn play_audio_pure_rust(samples: &[f32]) -> Result<(), String> {
    use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
    
    // Get default output device
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to get audio output device: {}", e))?;
    
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;
    
    // Create audio source from samples
    let source = SamplesBuffer::new(1, SAMPLE_RATE, samples.to_vec());
    
    sink.append(source);
    sink.sleep_until_end();
    
    Ok(())
}

/// Play embedded MP3 using pure Rust (rodio + symphonia)
fn play_embedded_mp3_pure_rust(mp3_data: &[u8]) -> Result<(), String> {
    use rodio::{Decoder, OutputStream, Sink};
    
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to get audio output device: {}", e))?;
    
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;
    
    // Decode MP3 from memory (pure Rust - symphonia!)
    let cursor = Cursor::new(mp3_data);
    let source = Decoder::new(cursor)
        .map_err(|e| format!("Failed to decode MP3: {}", e))?;
    
    sink.append(source);
    sink.sleep_until_end();
    
    Ok(())
}

/// Play audio file using pure Rust (rodio + symphonia)
fn play_file_pure_rust(path: &Path) -> Result<(), String> {
    use rodio::{Decoder, OutputStream, Sink};
    use std::fs::File;
    
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to get audio output device: {}", e))?;
    
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;
    
    // Open and decode file (pure Rust!)
    let file = File::open(path)
        .map_err(|e| format!("Failed to open audio file: {}", e))?;
    
    let source = Decoder::new(file)
        .map_err(|e| format!("Failed to decode audio file: {}", e))?;
    
    sink.append(source);
    sink.sleep_until_end();
    
    Ok(())
}

/// Embedded startup music (11MB MP3)
///
/// This is "Welcome Home Morning Star - Godking.mp3" embedded at compile time.
/// Makes petalTongue completely self-contained!
const EMBEDDED_STARTUP_MUSIC: &[u8] = include_bytes!("../assets/startup_music.mp3");

/// Startup audio configuration
pub struct StartupAudio {
    /// Path to startup music file (optional, for external override)
    startup_music_path: Option<PathBuf>,
    /// Whether to play signature tone
    play_signature: bool,
    /// Whether to play startup music
    play_music: bool,
    /// Use embedded music (default: true)
    use_embedded: bool,
}

impl Default for StartupAudio {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupAudio {
    /// Create new startup audio configuration
    #[must_use]
    pub fn new() -> Self {
        // Try to find external startup music (optional override)
        let startup_music_path = Self::find_startup_music();

        // Check if user wants to disable embedded music
        let use_embedded = std::env::var("PETALTONGUE_DISABLE_EMBEDDED_MUSIC")
            .map(|v| v != "1" && v.to_lowercase() != "true")
            .unwrap_or(true);

        if use_embedded {
            info!("🎵 Using embedded startup music (self-contained)");
        } else if startup_music_path.is_some() {
            info!("🎵 Using external startup music (environment override)");
        }

        Self {
            startup_music_path,
            play_signature: true,
            play_music: true,
            use_embedded,
        }
    }

    /// Find external startup music file (optional override)
    fn find_startup_music() -> Option<PathBuf> {
        // Only check environment variable for explicit override
        if let Ok(env_path) = std::env::var("PETALTONGUE_STARTUP_MUSIC") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                info!(
                    "🎵 Found external startup music override: {}",
                    path.display()
                );
                return Some(path);
            } else {
                warn!(
                    "🎵 PETALTONGUE_STARTUP_MUSIC set but file not found: {}",
                    path.display()
                );
            }
        }

        None
    }

    /// Get embedded startup music data
    #[must_use]
    pub fn get_embedded_music() -> &'static [u8] {
        EMBEDDED_STARTUP_MUSIC
    }

    /// Check if embedded music is available
    #[must_use]
    pub fn has_embedded_music() -> bool {
        !EMBEDDED_STARTUP_MUSIC.is_empty()
    }

    /// Generate signature tone
    ///
    /// Creates the petalTongue signature audio - a distinctive sound that identifies
    /// the application. Currently uses a pleasant chord progression.
    ///
    /// TODO: Design distinctive petalTongue signature sound
    #[must_use]
    pub fn generate_signature_tone() -> Vec<f32> {
        info!("🎵 Generating petalTongue signature tone...");

        // Signature: Ascending chord (C-E-G) with harmonic overtones
        // This creates a welcoming, flourishing sound
        let mut signature = Vec::new();

        // Note frequencies (C major chord)
        let notes = [
            261.63, // C4
            329.63, // E4
            392.00, // G4
        ];

        // Generate each note of the chord
        for (i, &freq) in notes.iter().enumerate() {
            let delay = i as f32 * 0.1; // Stagger notes slightly
            let duration = 0.5; // Each note lasts 0.5 seconds

            // Generate sine wave for this note
            let tone = generate_tone(freq, duration, Waveform::Sine, 0.3);

            // Add silence before this note (for stagger effect)
            let delay_samples = (SAMPLE_RATE as f32 * delay) as usize;
            signature.resize(signature.len().max(delay_samples), 0.0);

            // Mix this note into the signature
            for (j, &sample) in tone.iter().enumerate() {
                let index = delay_samples + j;
                if index >= signature.len() {
                    signature.push(sample);
                } else {
                    signature[index] += sample;
                }
            }
        }

        // Add harmonic overtone (octave up for sparkle)
        let harmonic = generate_tone(523.25, 0.4, Waveform::Triangle, 0.15); // C5
        for (j, &sample) in harmonic.iter().enumerate() {
            if j < signature.len() {
                signature[j] += sample;
            }
        }

        // Normalize to prevent clipping
        let max_amplitude = signature.iter().fold(0.0_f32, |max, &s| max.max(s.abs()));
        if max_amplitude > 0.0 {
            for sample in &mut signature {
                *sample /= max_amplitude;
                *sample *= 0.7; // Leave headroom
            }
        }

        info!("✨ Signature tone generated: {} samples", signature.len());
        signature
    }

    /// Play startup audio sequence
    ///
    /// Plays signature tone followed by startup music (if available).
    /// Non-blocking - spawns background thread.
    pub fn play(&self, _audio_system: &AudioSystem) {
        if !self.play_signature && !self.play_music {
            return;
        }

        info!("🎵 Starting petalTongue startup audio...");

        let play_signature = self.play_signature;
        let play_music = self.play_music;
        let music_path = self.startup_music_path.clone();

        // Spawn background thread for audio playback
        std::thread::spawn(move || {
            // 1. Play signature tone (pure Rust, always works)
            if play_signature {
                let signature = Self::generate_signature_tone();
                // Export to WAV and play using system
                use crate::audio_pure_rust::export_wav;

                let temp_dir = std::env::temp_dir();
                let wav_path = temp_dir.join("petaltongue_signature.wav");
                let wav_bytes = export_wav(&signature);

                // EVOLVED: Pure Rust audio playback (no external dependencies!)
                match play_audio_pure_rust(&signature) {
                    Ok(()) => {
                        info!("✅ Signature tone played with pure Rust (rodio)");
                    }
                    Err(e) => {
                        warn!("⚠️ Pure Rust audio failed: {}, writing to disk as fallback", e);
                        
                        // Fallback: Write to disk for manual playback
                        if let Err(e) = std::fs::write(&wav_path, wav_bytes) {
                            warn!("Failed to write signature tone: {}", e);
                        } else {
                            info!("🎵 Signature tone exported to {:?}", wav_path);
                        }
                    }
                }
            }

            // 2. Play startup music (embedded or external)
            if play_music {
                if Self::has_embedded_music() {
                    info!("🎵 Playing embedded startup music (pure Rust)...");
                    
                    // EVOLVED: Play embedded MP3 with pure Rust!
                    match play_embedded_mp3_pure_rust(Self::get_embedded_music()) {
                        Ok(()) => {
                            info!("✅ Embedded startup music played successfully (rodio)");
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to play embedded music: {}", e);
                        }
                    }
                } else if let Some(path) = music_path {
                    info!("🎵 Playing external startup music: {}", path.display());
                    
                    // EVOLVED: Play external file with pure Rust!
                    match play_file_pure_rust(&path) {
                        Ok(()) => {
                            info!("✅ External startup music played successfully (rodio)");
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to play external music: {}", e);
                        }
                    }
                } else {
                    info!("🎵 No startup music available (signature tone only)");
                }
            }

            info!("✨ Startup audio sequence complete");
        });
    }

    /// Enable/disable signature tone
    pub fn set_play_signature(&mut self, play: bool) {
        self.play_signature = play;
    }

    /// Enable/disable startup music
    pub fn set_play_music(&mut self, play: bool) {
        self.play_music = play;
    }

    /// Check if startup music is available (embedded or external)
    #[must_use]
    pub fn has_startup_music(&self) -> bool {
        self.startup_music_path.is_some() || (self.use_embedded && Self::has_embedded_music())
    }

    /// Get startup music path (external only)
    #[must_use]
    pub fn startup_music_path(&self) -> Option<&PathBuf> {
        self.startup_music_path.as_ref()
    }

    /// Check if using embedded music
    #[must_use]
    pub fn is_using_embedded(&self) -> bool {
        self.use_embedded && Self::has_embedded_music()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_tone_generation() {
        let signature = StartupAudio::generate_signature_tone();

        // Should have generated samples
        assert!(!signature.is_empty(), "Signature should generate samples");

        // Should be reasonable length (< 2 seconds)
        assert!(
            signature.len() < (SAMPLE_RATE * 2) as usize,
            "Signature should be under 2 seconds"
        );

        // All samples should be in valid range
        for (i, &sample) in signature.iter().enumerate() {
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Sample {} value {} out of range [-1.0, 1.0]",
                i,
                sample
            );
        }
    }

    #[test]
    fn test_signature_tone_length() {
        let signature = StartupAudio::generate_signature_tone();

        // Should be around 0.7 seconds (3 notes * 0.5s with stagger)
        let duration_secs = signature.len() as f32 / SAMPLE_RATE as f32;
        assert!(
            (0.5..=1.0).contains(&duration_secs),
            "Signature duration should be 0.5-1.0s, got {}s",
            duration_secs
        );
    }

    #[test]
    fn test_signature_tone_normalization() {
        let signature = StartupAudio::generate_signature_tone();

        // Find max amplitude
        let max_amplitude = signature.iter().fold(0.0_f32, |max, &s| max.max(s.abs()));

        // Should be normalized with headroom (not exceeding 0.7)
        assert!(
            max_amplitude <= 0.71,
            "Max amplitude {} should be normalized with headroom",
            max_amplitude
        );

        // Should have some signal (not all zeros)
        assert!(
            max_amplitude > 0.1,
            "Signature should have reasonable amplitude"
        );
    }

    #[test]
    fn test_startup_audio_creation() {
        let startup = StartupAudio::new();

        // Should have defaults set
        assert!(
            startup.play_signature,
            "Signature should be enabled by default"
        );
        assert!(startup.play_music, "Music should be enabled by default");
    }

    #[test]
    fn test_startup_audio_configuration() {
        let mut startup = StartupAudio::new();

        // Test signature toggle
        startup.set_play_signature(false);
        assert!(!startup.play_signature);

        startup.set_play_signature(true);
        assert!(startup.play_signature);

        // Test music toggle
        startup.set_play_music(false);
        assert!(!startup.play_music);

        startup.set_play_music(true);
        assert!(startup.play_music);
    }

    #[test]
    fn test_startup_music_path_detection() {
        let startup = StartupAudio::new();

        // Should attempt to find music (may or may not exist)
        // This test just verifies the detection runs without panic
        let _ = startup.has_startup_music();
        let _ = startup.startup_music_path();
    }

    #[test]
    fn test_has_startup_music() {
        let startup = StartupAudio::new();
        // With embedded music, has_startup_music() always returns true
        // unless explicitly disabled via PETALTONGUE_DISABLE_EMBEDDED_MUSIC
        assert!(
            startup.has_startup_music(),
            "has_startup_music should return true (embedded music available)"
        );
    }

    #[test]
    fn test_find_startup_music_paths() {
        // This test verifies the path search logic doesn't panic
        // Actual file existence depends on environment
        let startup = StartupAudio::new();

        if let Some(path) = startup.startup_music_path() {
            // If found, verify it's a path
            assert!(
                path.to_str().is_some(),
                "Startup music path should be valid UTF-8"
            );
        }
    }

    #[test]
    fn test_signature_tone_c_major_chord() {
        let signature = StartupAudio::generate_signature_tone();

        // Verify it's not empty (has actual audio data)
        assert!(
            signature.len() > SAMPLE_RATE as usize / 2,
            "Should have at least 0.5s of audio"
        );

        // Verify we have varied amplitudes (not flat/silence)
        let mut has_variation = false;
        let first_sample = signature[0];
        for &sample in signature.iter().skip(100) {
            if (sample - first_sample).abs() > 0.1 {
                has_variation = true;
                break;
            }
        }
        assert!(has_variation, "Signature should have amplitude variation");
    }

    #[test]
    fn test_startup_audio_with_both_disabled() {
        let mut startup = StartupAudio::new();
        startup.set_play_signature(false);
        startup.set_play_music(false);

        // Should handle disabled state without panic
        assert!(!startup.play_signature);
        assert!(!startup.play_music);
    }

    #[test]
    fn test_signature_generation_is_deterministic() {
        // Generate twice and compare
        let sig1 = StartupAudio::generate_signature_tone();
        let sig2 = StartupAudio::generate_signature_tone();

        // Should have same length
        assert_eq!(sig1.len(), sig2.len(), "Signature should be deterministic");

        // Should have same content
        for (i, (&s1, &s2)) in sig1.iter().zip(sig2.iter()).enumerate() {
            assert!(
                (s1 - s2).abs() < 0.0001,
                "Sample {} differs: {} vs {}",
                i,
                s1,
                s2
            );
        }
    }

    #[test]
    fn test_startup_audio_getters() {
        let startup = StartupAudio::new();

        // Test all getter methods
        let _ = startup.has_startup_music();
        let _ = startup.startup_music_path();
    }
}
