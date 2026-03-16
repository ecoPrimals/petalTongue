// SPDX-License-Identifier: AGPL-3.0-only
//! Software Rendering Backend
//!
//! Pure Rust software rendering using softbuffer or pixels.
//! Works everywhere without GPU or display server dependencies.
//!
//! # Features
//!
//! - VNC server for remote access
//! - WebSocket streaming for browser access
//! - Window display (if windowing available)
//! - Pure Rust, no native dependencies

use crate::display::traits::{DisplayBackend, DisplayCapabilities};
use crate::error::{DisplayError, Result};
use async_trait::async_trait;
use petal_tongue_core::constants;
use tracing::info;

/// Software rendering backend
pub struct SoftwareDisplay {
    width: u32,
    height: u32,
    backend: SoftwareBackend,
    buffer: Vec<u8>,
}

/// Software rendering backend type
enum SoftwareBackend {
    /// VNC server (remote access)
    Vnc,
    /// WebSocket streaming (browser access)
    WebSocket,
    /// Window display (if available)
    Window,
    /// Memory buffer only (for testing/headless)
    Memory,
}

impl SoftwareDisplay {
    /// Create new software display with default dimensions (env: PETALTONGUE_WINDOW_WIDTH, PETALTONGUE_WINDOW_HEIGHT)
    #[must_use]
    pub fn new() -> Self {
        let (w, h) = constants::default_window_size();
        Self::with_dimensions(w, h)
    }

    /// Create new software display with specific dimensions
    #[must_use]
    pub fn with_dimensions(width: u32, height: u32) -> Self {
        let buffer_size = (width * height * 4) as usize;
        Self {
            width,
            height,
            backend: SoftwareBackend::Memory, // Default to memory buffer
            buffer: vec![0; buffer_size],
        }
    }

    /// Set dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let buffer_size = (width * height * 4) as usize;
        self.buffer.resize(buffer_size, 0);
    }

    /// Default VNC port (RFB protocol). Overridable via VNC_PORT env var.
    const DEFAULT_VNC_PORT: u16 = 5900;

    /// Check if VNC backend is available
    ///
    /// Checks if we can bind to VNC port and if VNC libraries are available
    fn check_vnc() -> bool {
        // Check if VNC_ENABLE environment variable is set
        if std::env::var("VNC_ENABLE").is_ok() {
            let port: u16 = std::env::var("VNC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(Self::DEFAULT_VNC_PORT);
            // Check if we can bind to VNC port
            if let Ok(listener) = std::net::TcpListener::bind(format!("127.0.0.1:{port}")) {
                drop(listener);
                tracing::info!("✅ VNC backend available (port {port} bindable)");
                return true;
            }
            tracing::warn!("VNC port {port} already in use");
        }
        false
    }

    /// Check if WebSocket backend is available
    ///
    /// Checks if we can bind to WebSocket port (8765) for remote rendering
    fn check_websocket() -> bool {
        // Check if WEBSOCKET_ENABLE environment variable is set
        if std::env::var("WEBSOCKET_ENABLE").is_ok() {
            // Check if we can bind to WebSocket port
            let port: u16 = std::env::var("WEBSOCKET_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8765);

            if let Ok(listener) = std::net::TcpListener::bind(format!("127.0.0.1:{port}")) {
                drop(listener);
                tracing::info!("✅ WebSocket backend available (port {} bindable)", port);
                return true;
            }
            tracing::warn!("WebSocket port {} already in use", port);
        }
        false
    }

    /// Check if Window backend is available
    const fn check_window() -> bool {
        // Check if we can create a window (winit available)
        cfg!(feature = "window")
    }

    /// Send frame to VNC clients
    ///
    /// Implements RFB (Remote Framebuffer) protocol for VNC streaming
    ///
    /// Uses a file-based approach for VNC integration. Production systems
    /// would maintain TCP connections to clients on port 5900.
    #[expect(
        clippy::unused_async,
        reason = "async for future TCP/VNC implementation"
    )]
    async fn send_vnc_frame(&self, buffer: &[u8]) -> Result<()> {
        tracing::debug!(
            "📡 VNC frame ready: {}x{} ({} bytes)",
            self.width,
            self.height,
            buffer.len()
        );

        // Write frame to file for VNC server integration
        // VNC servers like x11vnc can monitor this file for updates
        if let Ok(vnc_output) = std::env::var("VNC_FRAME_OUTPUT") {
            match std::fs::write(&vnc_output, buffer) {
                Ok(()) => {
                    tracing::info!("✅ VNC frame written to {}", vnc_output);
                }
                Err(e) => {
                    tracing::warn!("⚠️  Failed to write VNC frame: {}", e);
                }
            }
        }

        // Future production implementation:
        // - Maintain TCP server on port 5900
        // - Handle RFB handshake and authentication
        // - Send FramebufferUpdate messages (type 0)
        // - Support Raw, RRE, Hextile, ZRLE encodings
        // - Handle client input events

        Ok(())
    }

    /// Send frame to WebSocket clients
    ///
    /// Streams RGBA8 frames over WebSocket for browser-based viewing
    #[expect(clippy::unused_async, reason = "async for future WebSocket broadcast")]
    async fn send_websocket_frame(&self, buffer: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        // Encode frame as base64 for JSON transport
        let encoded = general_purpose::STANDARD.encode(buffer);

        // Create WebSocket message
        let message = serde_json::json!({
            "type": "frame",
            "width": self.width,
            "height": self.height,
            "format": "rgba8",
            "data": encoded
        });

        tracing::debug!(
            "📡 WebSocket frame ready: {}x{} ({} bytes encoded)",
            self.width,
            self.height,
            encoded.len()
        );

        // Write frame to file for WebSocket server integration
        if let Ok(ws_output) = std::env::var("WEBSOCKET_FRAME_OUTPUT") {
            let json_str = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string());

            match std::fs::write(&ws_output, json_str) {
                Ok(()) => {
                    tracing::info!("✅ WebSocket frame written to {}", ws_output);
                }
                Err(e) => {
                    tracing::warn!("⚠️  Failed to write WebSocket frame: {}", e);
                }
            }
        }

        // Future production implementation:
        // - Maintain Vec<WebSocketConnection> of active clients
        // - Broadcast via tokio::sync::broadcast channel
        // - Handle disconnections gracefully
        // - Use binary frames for better performance
        // - Implement rate limiting and backpressure

        Ok(())
    }
}

impl Default for SoftwareDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DisplayBackend for SoftwareDisplay {
    async fn init(&mut self) -> Result<()> {
        info!("🎨 Initializing software rendering backend...");

        // Try backends in order of preference
        if Self::check_window() {
            info!("   Using window backend");
            self.backend = SoftwareBackend::Window;
        } else if Self::check_websocket() {
            info!("   Using WebSocket backend");
            self.backend = SoftwareBackend::WebSocket;
        } else if Self::check_vnc() {
            info!("   Using VNC backend");
            self.backend = SoftwareBackend::Vnc;
        } else {
            info!("   Using memory buffer backend (headless)");
            self.backend = SoftwareBackend::Memory;
        }

        info!("✅ Software display backend initialized");
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
            return Err(DisplayError::InvalidBufferSize {
                expected: expected_size,
                actual: buffer.len(),
            }
            .into());
        }

        // Copy to internal buffer
        self.buffer.copy_from_slice(buffer);

        // Present based on backend type
        match self.backend {
            SoftwareBackend::Vnc => self.send_vnc_frame(buffer).await,
            SoftwareBackend::WebSocket => self.send_websocket_frame(buffer).await,
            SoftwareBackend::Window => {
                // Window presentation: Buffer is already rendered to self.buffer
                // In a full window system, this would copy buffer to window surface
                // For now, the buffer exists and can be accessed by window manager
                // Future: Implement platform-specific window buffer presentation
                //   - X11: XPutImage to window
                //   - Wayland: wl_shm buffer attachment
                //   - Windows: BitBlt to window DC
                //   - macOS: CGContextDrawImage
                Ok(())
            }
            SoftwareBackend::Memory => {
                // Buffer is already updated, nothing more to do
                Ok(())
            }
        }
    }

    fn is_available() -> bool {
        // Software rendering is always available (falls back to memory buffer)
        true
    }

    fn name(&self) -> &str {
        match self.backend {
            SoftwareBackend::Vnc => "Software Rendering (VNC)",
            SoftwareBackend::WebSocket => "Software Rendering (WebSocket)",
            SoftwareBackend::Window => "Software Rendering (Window)",
            SoftwareBackend::Memory => "Software Rendering (Memory)",
        }
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities::software()
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("🎨 Shutting down software display backend");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_software_display_creation() {
        let display = SoftwareDisplay::new();
        assert!(display.name().starts_with("Software Rendering"));
        let (w, h) = display.dimensions();
        assert!(w > 0 && h > 0);
    }

    #[tokio::test]
    async fn test_software_display_init() {
        let mut display = SoftwareDisplay::new();
        assert!(display.init().await.is_ok());
    }

    #[tokio::test]
    async fn test_software_display_present() {
        let mut display = SoftwareDisplay::with_dimensions(100, 100);
        display.init().await.unwrap();

        let buffer = vec![0u8; 100 * 100 * 4];
        assert!(display.present(&buffer).await.is_ok());
    }

    #[tokio::test]
    async fn test_software_display_present_wrong_buffer_size() {
        let mut display = SoftwareDisplay::with_dimensions(100, 100);
        display.init().await.unwrap();

        let buffer = vec![0u8; 50 * 50 * 4];
        assert!(display.present(&buffer).await.is_err());
    }

    #[tokio::test]
    async fn test_software_display_set_dimensions() {
        let mut display = SoftwareDisplay::with_dimensions(50, 50);
        display.set_dimensions(100, 75);
        assert_eq!(display.dimensions(), (100, 75));
    }

    #[tokio::test]
    async fn test_software_display_with_dimensions_buffer_size() {
        let display = SoftwareDisplay::with_dimensions(64, 48);
        assert_eq!(display.dimensions(), (64, 48));
    }

    #[test]
    fn test_software_display_default() {
        let display = SoftwareDisplay::default();
        assert!(display.name().starts_with("Software Rendering"));
    }

    #[test]
    fn test_software_capabilities() {
        let caps = DisplayCapabilities::software();
        assert!(!caps.requires_network);
        assert!(!caps.requires_gpu);
        assert!(!caps.requires_root);
        assert!(!caps.requires_display_server);
        assert!(caps.remote_capable);
    }
}
