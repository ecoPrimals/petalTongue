// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovered Display Backend (tarpc)
//!
//! TRUE PRIMAL Evolution: Discovery via capability system, Performance via tarpc
//!
//! # Two-Phase Architecture
//!
//! **Phase 1 - Discovery** (Once at startup, ~50ms):
//! ```text
//! petalTongue → CapabilityDiscovery → biomeOS
//!   Query: "Who provides 'display' capability?"
//!   Response: PrimalEndpoint { tarpc: "tarpc://unix:/run/display-provider.sock" }
//! ```
//!
//! **Phase 2 - Performance** (Continuous, ~5-8ms):
//! ```text
//! petalTongue ←─ tarpc (binary RPC) ─→ display capability provider
//!   • Frame commits: 60 FPS
//!   • Input events: real-time
//!   • GPU compute: as needed
//! ```
//!
//! # Evolution from JSON-RPC
//!
//! **Previous**: JSON-RPC for everything (slower, more overhead)
//! **Current**: JSON-RPC discovery → tarpc performance (best of both)
//!
//! # Self-Knowledge
//!
//! petalTongue KNOWS:
//! - I need: "display" capability
//! - I speak: tarpc (primary), JSON-RPC (fallback)
//!
//! petalTongue NEVER KNOWS:
//! - Which concrete primal implements the capability (only the endpoint from discovery)
//! - Where that primal is located (biomeOS provides the endpoint)
//! - The provider's internal implementation

use crate::display::traits::{DisplayBackend, DisplayCapabilities};
use crate::error::{DisplayError, Result};
use petal_tongue_core::{
    biomeos_discovery::BiomeOsBackend,
    capability_discovery::{CapabilityDiscovery, CapabilityQuery},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Display backend using JSON-RPC to the provider discovered for the `display` capability
/// via `CapabilityDiscovery<BiomeOsBackend>`.
///
/// Discovery uses biomeOS Neural API (capability-based, no hardcoded provider names).
/// Transport uses JSON-RPC 2.0 over Unix socket — the universal IPC protocol.
pub struct DiscoveredDisplayBackendV2 {
    /// Capability discovery system
    discovery: Option<CapabilityDiscovery<BiomeOsBackend>>,

    /// JSON-RPC socket path (discovered from primal endpoint)
    jsonrpc_socket: Option<String>,

    /// Window ID returned by the display capability provider
    window_id: Option<String>,

    /// Display dimensions
    width: u32,
    height: u32,

    /// JSON-RPC request counter
    request_id: std::sync::atomic::AtomicU64,
}

/// Display capabilities from the provider (queried over tarpc)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisplayCapabilitiesResponse {
    displays: Vec<DisplayInfo>,
    input_devices: Vec<InputDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisplayInfo {
    id: String,
    connector: String,
    resolution: Resolution,
    refresh_rate: f64,
    connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Resolution {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputDeviceInfo {
    id: String,
    name: String,
    #[serde(rename = "type")]
    device_type: String,
}

/// Window creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowResponse {
    window_id: String,
    buffer_handle: String,
}

#[must_use]
pub const fn expected_rgba8_buffer_size(width: u32, height: u32) -> usize {
    (width as usize) * (height as usize) * 4
}

impl DiscoveredDisplayBackendV2 {
    /// Create a new backend with biomeOS-backed capability discovery
    ///
    /// # Errors
    ///
    /// Returns an error if the biomeOS discovery backend cannot be created from environment.
    pub fn new() -> Result<Self> {
        let backend = BiomeOsBackend::from_env()
            .map_err(|e| DisplayError::BiomeOsDiscoveryBackend(e.to_string()))?;

        let discovery = CapabilityDiscovery::new(backend);

        Ok(Self {
            discovery: Some(discovery),
            jsonrpc_socket: None,
            window_id: None,
            width: 1920,
            height: 1080,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Create with explicit JSON-RPC socket path (for testing)
    #[must_use]
    pub fn with_socket(socket_path: impl Into<String>) -> Self {
        Self {
            discovery: None,
            jsonrpc_socket: Some(socket_path.into()),
            window_id: None,
            width: 1920,
            height: 1080,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Discover the display capability provider and extract its JSON-RPC socket
    async fn discover_and_connect(&mut self) -> Result<()> {
        info!("Discovering display capability provider via biomeOS...");

        let discovery = self
            .discovery
            .as_ref()
            .ok_or(DisplayError::NoDiscoverySystem)?;

        let endpoint = discovery
            .discover_one(&CapabilityQuery::new("display"))
            .await
            .map_err(|e| DisplayError::DisplayDiscoveryFailed(e.to_string()))?;

        info!("Found display provider: {}", endpoint.id);
        debug!("Capabilities: {:?}", endpoint.capabilities);

        // Prefer JSON-RPC socket (universal IPC), fall back to tarpc endpoint string
        let socket = endpoint
            .endpoints
            .jsonrpc
            .or(endpoint.endpoints.tarpc)
            .ok_or(DisplayError::NoTarpcEndpoint)?;

        info!("Display provider socket: {socket}");
        self.jsonrpc_socket = Some(socket);

        Ok(())
    }

    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Send a JSON-RPC 2.0 request to the discovered display provider
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let socket_path = self
            .jsonrpc_socket
            .as_ref()
            .ok_or(DisplayError::NotConnectedToDisplay)?;

        let mut stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| DisplayError::BiomeOsConnect {
                path: socket_path.clone(),
                detail: e.to_string(),
            })?;

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_request_id(),
        });

        let request_str = serde_json::to_string(&request)
            .map_err(|e| DisplayError::BiomeOsParseJsonRpc(e.to_string()))?;

        stream
            .write_all(format!("{request_str}\n").as_bytes())
            .await
            .map_err(|e| DisplayError::BiomeOsConnect {
                path: socket_path.clone(),
                detail: e.to_string(),
            })?;
        stream.flush().await.map_err(|e| DisplayError::BiomeOsConnect {
            path: socket_path.clone(),
            detail: e.to_string(),
        })?;

        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| DisplayError::BiomeOsReadResponse(e.to_string()))?;

        let response: Value = serde_json::from_str(&response_line)
            .map_err(|e| DisplayError::BiomeOsParseJsonRpc(e.to_string()))?;

        if let Some(error) = response.get("error") {
            return Err(DisplayError::BiomeOsError(error.to_string()).into());
        }

        response
            .get("result")
            .cloned()
            .ok_or(DisplayError::BiomeOsNoResult)
            .map_err(Into::into)
    }

    /// Query display capabilities via JSON-RPC
    async fn query_capabilities(&self) -> Result<DisplayCapabilitiesResponse> {
        info!("Querying display capabilities via JSON-RPC...");

        let result = self
            .send_request("display.query_capabilities", json!({}))
            .await?;

        let caps: DisplayCapabilitiesResponse = serde_json::from_value(result)
            .map_err(|e| DisplayError::ParseDisplayCapabilities(e.to_string()))?;

        info!(
            "Found {} displays, {} input devices",
            caps.displays.len(),
            caps.input_devices.len()
        );

        Ok(caps)
    }

    /// Create window via JSON-RPC
    async fn create_window(&self, title: &str, width: u32, height: u32) -> Result<WindowResponse> {
        info!("Creating {width}x{height} window via JSON-RPC...");

        let params = json!({
            "title": title,
            "width": width,
            "height": height,
        });

        let result = self
            .send_request("display.create_window", params)
            .await?;

        let window: WindowResponse = serde_json::from_value(result)
            .map_err(|e| DisplayError::ParseWindowResponse(e.to_string()))?;

        info!("Window created: {}", window.window_id);

        Ok(window)
    }

    /// Commit frame via JSON-RPC (base64-encoded RGBA)
    async fn commit_frame(&self, buffer: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        let window_id = self
            .window_id
            .as_ref()
            .ok_or(DisplayError::NoWindowCreated)?;

        let encoded = general_purpose::STANDARD.encode(buffer);

        let params = json!({
            "window_id": window_id,
            "format": "rgba8",
            "width": self.width,
            "height": self.height,
            "data": encoded,
        });

        debug!("Committing frame via JSON-RPC ({} bytes)", buffer.len());

        self.send_request("display.commit_frame", params).await?;

        debug!("Frame committed");

        Ok(())
    }
}

impl DisplayBackend for DiscoveredDisplayBackendV2 {
    async fn init(&mut self) -> Result<()> {
        info!("Initializing discovered display backend (capability discovery + JSON-RPC)...");

        self.discover_and_connect().await?;

        let caps = self.query_capabilities().await?;

        let display_info = caps
            .displays
            .first()
            .ok_or(DisplayError::NoDisplaysAvailable)?;

        info!(
            "Display: {} ({}) {}x{} @ {}Hz",
            display_info.connector,
            display_info.id,
            display_info.resolution.width,
            display_info.resolution.height,
            display_info.refresh_rate
        );

        self.width = display_info.resolution.width;
        self.height = display_info.resolution.height;

        let window = self
            .create_window("petalTongue UI", self.width, self.height)
            .await?;
        self.window_id = Some(window.window_id.clone());

        info!(
            "Discovered display backend initialized: window={}, {}x{}, transport=JSON-RPC/UDS",
            window.window_id, self.width, self.height
        );

        Ok(())
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        let expected_size = expected_rgba8_buffer_size(self.width, self.height);
        if buffer.len() != expected_size {
            return Err(DisplayError::InvalidBufferSize {
                expected: expected_size,
                actual: buffer.len(),
            }
            .into());
        }

        self.commit_frame(buffer).await
    }

    fn is_available() -> bool {
        BiomeOsBackend::from_env().is_ok()
    }

    fn name(&self) -> &'static str {
        "Discovered Display (capability discovery)"
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities {
            requires_network: false,
            requires_gpu: false,
            requires_root: false,
            supports_resize: true,
            max_fps: 60,
            latency_ms: 10,
            requires_display_server: false,
            remote_capable: true,
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down discovered display backend (V2)");

        if let Some(window_id) = &self.window_id {
            let params = json!({ "window_id": window_id });

            match self.send_request("display.destroy_window", params).await {
                Ok(_) => info!("Window {window_id} destroyed"),
                Err(e) => warn!("Failed to destroy window {window_id} (non-fatal): {e}"),
            }
        }

        self.window_id = None;
        self.jsonrpc_socket = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_display() -> DiscoveredDisplayBackendV2 {
        DiscoveredDisplayBackendV2 {
            discovery: None,
            jsonrpc_socket: None,
            window_id: None,
            width: 1920,
            height: 1080,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    #[test]
    fn test_discovered_display_v2_capabilities() {
        let display = test_display();
        let caps = display.capabilities();
        assert!(!caps.requires_network);
        assert!(!caps.requires_gpu);
        assert!(!caps.requires_display_server);
        assert!(caps.remote_capable);
        assert_eq!(caps.max_fps, 60);
    }

    #[test]
    fn test_dimensions() {
        let display = test_display();
        assert_eq!(display.dimensions(), (1920, 1080));
    }

    #[test]
    fn test_expected_rgba8_buffer_size() {
        assert_eq!(expected_rgba8_buffer_size(1920, 1080), 1920 * 1080 * 4);
        assert_eq!(expected_rgba8_buffer_size(800, 600), 800 * 600 * 4);
        assert_eq!(expected_rgba8_buffer_size(1, 1), 4);
    }

    #[test]
    fn test_with_socket() {
        let display = DiscoveredDisplayBackendV2::with_socket("/tmp/test-display.sock");
        assert_eq!(display.dimensions(), (1920, 1080));
        assert!(display.name().contains("capability discovery"));
    }

    #[test]
    fn test_name() {
        let display = test_display();
        assert!(display.name().contains("capability discovery"));
    }

    #[test]
    fn test_capabilities_latency() {
        let display = test_display();
        let caps = display.capabilities();
        assert_eq!(caps.latency_ms, 10);
    }

    #[tokio::test]
    async fn test_present_invalid_buffer_size() {
        let mut display = DiscoveredDisplayBackendV2::with_socket("/tmp/nonexistent.sock");
        let wrong_buffer = vec![0u8; 100];
        let result = display.present(&wrong_buffer).await;
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("InvalidBufferSize")
                    || msg.contains("expected")
                    || msg.contains("buffer"),
                "expected buffer error: {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_present_buffer_too_small() {
        let mut display = DiscoveredDisplayBackendV2::with_socket("/tmp/nonexistent.sock");
        let expected = expected_rgba8_buffer_size(1920, 1080);
        let too_small = vec![0u8; expected - 1];
        let result = display.present(&too_small).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_no_window() {
        let mut display = DiscoveredDisplayBackendV2::with_socket("/tmp/nonexistent.sock");
        let result = display.shutdown().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_available() {
        let _ = DiscoveredDisplayBackendV2::is_available();
    }

    #[test]
    fn test_expected_rgba8_edge_cases() {
        assert_eq!(expected_rgba8_buffer_size(0, 0), 0);
        assert_eq!(expected_rgba8_buffer_size(1, 1), 4);
        assert_eq!(expected_rgba8_buffer_size(100, 100), 40_000);
    }
}
