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
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tracing::{info, warn};

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
    #[allow(dead_code)]
    Vnc,
    /// WebSocket streaming (browser access)
    #[allow(dead_code)]
    WebSocket,
    /// Window display (if available)
    #[allow(dead_code)]
    Window,
    /// Memory buffer only (for testing/headless)
    Memory,
}

impl SoftwareDisplay {
    /// Create new software display with default dimensions
    pub fn new() -> Self {
        Self::with_dimensions(1920, 1080)
    }

    /// Create new software display with specific dimensions
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

    /// Check if VNC backend is available
    fn check_vnc() -> bool {
        // TODO: Check if VNC server can be started
        false
    }

    /// Check if WebSocket backend is available
    fn check_websocket() -> bool {
        // TODO: Check if WebSocket server can be started
        false
    }

    /// Check if Window backend is available
    fn check_window() -> bool {
        // Check if we can create a window (winit available)
        cfg!(feature = "window")
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
            return Err(anyhow!(
                "Invalid buffer size: expected {}, got {}",
                expected_size,
                buffer.len()
            ));
        }

        // Copy to internal buffer
        self.buffer.copy_from_slice(buffer);

        // Present based on backend type
        match self.backend {
            SoftwareBackend::Vnc => {
                // TODO: Send to VNC clients
                Ok(())
            }
            SoftwareBackend::WebSocket => {
                // TODO: Send to WebSocket clients
                Ok(())
            }
            SoftwareBackend::Window => {
                // TODO: Present to window
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
        assert_eq!(display.name().starts_with("Software Rendering"), true);
        assert_eq!(display.dimensions(), (1920, 1080));
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

        // Create test buffer (100x100 RGBA)
        let buffer = vec![0u8; 100 * 100 * 4];
        assert!(display.present(&buffer).await.is_ok());
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

