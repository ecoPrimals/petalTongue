// SPDX-License-Identifier: AGPL-3.0-only
//! Primal Registration with Songbird
//!
//! Implements the `ipc.register` and `ipc.heartbeat` standards from `PRIMAL_IPC_PROTOCOL.md`.
//!
//! This module handles:
//! - Initial registration with Songbird on startup
//! - Periodic heartbeat to maintain registration
//! - Graceful handling when Songbird is unavailable

use anyhow::{Context, Result};
use petal_tongue_core::constants;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info, warn};

/// Primal registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRegistration {
    /// Primal name (e.g., "petaltongue")
    pub name: String,

    /// IPC endpoint (e.g., "/primal/petaltongue")
    pub endpoint: String,

    /// Capabilities this primal provides
    pub capabilities: Vec<String>,

    /// Version string (e.g., "1.4.0")
    pub version: String,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, Value>>,
}

impl PrimalRegistration {
    /// Create a new registration for petalTongue
    #[must_use]
    pub fn petaltongue() -> Self {
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "description".to_string(),
            json!("Universal representation system for ecoPrimals"),
        );
        metadata.insert(
            "ui_modes".to_string(),
            json!(["desktop", "tui", "web", "headless"]),
        );

        Self {
            name: "petaltongue".to_string(),
            endpoint: "/primal/petaltongue".to_string(),
            capabilities: vec![
                "ui.render".to_string(),
                "ui.visualization".to_string(),
                "ui.graph".to_string(),
                "graph.topology".to_string(),
                "graph.builder".to_string(),
                "modality.visual".to_string(),
                "modality.audio".to_string(),
            ],
            version: env!("CARGO_PKG_VERSION").to_string(),
            metadata: Some(metadata),
        }
    }
}

/// JSON-RPC client for Songbird communication
pub struct SongbirdClient {
    socket_path: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl SongbirdClient {
    /// Create a new Songbird client
    ///
    /// Capability-based discovery: uses discovery service socket (no hardcoded primal names).
    /// Socket path resolution (priority order):
    /// 1. `SONGBIRD_SOCKET` or `DISCOVERY_SERVICE_SOCKET` env (explicit override)
    /// 2. `discover_primal_socket` with capability-based socket name from constants
    /// 3. `SONGBIRD_SOCKET_FALLBACK` env or conventional path fallback
    #[must_use]
    pub fn new() -> Self {
        let socket_base = constants::discovery_service_socket_name();
        let socket_path = crate::socket_path::discover_primal_socket(&socket_base, None, None)
            .map_or_else(
                |_| {
                    std::env::var("SONGBIRD_SOCKET_FALLBACK")
                        .unwrap_or_else(|_| format!("/tmp/{socket_base}-nat0-default.sock"))
                },
                |p| p.to_string_lossy().to_string(),
            );
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Create a client with a custom socket path (for testing)
    #[must_use]
    pub fn with_socket_path(socket_path: String) -> Self {
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Check if Songbird is available
    pub async fn is_available(&self) -> bool {
        matches!(
            tokio::time::timeout(
                Duration::from_millis(100),
                UnixStream::connect(&self.socket_path),
            )
            .await,
            Ok(Ok(_))
        )
    }

    /// Register with Songbird
    ///
    /// Sends `ipc.register` to Songbird per `PRIMAL_IPC_PROTOCOL.md`
    ///
    /// # Errors
    ///
    /// Returns error if registration fails
    pub async fn register(&self, registration: &PrimalRegistration) -> Result<()> {
        let request_id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "ipc.register",
            "params": {
                "name": registration.name,
                "endpoint": registration.endpoint,
                "capabilities": registration.capabilities,
                "version": registration.version,
                "metadata": registration.metadata,
            },
            "id": request_id,
        });

        self.send_request(&request)
            .await
            .context("Failed to register with Songbird")
    }

    /// Send a heartbeat to Songbird
    ///
    /// Sends `ipc.heartbeat` to maintain registration per `PRIMAL_IPC_PROTOCOL.md`
    ///
    /// # Errors
    ///
    /// Returns error if heartbeat fails
    pub async fn heartbeat(&self, primal_name: &str) -> Result<()> {
        let request_id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "ipc.heartbeat",
            "params": {
                "name": primal_name,
            },
            "id": request_id,
        });

        self.send_request(&request)
            .await
            .context("Failed to send heartbeat to Songbird")
    }

    /// Send a JSON-RPC request to Songbird
    async fn send_request(&self, request: &Value) -> Result<()> {
        // Connect to Songbird
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to Songbird")?;

        // Send request (line-delimited JSON-RPC)
        let request_str = serde_json::to_string(request)?;
        stream
            .write_all(format!("{request_str}\n").as_bytes())
            .await?;
        stream.flush().await?;

        // Read response
        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .await
            .context("Failed to read response from Songbird")?;

        // Parse response
        let response: Value = serde_json::from_str(&response_line)
            .context("Failed to parse response from Songbird")?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            anyhow::bail!("Songbird returned error: {error}");
        }

        Ok(())
    }
}

impl Default for SongbirdClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Primal registration manager
///
/// Handles registration lifecycle:
/// - Initial registration on startup
/// - Periodic heartbeats
/// - Graceful handling when Songbird is unavailable
pub struct RegistrationManager {
    client: SongbirdClient,
    registration: PrimalRegistration,
    heartbeat_interval: Duration,
}

impl RegistrationManager {
    /// Create a new registration manager
    #[must_use]
    pub fn new(registration: PrimalRegistration) -> Self {
        Self {
            client: SongbirdClient::new(),
            registration,
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    /// Register with Songbird (if available)
    ///
    /// This should be called during primal startup.
    /// It will NOT fail if Songbird is unavailable - instead it will log a warning
    /// and allow the primal to continue operating.
    pub async fn register_on_startup(&self) {
        debug!("Checking if Songbird is available...");

        if !self.client.is_available().await {
            warn!(
                "Songbird not available at {}, continuing without registration",
                self.client.socket_path
            );
            warn!("Primal will operate standalone until Songbird becomes available");
            return;
        }

        info!("Songbird available, registering primal...");

        match self.client.register(&self.registration).await {
            Ok(()) => {
                info!(
                    "✅ Successfully registered '{}' with Songbird",
                    self.registration.name
                );
                info!("Capabilities: {:?}", self.registration.capabilities);
            }
            Err(e) => {
                error!("Failed to register with Songbird: {}", e);
                warn!("Continuing without registration (standalone mode)");
            }
        }
    }

    /// Start periodic heartbeat task
    ///
    /// Spawns a background task that sends heartbeats to Songbird
    /// at the configured interval.
    ///
    /// Returns a handle to the task that can be used to cancel it.
    pub fn spawn_heartbeat_task(&self) -> tokio::task::JoinHandle<()> {
        let client = self.client.clone();
        let primal_name = self.registration.name.clone();
        let interval = self.heartbeat_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                debug!("Sending heartbeat to Songbird...");

                match client.heartbeat(&primal_name).await {
                    Ok(()) => {
                        debug!("✅ Heartbeat successful");
                    }
                    Err(e) => {
                        warn!("Heartbeat failed: {} (Songbird may be unavailable)", e);
                    }
                }
            }
        })
    }
}

// Clone implementation for SongbirdClient (needed for spawning tasks)
impl Clone for SongbirdClient {
    fn clone(&self) -> Self {
        Self {
            socket_path: self.socket_path.clone(),
            request_id: std::sync::atomic::AtomicU64::new(
                self.request_id.load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_petaltongue_registration() {
        let reg = PrimalRegistration::petaltongue();

        assert_eq!(reg.name, "petaltongue");
        assert_eq!(reg.endpoint, "/primal/petaltongue");
        assert!(!reg.capabilities.is_empty());
        assert!(reg.capabilities.contains(&"ui.render".to_string()));
        assert!(reg.capabilities.contains(&"graph.topology".to_string()));
    }

    #[tokio::test]
    async fn test_songbird_unavailable() {
        let client = SongbirdClient::with_socket_path("/tmp/nonexistent-songbird.sock".to_string());
        let available = client.is_available().await;
        assert!(!available);
    }

    #[test]
    fn test_registration_metadata() {
        let reg = PrimalRegistration::petaltongue();
        assert!(reg.metadata.is_some());
        let meta = reg.metadata.as_ref().unwrap();
        assert!(meta.contains_key("description"));
        assert!(meta.contains_key("ui_modes"));
    }

    #[test]
    fn test_registration_serialization() {
        let reg = PrimalRegistration::petaltongue();
        let json = serde_json::to_string(&reg).unwrap();
        assert!(json.contains("petaltongue"));
        assert!(json.contains("ui.render"));
    }

    #[test]
    fn test_songbird_client_default() {
        let client = SongbirdClient::default();
        drop(client);
    }

    #[test]
    fn test_registration_manager_creation() {
        let reg = PrimalRegistration::petaltongue();
        let manager = RegistrationManager::new(reg);
        drop(manager);
    }
}
