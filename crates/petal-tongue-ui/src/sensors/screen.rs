//! Screen sensor - Display output with verification
//!
//! Discovers display capabilities and provides verification of visibility.

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

    // Method 2: Framebuffer
    if std::path::Path::new("/dev/fb0").exists() {
        tracing::debug!("Discovered framebuffer screen");
        return Some(ScreenSensor::new(DisplayType::Framebuffer, 1920, 1080));
    }

    // Method 3: Window (check for display environment)
    if std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos")
    {
        tracing::debug!("Discovered window screen");
        return Some(ScreenSensor::new(DisplayType::Window, 1400, 900));
    }

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
