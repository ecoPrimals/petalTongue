// SPDX-License-Identifier: AGPL-3.0-only
//! Audio Canvas - Direct Hardware Access (Pure Rust!)
//!
//! Inspired by Toadstool's WGPU pattern and framebuffer direct access.
//!
//! # Philosophy
//!
//! Just like we write pixels to `/dev/fb0` or use WGPU for `/dev/dri/card0`,
//! we can write audio samples directly to `/dev/snd/pcmC0D0p`!
//!
//! NO ALSA library, NO C dependencies - just raw device access!

use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Audio Canvas - Direct hardware access (like WGPU for graphics!)
pub struct AudioCanvas {
    device: File,
    device_path: PathBuf,
    sample_rate: u32,
    channels: u8,
}

impl AudioCanvas {
    /// Discover audio playback devices
    ///
    /// Scans `/dev/snd/` for PCM playback devices (pure Rust!)
    pub fn discover() -> Result<Vec<PathBuf>> {
        let mut devices = Vec::new();

        let snd_dir = Path::new("/dev/snd");
        if !snd_dir.exists() {
            warn!("/dev/snd not found - no audio devices available");
            return Ok(devices);
        }

        for entry in std::fs::read_dir(snd_dir).context("Failed to read /dev/snd directory")? {
            let path = entry?.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Find PCM playback devices (format: pcmC0D0p)
            // 'p' suffix = playback, 'c' suffix = capture
            if name.starts_with("pcm") && name.ends_with('p') {
                debug!("Found audio device: {}", path.display());
                devices.push(path);
            }
        }

        info!("🎨 Discovered {} audio canvas device(s)", devices.len());
        Ok(devices)
    }

    /// Open audio device for direct access
    ///
    /// Opens `/dev/snd/pcmC0D0p` directly - no ALSA library!
    pub fn open(device_path: &Path) -> Result<Self> {
        info!("🎨 Opening audio canvas: {}", device_path.display());

        let device = OpenOptions::new()
            .write(true)
            .open(device_path)
            .context(format!(
                "Failed to open audio device: {}",
                device_path.display()
            ))?;

        info!("✅ Audio canvas opened - direct hardware access!");

        Ok(Self {
            device,
            device_path: device_path.to_path_buf(),
            sample_rate: 44100,
            channels: 2,
        })
    }

    /// Open default audio device
    pub fn open_default() -> Result<Self> {
        let devices = Self::discover()?;

        if devices.is_empty() {
            anyhow::bail!("No audio devices found in /dev/snd/");
        }

        // Use first device
        Self::open(&devices[0])
    }

    /// Write audio samples directly to hardware!
    ///
    /// Samples are f32 in range [-1.0, 1.0]
    /// Converted to i16 PCM and written directly to device
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<()> {
        debug!("🎨 Writing {} samples to audio canvas", samples.len());

        // Convert f32 [-1.0, 1.0] to i16 PCM [-32768, 32767]
        let i16_samples: Vec<i16> = samples
            .iter()
            .map(|&s| {
                // Clamp to valid range
                let clamped = s.clamp(-1.0, 1.0);
                // Scale to i16 range
                (clamped * 32767.0) as i16
            })
            .collect();

        // Convert to bytes (safe transmute via bytemuck or manual)
        // EVOLVED: Safe Rust - convert i16 samples to bytes
        // Each i16 is 2 bytes in little-endian format (ALSA default)
        let mut bytes = Vec::with_capacity(i16_samples.len() * 2);
        for sample in &i16_samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        // Write directly to device!
        self.device
            .write_all(&bytes)
            .context("Failed to write samples to audio device")?;

        self.device
            .flush()
            .context("Failed to flush audio device")?;

        debug!("✅ Samples written to audio canvas");

        Ok(())
    }

    /// Get device path
    #[must_use]
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }

    /// Get sample rate
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get number of channels
    #[must_use]
    pub fn channels(&self) -> u8 {
        self.channels
    }
}

/// Platform-specific audio canvas (Linux)
#[cfg(target_os = "linux")]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    #[must_use]
    pub fn is_available() -> bool {
        Path::new("/dev/snd").exists()
    }
}

/// Platform-specific audio canvas (macOS)
#[cfg(target_os = "macos")]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    pub fn is_available() -> bool {
        // macOS uses /dev/audio or CoreAudio
        Path::new("/dev/audio").exists()
    }

    /// Discover audio devices on macOS
    pub fn discover() -> Result<Vec<PathBuf>> {
        // macOS typically uses /dev/audio
        if Path::new("/dev/audio").exists() {
            Ok(vec![PathBuf::from("/dev/audio")])
        } else {
            Ok(Vec::new())
        }
    }
}

/// Platform-specific audio canvas (Windows)
#[cfg(target_os = "windows")]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    pub fn is_available() -> bool {
        // Windows uses WDM/WASAPI - different approach needed
        false // TODO: Implement Windows direct access
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_devices() {
        // Should not panic
        let result = AudioCanvas::discover();

        match result {
            Ok(devices) => {
                println!("Found {} audio devices", devices.len());
                for device in devices {
                    println!("  - {}", device.display());
                }
            }
            Err(e) => {
                println!("Discovery failed (may be expected in test env): {}", e);
            }
        }
    }

    #[test]
    fn test_is_available() {
        let available = AudioCanvas::is_available();
        println!("Audio canvas available: {}", available);
    }

    #[test]
    #[ignore = "Requires audio hardware"]
    fn test_open_and_write() {
        let devices = AudioCanvas::discover().expect("Failed to discover devices");

        if devices.is_empty() {
            println!("No devices found - skipping test");
            return;
        }

        let mut canvas = AudioCanvas::open(&devices[0]).expect("Failed to open audio canvas");

        // Generate test tone (440 Hz A4)
        let sample_rate = 44100.0;
        let duration = 0.1; // 100ms
        let frequency = 440.0;

        let samples: Vec<f32> = (0..((sample_rate * duration) as usize))
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3
            })
            .collect();

        canvas
            .write_samples(&samples)
            .expect("Failed to write samples");

        println!("✅ Test tone played successfully!");
    }
}
