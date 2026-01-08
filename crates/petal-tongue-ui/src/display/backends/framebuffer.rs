//! Framebuffer Direct Rendering Backend
//!
//! Writes directly to Linux framebuffer (/dev/fb0) for console-mode GUI.
//! Perfect for embedded systems, kiosks, and headless servers.
//!
//! # Requirements
//!
//! - Linux system with framebuffer support
//! - Read/write access to /dev/fb0 (usually requires root)
//! - Framebuffer must be initialized (console mode)

use crate::display::traits::{DisplayBackend, DisplayCapabilities};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use tracing::{info, warn};

/// Framebuffer direct display backend
pub struct FramebufferDisplay {
    width: u32,
    height: u32,
    fb_device: Option<File>,
    buffer: Vec<u8>,
}

impl FramebufferDisplay {
    /// Create new framebuffer display
    pub fn new() -> Result<Self> {
        Ok(Self {
            width: 1920,
            height: 1080,
            fb_device: None,
            buffer: Vec::new(),
        })
    }

    /// Create framebuffer display with specific dimensions
    pub fn with_dimensions(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            width,
            height,
            fb_device: None,
            buffer: vec![0; (width * height * 4) as usize],
        })
    }

    /// Check if framebuffer device exists and is accessible
    fn check_framebuffer() -> bool {
        let fb_path = Path::new("/dev/fb0");
        if !fb_path.exists() {
            warn!("/dev/fb0 does not exist");
            return false;
        }

        // Try to open for reading (requires permissions)
        match File::open(fb_path) {
            Ok(_) => {
                info!("/dev/fb0 is accessible");
                true
            }
            Err(e) => {
                warn!("/dev/fb0 exists but not accessible: {}", e);
                false
            }
        }
    }

    /// Open framebuffer device
    fn open_framebuffer(&mut self) -> Result<()> {
        let fb_device = OpenOptions::new()
            .write(true)
            .read(true)
            .open("/dev/fb0")
            .map_err(|e| anyhow!("Failed to open /dev/fb0: {}", e))?;

        self.fb_device = Some(fb_device);
        Ok(())
    }

    /// Get framebuffer info (dimensions, etc.)
    fn get_framebuffer_info(&self) -> Result<(u32, u32)> {
        // TODO: Use ioctl to get actual framebuffer dimensions
        // For now, use configured dimensions
        Ok((self.width, self.height))
    }
}

impl Default for FramebufferDisplay {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[async_trait]
impl DisplayBackend for FramebufferDisplay {
    async fn init(&mut self) -> Result<()> {
        info!("🖥️  Initializing framebuffer display backend...");

        // Check if framebuffer is available
        if !Self::check_framebuffer() {
            return Err(anyhow!("Framebuffer /dev/fb0 not available"));
        }

        // Open framebuffer device
        self.open_framebuffer()?;

        // Get framebuffer info
        let (width, height) = self.get_framebuffer_info()?;
        info!("   Framebuffer dimensions: {}x{}", width, height);

        // Adjust buffer if needed
        if width != self.width || height != self.height {
            warn!(
                "   Configured dimensions ({}x{}) differ from framebuffer ({}x{})",
                self.width, self.height, width, height
            );
            self.width = width;
            self.height = height;
            self.buffer.resize((width * height * 4) as usize, 0);
        }

        info!("✅ Framebuffer display backend initialized");
        info!("   Device: /dev/fb0");
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

        // Write to framebuffer device
        if let Some(fb_device) = &mut self.fb_device {
            fb_device.write_all(buffer).map_err(|e| {
                anyhow!("Failed to write to framebuffer: {}", e)
            })?;
            fb_device.flush()?;
        } else {
            return Err(anyhow!("Framebuffer device not initialized"));
        }

        Ok(())
    }

    fn is_available() -> bool {
        Self::check_framebuffer()
    }

    fn name(&self) -> &str {
        "Framebuffer Direct (/dev/fb0)"
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities::framebuffer()
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("🖥️  Shutting down framebuffer display backend");
        self.fb_device = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let display = FramebufferDisplay::new();
        assert!(display.is_ok());
    }

    #[test]
    fn test_framebuffer_capabilities() {
        let caps = DisplayCapabilities::framebuffer();
        assert!(!caps.requires_network);
        assert!(!caps.requires_gpu);
        assert!(caps.requires_root); // Usually needs permissions
        assert!(!caps.requires_display_server);
        assert!(!caps.remote_capable); // Console only
        assert!(!caps.supports_resize); // Fixed by hardware
    }

    #[test]
    fn test_framebuffer_availability() {
        // This test will fail on systems without /dev/fb0
        // That's expected and correct behavior
        let available = FramebufferDisplay::is_available();
        info!("Framebuffer available: {}", available);
    }
}

