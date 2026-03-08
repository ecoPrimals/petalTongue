// SPDX-License-Identifier: AGPL-3.0-only
//! Direct Device Backend - Runtime Discovery of Audio Hardware
//!
//! Discovers direct audio devices at runtime:
//! - Linux: /dev/snd/pcmC*D*p
//! - macOS: /dev/audio
//! - FreeBSD: /dev/dsp*
//! - Future systems we don't know about yet!
//!
//! NO hardcoding - just discovers whatever direct devices exist!

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use anyhow::Result;
use async_trait::async_trait;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Discovered audio device
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub path: PathBuf,
    pub device_type: DeviceType,
}

/// Device type (detected heuristically)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    /// PCM device (Linux /dev/snd/pcmC*D*p)
    Pcm,
    /// Audio device (macOS /dev/audio)
    Audio,
    /// DSP device (FreeBSD /dev/dsp*)
    Dsp,
    /// Unknown device type
    Unknown,
}

/// Direct device backend
pub struct DirectBackend {
    device: DiscoveredDevice,
    file: Option<File>,
}

impl DirectBackend {
    /// Create from discovered device
    #[must_use]
    pub fn new(device: DiscoveredDevice) -> Self {
        Self { device, file: None }
    }

    /// Discover ALL direct audio devices at runtime
    ///
    /// This is NOT hardcoded to /dev/snd!
    /// We discover whatever direct devices exist.
    pub fn discover_all() -> Vec<Self> {
        let mut backends = Vec::new();

        // Discover devices using runtime heuristics
        for device in Self::discover_audio_devices() {
            info!(
                "✅ Discovered direct audio device: {} ({:?})",
                device.path.display(),
                device.device_type
            );
            backends.push(Self::new(device));
        }

        backends
    }

    /// Discover audio devices using runtime heuristics
    fn discover_audio_devices() -> Vec<DiscoveredDevice> {
        let mut devices = Vec::new();

        // Pattern 1: Linux /dev/snd/pcmC*D*p (ALSA PCM devices)
        if Path::new("/dev/snd").exists()
            && let Ok(entries) = std::fs::read_dir("/dev/snd")
        {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Find PCM playback devices (format: pcmC0D0p)
                // 'p' suffix = playback, 'c' suffix = capture
                if name.starts_with("pcm") && name.ends_with('p') {
                    devices.push(DiscoveredDevice {
                        path,
                        device_type: DeviceType::Pcm,
                    });
                }
            }
        }

        // Pattern 2: macOS /dev/audio
        if Path::new("/dev/audio").exists() {
            devices.push(DiscoveredDevice {
                path: PathBuf::from("/dev/audio"),
                device_type: DeviceType::Audio,
            });
        }

        // Pattern 3: FreeBSD /dev/dsp*
        for i in 0..10 {
            let path = if i == 0 {
                PathBuf::from("/dev/dsp")
            } else {
                PathBuf::from(format!("/dev/dsp{i}"))
            };

            if path.exists() {
                devices.push(DiscoveredDevice {
                    path,
                    device_type: DeviceType::Dsp,
                });
            }
        }

        // Extensible: New platforms add new patterns here!

        devices
    }

    /// Write samples to device (like `AudioCanvas`!)
    fn write_samples_to_device(file: &mut File, samples: &[f32]) -> Result<()> {
        // Convert f32 [-1.0, 1.0] to i16 PCM [-32768, 32767]
        let i16_samples: Vec<i16> = samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * 32767.0) as i16
            })
            .collect();

        // Convert to bytes - EVOLVED: Safe Rust using standard library
        // Each i16 is 2 bytes in little-endian format (ALSA default)
        let mut bytes = Vec::with_capacity(i16_samples.len() * 2);
        for sample in &i16_samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        // Write directly to device!
        file.write_all(&bytes)?;
        file.flush()?;

        Ok(())
    }
}

#[async_trait]
impl AudioBackend for DirectBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            name: format!("Direct Audio Device ({:?})", self.device.device_type),
            backend_type: BackendType::Direct,
            description: format!("Direct hardware access to {}", self.device.path.display()),
        }
    }

    fn priority(&self) -> u8 {
        40 // Lower priority than sockets (prefer user-level audio servers)
    }

    async fn is_available(&self) -> bool {
        self.device.path.exists()
    }

    async fn initialize(&mut self) -> Result<()> {
        info!(
            "🎨 Initializing direct audio device: {}",
            self.device.path.display()
        );

        // Open device for writing
        self.file = Some(
            std::fs::OpenOptions::new()
                .write(true)
                .open(&self.device.path)?,
        );

        Ok(())
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        debug!(
            "🎨 Direct playback: {} samples at {} Hz to {}",
            samples.len(),
            sample_rate,
            self.device.path.display()
        );

        // Ensure device is open
        if self.file.is_none() {
            self.initialize().await?;
        }

        // Write samples directly to hardware!
        if let Some(file) = &mut self.file {
            Self::write_samples_to_device(file, samples)?;
        }

        Ok(())
    }

    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: true,
            can_record: false,
            max_sample_rate: 48000,
            max_channels: 2,
            latency_estimate_ms: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_discovery() {
        // Should not panic
        let backends = DirectBackend::discover_all();
        println!("Discovered {} direct audio backend(s)", backends.len());

        for backend in &backends {
            let meta = backend.metadata();
            println!("  - {} at {}", meta.name, backend.device.path.display());
        }
    }
}
