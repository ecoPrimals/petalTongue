// SPDX-License-Identifier: AGPL-3.0-or-later
//! Direct device backend (incomplete — future ALSA / OSS path)
//!
//! # Cargo feature
//!
//! This module is compiled only with the **`audio-direct`** feature on `petal-tongue-ui`.
//! Default release builds omit it so the runtime does not register a non-functional backend.
//!
//! # Status
//!
//! Discovery of device nodes is implemented, but **playback is not**. Opening a PCM node and
//! writing raw samples requires ALSA `ioctl` setup (sample rate, format, buffers, etc.), which is
//! not implemented. Devices managed by PipeWire or PulseAudio can also block if opened without
//! going through those stacks.
//!
//! Callers should treat this module as a **placeholder** for a future direct-hardware path, not
//! as an active audio backend. [`AudioBackend::is_available`](crate::audio::traits::AudioBackend::is_available)
//! always returns `false`; [`initialize`](crate::audio::traits::AudioBackend::initialize) and
//! [`play_samples`](crate::audio::traits::AudioBackend::play_samples) return
//! [`AudioError::DirectDeviceUnavailable`](crate::error::AudioError::DirectDeviceUnavailable).
//!
//! Discovery heuristics (for when playback exists):
//! - Linux: `/dev/snd/pcmC*D*p`
//! - macOS: `/dev/audio`
//! - FreeBSD: `/dev/dsp*`

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use crate::error::{AudioError, Result};
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
}

impl DirectBackend {
    /// Create from discovered device
    #[must_use]
    pub const fn new(device: DiscoveredDevice) -> Self {
        Self { device }
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

    fn not_playable_message() -> String {
        "Direct PCM device access requires ALSA ioctl setup (sample rate, format, buffers) which is \
         not yet implemented. Use socket-based audio servers or software synthesis until this \
         backend is completed."
            .to_string()
    }

    /// Write samples to device (reserved for a future ALSA-backed implementation).
    #[allow(dead_code)] // Reserved for future ALSA-backed implementation
    pub(crate) fn write_samples_to_device(file: &mut File, samples: &[f32]) -> Result<()> {
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
        // Direct PCM device access requires ALSA ioctls for sample rate,
        // format, and buffer setup that are not yet implemented. Opening
        // a device managed by PipeWire/PulseAudio will also block
        // indefinitely. Report unavailable until proper ALSA setup is in place.
        false
    }

    async fn initialize(&mut self) -> Result<()> {
        Err(AudioError::DirectDeviceUnavailable(Self::not_playable_message()).into())
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        debug!(
            "🎨 Direct playback: {} samples at {} Hz to {} (not implemented)",
            samples.len(),
            sample_rate,
            self.device.path.display()
        );

        Err(AudioError::DirectDeviceUnavailable(Self::not_playable_message()).into())
    }

    fn capabilities(&self) -> AudioCapabilities {
        // Honest reporting: discovery exists but ALSA/ioctl playback path is not implemented.
        AudioCapabilities {
            can_play: false,
            can_record: false,
            max_sample_rate: 0,
            max_channels: 0,
            latency_estimate_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek};

    #[test]
    fn test_direct_discovery() {
        // Should not panic - tests runtime heuristic discovery
        let backends = DirectBackend::discover_all();
        for backend in &backends {
            let meta = backend.metadata();
            assert!(!meta.name.is_empty());
            assert_eq!(meta.backend_type, BackendType::Direct);
        }
    }

    #[test]
    fn test_write_samples_to_device_byte_output() {
        let mut file = tempfile::tempfile().unwrap();
        let samples: Vec<f32> = vec![1.0, -1.0, 0.0, 0.5];

        DirectBackend::write_samples_to_device(&mut file, &samples).unwrap();

        // Verify: f32 [-1,1] -> i16 little-endian (implementation uses 32767 scale)
        // 1.0 -> 32767, -1.0 -> -32767, 0.0 -> 0, 0.5 -> 16383
        file.rewind().unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(buf.len(), 8); // 4 samples * 2 bytes
        assert_eq!(buf[0..2], 32767i16.to_le_bytes());
        assert_eq!(buf[2..4], (-32767i16).to_le_bytes());
        assert_eq!(buf[4..6], 0i16.to_le_bytes());
        assert_eq!(buf[6..8], 16383i16.to_le_bytes());
    }

    #[tokio::test]
    async fn test_direct_backend_is_available_returns_false() {
        let device = DiscoveredDevice {
            path: PathBuf::from("/dev/null"),
            device_type: DeviceType::Pcm,
        };
        let backend = DirectBackend::new(device);
        assert!(
            !backend.is_available().await,
            "Direct backend reports unavailable (ALSA setup not implemented)"
        );
    }

    #[test]
    fn test_direct_backend_capabilities() {
        let device = DiscoveredDevice {
            path: PathBuf::from("/dev/null"),
            device_type: DeviceType::Pcm,
        };
        let backend = DirectBackend::new(device);
        let caps = backend.capabilities();
        assert!(
            !caps.can_play,
            "direct playback not implemented (ALSA ioctl path missing)"
        );
        assert!(!caps.can_record);
        assert_eq!(caps.max_sample_rate, 0);
        assert_eq!(caps.max_channels, 0);
        assert_eq!(caps.latency_estimate_ms, 0);
    }

    #[tokio::test]
    async fn test_direct_play_samples_returns_typed_error() {
        let device = DiscoveredDevice {
            path: PathBuf::from("/dev/null"),
            device_type: DeviceType::Pcm,
        };
        let mut backend = DirectBackend::new(device);
        let err = backend.play_samples(&[0.0_f32], 44100).await.unwrap_err();
        assert!(
            err.to_string().contains("ioctl") || err.to_string().contains("ALSA"),
            "expected DirectDeviceUnavailable message"
        );
    }
}
