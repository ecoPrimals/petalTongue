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
    pub const fn has_capability(&self, capability: SensorCapability) -> bool {
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

/// Events emitted by sensors (input devices, display backends, etc.).
///
/// Each variant carries a `timestamp` for ordering and latency measurement.
#[derive(Debug, Clone)]
pub enum SensorEvent {
    /// Mouse/pointer position update
    Position {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Mouse/pointer click event
    Click {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
        /// Which button was pressed
        button: MouseButton,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Scroll wheel event
    Scroll {
        /// Horizontal scroll delta
        delta_x: f32,
        /// Vertical scroll delta
        delta_y: f32,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Keyboard key press event
    KeyPress {
        /// Key that was pressed
        key: Key,
        /// Modifier keys held at press time
        modifiers: Modifiers,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Keyboard key release event
    KeyRelease {
        /// Key that was released
        key: Key,
        /// Modifier keys held at release time
        modifiers: Modifiers,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Generic button press event
    ButtonPress {
        /// Button identifier
        button: u8,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Audio input level measurement
    AudioLevel {
        /// Amplitude level
        amplitude: f32,
        /// Dominant frequency if available
        frequency: Option<f32>,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Temperature sensor reading
    Temperature {
        /// Temperature in Celsius
        celsius: f32,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Heartbeat confirmation from display backend
    Heartbeat {
        /// Round-trip latency
        latency: std::time::Duration,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Confirmation that a rendered frame was displayed
    FrameAcknowledged {
        /// Frame identifier
        frame_id: u64,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Display visibility changed (app focused/unfocused)
    DisplayVisible {
        /// Whether the display is visible to the user
        visible: bool,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Generic event for extensibility
    Generic {
        /// Event payload
        data: String,
        /// When the event occurred
        timestamp: Instant,
    },
}

impl SensorEvent {
    /// Get timestamp of this event
    #[must_use]
    pub const fn timestamp(&self) -> Instant {
        match self {
            Self::Position { timestamp, .. }
            | Self::Click { timestamp, .. }
            | Self::Scroll { timestamp, .. }
            | Self::KeyPress { timestamp, .. }
            | Self::KeyRelease { timestamp, .. }
            | Self::ButtonPress { timestamp, .. }
            | Self::AudioLevel { timestamp, .. }
            | Self::Temperature { timestamp, .. }
            | Self::Heartbeat { timestamp, .. }
            | Self::FrameAcknowledged { timestamp, .. }
            | Self::DisplayVisible { timestamp, .. }
            | Self::Generic { timestamp, .. } => *timestamp,
        }
    }

    /// Check if this is a user interaction event
    #[must_use]
    pub const fn is_user_interaction(&self) -> bool {
        matches!(
            self,
            Self::Click { .. }
                | Self::KeyPress { .. }
                | Self::ButtonPress { .. }
                | Self::Scroll { .. }
        )
    }

    /// Check if this is a confirmation event
    #[must_use]
    pub const fn is_confirmation(&self) -> bool {
        matches!(
            self,
            Self::Heartbeat { .. } | Self::FrameAcknowledged { .. } | Self::DisplayVisible { .. }
        )
    }
}

/// Mouse button identifier for click events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button (primary click)
    Left,
    /// Right mouse button (secondary click)
    Right,
    /// Middle mouse button (scroll wheel click)
    Middle,
    /// Other button with raw identifier
    Other(u8),
}

/// Key identifier for keyboard events (layout-agnostic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// Printable character key
    Char(char),
    /// Named key (e.g. "Space", "CapsLock") for non-printable keys
    Named(String),
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
    /// Arrow up
    Up,
    /// Arrow down
    Down,
    /// Arrow left
    Left,
    /// Arrow right
    Right,
    /// Function key (F1 = F(1), etc.)
    F(u8),
    /// Unknown or unmapped key
    Unknown,
}

/// Keyboard modifier state (Ctrl, Alt, Shift, Meta/Cmd).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    /// Control key held
    pub ctrl: bool,
    /// Alt/Option key held
    pub alt: bool,
    /// Shift key held
    pub shift: bool,
    /// Meta/Windows/Cmd key held
    pub meta: bool,
}

impl Modifiers {
    /// No modifiers pressed.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    /// Ctrl modifier only.
    #[must_use]
    pub const fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::none()
        }
    }
}
