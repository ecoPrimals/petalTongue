//! Mouse sensor - Spatial input device
//!
//! Discovers mouse capabilities and provides click/movement events.

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{self, Event, MouseButton as CrosstermButton, MouseEventKind};
use petal_tongue_core::{MouseButton, Sensor, SensorCapabilities, SensorEvent, SensorType};
use std::time::{Duration, Instant};

/// Mouse sensor implementation
pub struct MouseSensor {
    capabilities: SensorCapabilities,
    pointer_type: PointerType,
    last_position: Option<(f32, f32)>,
    last_click: Option<Instant>,
}

impl MouseSensor {
    /// Create new mouse sensor
    pub fn new(pointer_type: PointerType) -> Self {
        let capabilities = SensorCapabilities {
            sensor_type: SensorType::Mouse,
            input: true,
            output: false, // Mouse is input only
            spatial: true, // Provides X, Y coordinates
            temporal: true, // Timing of clicks
            continuous: true, // Continuous position updates
            discrete: true, // Discrete click events
            bidirectional: false,
        };
        
        Self {
            capabilities,
            pointer_type,
            last_position: None,
            last_click: None,
        }
    }
}

#[async_trait]
impl Sensor for MouseSensor {
    fn capabilities(&self) -> &SensorCapabilities {
        &self.capabilities
    }
    
    fn is_available(&self) -> bool {
        // Mouse is available if we're in a terminal or GUI
        true
    }
    
    async fn poll_events(&mut self) -> Result<Vec<SensorEvent>> {
        let mut events = Vec::new();
        
        // Non-blocking poll with very short timeout
        match self.pointer_type {
            PointerType::TerminalMouse => {
                if event::poll(Duration::from_millis(1))? {
                    if let Event::Mouse(mouse_event) = event::read()? {
                        let timestamp = Instant::now();
                        let x = mouse_event.column as f32;
                        let y = mouse_event.row as f32;
                        
                        match mouse_event.kind {
                            MouseEventKind::Down(btn) => {
                                self.last_click = Some(timestamp);
                                self.last_position = Some((x, y));
                                
                                events.push(SensorEvent::Click {
                                    x,
                                    y,
                                    button: map_button(btn),
                                    timestamp,
                                });
                            }
                            MouseEventKind::Moved => {
                                self.last_position = Some((x, y));
                                
                                events.push(SensorEvent::Position {
                                    x,
                                    y,
                                    timestamp,
                                });
                            }
                            MouseEventKind::ScrollDown => {
                                events.push(SensorEvent::Scroll {
                                    delta_x: 0.0,
                                    delta_y: -1.0,
                                    timestamp,
                                });
                            }
                            MouseEventKind::ScrollUp => {
                                events.push(SensorEvent::Scroll {
                                    delta_x: 0.0,
                                    delta_y: 1.0,
                                    timestamp,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        Ok(events)
    }
    
    fn last_activity(&self) -> Option<Instant> {
        self.last_click
    }
    
    fn name(&self) -> &str {
        match self.pointer_type {
            PointerType::TerminalMouse => "Terminal Mouse",
        }
    }
}

/// Pointer type (discovered at runtime)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerType {
    TerminalMouse,
}

/// Map crossterm button to our MouseButton enum
fn map_button(btn: CrosstermButton) -> MouseButton {
    match btn {
        CrosstermButton::Left => MouseButton::Left,
        CrosstermButton::Right => MouseButton::Right,
        CrosstermButton::Middle => MouseButton::Middle,
    }
}

/// Discover mouse capabilities
pub async fn discover() -> Option<MouseSensor> {
    // Check if terminal supports mouse events
    if atty::is(atty::Stream::Stdout) {
        tracing::debug!("Discovered terminal mouse");
        return Some(MouseSensor::new(PointerType::TerminalMouse));
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mouse_sensor_creation() {
        let sensor = MouseSensor::new(PointerType::TerminalMouse);
        assert_eq!(sensor.capabilities().sensor_type, SensorType::Mouse);
        assert!(sensor.capabilities().input);
        assert!(sensor.capabilities().spatial);
    }
    
    #[test]
    fn test_button_mapping() {
        assert_eq!(map_button(CrosstermButton::Left), MouseButton::Left);
        assert_eq!(map_button(CrosstermButton::Right), MouseButton::Right);
        assert_eq!(map_button(CrosstermButton::Middle), MouseButton::Middle);
    }
}

