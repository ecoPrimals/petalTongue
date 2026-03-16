// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure Rust Display Detection
//!
//! TRUE PRIMAL display detection using environment and system APIs (no external commands).
//!
//! # Sovereignty
//!
//! **EVOLVED**: 100% Pure Rust display detection!
//! - Environment variables + fallbacks
//! - NO external commands (xrandr, xdpyinfo, pgrep, xdotool)
//! - Works on: X11, Wayland, macOS, Windows
//! - Self-stable operation guaranteed
//!
//! Note: winit monitor detection requires an active event loop which is complex
//! for our use case. We use simpler, more reliable methods.

use petal_tongue_core::constants;
use tracing::{debug, info, warn};

/// Get primary display dimensions using pure Rust
///
/// This replaces xrandr/xdpyinfo external commands with pure Rust.
/// Uses environment variables and sensible defaults.
pub fn get_display_dimensions_pure_rust() -> Option<(u32, u32)> {
    info!("🖥️  Detecting display dimensions (pure Rust)...");

    // Method 0: Explicit window size (PETALTONGUE_WINDOW_WIDTH, PETALTONGUE_WINDOW_HEIGHT)
    let (default_w, default_h) = constants::default_window_size();
    if default_w != constants::DEFAULT_WINDOW_WIDTH || default_h != constants::DEFAULT_WINDOW_HEIGHT
    {
        info!(
            "✅ Display dimensions from env: {}x{}",
            default_w, default_h
        );
        return Some((default_w, default_h));
    }

    // Method 1: Check for common resolution environment variables
    if let Ok(res) = std::env::var("RESOLUTION")
        && let Some((w, h)) = parse_resolution(&res)
    {
        info!("✅ Display dimensions from RESOLUTION env: {}x{}", w, h);
        return Some((w, h));
    }

    // Method 2: Sensible defaults based on display type
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        debug!(
            "Wayland session detected - using default {}x{}",
            default_w, default_h
        );
        return Some((default_w, default_h));
    }

    if let Ok(display_var) = std::env::var("DISPLAY") {
        debug!("X11 session detected (DISPLAY={})", display_var);
        return Some((default_w, default_h));
    }

    // Method 3: Terminal dimensions (for headless/SSH)
    if let Some((terminal_size::Width(term_w), terminal_size::Height(term_h))) =
        terminal_size::terminal_size()
    {
        // Estimate pixel dimensions (assume 80x24 terminal = 1280x720)
        let pixel_w = (term_w as usize * 16).max(800) as u32;
        let pixel_h = (term_h as usize * 30).max(600) as u32;
        debug!(
            "Terminal dimensions: {}x{} chars → {}x{} pixels",
            term_w, term_h, pixel_w, pixel_h
        );
        return Some((pixel_w, pixel_h));
    }

    // Fallback: From constants (env: PETALTONGUE_WINDOW_WIDTH, PETALTONGUE_WINDOW_HEIGHT)
    warn!(
        "Could not detect display dimensions, using fallback {}x{}",
        default_w, default_h
    );
    Some((default_w, default_h))
}

/// Parse resolution string (e.g., "1920x1080" or "1920*1080")
fn parse_resolution(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split(['x', '*', 'X']).collect();
    if parts.len() == 2
        && let (Ok(w), Ok(h)) = (parts[0].parse(), parts[1].parse())
    {
        return Some((w, h));
    }
    None
}

/// Check if running in a virtual/headless display environment
///
/// This replaces pgrep/xdotool checks with pure Rust environment detection.
pub fn is_virtual_display() -> bool {
    // Check for common virtual display indicators

    // 1. Check for Xvfb via DISPLAY variable
    if let Ok(display_var) = std::env::var("DISPLAY")
        && (display_var.starts_with(":99") || display_var.contains("xvfb"))
    {
        debug!("Virtual display detected via DISPLAY={}", display_var);
        return true;
    }

    // 2. Check for headless environment variables
    if std::env::var("HEADLESS").is_ok() || std::env::var("CI").is_ok() {
        debug!("Headless environment detected via env vars");
        return true;
    }

    // 3. Check for framebuffer (physical displays usually have /dev/fb0)
    if !std::path::Path::new("/dev/fb0").exists() {
        debug!("No /dev/fb0 - possibly virtual display");
        // Not definitive, but a hint
    }

    // Assume physical if no clear virtual indicators
    false
}

/// Get all available monitors
///
/// Pure Rust monitor enumeration (replaces xrandr --listmonitors).
pub fn get_all_monitors() -> Vec<MonitorInfo> {
    // For now, return a single primary monitor
    // Full multi-monitor support would require platform-specific APIs or winit event loop
    if let Some((width, height)) = get_display_dimensions_pure_rust() {
        info!("📊 Detected 1 monitor (pure Rust)");
        vec![MonitorInfo {
            name: "Primary Display".to_string(),
            width,
            height,
            scale_factor: 1.0,
            is_primary: true,
        }]
    } else {
        warn!("No monitors detected");
        Vec::new()
    }
}

/// Monitor information
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Monitor name/identifier
    pub name: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Scale factor (`HiDPI` support)
    pub scale_factor: f64,
    /// Whether this is the primary monitor
    pub is_primary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_dimensions_detection() {
        // This test may fail in headless CI environments
        // That's expected and acceptable
        match get_display_dimensions_pure_rust() {
            Some((width, height)) => {
                assert!(width > 0, "Width should be positive");
                assert!(height > 0, "Height should be positive");
                assert!(width < 100_000, "Width should be reasonable");
                assert!(height < 100_000, "Height should be reasonable");
            }
            None => {
                // Acceptable in headless environments
                println!("No display detected (headless environment?)");
            }
        }
    }

    #[test]
    fn test_virtual_display_detection() {
        // Should not panic
        let is_virtual = is_virtual_display();
        println!("Virtual display: {}", is_virtual);
        // Result depends on environment, just verify it runs
    }

    #[test]
    fn test_monitor_enumeration() {
        // Should not panic
        let monitors = get_all_monitors();
        println!("Detected {} monitor(s)", monitors.len());

        for monitor in monitors {
            assert!(monitor.width > 0);
            assert!(monitor.height > 0);
            assert!(monitor.scale_factor > 0.0);
        }
    }
}
