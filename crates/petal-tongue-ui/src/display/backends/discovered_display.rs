// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovered Display Backend (via biomeOS capability query)
//!
//! TRUE PRIMAL Architecture: Discovery via biomeOS, Performance via tarpc
//!
//! # Architecture Principle
//!
//! **Discovery Phase** (Once at startup):
//! - petalTongue asks biomeOS: "Who provides 'display' capability?"
//! - biomeOS responds with the chosen provider's tarpc endpoint
//!
//! **Performance Phase** (Continuous):
//! - petalTongue ←─ tarpc ─→ display capability provider (direct binary RPC)
//! - Frame commits: 60 FPS (~5-8ms)
//! - Input events: real-time (~2-5ms)
//!
//! # Self-Knowledge
//!
//! petalTongue KNOWS:
//! - I need: "display", "input", "gpu.compute" capabilities
//! - I speak: JSON-RPC (discovery), tarpc (performance)
//!
//! petalTongue NEVER KNOWS:
//! - Which concrete primal implements `display` (only the endpoint from discovery)
//! - Where that primal runs (biomeOS provides the endpoint)
//!
//! # Integration Status
//!
//! ✅ Discovery: Via biomeOS (JSON-RPC, capability-based)
//! ✅ Performance: Via tarpc (direct, high-speed)
//! ✅ Display Runtime - DRM-based, Pure Rust, ARM64 + `x86_64`
//! ✅ Input System - Multi-touch (10+ fingers), Keyboard, Mouse
//! ✅ GPU Compute — capability-discovered compute stack (where advertised)
//!
//! # Reference
//!
//! See `wateringHole/fossilRecord/petaltongue-apr2026-archive/specs-archive/` for legacy integration notes (same capability-based discovery model).

use crate::display::traits::{DisplayBackend, DisplayCapabilities};
use crate::error::{DisplayError, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info};

/// Display backend discovered via biomeOS: JSON-RPC on the biomeOS socket routes to
/// whichever primal currently provides the `display` capability (no hardcoded provider name).
pub struct DiscoveredDisplayBackend {
    /// biomeOS socket path
    biomeos_socket: std::path::PathBuf,
    /// Window ID (from `display.create_window`)
    window_id: Option<String>,
    /// Buffer handle from the display capability provider
    buffer_handle: Option<String>,
    /// Display dimensions
    width: u32,
    height: u32,
    /// Request ID counter (for JSON-RPC)
    request_id: std::sync::atomic::AtomicU64,
}

/// Display capabilities response from the provider (via biomeOS)
#[derive(Debug, Clone, Deserialize)]
struct DisplayCapabilitiesResponse {
    displays: Vec<DisplayInfo>,
    input_devices: Vec<InputDeviceInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct DisplayInfo {
    id: String,
    connector: String,
    resolution: Resolution,
    refresh_rate: f64,
    connected: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct Resolution {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct InputDeviceInfo {
    id: String,
    name: String,
    #[serde(rename = "type")]
    device_type: String,
}

/// Window creation response
#[derive(Debug, Clone, Deserialize)]
struct WindowResponse {
    window_id: String,
    buffer_handle: String,
}

#[must_use]
pub const fn expected_rgba8_buffer_size(width: u32, height: u32) -> usize {
    (width as usize) * (height as usize) * 4
}

impl DiscoveredDisplayBackend {
    /// Create a new backend instance (discovers biomeOS socket from environment / defaults)
    ///
    /// # Errors
    ///
    /// Returns an error if biomeOS socket cannot be discovered.
    pub fn new() -> Result<Self> {
        let biomeos_socket = Self::discover_biomeos_socket()?;

        Ok(Self {
            biomeos_socket,
            window_id: None,
            buffer_handle: None,
            width: 1920,
            height: 1080,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Create with explicit biomeOS socket path
    pub fn with_socket(socket_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            biomeos_socket: socket_path.into(),
            window_id: None,
            buffer_handle: None,
            width: 1920,
            height: 1080,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Discover biomeOS socket path
    fn discover_biomeos_socket() -> Result<std::path::PathBuf> {
        // 1. Environment variable
        if let Ok(path) = std::env::var("BIOMEOS_SOCKET") {
            return Ok(path.into());
        }

        // 2. XDG runtime directory
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let path = std::path::PathBuf::from(runtime_dir).join(format!(
                "{}.sock",
                petal_tongue_core::constants::biomeos_socket_name()
            ));
            if path.exists() {
                return Ok(path);
            }
        }

        // 3. Fallback: capability-based discovery via petal-tongue-core constants
        Ok(petal_tongue_core::constants::biomeos_legacy_socket())
    }

    /// Send JSON-RPC request to biomeOS
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        // Connect to biomeOS
        let mut stream = UnixStream::connect(&self.biomeos_socket)
            .await
            .map_err(|e| DisplayError::BiomeOsConnect {
                path: self.biomeos_socket.display().to_string(),
                detail: e.to_string(),
            })?;

        // Prepare JSON-RPC 2.0 request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_request_id(),
        });

        // Send request (line-delimited JSON-RPC)
        let request_str = serde_json::to_string(&request)?;
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
            .map_err(|e| DisplayError::BiomeOsReadResponse(e.to_string()))?;

        // Parse response
        let response: Value = serde_json::from_str(&response_line)
            .map_err(|e| DisplayError::BiomeOsParseJsonRpc(e.to_string()))?;

        // Check for error
        if let Some(error) = response.get("error") {
            return Err(DisplayError::BiomeOsError(error.to_string()).into());
        }

        // Extract result
        response
            .get("result")
            .cloned()
            .ok_or(DisplayError::BiomeOsNoResult)
            .map_err(Into::into)
    }

    /// Get next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Query display capabilities via biomeOS (capability-based, provider-agnostic)
    async fn query_capabilities(&self) -> Result<DisplayCapabilitiesResponse> {
        info!("🌸 Querying display capabilities via biomeOS...");

        let result = self
            .send_request("display.query_capabilities", json!({}))
            .await?;

        let caps: DisplayCapabilitiesResponse = serde_json::from_value(result)
            .map_err(|e| DisplayError::ParseDisplayCapabilities(e.to_string()))?;

        info!(
            "✅ Found {} displays, {} input devices",
            caps.displays.len(),
            caps.input_devices.len()
        );

        Ok(caps)
    }

    /// Create window via biomeOS display capability provider
    async fn create_window(&self, title: &str, width: u32, height: u32) -> Result<WindowResponse> {
        info!("🌸 Creating {width}x{height} window via biomeOS display provider...");

        let params = json!({
            "title": title,
            "width": width,
            "height": height,
        });

        let result = self.send_request("display.create_window", params).await?;

        let window: WindowResponse = serde_json::from_value(result)
            .map_err(|e| DisplayError::ParseWindowResponse(e.to_string()))?;

        info!("✅ Window created: {}", window.window_id);

        Ok(window)
    }

    /// Commit frame to the display capability provider via biomeOS
    async fn commit_frame(&self, buffer: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        let window_id = self
            .window_id
            .as_ref()
            .ok_or(DisplayError::NoWindowCreated)?;

        // Encode buffer as base64 for JSON transport
        let encoded = general_purpose::STANDARD.encode(buffer);

        let params = json!({
            "window_id": window_id,
            "format": "rgba8",
            "width": self.width,
            "height": self.height,
            "data": encoded,
        });

        debug!(
            "🎨 Committing {}x{} frame ({} bytes)",
            self.width,
            self.height,
            buffer.len()
        );

        self.send_request("display.commit_frame", params).await?;

        debug!("✅ Frame committed successfully");

        Ok(())
    }
}

impl DisplayBackend for DiscoveredDisplayBackend {
    async fn init(&mut self) -> Result<()> {
        info!("🌸🦈 Initializing discovered display backend (biomeOS JSON-RPC)...");
        info!("   Socket: {}", self.biomeos_socket.display());

        // 1. Query display capabilities
        let caps = self.query_capabilities().await?;

        // 2. Select primary display (prefer connected outputs)
        let display_info = caps
            .displays
            .iter()
            .find(|d| d.connected)
            .or_else(|| caps.displays.first())
            .ok_or(DisplayError::NoDisplaysFromBackend)?;

        info!(
            "   Display: {} ({}) connected={}",
            display_info.connector, display_info.id, display_info.connected
        );
        info!(
            "   Resolution: {}x{} @ {}Hz",
            display_info.resolution.width,
            display_info.resolution.height,
            display_info.refresh_rate
        );

        for dev in &caps.input_devices {
            debug!(
                "   Input device: {} id={} type={}",
                dev.name, dev.id, dev.device_type
            );
        }

        // Update dimensions from actual display
        self.width = display_info.resolution.width;
        self.height = display_info.resolution.height;

        // 3. Create window
        let window = self
            .create_window("petalTongue UI", self.width, self.height)
            .await?;
        self.window_id = Some(window.window_id);
        self.buffer_handle = Some(window.buffer_handle);

        info!("✅ Discovered display backend initialized");
        if let Some(window_id) = &self.window_id {
            info!("   Window: {}", window_id);
        }
        info!("   Dimensions: {}x{}", self.width, self.height);

        Ok(())
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        let expected_size = expected_rgba8_buffer_size(self.width, self.height);
        if buffer.len() != expected_size {
            return Err(DisplayError::InvalidBufferSizeDetailed {
                expected: expected_size,
                width: self.width,
                height: self.height,
                actual: buffer.len(),
            }
            .into());
        }

        self.commit_frame(buffer).await
    }

    /// Returns true when a biomeOS Unix socket path exists so capability queries can reach the orchestrator.
    fn is_available() -> bool {
        // Check if biomeOS socket exists (capability-based discovery)
        let socket_paths = [
            petal_tongue_core::constants::biomeos_legacy_socket(),
            std::path::PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_default()).join(
                format!(
                    "{}.sock",
                    petal_tongue_core::constants::biomeos_socket_name()
                ),
            ),
        ];

        socket_paths.iter().any(|p| p.exists())
    }

    fn name(&self) -> &'static str {
        "Discovered Display (via biomeOS)"
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities {
            requires_network: false, // Unix socket is local
            requires_gpu: false,     // display provider may use GPU server-side
            requires_root: false,
            supports_resize: true,
            max_fps: 60,                    // VSync from DRM
            latency_ms: 10,                 // Low latency via biomeOS
            requires_display_server: false, // Direct DRM
            remote_capable: true,           // Can work over network if needed
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("🌸 Shutting down discovered display backend");

        if let Some(window_id) = &self.window_id {
            info!("   Window: {}", window_id);
        }

        self.window_id = None;
        self.buffer_handle = None;

        Ok(())
    }
}

#[cfg(test)]
#[path = "discovered_display_tests.rs"]
mod tests;
