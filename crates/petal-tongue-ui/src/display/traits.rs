// SPDX-License-Identifier: AGPL-3.0-only
//! Display backend traits and types

use anyhow::Result;
use async_trait::async_trait;

/// Display backend trait - implemented by all display systems
#[async_trait]
pub trait DisplayBackend: Send + Sync {
    /// Initialize the display backend
    async fn init(&mut self) -> Result<()>;

    /// Get display dimensions (width, height)
    fn dimensions(&self) -> (u32, u32);

    /// Present a frame (RGBA8 pixel buffer)
    ///
    /// Buffer format: width * height * 4 bytes (RGBA)
    /// Buffer layout: row-major, top-left origin
    async fn present(&mut self, buffer: &[u8]) -> Result<()>;

    /// Check if this backend is available on the current system
    fn is_available() -> bool
    where
        Self: Sized;

    /// Backend name for logging/UI
    fn name(&self) -> &str;

    /// Performance and capability characteristics
    fn capabilities(&self) -> DisplayCapabilities;

    /// Shutdown the display backend gracefully
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Display backend capabilities and performance characteristics
#[derive(Debug, Clone)]
pub struct DisplayCapabilities {
    /// Requires network connection (e.g., Toadstool)
    pub requires_network: bool,

    /// Requires GPU (e.g., OpenGL)
    pub requires_gpu: bool,

    /// Requires root/elevated privileges (e.g., framebuffer)
    pub requires_root: bool,

    /// Supports window resizing
    pub supports_resize: bool,

    /// Maximum sustained frame rate
    pub max_fps: u32,

    /// Average frame latency in milliseconds
    pub latency_ms: u32,

    /// Requires display server (X11/Wayland)
    pub requires_display_server: bool,

    /// Can run over SSH/remote
    pub remote_capable: bool,
}

impl DisplayCapabilities {
    /// Toadstool WASM capabilities
    #[must_use]
    pub const fn toadstool() -> Self {
        Self {
            requires_network: true,
            requires_gpu: false, // Toadstool handles GPU
            requires_root: false,
            supports_resize: true,
            max_fps: 60,
            latency_ms: 20, // Network latency
            requires_display_server: false,
            remote_capable: true,
        }
    }

    /// Software rendering capabilities
    #[must_use]
    pub const fn software() -> Self {
        Self {
            requires_network: false,
            requires_gpu: false,
            requires_root: false,
            supports_resize: true,
            max_fps: 60,
            latency_ms: 16, // ~60 FPS
            requires_display_server: false,
            remote_capable: true,
        }
    }

    /// Framebuffer direct capabilities
    #[must_use]
    pub const fn framebuffer() -> Self {
        Self {
            requires_network: false,
            requires_gpu: false,
            requires_root: true, // Usually needs framebuffer access
            supports_resize: false,
            max_fps: 60,
            latency_ms: 10, // Very low latency
            requires_display_server: false,
            remote_capable: false, // Console only
        }
    }

    /// External display server capabilities
    #[must_use]
    pub const fn external() -> Self {
        Self {
            requires_network: false,
            requires_gpu: true, // Usually OpenGL
            requires_root: false,
            supports_resize: true,
            max_fps: 144,  // Can be very high
            latency_ms: 8, // Very low latency
            requires_display_server: true,
            remote_capable: false, // Display server must be local
        }
    }
}

/// Display backend priority (lower is better)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendPriority {
    /// Tier 1: Highest priority (Toadstool network effect)
    Toadstool = 1,
    /// Tier 2: High priority (Pure Rust software)
    Software = 2,
    /// Tier 3: Medium priority (Framebuffer direct)
    Framebuffer = 3,
    /// Tier 4: Low priority (External fallback)
    External = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_capabilities_toadstool() {
        let caps = DisplayCapabilities::toadstool();
        assert!(caps.requires_network);
        assert!(!caps.requires_gpu);
        assert!(!caps.requires_root);
        assert!(caps.supports_resize);
        assert_eq!(caps.max_fps, 60);
        assert!(caps.remote_capable);
    }

    #[test]
    fn test_display_capabilities_software() {
        let caps = DisplayCapabilities::software();
        assert!(!caps.requires_network);
        assert!(!caps.requires_gpu);
        assert!(!caps.requires_root);
        assert!(caps.supports_resize);
        assert_eq!(caps.max_fps, 60);
    }

    #[test]
    fn test_display_capabilities_framebuffer() {
        let caps = DisplayCapabilities::framebuffer();
        assert!(caps.requires_root);
        assert!(!caps.supports_resize);
        assert!(!caps.remote_capable);
    }

    #[test]
    fn test_display_capabilities_external() {
        let caps = DisplayCapabilities::external();
        assert!(caps.requires_gpu);
        assert!(caps.requires_display_server);
        assert!(!caps.remote_capable);
        assert_eq!(caps.max_fps, 144);
    }

    #[test]
    fn test_backend_priority_ordering() {
        assert!(BackendPriority::Toadstool < BackendPriority::Software);
        assert!(BackendPriority::Software < BackendPriority::Framebuffer);
        assert!(BackendPriority::Framebuffer < BackendPriority::External);
    }
}
