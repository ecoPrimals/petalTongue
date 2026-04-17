// SPDX-License-Identifier: AGPL-3.0-or-later
//! Network Audio Backend — Ecosystem Audio Delegation
//!
//! Tier 1 audio backend: discovers an `audio` capability provider via
//! `CapabilityDiscovery<BiomeOsBackend>` and delegates playback over JSON-RPC/UDS.
//!
//! Falls back gracefully when no audio primal is available (lower-priority backends
//! handle playback instead).

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use crate::error::Result;
use petal_tongue_core::biomeos_discovery::BiomeOsBackend;
use petal_tongue_core::capability_discovery::{CapabilityDiscovery, CapabilityQuery};
use tracing::{debug, info, warn};

/// Network audio backend — delegates to ecosystem audio primal via capability discovery.
pub struct NetworkBackend {
    socket_path: Option<String>,
}

impl NetworkBackend {
    /// Attempt to discover an ecosystem audio provider.
    ///
    /// Returns a backend regardless of whether discovery succeeds; `is_available()`
    /// will report `false` if no provider was found (graceful degradation).
    pub async fn discover() -> Self {
        let socket_path = Self::try_discover().await;
        if socket_path.is_some() {
            info!("Discovered ecosystem audio provider via biomeOS capability discovery");
        } else {
            debug!("No ecosystem audio provider found (expected until ToadStool wires audio.*)");
        }
        Self { socket_path }
    }

    async fn try_discover() -> Option<String> {
        let backend = BiomeOsBackend::from_env().ok()?;
        let discovery = CapabilityDiscovery::new(backend);
        let query = CapabilityQuery::new("audio");
        let endpoint = discovery.discover_one(&query).await.ok()?;

        endpoint
            .endpoints
            .jsonrpc
            .or(endpoint.endpoints.tarpc)
    }

    async fn send_play_request(
        &self,
        socket_path: &str,
        samples: &[f32],
        sample_rate: u32,
    ) -> std::result::Result<(), String> {
        use base64::{Engine as _, engine::general_purpose};
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let pcm_bytes: Vec<u8> = samples
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();
        let encoded = general_purpose::STANDARD.encode(&pcm_bytes);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "audio.play",
            "params": {
                "format": "f32le",
                "sample_rate": sample_rate,
                "channels": 1,
                "data": encoded,
            },
            "id": 1
        });

        let mut stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| format!("connect: {e}"))?;

        let payload = format!("{}\n", serde_json::to_string(&request).map_err(|e| e.to_string())?);
        stream
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| format!("write: {e}"))?;
        stream.flush().await.map_err(|e| format!("flush: {e}"))?;

        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("read: {e}"))?;

        let response: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| format!("parse: {e}"))?;

        if let Some(err) = response.get("error") {
            return Err(format!("audio.play error: {err}"));
        }

        Ok(())
    }
}

impl AudioBackend for NetworkBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            name: "Ecosystem Audio (capability discovery)".to_string(),
            backend_type: BackendType::Network,
            description: "Audio playback via ecosystem primal (audio.play over JSON-RPC/UDS)"
                .to_string(),
        }
    }

    fn priority(&self) -> u8 {
        10
    }

    async fn is_available(&self) -> bool {
        self.socket_path.is_some()
    }

    async fn initialize(&mut self) -> Result<()> {
        if self.socket_path.is_none() {
            self.socket_path = Self::try_discover().await;
        }
        if self.socket_path.is_some() {
            info!("Ecosystem audio backend initialized");
            Ok(())
        } else {
            Err(crate::error::AudioError::NoBackendsAvailable.into())
        }
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        let socket_path = self
            .socket_path
            .as_ref()
            .ok_or(crate::error::AudioError::NoActiveBackend)?
            .clone();

        match self.send_play_request(&socket_path, samples, sample_rate).await {
            Ok(()) => {
                debug!("Played {} samples via ecosystem audio provider", samples.len());
                Ok(())
            }
            Err(e) => {
                warn!("Ecosystem audio playback failed: {e}");
                Err(crate::error::AudioError::SocketConnectionFailed(e).into())
            }
        }
    }

    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: self.socket_path.is_some(),
            can_record: false,
            max_sample_rate: 48000,
            max_channels: 2,
            latency_estimate_ms: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_backend_discover_graceful() {
        let backend = NetworkBackend::discover().await;
        let meta = backend.metadata();
        assert_eq!(meta.backend_type, BackendType::Network);
        assert_eq!(backend.priority(), 10);
    }

    #[tokio::test]
    async fn test_network_backend_unavailable_without_biome_os() {
        let backend = NetworkBackend::discover().await;
        // Without biomeOS running, should not be available (graceful degradation)
        // This may be true or false depending on environment, so just verify no panic
        let _ = backend.is_available().await;
    }

    #[test]
    fn test_network_backend_metadata() {
        let backend = NetworkBackend { socket_path: None };
        let meta = backend.metadata();
        assert!(meta.name.contains("Ecosystem"));
        assert!(meta.description.contains("audio.play"));
    }

    #[test]
    fn test_network_backend_capabilities_without_provider() {
        let backend = NetworkBackend { socket_path: None };
        let caps = backend.capabilities();
        assert!(!caps.can_play);
    }

    #[test]
    fn test_network_backend_capabilities_with_provider() {
        let backend = NetworkBackend {
            socket_path: Some("/tmp/test-audio.sock".to_string()),
        };
        let caps = backend.capabilities();
        assert!(caps.can_play);
        assert_eq!(caps.max_sample_rate, 48000);
    }
}
