//! Web Audio API - Universal Audio Abstraction
//!
//! TRUE PRIMAL sovereignty through web standards!
//!
//! # Architecture
//!
//! Uses Web Audio API (like WGPU uses WebGPU for graphics):
//! - 100% Pure Rust (no C dependencies!)
//! - Universal abstraction (works everywhere)
//! - Desktop: Linux, macOS, Windows
//! - Web: WASM
//! - Edge: Small embeddings
//!
//! # Inspiration
//!
//! Toadstool uses WGPU successfully for graphics.
//! petalTongue uses Web Audio for audio.
//! Same pattern, same success!

use anyhow::{Context, Result};
use std::io::Cursor;
use tracing::{debug, info, warn};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

/// Universal audio player using Web Audio API
///
/// Works on all platforms without C dependencies!
pub struct WebAudioPlayer {
    context: AudioContext,
}

impl WebAudioPlayer {
    /// Create a new Web Audio player
    ///
    /// This creates a cross-platform audio context that works on:
    /// - Linux (via cpal/alsa or other backends)
    /// - macOS (via CoreAudio)
    /// - Windows (via WASAPI)
    /// - Web (via WebAudio natively)
    /// - Edge devices (embedded implementations)
    pub fn new() -> Result<Self> {
        info!("🌐 Initializing Web Audio API (universal abstraction)");
        
        let context = AudioContext::default();
        
        info!("✅ Web Audio context created - pure Rust, no C dependencies!");
        
        Ok(Self { context })
    }
    
    /// Play audio samples
    ///
    /// Pure Rust playback - works everywhere!
    pub fn play_samples(&self, samples: &[f32], sample_rate: f32) -> Result<()> {
        debug!(
            "🔊 Playing {} samples at {} Hz (Web Audio)",
            samples.len(),
            sample_rate
        );
        
        // Create audio buffer
        let mut buffer = self.context.create_buffer(
            1,  // mono
            samples.len(),
            sample_rate,
        );
        
        // Copy samples to buffer
        buffer.copy_to_channel(samples, 0)
            .context("Failed to copy samples to audio buffer")?;
        
        // Create buffer source node
        let source = self.context.create_buffer_source();
        source.set_buffer(buffer);
        source.connect(&self.context.destination());
        
        // Play!
        source.start();
        
        info!("✅ Audio playback started (Web Audio API)");
        
        Ok(())
    }
    
    /// Play MP3 data (decoded with symphonia)
    ///
    /// 100% Pure Rust: symphonia decoding + Web Audio playback!
    pub fn play_mp3(&self, mp3_data: &[u8]) -> Result<()> {
        info!("🎵 Decoding MP3 ({} bytes) with symphonia...", mp3_data.len());
        
        // Decode with symphonia (pure Rust!)
        let decoded = decode_audio_with_symphonia(mp3_data)
            .context("Failed to decode MP3 with symphonia")?;
        
        info!(
            "✅ Decoded: {} samples at {} Hz",
            decoded.samples.len(),
            decoded.sample_rate
        );
        
        // Play decoded samples
        self.play_samples(&decoded.samples, decoded.sample_rate)?;
        
        Ok(())
    }
    
    /// Play WAV data (decoded with symphonia)
    pub fn play_wav(&self, wav_data: &[u8]) -> Result<()> {
        info!("🎵 Decoding WAV ({} bytes) with symphonia...", wav_data.len());
        
        let decoded = decode_audio_with_symphonia(wav_data)
            .context("Failed to decode WAV with symphonia")?;
        
        self.play_samples(&decoded.samples, decoded.sample_rate)?;
        
        Ok(())
    }
    
    /// Get the audio context sample rate
    pub fn sample_rate(&self) -> f32 {
        self.context.sample_rate()
    }
}

impl Default for WebAudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create WebAudioPlayer")
    }
}

/// Decoded audio data
struct DecodedAudio {
    samples: Vec<f32>,
    sample_rate: f32,
}

/// Decode audio with symphonia (pure Rust!)
///
/// Supports: MP3, WAV, FLAC, OGG, AAC
fn decode_audio_with_symphonia(audio_data: &[u8]) -> Result<DecodedAudio> {
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    
    // Create media source from bytes
    let cursor = Cursor::new(audio_data);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    
    // Probe the format
    let mut hint = Hint::new();
    hint.with_extension("mp3");  // Hint for MP3, but symphonia auto-detects
    
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .context("Failed to probe audio format")?;
    
    let mut format = probed.format;
    
    // Get the default track
    let track = format
        .default_track()
        .context("No default audio track found")?;
    
    // Create decoder
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("Failed to create audio decoder")?;
    
    let sample_rate = track
        .codec_params
        .sample_rate
        .context("No sample rate in audio")? as f32;
    
    let mut samples = Vec::new();
    
    // Decode all packets
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,  // End of stream
        };
        
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(_) => continue,  // Skip errors
        };
        
        // Convert to f32 samples
        match decoded {
            AudioBufferRef::F32(buf) => {
                // Already f32
                for &sample in buf.chan(0) {
                    samples.push(sample);
                }
            }
            AudioBufferRef::U8(buf) => {
                // Convert u8 to f32
                for &sample in buf.chan(0) {
                    samples.push((f32::from(sample) - 128.0) / 128.0);
                }
            }
            AudioBufferRef::U16(buf) => {
                // Convert u16 to f32
                for &sample in buf.chan(0) {
                    samples.push((f32::from(sample) - 32768.0) / 32768.0);
                }
            }
            AudioBufferRef::U24(buf) => {
                // Convert u24 to f32
                for &sample in buf.chan(0) {
                    samples.push((sample.into_i32() as f32 - 8388608.0) / 8388608.0);
                }
            }
            AudioBufferRef::U32(buf) => {
                // Convert u32 to f32
                for &sample in buf.chan(0) {
                    samples.push((sample as f32 - 2147483648.0) / 2147483648.0);
                }
            }
            AudioBufferRef::S8(buf) => {
                // Convert i8 to f32
                for &sample in buf.chan(0) {
                    samples.push(f32::from(sample) / 128.0);
                }
            }
            AudioBufferRef::S16(buf) => {
                // Convert i16 to f32
                for &sample in buf.chan(0) {
                    samples.push(f32::from(sample) / 32768.0);
                }
            }
            AudioBufferRef::S24(buf) => {
                // Convert i24 to f32
                for &sample in buf.chan(0) {
                    samples.push(sample.into_i32() as f32 / 8388608.0);
                }
            }
            AudioBufferRef::S32(buf) => {
                // Convert i32 to f32
                for &sample in buf.chan(0) {
                    samples.push(sample as f32 / 2147483648.0);
                }
            }
            AudioBufferRef::F64(buf) => {
                // Convert f64 to f32
                for &sample in buf.chan(0) {
                    samples.push(sample as f32);
                }
            }
        }
    }
    
    debug!("Decoded {} samples at {} Hz", samples.len(), sample_rate);
    
    Ok(DecodedAudio {
        samples,
        sample_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web_audio_player_creation() {
        // Should create successfully
        let player = WebAudioPlayer::new();
        assert!(player.is_ok(), "Should create Web Audio player");
        
        let player = player.unwrap();
        assert!(player.sample_rate() > 0.0, "Should have valid sample rate");
    }
    
    #[test]
    fn test_play_samples() {
        let player = WebAudioPlayer::new().expect("Failed to create player");
        
        // Generate test tone
        let sample_rate = 44100.0;
        let duration = 0.1;  // 100ms
        let frequency = 440.0;  // A4
        
        let samples: Vec<f32> = (0..((sample_rate * duration) as usize))
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3
            })
            .collect();
        
        // Should not panic
        let result = player.play_samples(&samples, sample_rate);
        assert!(result.is_ok(), "Should play samples without error");
    }
}

