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
use async_trait::async_trait;
use petal_tongue_core::{
    biomeos_discovery::BiomeOsBackend,
    capability_discovery::{CapabilityDiscovery, CapabilityQuery},
};
use petal_tongue_ipc::tarpc_client::TarpcClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Display backend using tarpc to the provider discovered for the `display` capability (biomeOS discovery).
pub struct DiscoveredDisplayBackendV2 {
    /// Capability discovery system
    discovery: Option<CapabilityDiscovery>,

    /// tarpc client (high-performance binary RPC)
    tarpc_client: Option<TarpcClient>,

    /// Window ID returned by the display capability provider
    window_id: Option<String>,

    /// Display dimensions
    width: u32,
    height: u32,
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
        // Create discovery system with biomeOS backend
        let backend = BiomeOsBackend::from_env()
            .map_err(|e| DisplayError::BiomeOsDiscoveryBackend(e.to_string()))?;

        let discovery = CapabilityDiscovery::new(Box::new(backend));

        Ok(Self {
            discovery: Some(discovery),
            tarpc_client: None,
            window_id: None,
            width: 1920,
            height: 1080,
        })
    }

    /// Create with explicit tarpc client (for testing)
    #[must_use]
    pub const fn with_client(client: TarpcClient) -> Self {
        Self {
            discovery: None,
            tarpc_client: Some(client),
            window_id: None,
            width: 1920,
            height: 1080,
        }
    }

    /// Discover and connect to the display capability provider via tarpc
    async fn discover_and_connect(&mut self) -> Result<()> {
        info!("🌸 Discovering display capability provider via biomeOS...");

        let discovery = self
            .discovery
            .as_ref()
            .ok_or(DisplayError::NoDiscoverySystem)?;

        // Query for display capability (no hardcoded provider name)
        let endpoint = discovery
            .discover_one(&CapabilityQuery::new("display"))
            .await
            .map_err(|e| DisplayError::DisplayDiscoveryFailed(e.to_string()))?;

        info!("✅ Found display provider: {}", endpoint.id);
        info!("   Capabilities: {:?}", endpoint.capabilities);

        // Extract tarpc endpoint
        let tarpc_endpoint = endpoint
            .endpoints
            .tarpc
            .ok_or(DisplayError::NoTarpcEndpoint)?;

        info!("🔌 Connecting via tarpc: {}", tarpc_endpoint);

        // Connect via tarpc for high-performance communication
        // Note: TarpcClient::new() creates client with lazy connection
        let client = TarpcClient::new(&tarpc_endpoint)
            .map_err(|e| DisplayError::TarpcClientCreation(e.to_string()))?;

        self.tarpc_client = Some(client);

        info!("✅ Connected to display provider via tarpc");

        Ok(())
    }

    /// Get tarpc client (ensures connected)
    fn client(&self) -> Result<&TarpcClient> {
        self.tarpc_client
            .as_ref()
            .ok_or(DisplayError::NotConnectedToDisplay)
            .map_err(Into::into)
    }

    /// Query display capabilities via tarpc
    async fn query_capabilities(&self) -> Result<DisplayCapabilitiesResponse> {
        info!("🌸 Querying display capabilities via tarpc...");

        let result = self
            .client()?
            .call_method("display.query_capabilities", Some(serde_json::json!({})))
            .await
            .map_err(|e| DisplayError::QueryCapabilitiesFailed(e.to_string()))?;

        let caps: DisplayCapabilitiesResponse = serde_json::from_value(result)
            .map_err(|e| DisplayError::ParseDisplayCapabilities(e.to_string()))?;

        info!(
            "✅ Found {} displays, {} input devices",
            caps.displays.len(),
            caps.input_devices.len()
        );

        Ok(caps)
    }

    /// Create window via tarpc
    async fn create_window(&self, title: &str, width: u32, height: u32) -> Result<WindowResponse> {
        info!("🌸 Creating {}x{} window via tarpc...", width, height);

        let params = serde_json::json!({
            "title": title,
            "width": width,
            "height": height,
        });

        let result = self
            .client()?
            .call_method("display.create_window", Some(params))
            .await
            .map_err(|e| DisplayError::CreateWindowFailed(e.to_string()))?;

        let window: WindowResponse = serde_json::from_value(result)
            .map_err(|e| DisplayError::ParseWindowResponse(e.to_string()))?;

        info!("✅ Window created: {}", window.window_id);

        Ok(window)
    }

    /// Commit frame via tarpc (high-performance binary RPC)
    async fn commit_frame(&self, buffer: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        let window_id = self
            .window_id
            .as_ref()
            .ok_or(DisplayError::NoWindowCreated)?;

        // tarpc can handle binary data efficiently
        // For now, we use base64 encoding for compatibility
        let encoded = general_purpose::STANDARD.encode(buffer);

        let params = serde_json::json!({
            "window_id": window_id,
            "format": "rgba8",
            "width": self.width,
            "height": self.height,
            "data": encoded,
        });

        debug!("🎨 Committing frame via tarpc ({} bytes)", buffer.len());

        self.client()?
            .call_method("display.commit_frame", Some(params))
            .await
            .map_err(|e| DisplayError::CommitFrameFailed(e.to_string()))?;

        debug!("✅ Frame committed");

        Ok(())
    }
}

#[async_trait]
impl DisplayBackend for DiscoveredDisplayBackendV2 {
    async fn init(&mut self) -> Result<()> {
        info!("🌸🦈 Initializing discovered display backend (tarpc)...");

        // Phase 1: Discover display provider via capability system
        self.discover_and_connect().await?;

        // Phase 2: Query capabilities via tarpc (high-performance)
        let caps = self.query_capabilities().await?;

        // Select primary display
        let display_info = caps
            .displays
            .first()
            .ok_or(DisplayError::NoDisplaysAvailable)?;

        info!(
            "   Display: {} ({})",
            display_info.connector, display_info.id
        );
        info!(
            "   Resolution: {}x{} @ {}Hz",
            display_info.resolution.width,
            display_info.resolution.height,
            display_info.refresh_rate
        );

        // Update dimensions from actual display
        self.width = display_info.resolution.width;
        self.height = display_info.resolution.height;

        // Create window via tarpc
        let window = self
            .create_window("petalTongue UI", self.width, self.height)
            .await?;
        self.window_id = Some(window.window_id.clone());

        info!("✅ Discovered display backend initialized (tarpc)");
        info!("   Window: {}", window.window_id);
        info!("   Dimensions: {}x{}", self.width, self.height);
        info!("   Transport: tarpc (high-performance binary RPC)");

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
        // Try to create discovery system
        BiomeOsBackend::from_env().is_ok()
    }

    fn name(&self) -> &'static str {
        "Discovered Display (tarpc)"
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities {
            requires_network: false,
            requires_gpu: false,
            requires_root: false,
            supports_resize: true,
            max_fps: 60,
            latency_ms: 8, // Improved with tarpc
            requires_display_server: false,
            remote_capable: true,
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("🌸 Shutting down discovered display backend (tarpc)");

        // Destroy window via tarpc if connected
        if let Some(window_id) = &self.window_id {
            info!("   Destroying window: {}", window_id);

            if let Some(client) = &self.tarpc_client {
                let params = serde_json::json!({
                    "window_id": window_id,
                });

                // Best-effort window destruction (don't fail shutdown if this fails)
                match client
                    .call_method("display.destroy_window", Some(params))
                    .await
                {
                    Ok(_) => info!("   ✅ Window destroyed"),
                    Err(e) => warn!("   ⚠️ Failed to destroy window (non-fatal): {}", e),
                }
            }
        }

        self.window_id = None;
        self.tarpc_client = None;

        info!("✅ Discovered display backend (tarpc) shutdown complete");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_display_v2_capabilities() {
        let display = DiscoveredDisplayBackendV2 {
            discovery: None,
            tarpc_client: None,
            window_id: None,
            width: 1920,
            height: 1080,
        };

        let caps = display.capabilities();
        assert!(!caps.requires_network);
        assert!(!caps.requires_gpu);
        assert!(!caps.requires_display_server);
        assert!(caps.remote_capable);
        assert_eq!(caps.max_fps, 60);
    }

    #[test]
    fn test_dimensions() {
        let display = DiscoveredDisplayBackendV2 {
            discovery: None,
            tarpc_client: None,
            window_id: None,
            width: 1920,
            height: 1080,
        };

        assert_eq!(display.dimensions(), (1920, 1080));
    }

    #[test]
    fn test_expected_rgba8_buffer_size() {
        assert_eq!(expected_rgba8_buffer_size(1920, 1080), 1920 * 1080 * 4);
        assert_eq!(expected_rgba8_buffer_size(800, 600), 800 * 600 * 4);
        assert_eq!(expected_rgba8_buffer_size(1, 1), 4);
    }

    #[test]
    fn test_with_client() {
        use petal_tongue_ipc::tarpc_client::TarpcClient;
        let client = TarpcClient::new("tarpc://localhost:9999").unwrap();
        let display = DiscoveredDisplayBackendV2::with_client(client);
        assert_eq!(display.dimensions(), (1920, 1080));
        assert_eq!(display.name(), "Discovered Display (tarpc)");
    }

    #[test]
    fn test_name() {
        let display = DiscoveredDisplayBackendV2 {
            discovery: None,
            tarpc_client: None,
            window_id: None,
            width: 1920,
            height: 1080,
        };
        assert!(display.name().contains("tarpc"));
    }

    #[test]
    fn test_capabilities_latency() {
        let display = DiscoveredDisplayBackendV2 {
            discovery: None,
            tarpc_client: None,
            window_id: None,
            width: 1920,
            height: 1080,
        };
        let caps = display.capabilities();
        assert_eq!(caps.latency_ms, 8);
    }

    #[tokio::test]
    async fn test_present_invalid_buffer_size() {
        use petal_tongue_ipc::tarpc_client::TarpcClient;
        let client = TarpcClient::new("tarpc://localhost:9999").unwrap();
        let mut display = DiscoveredDisplayBackendV2::with_client(client);
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
        use petal_tongue_ipc::tarpc_client::TarpcClient;
        let client = TarpcClient::new("tarpc://localhost:9999").unwrap();
        let mut display = DiscoveredDisplayBackendV2::with_client(client);
        let expected = expected_rgba8_buffer_size(1920, 1080);
        let too_small = vec![0u8; expected - 1];
        let result = display.present(&too_small).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_no_window() {
        use petal_tongue_ipc::tarpc_client::TarpcClient;
        let client = TarpcClient::new("tarpc://localhost:9999").unwrap();
        let mut display = DiscoveredDisplayBackendV2::with_client(client);
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
