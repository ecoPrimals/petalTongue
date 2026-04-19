// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio Canvas - Direct Hardware Access (Pure Rust!)
//!
//! Inspired by the WGPU GPU-provider pattern and framebuffer direct access.
//!
//! # Philosophy
//!
//! Just like we write pixels to `/dev/fb0` or use WGPU for `/dev/dri/card0`,
//! we can write audio samples directly to `/dev/snd/pcmC0D0p`!
//!
//! NO ALSA library, NO C dependencies - just raw device access!

use crate::error::{AudioError, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use tracing::{debug, info, warn};
#[cfg(not(target_os = "linux"))]
use tracing::{debug, info};

/// Audio Canvas - Direct hardware access (like WGPU for graphics!)
pub struct AudioCanvas {
    device: File,
    device_path: PathBuf,
    sample_rate: u32,
    channels: u8,
}

impl AudioCanvas {
    /// Open audio device for direct access
    ///
    /// # Errors
    ///
    /// Returns an error if the device cannot be opened for writing.
    pub fn open(device_path: &Path) -> Result<Self> {
        info!("🎨 Opening audio canvas: {}", device_path.display());

        let device = OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| {
                crate::error::UiError::Generic(format!(
                    "Failed to open audio device: {}: {e}",
                    device_path.display()
                ))
            })?;

        info!("✅ Audio canvas opened - direct hardware access!");

        Ok(Self {
            device,
            device_path: device_path.to_path_buf(),
            sample_rate: 44100,
            channels: 2,
        })
    }

    /// Open default audio device
    ///
    /// # Errors
    ///
    /// Returns an error if no devices are discovered or the first device cannot be opened.
    pub fn open_default() -> Result<Self> {
        let devices = Self::discover()?;

        if devices.is_empty() {
            return Err(AudioError::NoAudioDevices.into());
        }

        Self::open(&devices[0])
    }

    /// Write audio samples directly to hardware
    ///
    /// Samples are f32 in range \[-1.0, 1.0\], converted to i16 LE PCM.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the device or flushing fails.
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<()> {
        debug!("🎨 Writing {} samples to audio canvas", samples.len());

        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for &s in samples {
            let clamped = s.clamp(-1.0, 1.0);
            bytes.extend_from_slice(&((clamped * 32767.0) as i16).to_le_bytes());
        }

        self.device
            .write_all(&bytes)
            .map_err(crate::error::UiError::from)?;

        self.device.flush().map_err(crate::error::UiError::from)?;

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
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get number of channels
    #[must_use]
    pub const fn channels(&self) -> u8 {
        self.channels
    }
}

#[cfg(target_os = "linux")]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    #[must_use]
    pub fn is_available() -> bool {
        Path::new("/dev/snd").exists()
    }

    /// Discover audio playback devices
    ///
    /// Scans `/dev/snd/` for PCM playback devices (pure Rust).
    ///
    /// # Errors
    ///
    /// Returns an error if `/dev/snd` cannot be read.
    pub fn discover() -> Result<Vec<PathBuf>> {
        let mut devices = Vec::new();

        let snd_dir = Path::new("/dev/snd");
        if !snd_dir.exists() {
            warn!("/dev/snd not found - no audio devices available");
            return Ok(devices);
        }

        for entry in std::fs::read_dir(snd_dir).map_err(|e| {
            crate::error::UiError::Generic(format!("Failed to read /dev/snd directory: {e}"))
        })? {
            let path = entry.map_err(crate::error::UiError::Io)?.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if name.starts_with("pcm") && name.ends_with('p') {
                debug!("Found audio device: {}", path.display());
                devices.push(path);
            }
        }

        info!("🎨 Discovered {} audio canvas device(s)", devices.len());
        Ok(devices)
    }
}

#[cfg(target_os = "macos")]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    #[must_use]
    pub fn is_available() -> bool {
        Path::new("/dev/audio").exists()
    }

    /// Discover audio devices on macOS
    ///
    /// # Errors
    ///
    /// Returns an error if device enumeration fails.
    pub fn discover() -> Result<Vec<PathBuf>> {
        if Path::new("/dev/audio").exists() {
            Ok(vec![PathBuf::from("/dev/audio")])
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(target_os = "windows")]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    #[must_use]
    pub fn is_available() -> bool {
        false
    }

    /// Discover audio devices on Windows (not yet implemented)
    ///
    /// # Errors
    ///
    /// Always returns an empty list (WDM/WASAPI requires platform-specific integration).
    pub fn discover() -> Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
impl AudioCanvas {
    /// Check if audio canvas is available on this platform
    #[must_use]
    pub fn is_available() -> bool {
        false
    }

    /// Discover audio devices (unsupported platform)
    ///
    /// # Errors
    ///
    /// Always returns an empty list.
    pub fn discover() -> Result<Vec<PathBuf>> {
        Ok(Vec::new())
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
