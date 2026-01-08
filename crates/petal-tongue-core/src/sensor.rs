//! Sensor abstraction layer - Universal input system
//!
//! petalTongue discovers sensors at runtime and understands their capabilities.
//! No hardcoded knowledge of specific devices - only capability-based discovery.

use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

/// Universal sensor trait - any input device implements this
#[async_trait]
pub trait Sensor: Send + Sync {
    /// Get sensor capabilities
    fn capabilities(&self) -> &SensorCapabilities;
    
    /// Check if sensor is currently available
    fn is_available(&self) -> bool;
    
    /// Poll for new events (non-blocking)
    async fn poll_events(&mut self) -> Result<Vec<SensorEvent>>;
    
    /// Get last activity timestamp
    fn last_activity(&self) -> Option<Instant>;
    
    /// Get sensor name (for logging/debugging)
    fn name(&self) -> &str;
}

/// Describes what a sensor can do
#[derive(Debug, Clone)]
pub struct SensorCapabilities {
    /// Type of sensor
    pub sensor_type: SensorType,
    
    /// Can receive input
    pub input: bool,
    
    /// Can provide output
    pub output: bool,
    
    /// Provides spatial data (x, y coordinates)
    pub spatial: bool,
    
    /// Provides temporal data (timing, rhythm)
    pub temporal: bool,
    
    /// Continuous values (analog)
    pub continuous: bool,
    
    /// Discrete events (digital)
    pub discrete: bool,
    
    /// Bidirectional (input AND output)
    pub bidirectional: bool,
}

impl SensorCapabilities {
    /// Check if sensor has a specific capability
    pub fn has_capability(&self, capability: SensorCapability) -> bool {
        match capability {
            SensorCapability::Input => self.input,
            SensorCapability::Output => self.output,
            SensorCapability::Spatial => self.spatial,
            SensorCapability::Temporal => self.temporal,
            SensorCapability::Continuous => self.continuous,
            SensorCapability::Discrete => self.discrete,
            SensorCapability::Bidirectional => self.bidirectional,
        }
    }
}

/// Specific capabilities to query for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorCapability {
    Input,
    Output,
    Spatial,
    Temporal,
    Continuous,
    Discrete,
    Bidirectional,
}

/// Types of sensors (discovered at runtime)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorType {
    /// Display output (terminal, framebuffer, window)
    Screen,
    
    /// Discrete input device (keys, buttons)
    Keyboard,
    
    /// Spatial input device (pointing, clicking)
    Mouse,
    
    /// Audio input/output (microphone, speaker)
    Audio,
    
    /// Visual input (camera, image sensor)
    Camera,
    
    /// Motion detection (accelerometer, gyroscope)
    Motion,
    
    /// Location awareness (GPS, network location)
    Location,
    
    /// Biometric sensor (heart rate, temperature, etc.)
    Biometric,
    
    /// Environmental sensor (temperature, humidity, etc.)
    Environmental,
    
    /// Network sensor (primal discovery, health)
    Network,
    
    /// Unknown sensor type
    Unknown,
}

/// Events from sensors
#[derive(Debug, Clone)]
pub enum SensorEvent {
    // Spatial events
    Position {
        x: f32,
        y: f32,
        timestamp: Instant,
    },
    
    Click {
        x: f32,
        y: f32,
        button: MouseButton,
        timestamp: Instant,
    },
    
    Scroll {
        delta_x: f32,
        delta_y: f32,
        timestamp: Instant,
    },
    
    // Discrete events
    KeyPress {
        key: Key,
        modifiers: Modifiers,
        timestamp: Instant,
    },
    
    KeyRelease {
        key: Key,
        modifiers: Modifiers,
        timestamp: Instant,
    },
    
    ButtonPress {
        button: u8,
        timestamp: Instant,
    },
    
    // Continuous events
    AudioLevel {
        amplitude: f32,
        frequency: Option<f32>,
        timestamp: Instant,
    },
    
    Temperature {
        celsius: f32,
        timestamp: Instant,
    },
    
    // Confirmation events (sensory feedback)
    Heartbeat {
        latency: std::time::Duration,
        timestamp: Instant,
    },
    
    FrameAcknowledged {
        frame_id: u64,
        timestamp: Instant,
    },
    
    DisplayVisible {
        visible: bool,
        timestamp: Instant,
    },
    
    // Generic event for unknown types
    Generic {
        data: String,
        timestamp: Instant,
    },
}

impl SensorEvent {
    /// Get timestamp of this event
    pub fn timestamp(&self) -> Instant {
        match self {
            SensorEvent::Position { timestamp, .. } => *timestamp,
            SensorEvent::Click { timestamp, .. } => *timestamp,
            SensorEvent::Scroll { timestamp, .. } => *timestamp,
            SensorEvent::KeyPress { timestamp, .. } => *timestamp,
            SensorEvent::KeyRelease { timestamp, .. } => *timestamp,
            SensorEvent::ButtonPress { timestamp, .. } => *timestamp,
            SensorEvent::AudioLevel { timestamp, .. } => *timestamp,
            SensorEvent::Temperature { timestamp, .. } => *timestamp,
            SensorEvent::Heartbeat { timestamp, .. } => *timestamp,
            SensorEvent::FrameAcknowledged { timestamp, .. } => *timestamp,
            SensorEvent::DisplayVisible { timestamp, .. } => *timestamp,
            SensorEvent::Generic { timestamp, .. } => *timestamp,
        }
    }
    
    /// Check if this is a user interaction event
    pub fn is_user_interaction(&self) -> bool {
        matches!(
            self,
            SensorEvent::Click { .. }
                | SensorEvent::KeyPress { .. }
                | SensorEvent::ButtonPress { .. }
                | SensorEvent::Scroll { .. }
        )
    }
    
    /// Check if this is a confirmation event
    pub fn is_confirmation(&self) -> bool {
        matches!(
            self,
            SensorEvent::Heartbeat { .. }
                | SensorEvent::FrameAcknowledged { .. }
                | SensorEvent::DisplayVisible { .. }
        )
    }
}

/// Mouse button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Key identifier (no hardcoded keyboard layout)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// Character key
    Char(char),
    
    /// Named key
    Named(String),
    
    /// Special keys
    Escape,
    Enter,
    Tab,
    Backspace,
    Delete,
    
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    
    /// Function keys
    F(u8),
    
    /// Unknown key
    Unknown,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }
    
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::none()
        }
    }
}

/// Sensor registry - discovers and manages all sensors
pub struct SensorRegistry {
    sensors: Vec<Box<dyn Sensor>>,
    last_poll: Option<Instant>,
}

impl SensorRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
            last_poll: None,
        }
    }
    
    /// Register a sensor
    pub fn register(&mut self, sensor: Box<dyn Sensor>) {
        self.sensors.push(sensor);
    }
    
    /// Get all sensors
    pub fn sensors(&self) -> &[Box<dyn Sensor>] {
        &self.sensors
    }
    
    /// Get sensors by type
    pub fn sensors_by_type(&self, sensor_type: SensorType) -> Vec<&Box<dyn Sensor>> {
        self.sensors
            .iter()
            .filter(|s| s.capabilities().sensor_type == sensor_type)
            .collect()
    }
    
    /// Check if we have a sensor with specific capability
    pub fn has_capability(&self, capability: SensorCapability) -> bool {
        self.sensors
            .iter()
            .any(|s| s.capabilities().has_capability(capability))
    }
    
    /// Poll all sensors for events
    pub async fn poll_all(&mut self) -> Result<Vec<SensorEvent>> {
        let mut all_events = Vec::new();
        
        for sensor in &mut self.sensors {
            if sensor.is_available() {
                match sensor.poll_events().await {
                    Ok(events) => all_events.extend(events),
                    Err(e) => {
                        tracing::warn!("Error polling sensor {}: {}", sensor.name(), e);
                    }
                }
            }
        }
        
        self.last_poll = Some(Instant::now());
        Ok(all_events)
    }
    
    /// Get count of active sensors
    pub fn active_count(&self) -> usize {
        self.sensors.iter().filter(|s| s.is_available()).count()
    }
    
    /// Get sensor statistics
    pub fn stats(&self) -> SensorStats {
        let total = self.sensors.len();
        let active = self.active_count();
        let has_input = self.has_capability(SensorCapability::Input);
        let has_output = self.has_capability(SensorCapability::Output);
        let has_bidirectional = self.has_capability(SensorCapability::Bidirectional);
        
        SensorStats {
            total,
            active,
            has_input,
            has_output,
            has_bidirectional,
            last_poll: self.last_poll,
        }
    }
}

impl Default for SensorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Sensor statistics
#[derive(Debug, Clone)]
pub struct SensorStats {
    pub total: usize,
    pub active: usize,
    pub has_input: bool,
    pub has_output: bool,
    pub has_bidirectional: bool,
    pub last_poll: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sensor_capabilities() {
        let caps = SensorCapabilities {
            sensor_type: SensorType::Keyboard,
            input: true,
            output: false,
            spatial: false,
            temporal: true,
            continuous: false,
            discrete: true,
            bidirectional: false,
        };
        
        assert!(caps.has_capability(SensorCapability::Input));
        assert!(!caps.has_capability(SensorCapability::Output));
        assert!(caps.has_capability(SensorCapability::Discrete));
    }
    
    #[test]
    fn test_sensor_event_classification() {
        let click = SensorEvent::Click {
            x: 100.0,
            y: 200.0,
            button: MouseButton::Left,
            timestamp: Instant::now(),
        };
        
        assert!(click.is_user_interaction());
        assert!(!click.is_confirmation());
        
        let heartbeat = SensorEvent::Heartbeat {
            latency: std::time::Duration::from_millis(10),
            timestamp: Instant::now(),
        };
        
        assert!(!heartbeat.is_user_interaction());
        assert!(heartbeat.is_confirmation());
    }
    
    #[test]
    fn test_modifiers() {
        let none = Modifiers::none();
        assert!(!none.ctrl && !none.alt && !none.shift && !none.meta);
        
        let ctrl = Modifiers::ctrl();
        assert!(ctrl.ctrl && !ctrl.alt);
    }
}

