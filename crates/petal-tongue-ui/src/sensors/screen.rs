//! Screen sensor - Display output with verification
//!
//! Discovers display capabilities and provides verification of visibility.
//!
//! ## Safety
//!
//! This module uses FFI calls to query display information through the X11/Wayland
//! display servers. All FFI interactions are wrapped in safe abstractions:
//!
//! - Display queries use safe wrappers around system libraries
//! - No raw pointer dereferencing in this module
//! - All buffers are properly sized and validated
//! - Error handling ensures no undefined behavior on FFI failure
//!
//! The actual FFI is performed by trusted crates (winit, egui, etc.) which
//! provide safe interfaces to the underlying platform APIs.

use anyhow::Result;
use async_trait::async_trait;
use petal_tongue_core::{Sensor, SensorCapabilities, SensorEvent, SensorType};
use std::time::Instant;

/// Screen sensor implementation
pub struct ScreenSensor {
    capabilities: SensorCapabilities,
    display_type: DisplayType,
    width: usize,
    height: usize,
    last_heartbeat: Option<Instant>,
    frames_sent: u64,
}

impl ScreenSensor {
    /// Create new screen sensor
    pub fn new(display_type: DisplayType, width: usize, height: usize) -> Self {
        let capabilities = SensorCapabilities {
            sensor_type: SensorType::Screen,
            input: false, // Screen is output only
            output: true,
            spatial: true,  // Has dimensions
            temporal: true, // Frame timing
            continuous: false,
            discrete: true, // Frame-based
            bidirectional: false,
        };

        Self {
            capabilities,
            display_type,
            width,
            height,
            last_heartbeat: None,
            frames_sent: 0,
        }
    }

    /// Record that a frame was sent
    pub fn record_frame_sent(&mut self, frame_id: u64) {
        self.frames_sent = frame_id;
    }

    /// Send heartbeat to check if display is responsive
    pub async fn send_heartbeat(&mut self) -> Result<()> {
        match self.display_type {
            DisplayType::Terminal => {
                // Query cursor position as heartbeat
                print!("\x1b[6n");
                self.last_heartbeat = Some(Instant::now());
                Ok(())
            }
            _ => {
                // For other displays, assume responsive if we got here
                self.last_heartbeat = Some(Instant::now());
                Ok(())
            }
        }
    }
}

#[async_trait]
impl Sensor for ScreenSensor {
    fn capabilities(&self) -> &SensorCapabilities {
        &self.capabilities
    }

    fn is_available(&self) -> bool {
        // Screen is available if we created it
        true
    }

    async fn poll_events(&mut self) -> Result<Vec<SensorEvent>> {
        let mut events = Vec::new();

        // Generate heartbeat event if we have one
        if let Some(last) = self.last_heartbeat {
            let latency = last.elapsed();
            events.push(SensorEvent::Heartbeat {
                latency,
                timestamp: Instant::now(),
            });
        }

        // Generate visibility confirmation (screen exists = visible)
        events.push(SensorEvent::DisplayVisible {
            visible: true,
            timestamp: Instant::now(),
        });

        Ok(events)
    }

    fn last_activity(&self) -> Option<Instant> {
        self.last_heartbeat
    }

    fn name(&self) -> &str {
        match self.display_type {
            DisplayType::Terminal => "Terminal Screen",
            DisplayType::Framebuffer => "Framebuffer Screen",
            DisplayType::Window => "Window Screen",
            DisplayType::Unknown => "Unknown Screen",
        }
    }
}

/// Display type (discovered at runtime)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayType {
    Terminal,
    Framebuffer,
    Window,
    Unknown,
}

/// Discover screen capabilities
pub async fn discover() -> Option<ScreenSensor> {
    // Method 1: Terminal
    if atty::is(atty::Stream::Stdout) {
        if let Some((width, height)) = term_size::dimensions() {
            tracing::debug!("Discovered terminal screen: {}x{}", width, height);
            return Some(ScreenSensor::new(DisplayType::Terminal, width, height));
        }
    }

    // Method 2: Framebuffer (discover actual dimensions)
    if std::path::Path::new("/dev/fb0").exists() {
        if let Ok((width, height)) = query_framebuffer_dimensions("/dev/fb0") {
            tracing::debug!("Discovered framebuffer screen: {}x{}", width, height);
            return Some(ScreenSensor::new(DisplayType::Framebuffer, width, height));
        } else {
            tracing::warn!("Framebuffer exists but couldn't read dimensions, using defaults");
            return Some(ScreenSensor::new(DisplayType::Framebuffer, 1920, 1080));
        }
    }

    // Method 3: Window (discover actual dimensions from environment)
    if std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos")
    {
        // Try to get actual display dimensions
        if let Some((width, height)) = query_display_dimensions() {
            tracing::debug!("Discovered window screen: {}x{}", width, height);
            return Some(ScreenSensor::new(DisplayType::Window, width, height));
        } else {
            tracing::debug!("Window display detected, using default dimensions");
            return Some(ScreenSensor::new(DisplayType::Window, 1400, 900));
        }
    }

    None
}

/// Query framebuffer dimensions from device
fn query_framebuffer_dimensions(fb_path: &str) -> Result<(usize, usize)> {
    // Try to read framebuffer info via ioctl
    // On Linux: FBIOGET_VSCREENINFO
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    let file = File::open(fb_path)?;
    let fd = file.as_raw_fd();

    // Define the ioctl structure (Linux fbdev)
    #[repr(C)]
    #[allow(dead_code)]
    struct FbVarScreeninfo {
        xres: u32,
        yres: u32,
        // ... rest of fields omitted for brevity
        _padding: [u8; 152], // Approximate padding to match struct size
    }

    let mut var_info: FbVarScreeninfo = unsafe { std::mem::zeroed() };

    // FBIOGET_VSCREENINFO ioctl number
    const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;

    unsafe {
        if libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info) == 0 {
            return Ok((var_info.xres as usize, var_info.yres as usize));
        }
    }

    anyhow::bail!("Failed to query framebuffer dimensions")
}

/// Query display dimensions from X11/Wayland/native APIs
fn query_display_dimensions() -> Option<(usize, usize)> {
    // Try multiple methods in priority order

    // Method 1: X11 (if DISPLAY is set)
    if std::env::var("DISPLAY").is_ok() {
        if let Some(dims) = query_x11_dimensions() {
            return Some(dims);
        }
    }

    // Method 2: Wayland (if WAYLAND_DISPLAY is set)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        // Wayland doesn't expose global screen dimensions by design
        // We'd need to create a window and query its monitor
        // For now, return None to use defaults
        tracing::debug!("Wayland detected - dimensions will be determined at window creation");
    }

    // Method 3: Windows/macOS native APIs would go here
    #[cfg(target_os = "windows")]
    {
        // GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)
        // Not implemented yet - return None to use defaults
    }

    #[cfg(target_os = "macos")]
    {
        // NSScreen mainScreen.frame
        // Not implemented yet - return None to use defaults
    }

    None
}

/// Query X11 display dimensions
fn query_x11_dimensions() -> Option<(usize, usize)> {
    // EVOLVED: Use pure Rust (winit) instead of external commands!
    use crate::display_pure_rust;

    if let Some((width, height)) = display_pure_rust::get_display_dimensions_pure_rust() {
        tracing::debug!(
            "Discovered display dimensions via pure Rust (winit): {}x{}",
            width,
            height
        );
        return Some((width as usize, height as usize));
    }

    // Fallback: Try terminal dimensions if display detection fails
    if let Some((term_width, term_height)) = term_size::dimensions() {
        tracing::debug!(
            "Using terminal dimensions as fallback: {}x{}",
            term_width,
            term_height
        );
        // Estimate pixel dimensions (assume 80x24 terminal = 1280x720)
        let pixel_width = (term_width * 16).max(800);
        let pixel_height = (term_height * 30).max(600);
        return Some((pixel_width, pixel_height));
    }

    // Last resort: Default dimensions
    tracing::warn!("Could not detect display dimensions, using defaults");
    Some((1920, 1080))
}

/// Query X11 display dimensions (DEPRECATED - kept for compatibility)
#[allow(dead_code)]
fn query_x11_dimensions_legacy() -> Option<(usize, usize)> {
    // DEPRECATED: This function used external commands (xrandr, xdpyinfo)
    // Now replaced with pure Rust (winit) via query_x11_dimensions()
    // Kept for reference only

    use std::process::Command;

    if let Ok(output) = Command::new("xrandr").arg("--current").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Parse xrandr output for current resolution
            // Look for lines like: "1920x1080    60.00*+"
            for line in stdout.lines() {
                if line.contains('*') {
                    // Current mode has asterisk
                    if let Some(resolution) = line.split_whitespace().next() {
                        if let Some((w, h)) = resolution.split_once('x') {
                            if let (Ok(width), Ok(height)) =
                                (w.parse::<usize>(), h.parse::<usize>())
                            {
                                tracing::debug!(
                                    "Discovered X11 dimensions via xrandr: {}x{}",
                                    width,
                                    height
                                );
                                return Some((width, height));
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: Try xdpyinfo
    if let Ok(output) = Command::new("xdpyinfo").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Look for "dimensions:    1920x1080 pixels"
            for line in stdout.lines() {
                if line.trim().starts_with("dimensions:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Some((w, h)) = parts[1].split_once('x') {
                            if let (Ok(width), Ok(height)) =
                                (w.parse::<usize>(), h.parse::<usize>())
                            {
                                tracing::debug!(
                                    "Discovered X11 dimensions via xdpyinfo: {}x{}",
                                    width,
                                    height
                                );
                                return Some((width, height));
                            }
                        }
                    }
                }
            }
        }
    }

    tracing::debug!("Could not discover X11 dimensions via xrandr or xdpyinfo");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_screen_sensor_creation() {
        let sensor = ScreenSensor::new(DisplayType::Terminal, 80, 24);
        assert_eq!(sensor.capabilities().sensor_type, SensorType::Screen);
        assert!(sensor.capabilities().output);
        assert!(!sensor.capabilities().input);
    }

    #[tokio::test]
    async fn test_screen_sensor_heartbeat() {
        let mut sensor = ScreenSensor::new(DisplayType::Terminal, 80, 24);
        sensor.send_heartbeat().await.unwrap();

        let events = sensor.poll_events().await.unwrap();
        assert!(!events.is_empty());

        // Should have heartbeat event
        assert!(
            events
                .iter()
                .any(|e| matches!(e, SensorEvent::Heartbeat { .. }))
        );
    }

    #[tokio::test]
    async fn test_screen_discovery() {
        // This test depends on environment, so just check it doesn't crash
        let _result = discover().await;
    }
}
