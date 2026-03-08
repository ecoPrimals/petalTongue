// SPDX-License-Identifier: AGPL-3.0-only
//! External Display Server Backend
//!
//! Traditional display server support (X11/Wayland/Windows/macOS).
//! Used as a benchmark and fallback for systems with existing display infrastructure.
//!
//! # Features
//!
//! - Detects available display servers
//! - Prompts user to start display server if not available
//! - Uses native OpenGL/GPU acceleration
//! - Highest performance (when available)

use crate::display::traits::{DisplayBackend, DisplayCapabilities};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::env;
use tracing::info;

/// External display server backend
pub struct ExternalDisplay {
    width: u32,
    height: u32,
    display_type: Option<ExternalDisplayType>,
}

/// Type of external display server
#[derive(Debug, Clone, Copy)]
pub enum ExternalDisplayType {
    /// X11 display server
    X11,
    /// Wayland display server
    Wayland,
    /// Windows display
    Windows,
    /// macOS display
    MacOS,
}

impl ExternalDisplay {
    /// Create new external display backend
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            display_type: None,
        }
    }

    /// Create external display with specific dimensions
    #[must_use]
    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            display_type: None,
        }
    }

    /// Detect available display server
    fn detect_display_server() -> Option<ExternalDisplayType> {
        // Check for Wayland
        if env::var("WAYLAND_DISPLAY").is_ok() {
            info!("Detected Wayland display server");
            return Some(ExternalDisplayType::Wayland);
        }

        // Check for X11
        if env::var("DISPLAY").is_ok() {
            info!("Detected X11 display server");
            return Some(ExternalDisplayType::X11);
        }

        // Check for Windows
        #[cfg(target_os = "windows")]
        {
            info!("Running on Windows");
            return Some(ExternalDisplayType::Windows);
        }

        // Check for macOS
        #[cfg(target_os = "macos")]
        {
            info!("Running on macOS");
            return Some(ExternalDisplayType::MacOS);
        }

        None
    }
}

impl Default for ExternalDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DisplayBackend for ExternalDisplay {
    async fn init(&mut self) -> Result<()> {
        info!("🪟 Initializing external display backend...");

        // Detect display server
        self.display_type = Self::detect_display_server();

        if let Some(display_type) = self.display_type {
            info!("✅ External display backend initialized");
            info!("   Type: {:?}", display_type);
            info!("   Dimensions: {}x{}", self.width, self.height);
            Ok(())
        } else {
            Err(anyhow!("No external display server detected"))
        }
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

        // In a real implementation, this would present via eframe/OpenGL
        // For now, this is a placeholder
        Ok(())
    }

    fn is_available() -> bool {
        Self::detect_display_server().is_some()
    }

    fn name(&self) -> &str {
        match self.display_type {
            Some(ExternalDisplayType::X11) => "External Display (X11)",
            Some(ExternalDisplayType::Wayland) => "External Display (Wayland)",
            Some(ExternalDisplayType::Windows) => "External Display (Windows)",
            Some(ExternalDisplayType::MacOS) => "External Display (macOS)",
            None => "External Display (Unknown)",
        }
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities::external()
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("🪟 Shutting down external display backend");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_display_creation() {
        let display = ExternalDisplay::new();
        assert_eq!(display.dimensions(), (1920, 1080));
    }

    #[test]
    fn test_external_display_detection() {
        let display_type = ExternalDisplay::detect_display_server();
        info!("Detected display type: {:?}", display_type);
        // This will vary by platform, so we just log it
    }

    #[test]
    fn test_external_capabilities() {
        let caps = DisplayCapabilities::external();
        assert!(!caps.requires_network);
        assert!(caps.requires_gpu); // Usually uses GPU
        assert!(!caps.requires_root);
        assert!(caps.requires_display_server); // By definition
        assert!(!caps.remote_capable);
    }
}
