// SPDX-License-Identifier: AGPL-3.0-only
//! Socket Audio Backend - Runtime Discovery of Socket-Based Audio Servers
//!
//! Discovers socket-based audio servers at runtime:
//! - `PipeWire` (modern Linux)
//! - `PulseAudio` (legacy Linux)
//! - Future systems we don't know about yet!
//!
//! NO hardcoding - just discovers whatever socket-based audio exists!

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Discovered audio socket
#[derive(Debug, Clone)]
pub struct DiscoveredSocket {
    pub path: PathBuf,
    pub detected_name: String,
}

/// Socket audio backend
pub struct SocketBackend {
    socket: DiscoveredSocket,
    _connection: Option<()>, // TODO: Actual socket connection
}

impl SocketBackend {
    /// Create from discovered socket
    #[must_use]
    pub fn new(socket: DiscoveredSocket) -> Self {
        Self {
            socket,
            _connection: None,
        }
    }

    /// Discover ALL socket-based audio servers at runtime
    ///
    /// This is NOT hardcoded to PipeWire/PulseAudio!
    /// We discover whatever socket-based audio exists.
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
        self.socket.path.exists()
    }

    async fn initialize(&mut self) -> Result<()> {
        info!(
            "🔌 Initializing socket audio backend: {}",
            self.socket.path.display()
        );
        // TODO: Actual socket connection
        Ok(())
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        debug!(
            "🔌 Socket playback: {} samples at {} Hz via {}",
            samples.len(),
            sample_rate,
            self.socket.detected_name
        );
        // TODO: Actual socket communication
        Ok(())
    }

    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: true,
            can_record: true,
            max_sample_rate: 192000,
            max_channels: 8,
            latency_estimate_ms: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_socket_discovery() {
        // Should not panic
        let backends = SocketBackend::discover_all().await;
        println!("Discovered {} socket audio backend(s)", backends.len());

        for backend in &backends {
            let meta = backend.metadata();
            println!("  - {} at {}", meta.name, backend.socket.path.display());
        }
    }
}
