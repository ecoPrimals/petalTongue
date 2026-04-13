// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio Discovery - Runtime detection of audio capabilities
//!
//! TRUE PRIMAL approach: Discovers what audio systems are available at runtime
//! and selects the best option with graceful fallback.
//!
//! # Discovery Chain
//!
//! 1. **`PipeWire`** (preferred) - Modern, user-level, no permissions
//! 2. **`PulseAudio`** (fallback) - Legacy, user-level, no permissions
//! 3. **Direct ALSA** (fallback) - Requires audio group
//! 4. **Silent Mode** (graceful) - No audio, visual-only
//!
//! # Pattern Alignment
//!
//! This follows the same pattern as discovery/registry-based discovery:
//! - Unix socket discovery
//! - Runtime capability detection
//! - Graceful degradation
//! - No hard dependencies

use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Audio backend type discovered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioBackendType {
    /// `PipeWire` (modern, preferred)
    PipeWire,
    /// `PulseAudio` (legacy, fallback)
    PulseAudio,
    /// Direct ALSA device access
    DirectALSA,
    /// No audio available (silent mode)
    Silent,
}

/// Discovered audio socket
#[derive(Debug, Clone)]
pub struct AudioSocket {
    /// Socket path
    pub path: PathBuf,
    /// Backend type
    pub backend_type: AudioBackendType,
    /// Whether socket is accessible
    pub accessible: bool,
}

/// Audio discovery results
#[derive(Debug, Clone)]
pub struct AudioDiscovery {
    /// Available audio sockets (`PipeWire`, `PulseAudio`)
    pub sockets: Vec<AudioSocket>,
    /// Available direct ALSA devices
    pub direct_devices: Vec<PathBuf>,
    /// Preferred backend
    pub preferred: AudioBackendType,
}

impl AudioDiscovery {
    /// Discover all available audio backends
    ///
    /// This follows TRUE PRIMAL principles:
    /// - Self-discovery at runtime
    /// - No hard dependencies
    /// - Graceful degradation
    pub fn discover() -> Self {
        info!("🎵 Discovering audio capabilities (TRUE PRIMAL)...");

        let mut sockets = Vec::new();

        // 1. Try PipeWire (preferred - modern, user-level)
        if let Some(socket) = Self::discover_pipewire() {
            info!("✅ PipeWire discovered: {}", socket.path.display());
            sockets.push(socket);
        }

        // 2. Try PulseAudio (fallback - legacy, user-level)
        if let Some(socket) = Self::discover_pulseaudio() {
            info!("✅ PulseAudio discovered: {}", socket.path.display());
            sockets.push(socket);
        }

        // 3. Try direct ALSA devices (fallback - requires audio group)
        let direct_devices = Self::discover_direct_alsa();
        if !direct_devices.is_empty() {
            info!(
                "✅ Direct ALSA devices discovered: {} device(s)",
                direct_devices.len()
            );
        }

        // Determine preferred backend
        let preferred = Self::determine_preferred(&sockets, &direct_devices);

        info!("🎯 Preferred audio backend: {:?}", preferred);

        Self {
            sockets,
            direct_devices,
            preferred,
        }
    }

    /// Discover `PipeWire` socket
    ///
    /// `PipeWire` is the modern Linux audio/video server.
    /// It exposes a Unix socket at /run/user/$UID/pipewire-0
    fn discover_pipewire() -> Option<AudioSocket> {
        debug!("🔍 Searching for PipeWire socket...");

        // Get user runtime directory (using safe helper from core)
        let runtime_dir = petal_tongue_core::system_info::get_user_runtime_dir();

        let socket_path = runtime_dir.join("pipewire-0");

        if socket_path.exists() {
            debug!("Found PipeWire socket: {}", socket_path.display());

            // Check if we can access it
            let accessible = socket_path
                .metadata()
                .map(|m| {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = m.permissions().mode();
                    // Check if socket is accessible (user or world readable/writable)
                    (mode & 0o600) != 0 || (mode & 0o006) != 0
                })
                .unwrap_or(false);

            Some(AudioSocket {
                path: socket_path,
                backend_type: AudioBackendType::PipeWire,
                accessible,
            })
        } else {
            debug!("PipeWire socket not found: {}", socket_path.display());
            None
        }
    }

    /// Discover `PulseAudio` socket
    ///
    /// `PulseAudio` is the legacy Linux audio server.
    /// It exposes a Unix socket at /run/user/$UID/pulse/native
    fn discover_pulseaudio() -> Option<AudioSocket> {
        debug!("🔍 Searching for PulseAudio socket...");

        // Get user runtime directory (using safe helper from core)
        let runtime_dir = petal_tongue_core::system_info::get_user_runtime_dir();

        let socket_path = runtime_dir.join("pulse/native");

        if socket_path.exists() {
            debug!("Found PulseAudio socket: {}", socket_path.display());

            let accessible = socket_path
                .metadata()
                .map(|m| {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = m.permissions().mode();
                    (mode & 0o600) != 0 || (mode & 0o006) != 0
                })
                .unwrap_or(false);

            Some(AudioSocket {
                path: socket_path,
                backend_type: AudioBackendType::PulseAudio,
                accessible,
            })
        } else {
            debug!("PulseAudio socket not found: {}", socket_path.display());
            None
        }
    }

    /// Discover direct ALSA devices
    ///
    /// Fallback for minimal systems without PipeWire/PulseAudio.
    /// Requires audio group membership.
    fn discover_direct_alsa() -> Vec<PathBuf> {
        debug!("🔍 Searching for direct ALSA devices...");

        let mut devices = Vec::new();
        let snd_dir = Path::new("/dev/snd");

        if !snd_dir.exists() {
            debug!("/dev/snd not found");
            return devices;
        }

        if let Ok(entries) = std::fs::read_dir(snd_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Find PCM playback devices (format: pcmC0D0p)
                // 'p' suffix = playback, 'c' suffix = capture
                if name.starts_with("pcm") && name.ends_with('p') {
                    debug!("Found ALSA device: {}", path.display());
                    devices.push(path);
                }
            }
        }

        devices
    }

    /// Determine preferred backend from discovery results
    fn determine_preferred(
        sockets: &[AudioSocket],
        direct_devices: &[PathBuf],
    ) -> AudioBackendType {
        // Prefer PipeWire (modern, no permissions needed)
        if sockets
            .iter()
            .any(|s| s.backend_type == AudioBackendType::PipeWire && s.accessible)
        {
            return AudioBackendType::PipeWire;
        }

        // Fallback to PulseAudio (legacy, no permissions needed)
        if sockets
            .iter()
            .any(|s| s.backend_type == AudioBackendType::PulseAudio && s.accessible)
        {
            return AudioBackendType::PulseAudio;
        }

        // Fallback to direct ALSA (requires audio group)
        if !direct_devices.is_empty() {
            return AudioBackendType::DirectALSA;
        }

        // Graceful degradation: silent mode
        AudioBackendType::Silent
    }

    /// Check if audio is available
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.preferred != AudioBackendType::Silent
    }

    /// Get human-readable status
    #[must_use]
    pub fn status_message(&self) -> String {
        match self.preferred {
            AudioBackendType::PipeWire => {
                "Audio available via PipeWire (modern, no permissions needed)".to_string()
            }
            AudioBackendType::PulseAudio => {
                "Audio available via PulseAudio (legacy, no permissions needed)".to_string()
            }
            AudioBackendType::DirectALSA => {
                "Audio available via direct ALSA (requires audio group)".to_string()
            }
            AudioBackendType::Silent => {
                "No audio available - running in silent mode (visual-only)".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_discovery() {
        // This test will attempt to discover audio backends.
        // The actual availability depends on the environment,
        // but it should not panic and should return valid results.
        let discovery = AudioDiscovery::discover();

        // Should have a preferred backend (even if Silent)
        assert!(
            matches!(
                discovery.preferred,
                AudioBackendType::PipeWire
                    | AudioBackendType::PulseAudio
                    | AudioBackendType::DirectALSA
                    | AudioBackendType::Silent
            ),
            "Discovery should determine a valid preferred backend"
        );

        // Status message should be non-empty
        assert!(
            !discovery.status_message().is_empty(),
            "Status message should be non-empty"
        );
    }

    #[test]
    fn test_determine_preferred_pipewire() {
        let sockets = vec![AudioSocket {
            path: PathBuf::from("/run/user/1000/pipewire-0"),
            backend_type: AudioBackendType::PipeWire,
            accessible: true,
        }];
        let direct = vec![];

        let preferred = AudioDiscovery::determine_preferred(&sockets, &direct);
        assert_eq!(preferred, AudioBackendType::PipeWire);
    }

    #[test]
    fn test_determine_preferred_fallback() {
        let sockets = vec![];
        let direct = vec![PathBuf::from("/dev/snd/pcmC0D0p")];

        let preferred = AudioDiscovery::determine_preferred(&sockets, &direct);
        assert_eq!(preferred, AudioBackendType::DirectALSA);
    }

    #[test]
    fn test_determine_preferred_silent() {
        let sockets = vec![];
        let direct = vec![];

        let preferred = AudioDiscovery::determine_preferred(&sockets, &direct);
        assert_eq!(preferred, AudioBackendType::Silent);
    }
}
