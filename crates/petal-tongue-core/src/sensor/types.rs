// SPDX-License-Identifier: AGPL-3.0-only
//! Sensor types: trait, capabilities, events, input types.

use std::time::Instant;

use async_trait::async_trait;

/// Universal sensor trait - any input device implements this
#[async_trait]
pub trait Sensor: Send + Sync {
    /// Get sensor capabilities
    fn capabilities(&self) -> &SensorCapabilities;

    /// Check if sensor is currently available
    fn is_available(&self) -> bool;

    /// Poll for new events (non-blocking)
    async fn poll_events(&mut self) -> anyhow::Result<Vec<SensorEvent>>;

    /// Get last activity timestamp
    fn last_activity(&self) -> Option<Instant>;

    /// Get sensor name (for logging/debugging)
    fn name(&self) -> &str;
}

/// Describes what a sensor can do
#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools)]
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
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum SensorEvent {
    /// Mouse/pointer position update
    Position { x: f32, y: f32, timestamp: Instant },

    /// Mouse/pointer click event
    Click {
        x: f32,
        y: f32,
        button: MouseButton,
        timestamp: Instant,
    },

    /// Scroll wheel event
    Scroll {
        delta_x: f32,
        delta_y: f32,
        timestamp: Instant,
    },

    /// Keyboard key press event
    KeyPress {
        key: Key,
        modifiers: Modifiers,
        timestamp: Instant,
    },

    /// Keyboard key release event
    KeyRelease {
        key: Key,
        modifiers: Modifiers,
        timestamp: Instant,
    },

    /// Generic button press event
    ButtonPress { button: u8, timestamp: Instant },

    /// Audio input level measurement
    AudioLevel {
        amplitude: f32,
        frequency: Option<f32>,
        timestamp: Instant,
    },

    /// Temperature sensor reading
    Temperature { celsius: f32, timestamp: Instant },

    /// Heartbeat confirmation from display backend
    Heartbeat {
        latency: std::time::Duration,
        timestamp: Instant,
    },

    /// Confirmation that a rendered frame was displayed
    FrameAcknowledged { frame_id: u64, timestamp: Instant },

    /// Display visibility changed (app focused/unfocused)
    DisplayVisible { visible: bool, timestamp: Instant },

    /// Generic event for extensibility
    Generic { data: String, timestamp: Instant },
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
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Key identifier (no hardcoded keyboard layout)
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Named(String),
    Escape,
    Enter,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    F(u8),
    Unknown,
}

/// Keyboard modifiers
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    #[allow(missing_docs)]
    #[must_use]
    pub fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    #[allow(missing_docs)]
    #[must_use]
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::none()
        }
    }
}
