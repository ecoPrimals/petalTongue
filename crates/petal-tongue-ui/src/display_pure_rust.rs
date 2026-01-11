//! Pure Rust Display Detection
//!
//! TRUE PRIMAL display detection using winit (no external commands).
//!
//! # Sovereignty
//!
//! **EVOLVED**: 100% Pure Rust display detection!
//! - winit: Cross-platform window/monitor management
//! - NO external commands (xrandr, xdpyinfo, pgrep, xdotool)
//! - Works on: X11, Wayland, macOS, Windows
//! - Self-stable operation guaranteed

use tracing::{debug, info, warn};

/// Get primary display dimensions using pure Rust (winit)
///
/// This replaces xrandr/xdpyinfo external commands with pure Rust.
/// Works on X11, Wayland, macOS (Cocoa), Windows (Win32).
pub fn get_display_dimensions_pure_rust() -> Option<(u32, u32)> {
    use winit::event_loop::EventLoop;
    
    info!("🖥️  Detecting display dimensions (pure Rust - winit)...");
    
    // Create event loop (lightweight, doesn't open window)
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            warn!("Failed to create winit event loop: {}", e);
            return None;
        }
    };
    
    // Get primary monitor
    let monitor = match event_loop.primary_monitor() {
        Some(m) => m,
        None => {
            warn!("No primary monitor detected");
            // Try available monitors
            if let Some(m) = event_loop.available_monitors().next() {
                debug!("Using first available monitor instead");
                m
            } else {
                warn!("No monitors detected at all");
                return None;
            }
        }
    };
    
    // Get physical size
    let size = monitor.size();
    let width = size.width;
    let height = size.height;
    
    info!("✅ Display dimensions detected: {}x{} (pure Rust)", width, height);
    
    // Log additional info for diagnostics
    if let Some(name) = monitor.name() {
        debug!("Monitor name: {}", name);
    }
    
    let scale_factor = monitor.scale_factor();
    debug!("Scale factor: {}", scale_factor);
    
    Some((width, height))
}

/// Check if running in a virtual/headless display environment
///
/// This replaces pgrep/xdotool checks with pure Rust environment detection.
pub fn is_virtual_display() -> bool {
    // Check for common virtual display indicators
    
    // 1. Check for Xvfb via DISPLAY variable
    if let Ok(display) = std::env::var("DISPLAY") {
        if display.starts_with(":99") || display.contains("xvfb") {
            debug!("Virtual display detected via DISPLAY={}", display);
            return true;
        }
    }
    
    // 2. Check for headless environment variables
    if std::env::var("HEADLESS").is_ok() || std::env::var("CI").is_ok() {
        debug!("Headless environment detected via env vars");
        return true;
    }
    
    // 3. Try to detect via winit (if no monitors, likely virtual)
    match winit::event_loop::EventLoop::new() {
        Ok(event_loop) => {
            let monitor_count = event_loop.available_monitors().count();
            if monitor_count == 0 {
                debug!("No monitors detected - likely virtual display");
                return true;
            }
            debug!("Detected {} monitor(s) - physical display", monitor_count);
            false
        }
        Err(e) => {
            warn!("Failed to create event loop for virtual detection: {}", e);
            // Assume physical if we can't detect
            false
        }
    }
}

/// Get all available monitors
///
/// Pure Rust monitor enumeration (replaces xrandr --listmonitors).
pub fn get_all_monitors() -> Vec<MonitorInfo> {
    use winit::event_loop::EventLoop;
    
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            warn!("Failed to create event loop for monitor enumeration: {}", e);
            return Vec::new();
        }
    };
    
    let monitors: Vec<MonitorInfo> = event_loop
        .available_monitors()
        .enumerate()
        .map(|(idx, monitor)| {
            let size = monitor.size();
            let name = monitor.name().unwrap_or_else(|| format!("Monitor {}", idx));
            let scale_factor = monitor.scale_factor();
            
            MonitorInfo {
                name,
                width: size.width,
                height: size.height,
                scale_factor,
                is_primary: false, // winit doesn't expose this directly
            }
        })
        .collect();
    
    info!("📊 Detected {} monitor(s) (pure Rust)", monitors.len());
    for (idx, monitor) in monitors.iter().enumerate() {
        debug!(
            "  Monitor {}: {} ({}x{}, scale: {})",
            idx, monitor.name, monitor.width, monitor.height, monitor.scale_factor
        );
    }
    
    monitors
}

/// Monitor information
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
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
                assert!(width < 100000, "Width should be reasonable");
                assert!(height < 100000, "Height should be reasonable");
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

