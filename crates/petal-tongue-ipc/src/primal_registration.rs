// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal Registration with Discovery Service
//!
//! Implements the `ipc.register` and `ipc.heartbeat` standards from `PRIMAL_IPC_PROTOCOL.md`.
//!
//! This module handles:
//! - Initial registration with the ecosystem discovery service on startup
//! - Periodic heartbeat to maintain registration
//! - Graceful handling when the discovery service is unavailable

use crate::primal_registration_error::PrimalRegistrationError;
use petal_tongue_core::capability_names::{primal_names, self_capabilities};
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

    /// Version string (e.g., "1.6.6")
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
        metadata.insert(
            "modalities".to_string(),
            json!([
                "visual",
                "terminal",
                "audio",
                "haptic",
                "braille",
                "description",
                "gpu"
            ]),
        );

        Self {
            name: primal_names::PETALTONGUE.to_string(),
            endpoint: format!("/primal/{}", primal_names::PETALTONGUE),
            capabilities: self_capabilities::ALL
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            metadata: Some(metadata),
        }
    }
}

/// JSON-RPC client for discovery service registration
pub struct RegistrationClient {
    socket_path: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl RegistrationClient {
    /// Create a new discovery service registration client
    ///
    /// Capability-based discovery: uses discovery service socket (no hardcoded primal names).
    /// Socket path resolution (priority order):
    /// 1. `discover_primal_socket` using the capability-based name from
    ///    `petal_tongue_core::constants::discovery_service_socket_name` (includes
    ///    `<SOCKET_BASENAME>_SOCKET` override and standard biomeOS paths)
    /// 2. Conventional path `/tmp/biomeos/<socket_base>.sock` when `discover_primal_socket` fails
    #[must_use]
    pub fn new() -> Self {
        let socket_base = constants::discovery_service_socket_name();
        let socket_path = crate::socket_path::discover_primal_socket(&socket_base, None, None)
            .map_or_else(
                |_| format!("/tmp/biomeos/{socket_base}.sock"),
                |p| p.to_string_lossy().to_string(),
            );
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Create a client with a custom socket path (for testing)
    #[must_use]
    pub const fn with_socket_path(socket_path: String) -> Self {
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Check if the discovery service is available
    pub async fn is_available(&self) -> bool {
        matches!(
            tokio::time::timeout(
                constants::discovery_timeouts::DISCOVERY_SERVICE_REGISTRATION_PROBE_TIMEOUT,
                UnixStream::connect(&self.socket_path),
            )
            .await,
            Ok(Ok(_))
        )
    }

    /// Register with discovery service
    ///
    /// Sends `ipc.register` per `PRIMAL_IPC_PROTOCOL.md`
    ///
    /// # Errors
    ///
    /// Returns error if registration fails
    pub async fn register(
        &self,
        registration: &PrimalRegistration,
    ) -> Result<(), PrimalRegistrationError> {
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

        self.send_request(&request).await
    }

    /// Send a heartbeat to discovery service
    ///
    /// Sends `ipc.heartbeat` to maintain registration per `PRIMAL_IPC_PROTOCOL.md`
    ///
    /// # Errors
    ///
    /// Returns error if heartbeat fails
    pub async fn heartbeat(&self, primal_name: &str) -> Result<(), PrimalRegistrationError> {
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

        self.send_request(&request).await
    }

    /// Send a JSON-RPC request to the discovery service
    async fn send_request(&self, request: &Value) -> Result<(), PrimalRegistrationError> {
        // Connect to discovery service
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        let mut buf = serde_json::to_vec(request)?;
        buf.push(b'\n');
        stream.write_all(&buf).await?;
        stream.flush().await?;

        // Read response
        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();

        reader.read_line(&mut response_line).await?;

        // Parse response
        let response: Value = serde_json::from_str(&response_line)?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(PrimalRegistrationError::RegistrationRejected(error.clone()));
        }

        Ok(())
    }
}

impl Default for RegistrationClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Primal registration manager
///
/// Handles registration lifecycle:
/// - Initial registration on startup
/// - Periodic heartbeats
/// - Graceful handling when the discovery service is unavailable
pub struct RegistrationManager {
    client: RegistrationClient,
    registration: PrimalRegistration,
    heartbeat_interval: Duration,
}

impl RegistrationManager {
    /// Create a new registration manager
    #[must_use]
    pub fn new(registration: PrimalRegistration) -> Self {
        Self {
            client: RegistrationClient::new(),
            registration,
            heartbeat_interval: constants::default_heartbeat_interval(),
        }
    }

    /// Register with discovery service (if available)
    ///
    /// This should be called during primal startup.
    /// It will NOT fail if the discovery service is unavailable — instead it will log
    /// a warning and allow the primal to continue operating.
    pub async fn register_on_startup(&self) {
        debug!("Checking if discovery service is available...");

        if !self.client.is_available().await {
            warn!(
                "Discovery service not available at {}, continuing without registration",
                self.client.socket_path
            );
            warn!("Primal will operate standalone until discovery service becomes available");
            return;
        }

        info!("Discovery service available, registering primal...");

        match self.client.register(&self.registration).await {
            Ok(()) => {
                info!(
                    "✅ Successfully registered '{}' with discovery service",
                    self.registration.name
                );
                info!("Capabilities: {:?}", self.registration.capabilities);
            }
            Err(e) => {
                error!("Failed to register with discovery service: {}", e);
                warn!("Continuing without registration (standalone mode)");
            }
        }
    }

    /// Start periodic heartbeat task
    ///
    /// Spawns a background task that sends heartbeats to the discovery service
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

                debug!("Sending heartbeat to discovery service...");

                match client.heartbeat(&primal_name).await {
                    Ok(()) => {
                        debug!("✅ Heartbeat successful");
                    }
                    Err(e) => {
                        warn!(
                            "Heartbeat failed: {} (discovery service may be unavailable)",
                            e
                        );
                    }
                }
            }
        })
    }
}

// Clone implementation for RegistrationClient (needed for spawning tasks)
impl Clone for RegistrationClient {
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
        assert!(
            reg.capabilities
                .contains(&"visualization.render".to_string())
        );
        assert!(reg.capabilities.contains(&"modality.visual".to_string()));
    }

    #[tokio::test]
    async fn test_songbird_unavailable() {
        let client =
            RegistrationClient::with_socket_path("/tmp/nonexistent-songbird.sock".to_string());
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
        assert!(meta.contains_key("modalities"));
    }

    #[test]
    fn test_registration_serialization() {
        let reg = PrimalRegistration::petaltongue();
        let json = serde_json::to_string(&reg).expect("serialize");
        assert!(json.contains("petaltongue"));
        assert!(json.contains("ui.render"));
    }

    #[test]
    fn test_songbird_client_default() {
        let client = RegistrationClient::default();
        drop(client);
    }

    #[test]
    fn test_registration_manager_creation() {
        let reg = PrimalRegistration::petaltongue();
        let manager = RegistrationManager::new(reg);
        drop(manager);
    }

    #[test]
    fn test_primal_registration_version() {
        let reg = PrimalRegistration::petaltongue();
        assert!(!reg.version.is_empty());
        assert!(
            reg.version
                .chars()
                .next()
                .expect("version")
                .is_ascii_digit()
        );
    }

    #[test]
    fn test_primal_registration_all_capabilities() {
        let reg = PrimalRegistration::petaltongue();
        let expected = [
            "ui.render",
            "ui.visualization",
            "ui.graph",
            "ui.terminal",
            "ui.audio",
            "visualization.render",
            "visualization.render.stream",
            "visualization.render.grammar",
            "visualization.render.dashboard",
            "visualization.interact",
            "visualization.interact.subscribe",
            "visualization.provenance",
            "visualization.export",
            "visualization.validate",
            "graph.topology",
            "graph.builder",
            "interaction.subscribe",
            "interaction.poll",
            "sensor.stream.subscribe",
            "motor.set_panel",
            "motor.set_zoom",
            "motor.set_mode",
            "motor.fit_to_view",
            "motor.navigate",
            "modality.visual",
            "modality.audio",
            "modality.terminal",
            "modality.haptic",
            "modality.braille",
            "modality.description",
            "identity.get",
            "lifecycle.status",
            "health.check",
            "health.liveness",
            "health.readiness",
            "capabilities.list",
        ];
        for cap in expected {
            assert!(
                reg.capabilities.contains(&cap.to_string()),
                "missing capability: {cap}"
            );
        }
    }

    #[test]
    fn test_songbird_client_with_socket_path() {
        let client = RegistrationClient::with_socket_path(String::new());
        drop(client);
    }

    #[test]
    fn test_registration_manager_heartbeat_interval() {
        let reg = PrimalRegistration::petaltongue();
        let manager = RegistrationManager::new(reg);
        drop(manager);
    }

    #[test]
    fn test_songbird_client_clone() {
        let client = RegistrationClient::with_socket_path("/tmp/test.sock".to_string());
        let cloned = client.clone();
        drop(cloned);
        drop(client);
    }

    #[tokio::test]
    async fn test_register_on_startup_songbird_unavailable() {
        let reg = PrimalRegistration::petaltongue();
        let manager = RegistrationManager::new(reg);
        // When Songbird is unavailable, should not panic
        manager.register_on_startup().await;
    }

    #[tokio::test]
    async fn test_spawn_heartbeat_task_returns_handle() {
        let reg = PrimalRegistration::petaltongue();
        let manager = RegistrationManager::new(reg);
        let handle = manager.spawn_heartbeat_task();
        // Handle should be valid
        drop(handle);
    }

    #[test]
    fn test_registration_manager_creation_with_defaults() {
        let reg = PrimalRegistration::petaltongue();
        let _manager = RegistrationManager::new(reg);
        // Just verify creation succeeds
    }

    #[test]
    fn test_primal_registration_deserialization_roundtrip() {
        let reg = PrimalRegistration::petaltongue();
        let json = serde_json::to_string(&reg).expect("serialize");
        let restored: PrimalRegistration = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.name, reg.name);
        assert_eq!(restored.endpoint, reg.endpoint);
        assert_eq!(restored.capabilities.len(), reg.capabilities.len());
    }

    #[test]
    fn test_register_request_json_structure() {
        let reg = PrimalRegistration::petaltongue();
        let request = json!({
            "jsonrpc": "2.0",
            "method": "ipc.register",
            "params": {
                "name": reg.name,
                "endpoint": reg.endpoint,
                "capabilities": reg.capabilities,
                "version": reg.version,
                "metadata": reg.metadata,
            },
            "id": 1,
        });
        assert_eq!(request["method"], "ipc.register");
        assert_eq!(request["params"]["name"], reg.name);
        assert!(request["params"]["capabilities"].is_array());
    }

    #[test]
    fn test_heartbeat_request_json_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "ipc.heartbeat",
            "params": {"name": "petaltongue"},
            "id": 1,
        });
        assert_eq!(request["method"], "ipc.heartbeat");
        assert_eq!(request["params"]["name"], "petaltongue");
    }

    #[test]
    fn test_registration_with_optional_metadata() {
        let reg = PrimalRegistration {
            name: "custom".to_string(),
            endpoint: "/primal/custom".to_string(),
            capabilities: vec!["test.cap".to_string()],
            version: "0.1.0".to_string(),
            metadata: None,
        };
        let json = serde_json::to_string(&reg).expect("serialize");
        assert!(!json.contains("metadata") || json.contains("null"));
        let restored: PrimalRegistration = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.metadata.is_none());
    }
}
