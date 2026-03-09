// SPDX-License-Identifier: AGPL-3.0-only
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
#[allow(clippy::struct_excessive_bools)]
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
    #[must_use]
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

/// Specific capabilities to query for - sensor characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorCapability {
    /// Accepts input from user/environment
    Input,
    /// Provides output to user/environment
    Output,
    /// Tracks spatial position/movement
    Spatial,
    /// Tracks temporal changes/events
    Temporal,
    /// Provides continuous stream of data
    Continuous,
    /// Provides discrete events/readings
    Discrete,
    /// Supports bidirectional communication
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
    /// Mouse/pointer position update
    Position {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
        /// When event occurred
        timestamp: Instant,
    },

    /// Mouse/pointer click event
    Click {
        /// X coordinate of click
        x: f32,
        /// Y coordinate of click
        y: f32,
        /// Which button was clicked
        button: MouseButton,
        /// When click occurred
        timestamp: Instant,
    },

    /// Scroll wheel event
    Scroll {
        /// Horizontal scroll delta
        delta_x: f32,
        /// Vertical scroll delta
        delta_y: f32,
        /// When scroll occurred
        timestamp: Instant,
    },

    // Discrete events
    /// Keyboard key press event
    KeyPress {
        /// Which key was pressed
        key: Key,
        /// Active modifier keys (Ctrl, Shift, Alt, etc.)
        modifiers: Modifiers,
        /// When key was pressed
        timestamp: Instant,
    },

    /// Keyboard key release event
    KeyRelease {
        /// Which key was released
        key: Key,
        /// Active modifier keys
        modifiers: Modifiers,
        /// When key was released
        timestamp: Instant,
    },

    /// Generic button press event
    ButtonPress {
        /// Button identifier
        button: u8,
        /// When button was pressed
        timestamp: Instant,
    },

    // Continuous events
    /// Audio input level measurement
    AudioLevel {
        /// Sound amplitude (0.0 to 1.0)
        amplitude: f32,
        /// Dominant frequency if detected
        frequency: Option<f32>,
        /// When measured
        timestamp: Instant,
    },

    /// Temperature sensor reading
    Temperature {
        /// Temperature in Celsius
        celsius: f32,
        /// When measured
        timestamp: Instant,
    },

    // Confirmation events (sensory feedback)
    /// Heartbeat confirmation from display backend
    Heartbeat {
        /// Round-trip latency
        latency: std::time::Duration,
        /// When measured
        timestamp: Instant,
    },

    /// Confirmation that a rendered frame was displayed
    FrameAcknowledged {
        /// ID of acknowledged frame
        frame_id: u64,
        /// When acknowledged
        timestamp: Instant,
    },

    /// Display visibility changed (app focused/unfocused)
    DisplayVisible {
        /// Whether display is currently visible
        visible: bool,
        /// When visibility changed
        timestamp: Instant,
    },

    // Generic event for unknown types
    /// Generic event for extensibility
    Generic {
        /// Event data as string
        data: String,
        /// When event occurred
        timestamp: Instant,
    },
}

impl SensorEvent {
    /// Get timestamp of this event
    #[must_use]
    pub fn timestamp(&self) -> Instant {
        match self {
            SensorEvent::Position { timestamp, .. }
            | SensorEvent::Click { timestamp, .. }
            | SensorEvent::Scroll { timestamp, .. }
            | SensorEvent::KeyPress { timestamp, .. }
            | SensorEvent::KeyRelease { timestamp, .. }
            | SensorEvent::ButtonPress { timestamp, .. }
            | SensorEvent::AudioLevel { timestamp, .. }
            | SensorEvent::Temperature { timestamp, .. }
            | SensorEvent::Heartbeat { timestamp, .. }
            | SensorEvent::FrameAcknowledged { timestamp, .. }
            | SensorEvent::DisplayVisible { timestamp, .. }
            | SensorEvent::Generic { timestamp, .. } => *timestamp,
        }
    }

    /// Check if this is a user interaction event
    #[must_use]
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
    #[must_use]
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
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button
    Middle,
    /// Other mouse button (by ID)
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
    /// Escape key
    Escape,
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,

    /// Arrow keys
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,

    /// Function keys
    F(u8),

    /// Unknown key
    Unknown,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    /// Control key pressed
    pub ctrl: bool,
    /// Alt key pressed
    pub alt: bool,
    /// Shift key pressed
    pub shift: bool,
    /// Meta/Command/Windows key pressed
    pub meta: bool,
}

impl Modifiers {
    /// No modifiers pressed
    #[must_use]
    pub fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    /// Only Ctrl modifier pressed
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn sensors(&self) -> &[Box<dyn Sensor>] {
        &self.sensors
    }

    /// Get sensors by type
    #[must_use]
    pub fn sensors_by_type(&self, sensor_type: SensorType) -> Vec<&dyn Sensor> {
        self.sensors
            .iter()
            .filter(|s| s.capabilities().sensor_type == sensor_type)
            .map(std::convert::AsRef::as_ref)
            .collect()
    }

    /// Check if we have a sensor with specific capability
    #[must_use]
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
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.sensors.iter().filter(|s| s.is_available()).count()
    }

    /// Get sensor statistics
    #[must_use]
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

/// Sensor statistics for runtime tracking
#[derive(Debug, Clone)]
pub struct SensorStats {
    /// Total number of sensors registered
    pub total: usize,
    /// Number of currently active sensors
    pub active: usize,
    /// Whether this sensor can receive input
    pub has_input: bool,
    /// Whether this sensor can provide output
    pub has_output: bool,
    /// Whether this sensor supports bidirectional communication
    pub has_bidirectional: bool,
    /// Timestamp of last poll operation
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
