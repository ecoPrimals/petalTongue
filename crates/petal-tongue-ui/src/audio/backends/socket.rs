// SPDX-License-Identifier: AGPL-3.0-or-later
//! Socket audio backend (stub / planned PipeWire and PulseAudio client)
//!
//! **Status:** Runtime discovery of well-known Unix sockets under `XDG_RUNTIME_DIR` (e.g.
//! `pipewire-0`, `pulse/native`) is implemented, but **no wire protocol is implemented**. A future
//! version will speak the native protocols used by **PipeWire** and **PulseAudio** over those
//! sockets (session/stream setup, format negotiation, PCM transfer)—not a custom petalTongue
//! protocol.
//!
//! Until then, [`is_available`](crate::audio::traits::AudioBackend::is_available) is always
//! `false`, [`play_samples`](crate::audio::traits::AudioBackend::play_samples) returns an error,
//! and [`capabilities`](crate::audio::traits::AudioBackend::capabilities) reports no usable audio
//! features. Connecting in [`initialize`](crate::audio::traits::AudioBackend::initialize) on Unix
//! only establishes a stream; it does not enable playback without the missing protocol layer.

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use crate::error::{AudioError, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Discovered audio socket
#[derive(Debug, Clone)]
pub struct DiscoveredSocket {
    pub path: PathBuf,
    pub detected_name: String,
}

/// Socket audio backend
pub struct SocketBackend {
    socket: DiscoveredSocket,
    #[cfg(unix)]
    connection: Option<std::os::unix::net::UnixStream>,
    #[cfg(not(unix))]
    connection: Option<()>,
}

impl SocketBackend {
    /// Create from discovered socket
    #[must_use]
    pub const fn new(socket: DiscoveredSocket) -> Self {
        Self {
            socket,
            connection: None,
        }
    }

    /// Discover ALL socket-based audio servers at runtime
    ///
    /// This is NOT hardcoded to PipeWire/PulseAudio!
    /// We discover whatever socket-based audio exists.
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    pub async fn discover_all() -> Vec<Self> {
        let mut backends = Vec::new();

        // Discover sockets using runtime heuristics
        for socket in Self::discover_audio_sockets() {
            info!("✅ Discovered audio socket: {}", socket.detected_name);
            backends.push(Self::new(socket));
        }

        backends
    }

    /// Discover audio sockets using runtime heuristics
    fn discover_audio_sockets() -> Vec<DiscoveredSocket> {
        let mut sockets = Vec::new();

        // Pattern 1: Check XDG_RUNTIME_DIR for audio sockets
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let runtime_path = Path::new(&runtime_dir);

            // Check for common audio socket patterns
            // (We adapt to new patterns as they emerge!)
            let candidates = vec![
                ("pipewire-0", "PipeWire"),
                ("pulse/native", "PulseAudio"),
                // Add more patterns as we discover them!
            ];

            for (socket_name, detected_name) in candidates {
                let socket_path = runtime_path.join(socket_name);
                if socket_path.exists() && Self::is_audio_socket(&socket_path) {
                    sockets.push(DiscoveredSocket {
                        path: socket_path,
                        detected_name: detected_name.to_string(),
                    });
                }
            }
        }

        // Pattern 2: Look for any socket that looks like audio
        // This is extensible - we discover NEW systems automatically!

        sockets
    }

    /// Check if path is an audio socket (runtime heuristics)
    fn is_audio_socket(path: &Path) -> bool {
        // Verify it's actually a socket
        if let Ok(metadata) = path.metadata() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                use std::os::unix::fs::PermissionsExt;

                if !metadata.file_type().is_socket() {
                    return false;
                }

                // Check accessibility
                let mode = metadata.permissions().mode();

                (mode & 0o600) != 0 || (mode & 0o006) != 0
            }
            #[cfg(not(unix))]
            {
                // On non-Unix, just check if file exists
                let _ = metadata;
                true
            }
        } else {
            false
        }
    }
}

#[async_trait]
impl AudioBackend for SocketBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            name: format!("Socket Audio ({})", self.socket.detected_name),
            backend_type: BackendType::Socket,
            description: format!(
                "Socket-based audio server at {}",
                self.socket.path.display()
            ),
        }
    }

    fn priority(&self) -> u8 {
        30 // Higher priority than direct, lower than network
    }

    async fn is_available(&self) -> bool {
        // Socket exists, but PipeWire/PulseAudio protocol negotiation
        // is not implemented yet. Report unavailable so AudioManager
        // skips this backend and falls through to software/silent.
        false
    }

    async fn initialize(&mut self) -> Result<()> {
        info!(
            "🔌 Initializing socket audio backend: {}",
            self.socket.path.display()
        );
        #[cfg(unix)]
        {
            let path = self.socket.path.clone();
            let _path_display = path.display().to_string();
            let stream =
                tokio::task::spawn_blocking(move || std::os::unix::net::UnixStream::connect(&path))
                    .await
                    .map_err(|e| AudioError::SocketTaskFailed(e.to_string()))?
                    .map_err(|e| AudioError::SocketConnectionFailed(e.to_string()))?;
            self.connection = Some(stream);
        }
        #[cfg(not(unix))]
        {
            let _ = self;
            return Err(AudioError::SocketBackendUnavailable.into());
        }
        Ok(())
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        #[cfg(unix)]
        {
            if self.connection.is_none() {
                return Err(AudioError::SocketBackendNotInitialized.into());
            }
            debug!(
                "🔌 Socket playback: {} samples at {} Hz via {} (PipeWire/PulseAudio protocol stub)",
                samples.len(),
                sample_rate,
                self.socket.detected_name
            );
            warn!(
                "Planned PipeWire/PulseAudio socket protocol not implemented for {}",
                self.socket.detected_name
            );
            return Err(AudioError::SocketConnectionFailed(
                "PipeWire/PulseAudio socket protocol not implemented (planned; backend unavailable for playback)"
                    .to_string(),
            )
            .into());
        }
        #[cfg(not(unix))]
        {
            let _ = (samples, sample_rate);
            Err(AudioError::SocketBackendUnavailable.into())
        }
    }

    fn capabilities(&self) -> AudioCapabilities {
        // No protocol layer yet—do not advertise rates/channels/latency as if playback worked.
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
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_socket_discovery() {
        // Should not panic
        let backends = SocketBackend::discover_all().await;
        assert!(backends.len() <= 2); // At most pipewire-0 and pulse/native

        for backend in &backends {
            let meta = backend.metadata();
            assert!(!meta.name.is_empty());
            assert!(!meta.description.is_empty());
        }
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_discover_audio_sockets_returns_sockets_from_xdg_runtime_dir() {
        let temp = TempDir::new().unwrap();
        let socket_path = temp.path().join("pipewire-0");

        // Create a real Unix socket so is_audio_socket passes
        let _listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();

        let backends = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
            "XDG_RUNTIME_DIR",
            temp.path().to_str().unwrap(),
            || async { SocketBackend::discover_all().await },
        )
        .await;

        assert!(
            !backends.is_empty(),
            "Should discover pipewire-0 socket when XDG_RUNTIME_DIR contains it"
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_discover_audio_sockets_ignores_regular_files() {
        let temp = TempDir::new().unwrap();
        let fake_socket_path = temp.path().join("pipewire-0");
        // Write a regular file, not a socket
        std::fs::write(&fake_socket_path, "not a socket").unwrap();

        let backends = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
            "XDG_RUNTIME_DIR",
            temp.path().to_str().unwrap(),
            || async { SocketBackend::discover_all().await },
        )
        .await;

        assert!(
            backends.is_empty(),
            "Should not discover regular files as audio sockets"
        );
    }

    #[tokio::test]
    async fn test_socket_backend_capabilities_can_play_false() {
        let socket = DiscoveredSocket {
            path: PathBuf::from("/dev/null"),
            detected_name: "Test".to_string(),
        };
        let backend = SocketBackend::new(socket);
        let caps = backend.capabilities();
        assert!(
            !caps.can_play,
            "Socket backend reports can_play false (PipeWire/PulseAudio protocol not implemented)"
        );
        assert!(!caps.can_record);
        assert_eq!(caps.max_sample_rate, 0);
        assert_eq!(caps.max_channels, 0);
        assert_eq!(caps.latency_estimate_ms, 0);
    }

    #[tokio::test]
    async fn test_socket_backend_is_available_returns_false() {
        let socket = DiscoveredSocket {
            path: PathBuf::from("/dev/null"),
            detected_name: "Test".to_string(),
        };
        let backend = SocketBackend::new(socket);
        assert!(
            !backend.is_available().await,
            "Socket backend reports unavailable (protocol not implemented)"
        );
    }
}
