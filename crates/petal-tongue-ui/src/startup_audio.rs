// SPDX-License-Identifier: AGPL-3.0-or-later
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

use crate::audio::AudioSystemV2;
#[cfg(test)]
use crate::audio_pure_rust::SAMPLE_RATE;
use crate::audio_pure_rust::{Waveform, generate_tone};
use bytes::Bytes;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Play audio samples using Audio Canvas (direct hardware access!)
///
/// EVOLVED: Like WGPU for graphics, we write directly to /dev/snd!
/// NO rodio, NO cpal, NO ALSA library - just raw device access!
fn play_audio_pure_rust(samples: &[f32]) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;

    // Open audio device directly (like opening framebuffer!)
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {e}"))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(samples)
        .map_err(|e| format!("Failed to write samples: {e}"))?;

    Ok(())
}

/// Play embedded MP3 using Audio Canvas + symphonia (100% pure Rust!)
fn play_embedded_mp3_pure_rust(mp3_data: Bytes) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;

    // Decode MP3 with symphonia (pure Rust!)
    let decoded =
        decode_audio_symphonia(mp3_data).map_err(|e| format!("Failed to decode MP3: {e}"))?;

    // Open audio canvas
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {e}"))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(&decoded.samples)
        .map_err(|e| format!("Failed to write samples: {e}"))?;

    Ok(())
}

/// Decode audio with symphonia (pure Rust!)
///
/// # Errors
///
/// Returns an error if format probing fails, no default track exists, decoder creation fails, or sample rate is missing.
pub fn decode_audio_symphonia(audio_data: Bytes) -> Result<DecodedAudio, String> {
    use std::io::Cursor;
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let cursor = Cursor::new(audio_data);
    let mss = MediaSourceStream::new(Box::new(cursor), MediaSourceStreamOptions::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Probe failed: {e}"))?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("No default track")?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Decoder creation failed: {e}"))?;

    let sample_rate = track.codec_params.sample_rate.ok_or("No sample rate")? as f32;

    let mut samples = Vec::new();

    while let Ok(packet) = format.next_packet() {
        let Ok(decoded_buf) = decoder.decode(&packet) else {
            continue;
        };

        // Convert to f32 samples
        match decoded_buf {
            AudioBufferRef::F32(buf) => {
                for &sample in buf.chan(0) {
                    samples.push(sample);
                }
            }
            AudioBufferRef::S16(buf) => {
                for &sample in buf.chan(0) {
                    samples.push(f32::from(sample) / 32768.0);
                }
            }
            _ => {}
        }
    }

    Ok(DecodedAudio {
        samples,
        sample_rate,
    })
}

/// Decoded audio data
pub struct DecodedAudio {
    /// Audio samples as f32 in range [-1.0, 1.0]
    pub samples: Vec<f32>,
    /// Sample rate in Hz
    pub sample_rate: f32,
}

/// Play audio file using Audio Canvas + symphonia (100% pure Rust!)
fn play_file_pure_rust(path: &Path) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;
    use std::fs;

    let data = Bytes::from(fs::read(path).map_err(|e| format!("Failed to read audio file: {e}"))?);
    let decoded =
        decode_audio_symphonia(data).map_err(|e| format!("Failed to decode audio: {e}"))?;

    // Open audio canvas
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {e}"))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(&decoded.samples)
        .map_err(|e| format!("Failed to write samples: {e}"))?;

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
            }
            warn!(
                "🎵 PETALTONGUE_STARTUP_MUSIC set but file not found: {}",
                path.display()
            );
        }

        None
    }

    /// Get embedded startup music data
    #[must_use]
    pub const fn get_embedded_music() -> &'static [u8] {
        EMBEDDED_STARTUP_MUSIC
    }

    /// Check if embedded music is available
    #[must_use]
    pub const fn has_embedded_music() -> bool {
        !EMBEDDED_STARTUP_MUSIC.is_empty()
    }

    /// Generate signature tone
    ///
    /// Creates the petalTongue "bloom" sound - a quick ascending arpeggio
    /// (C5, E5, G5) that identifies the application.
    #[must_use]
    pub fn generate_signature_tone() -> Vec<f32> {
        info!("🎵 Generating petalTongue signature tone (bloom)...");

        // Bloom: 3-note ascending arpeggio, ~80ms per note, ~250ms total
        let notes = [
            (523.25, 0.08), // C5
            (659.25, 0.08), // E5
            (784.00, 0.08), // G5
        ];

        let mut signature = Vec::new();
        for &(freq, duration) in &notes {
            let tone = generate_tone(freq, duration, Waveform::Sine, 0.35);
            signature.extend(tone);
        }

        // Normalize to prevent clipping
        let max_amplitude = signature.iter().fold(0.0_f32, |max, &s| max.max(s.abs()));
        if max_amplitude > 0.0 {
            for sample in &mut signature {
                *sample /= max_amplitude;
                *sample *= 0.7; // Leave headroom
            }
        }

        info!(
            "✨ Signature tone generated: {} samples (~250ms bloom)",
            signature.len()
        );
        signature
    }

    /// Play startup audio sequence
    ///
    /// Plays signature tone followed by startup music (if available).
    /// Non-blocking - spawns background thread.
    pub fn play(&self, _audio_system: &AudioSystemV2) {
        if !self.play_signature && !self.play_music {
            return;
        }

        info!("🎵 Starting petalTongue startup audio...");

        let play_signature = self.play_signature;
        let play_music = self.play_music;
        let music_path = self.startup_music_path.clone();

        // Spawn background thread for audio playback
        std::thread::spawn(move || {
            use crate::audio_pure_rust::export_wav;

            // 1. Play signature tone (pure Rust, always works)
            if play_signature {
                let signature = Self::generate_signature_tone();

                let temp_dir = std::env::temp_dir();
                let wav_path = temp_dir.join("petaltongue_signature.wav");
                let wav_bytes = export_wav(&signature);

                // EVOLVED: Pure Rust audio playback (no external dependencies!)
                match play_audio_pure_rust(&signature) {
                    Ok(()) => {
                        info!("✅ Signature tone played with pure Rust (rodio)");
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ Pure Rust audio failed: {}, writing to disk as fallback",
                            e
                        );

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

                    match play_embedded_mp3_pure_rust(
                        Bytes::from_static(Self::get_embedded_music()),
                    ) {
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
    pub const fn set_play_signature(&mut self, play: bool) {
        self.play_signature = play;
    }

    /// Enable/disable startup music
    pub const fn set_play_music(&mut self, play: bool) {
        self.play_music = play;
    }

    /// Check if startup music is available (embedded or external)
    #[must_use]
    pub const fn has_startup_music(&self) -> bool {
        self.startup_music_path.is_some() || (self.use_embedded && Self::has_embedded_music())
    }

    /// Get startup music path (external only)
    #[must_use]
    pub const fn startup_music_path(&self) -> Option<&PathBuf> {
        self.startup_music_path.as_ref()
    }

    /// Check if using embedded music
    #[must_use]
    pub const fn is_using_embedded(&self) -> bool {
        self.use_embedded && Self::has_embedded_music()
    }
}

#[cfg(test)]
#[path = "startup_audio_tests.rs"]
mod tests;
