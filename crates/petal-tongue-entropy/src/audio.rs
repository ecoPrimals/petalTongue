//! Audio entropy capture (singing, speaking)
//!
//! Captures voice/audio with timing, pitch dynamics, and spectral patterns.
//! 
//! ## Design Principles
//! 
//! - **Zero mocks**: Real microphone capture only (testing uses recorded samples)
//! - **Stream-only**: Never persists audio data
//! - **Real-time quality**: Immediate feedback during capture
//! - **Modern Rust**: Safe, idiomatic, zero unsafe code
//!
//! ## Architecture
//!
//! ```text
//! Microphone → cpal Stream → Sample Buffer → Quality Analysis → User Feedback
//!                                ↓
//!                          Streaming API (encrypted)
//! ```

use crate::types::*;
use crate::quality::{shannon_entropy, variance, timing_entropy, weighted_quality};
use anyhow::{anyhow, bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Sample, SampleFormat, SampleRate, Stream, StreamConfig};
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Audio sample rate (44.1 kHz - CD quality)
pub const AUDIO_SAMPLE_RATE: u32 = 44100;

/// Buffer size for real-time processing (100ms chunks)
pub const AUDIO_BUFFER_SIZE: usize = 4410; // 44100 samples/sec * 0.1 sec

/// Maximum capture duration (5 minutes)
pub const MAX_CAPTURE_DURATION: Duration = Duration::from_secs(300);

/// Minimum capture duration for quality (3 seconds)
pub const MIN_CAPTURE_DURATION: Duration = Duration::from_secs(3);

/// Audio entropy capturer
///
/// Captures audio from a microphone in real-time, analyzes quality,
/// and provides immediate feedback to the user.
///
/// ## Example
///
/// ```no_run
/// use petal_tongue_entropy::audio::AudioEntropyCapture;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create capturer with default device
/// let mut capture = AudioEntropyCapture::new()?;
///
/// // Start capturing
/// capture.start()?;
///
/// // ... user sings/speaks ...
///
/// // Check quality in real-time
/// let quality = capture.assess_quality();
/// println!("Quality: {:.1}%", quality.overall_quality * 100.0);
///
/// // Stop and finalize
/// capture.stop()?;
/// let entropy = capture.finalize()?;
/// # Ok(())
/// # }
/// ```
pub struct AudioEntropyCapture {
    /// Audio host (platform-specific)
    host: Host,
    
    /// Audio device (microphone)
    device: Device,
    
    /// Stream configuration
    config: StreamConfig,
    
    /// Active audio stream (None when stopped)
    stream: Option<Stream>,
    
    /// Shared buffer for captured samples
    buffer: Arc<Mutex<AudioBuffer>>,
    
    /// Capture start time
    start_time: Option<Instant>,
    
    /// Capture state
    state: CaptureState,
}

/// Internal audio buffer
struct AudioBuffer {
    /// Raw audio samples (f32)
    samples: Vec<f32>,
    
    /// Sample timestamps (for timing entropy)
    timestamps: Vec<Duration>,
    
    /// Peak amplitudes per chunk
    peaks: Vec<f32>,
    
    /// Buffer start time
    start_time: Option<Instant>,
}

/// Capture state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureState {
    /// Not started
    Idle,
    
    /// Actively capturing
    Recording,
    
    /// Stopped, ready to finalize
    Stopped,
    
    /// Finalized (cannot reuse)
    Finalized,
}

impl AudioEntropyCapture {
    /// Create a new audio entropy capturer with the default input device
    pub fn new() -> Result<Self> {
        Self::with_device_name(None)
    }
    
    /// Create a capturer with a specific device name
    ///
    /// If `device_name` is `None`, uses the default input device.
    pub fn with_device_name(device_name: Option<&str>) -> Result<Self> {
        info!("Initializing audio entropy capture");
        
        // Get audio host (platform-specific)
        let host = cpal::default_host();
        debug!("Audio host: {:?}", host.id());
        
        // Get input device
        let device = if let Some(name) = device_name {
            // Find device by name
            host.input_devices()?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .ok_or_else(|| anyhow!("Audio device '{}' not found", name))?
        } else {
            // Use default input device
            host.default_input_device()
                .ok_or_else(|| anyhow!("No default audio input device found"))?
        };
        
        info!("Using audio device: {}", device.name()?);
        
        // Get supported config
        let supported_config = device
            .default_input_config()
            .context("Failed to get default input config")?;
        
        debug!("Default audio config: {:?}", supported_config);
        
        // Create stream config (prefer 44.1kHz, mono)
        let config = StreamConfig {
            channels: 1, // Mono
            sample_rate: SampleRate(AUDIO_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Fixed(AUDIO_BUFFER_SIZE as u32),
        };
        
        // Create shared buffer
        let buffer = Arc::new(Mutex::new(AudioBuffer {
            samples: Vec::with_capacity(AUDIO_SAMPLE_RATE as usize * 60), // 1 minute initial capacity
            timestamps: Vec::with_capacity(1000),
            peaks: Vec::with_capacity(1000),
            start_time: None,
        }));
        
        Ok(Self {
            host,
            device,
            config,
            stream: None,
            buffer,
            start_time: None,
            state: CaptureState::Idle,
        })
    }
    
    /// Start capturing audio
    pub fn start(&mut self) -> Result<()> {
        if self.state != CaptureState::Idle {
            bail!("Cannot start: capture is already {:?}", self.state);
        }
        
        info!("Starting audio capture");
        
        let buffer = Arc::clone(&self.buffer);
        let start_time = Instant::now();
        
        // Initialize buffer start time
        {
            let mut buf = buffer.lock().unwrap();
            buf.start_time = Some(start_time);
        }
        
        // Build audio stream
        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Audio callback (runs in real-time audio thread)
                let mut buf = buffer.lock().unwrap();
                
                // Calculate timestamp for this chunk
                let timestamp = Instant::now().duration_since(start_time);
                
                // Calculate peak amplitude
                let peak = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                
                // Store samples, timestamp, and peak
                buf.samples.extend_from_slice(data);
                buf.timestamps.push(timestamp);
                buf.peaks.push(peak);
            },
            |err| {
                error!("Audio stream error: {}", err);
            },
            None, // No timeout
        )?;
        
        // Start the stream
        stream.play()?;
        
        self.stream = Some(stream);
        self.start_time = Some(start_time);
        self.state = CaptureState::Recording;
        
        info!("Audio capture started");
        Ok(())
    }
    
    /// Stop capturing audio
    pub fn stop(&mut self) -> Result<()> {
        if self.state != CaptureState::Recording {
            bail!("Cannot stop: capture is not recording (state: {:?})", self.state);
        }
        
        info!("Stopping audio capture");
        
        // Drop the stream (stops capture)
        self.stream = None;
        self.state = CaptureState::Stopped;
        
        // Check minimum duration
        if let Some(start) = self.start_time {
            let duration = Instant::now().duration_since(start);
            if duration < MIN_CAPTURE_DURATION {
                warn!(
                    "Capture duration ({:.1}s) is below minimum ({:.1}s) - quality may be low",
                    duration.as_secs_f64(),
                    MIN_CAPTURE_DURATION.as_secs_f64()
                );
            }
        }
        
        info!("Audio capture stopped");
        Ok(())
    }
    
    /// Assess current capture quality (can be called while recording)
    pub fn assess_quality(&self) -> AudioQualityMetrics {
        let buf = self.buffer.lock().unwrap();
        
        if buf.samples.is_empty() {
            return AudioQualityMetrics::default();
        }
        
        // Calculate various entropy measures
        let amplitude_entropy = self.calculate_amplitude_entropy(&buf);
        let timing_entropy_val = timing_entropy(&buf.timestamps);
        let spectral_entropy = self.calculate_spectral_entropy(&buf);
        let dynamic_range = self.calculate_dynamic_range(&buf);
        
        // Weighted overall quality
        let overall_quality = weighted_quality(&[
            (amplitude_entropy, 0.25),
            (timing_entropy_val, 0.25),
            (spectral_entropy, 0.30),
            (dynamic_range, 0.20),
        ]);
        
        AudioQualityMetrics {
            amplitude_entropy,
            timing_entropy: timing_entropy_val,
            spectral_entropy,
            dynamic_range,
            overall_quality,
        }
    }
    
    /// Calculate amplitude entropy (variation in volume)
    fn calculate_amplitude_entropy(&self, buf: &AudioBuffer) -> f64 {
        if buf.peaks.is_empty() {
            return 0.0;
        }
        
        // Quantize peaks into bins for entropy calculation
        let bins = 32; // 32 volume levels
        let peak_bins: Vec<u8> = buf.peaks
            .iter()
            .map(|&p| (p * bins as f32).min(bins as f32 - 1.0) as u8)
            .collect();
        
        shannon_entropy(&peak_bins)
    }
    
    /// Calculate spectral entropy (frequency distribution)
    fn calculate_spectral_entropy(&self, buf: &AudioBuffer) -> f64 {
        if buf.samples.len() < 1024 {
            return 0.0; // Not enough samples for FFT
        }
        
        // Take a representative chunk (middle of recording)
        let chunk_size = 1024;
        let start = (buf.samples.len() / 2).saturating_sub(chunk_size / 2);
        let chunk = &buf.samples[start..start + chunk_size.min(buf.samples.len() - start)];
        
        // Perform FFT
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(chunk.len());
        
        let mut spectrum: Vec<Complex<f32>> = chunk
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();
        
        fft.process(&mut spectrum);
        
        // Calculate magnitude spectrum
        let magnitudes: Vec<f32> = spectrum
            .iter()
            .take(spectrum.len() / 2) // Only positive frequencies
            .map(|c| c.norm())
            .collect();
        
        // Quantize into frequency bins for entropy
        let max_mag = magnitudes.iter().copied().fold(0.0f32, f32::max);
        if max_mag == 0.0 {
            return 0.0;
        }
        
        let bins = 16; // 16 frequency bands
        let freq_bins: Vec<u8> = magnitudes
            .iter()
            .map(|&m| ((m / max_mag) * bins as f32).min(bins as f32 - 1.0) as u8)
            .collect();
        
        shannon_entropy(&freq_bins)
    }
    
    /// Calculate dynamic range (variation in amplitude)
    fn calculate_dynamic_range(&self, buf: &AudioBuffer) -> f64 {
        if buf.peaks.is_empty() {
            return 0.0;
        }
        
        // Calculate variance of peak amplitudes
        variance(&buf.peaks)
    }
    
    /// Get current capture duration
    pub fn duration(&self) -> Option<Duration> {
        self.start_time.map(|start| Instant::now().duration_since(start))
    }
    
    /// Get number of samples captured
    pub fn sample_count(&self) -> usize {
        self.buffer.lock().unwrap().samples.len()
    }
    
    /// Finalize and create entropy data
    ///
    /// This consumes the capturer and returns the audio entropy data.
    /// The audio data is securely zeroized after finalization.
    pub fn finalize(mut self) -> Result<AudioEntropyData> {
        if self.state == CaptureState::Finalized {
            bail!("Cannot finalize: already finalized");
        }
        
        if self.state == CaptureState::Recording {
            self.stop()?;
        }
        
        info!("Finalizing audio capture");
        
        let duration = self.start_time
            .map(|start| Instant::now().duration_since(start))
            .unwrap_or_default();
        
        if duration < MIN_CAPTURE_DURATION {
            warn!("Finalizing with duration below minimum: {:.1}s", duration.as_secs_f64());
        }
        
        let quality_metrics = self.assess_quality();
        
        // Extract data from buffer
        let mut buf = self.buffer.lock().unwrap();
        
        // Calculate overall statistics
        let sample_count = buf.samples.len();
        let peak_amplitude = buf.peaks.iter().copied().fold(0.0f32, f32::max);
        let avg_amplitude = if !buf.peaks.is_empty() {
            buf.peaks.iter().sum::<f32>() / buf.peaks.len() as f32
        } else {
            0.0
        };
        
        // Move data (will be zeroized on Drop)
        let samples = std::mem::take(&mut buf.samples);
        let peaks = std::mem::take(&mut buf.peaks);
        let timestamps = std::mem::take(&mut buf.timestamps);
        
        self.state = CaptureState::Finalized;
        
        info!(
            "Audio finalized: {:.1}s, {} samples, quality: {:.1}%",
            duration.as_secs_f64(),
            sample_count,
            quality_metrics.overall_quality * 100.0
        );
        
        Ok(AudioEntropyData {
            samples,
            sample_rate: AUDIO_SAMPLE_RATE,
            duration,
            peaks,
            timestamps,
            peak_amplitude,
            avg_amplitude,
            quality_metrics,
        })
    }
}

/// List available audio input devices
pub fn list_audio_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices: Vec<String> = host
        .input_devices()?
        .filter_map(|d| d.name().ok())
        .collect();
    
    Ok(devices)
}

/// Get the default audio input device name
pub fn default_audio_device() -> Result<String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow!("No default audio input device found"))?;
    
    device.name().context("Failed to get device name")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_audio_devices() {
        // This should work on any system with audio
        match list_audio_devices() {
            Ok(devices) => {
                println!("Found {} audio devices:", devices.len());
                for device in devices {
                    println!("  - {}", device);
                }
            }
            Err(e) => {
                println!("Note: No audio devices found (may be headless): {}", e);
            }
        }
    }
    
    #[test]
    fn test_default_audio_device() {
        match default_audio_device() {
            Ok(device) => {
                println!("Default audio device: {}", device);
            }
            Err(e) => {
                println!("Note: No default audio device (may be headless): {}", e);
            }
        }
    }
    
    #[test]
    fn test_audio_capture_creation() {
        // This test may fail on headless systems (CI)
        match AudioEntropyCapture::new() {
            Ok(capture) => {
                println!("Audio capture created successfully");
                assert_eq!(capture.state, CaptureState::Idle);
                assert_eq!(capture.sample_count(), 0);
            }
            Err(e) => {
                println!("Note: Audio capture unavailable (may be headless): {}", e);
            }
        }
    }
    
    // NOTE: Real capture tests are in integration tests,
    // as they require user interaction and cannot be automated.
}

