//! Toadstool WASM Rendering Backend
//!
//! Leverages Toadstool's GPU capabilities via WASM for rendering.
//! This demonstrates primal collaboration and network effect.
//!
//! # Flow
//!
//! 1. Discover Toadstool via capability query ("wasm-rendering")
//! 2. Send egui rendering commands to Toadstool
//! 3. Toadstool renders via WASM module (GPU-accelerated if available)
//! 4. Receive pixel buffer (RGBA8)
//! 5. Display locally via software renderer

use crate::display::traits::{DisplayBackend, DisplayCapabilities};
use crate::universal_discovery;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Toadstool WASM display backend
pub struct ToadstoolDisplay {
    endpoint: String,
    width: u32,
    height: u32,
    client: Option<ToadstoolClient>,
}

/// Rendering request to Toadstool
#[derive(Debug, Clone, Serialize)]
struct RenderRequest {
    width: u32,
    height: u32,
    commands: Vec<u8>, // Serialized egui commands
}

/// Rendering response from Toadstool
#[derive(Debug, Clone, Deserialize)]
struct RenderResponse {
    pixels: Vec<u8>, // RGBA8 pixel buffer
    width: u32,
    height: u32,
}

/// Toadstool client (JSON-RPC or TARPC)
struct ToadstoolClient {
    endpoint: String,
    protocol: Protocol,
}

#[derive(Debug, Clone, Copy)]
enum Protocol {
    JsonRpc,
    Tarpc,
    Http,
}

impl ToadstoolDisplay {
    /// Discover Toadstool rendering capability
    pub async fn discover() -> Result<Self> {
        info!("🌸 Discovering Toadstool WASM rendering capability...");

        // Use infant discovery pattern to find Toadstool
        let discovery = universal_discovery::UniversalDiscovery::new();
        let services = discovery.discover_capability("wasm-rendering").await?;

        let endpoints: Vec<String> = services.iter().map(|s| s.endpoint.clone()).collect();

        if endpoints.is_empty() {
            return Err(anyhow!("No Toadstool WASM renderer found"));
        }

        let endpoint = endpoints[0].clone();
        info!("✅ Found Toadstool at: {}", endpoint);

        Ok(Self {
            endpoint,
            width: 1920,
            height: 1080,
            client: None,
        })
    }

    /// Create new Toadstool display with explicit endpoint
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            width: 1920,
            height: 1080,
            client: None,
        }
    }

    /// Set dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Send render request to Toadstool
    ///
    /// Sends an RGBA8 frame buffer to the Toadstool WASM renderer for remote display.
    /// Supports multiple transport protocols (HTTP, tarpc, JSON-RPC).
    async fn render_via_toadstool(&mut self, buffer: &[u8]) -> Result<()> {
        // Validate buffer size
        let expected_size = (self.width * self.height * 4) as usize; // RGBA8 = 4 bytes per pixel
        if buffer.len() != expected_size {
            return Err(anyhow!(
                "Buffer size mismatch: expected {} bytes ({}x{}x4), got {}",
                expected_size,
                self.width,
                self.height,
                buffer.len()
            ));
        }

        info!(
            "🎨 Rendering {}x{} frame via Toadstool ({} bytes -> {})",
            self.width,
            self.height,
            buffer.len(),
            self.endpoint
        );

        // Determine protocol from endpoint
        if self.endpoint.starts_with("http://") || self.endpoint.starts_with("https://") {
            self.render_via_http(buffer).await
        } else if self.endpoint.starts_with("tarpc://") {
            self.render_via_tarpc(buffer).await
        } else {
            self.render_via_jsonrpc(buffer).await
        }
    }

    /// HTTP rendering protocol
    async fn render_via_http(&self, buffer: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        // Encode buffer as base64 for JSON transport
        let encoded = general_purpose::STANDARD.encode(buffer);

        // Create render request
        let request = serde_json::json!({
            "width": self.width,
            "height": self.height,
            "format": "rgba8",
            "data": encoded
        });

        // Send POST request to toadstool
        let url = format!("{}/api/v1/render", self.endpoint);
        let client = reqwest::Client::new();

        match client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("✅ Frame rendered successfully via Toadstool HTTP");
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Toadstool HTTP render failed: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => {
                warn!("Failed to send frame to Toadstool: {}", e);
                Err(anyhow!("HTTP request failed: {}", e))
            }
        }
    }

    /// tarpc rendering protocol (for local high-performance rendering)
    async fn render_via_tarpc(&self, buffer: &[u8]) -> Result<()> {
        // tarpc requires protobufor similar serialization
        // For now, fall back to HTTP-like JSON protocol
        warn!("tarpc protocol not fully implemented, using fallback");

        // In production, would use tarpc client:
        // let client = ToadstoolRenderClient::connect(self.endpoint).await?;
        // client.render_frame(self.width, self.height, buffer.to_vec()).await?;

        Err(anyhow!("tarpc protocol requires toadstool client library"))
    }

    /// JSON-RPC rendering protocol
    async fn render_via_jsonrpc(&self, buffer: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        let encoded = general_purpose::STANDARD.encode(buffer);

        // JSON-RPC 2.0 request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "render_frame",
            "params": {
                "width": self.width,
                "height": self.height,
                "format": "rgba8",
                "data": encoded
            },
            "id": 1
        });

        // Send to WebSocket or HTTP JSON-RPC endpoint
        let client = reqwest::Client::new();

        match client
            .post(&self.endpoint)
            .json(&request)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("✅ Frame rendered successfully via Toadstool JSON-RPC");
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Toadstool JSON-RPC render failed: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => {
                warn!("Failed to send frame to Toadstool: {}", e);
                Err(anyhow!("JSON-RPC request failed: {}", e))
            }
        }
    }
}

#[async_trait]
impl DisplayBackend for ToadstoolDisplay {
    async fn init(&mut self) -> Result<()> {
        info!("🌸 Initializing Toadstool WASM display backend...");

        // Determine protocol (try TARPC first, then JSON-RPC, then HTTP)
        let protocol = if self.endpoint.starts_with("unix://") {
            Protocol::JsonRpc
        } else if self.endpoint.starts_with("http://") || self.endpoint.starts_with("https://") {
            Protocol::Http
        } else {
            Protocol::Tarpc
        };

        self.client = Some(ToadstoolClient {
            endpoint: self.endpoint.clone(),
            protocol,
        });

        info!("✅ Toadstool display backend initialized");
        info!("   Endpoint: {}", self.endpoint);
        info!("   Protocol: {:?}", protocol);
        info!("   Dimensions: {}x{}", self.width, self.height);

        Ok(())
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        // Verify buffer size
        let expected_size = (self.width * self.height * 4) as usize;
        if buffer.len() != expected_size {
            return Err(anyhow!(
                "Invalid buffer size: expected {}, got {}",
                expected_size,
                buffer.len()
            ));
        }

        self.render_via_toadstool(buffer).await
    }

    fn is_available() -> bool {
        // Check if we can discover Toadstool
        // This is a synchronous check, so we just return true and fail gracefully during init
        true
    }

    fn name(&self) -> &str {
        "Toadstool WASM Rendering"
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities::toadstool()
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("🌸 Shutting down Toadstool display backend");
        self.client = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toadstool_display_creation() {
        let display = ToadstoolDisplay::new("http://localhost:8080".to_string());
        assert_eq!(display.name(), "Toadstool WASM Rendering");
        assert_eq!(display.dimensions(), (1920, 1080));
    }

    #[test]
    fn test_toadstool_capabilities() {
        let caps = DisplayCapabilities::toadstool();
        assert!(caps.requires_network);
        assert!(!caps.requires_gpu); // Toadstool handles GPU
        assert!(!caps.requires_root);
        assert!(!caps.requires_display_server);
        assert!(caps.remote_capable);
    }
}
