// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensor types: trait, capabilities, events, input types.

use std::time::Instant;

use async_trait::async_trait;

/// Typed error for sensor polling failures.
#[derive(Debug, thiserror::Error)]
pub enum SensorError {
    /// The sensor device is not currently available.
    #[error("sensor unavailable: {0}")]
    Unavailable(String),

    /// An I/O error occurred during polling.
    #[error("sensor I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The sensor returned malformed or unparseable data.
    #[error("sensor data error: {0}")]
    DataFormat(String),

    /// A timeout occurred waiting for sensor response.
    #[error("sensor timeout: {0}")]
    Timeout(String),
}

/// Universal sensor trait - any input device implements this
#[async_trait]
pub trait Sensor: Send + Sync {
    /// Get sensor capabilities
    fn capabilities(&self) -> &SensorCapabilities;

    /// Check if sensor is currently available
    fn is_available(&self) -> bool;

    /// Poll for new events (non-blocking)
    async fn poll_events(&mut self) -> Result<Vec<SensorEvent>, SensorError>;

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
    /// Display output (terminal, framebuffer, surface)
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

    /// Touchscreen / pressure surface
    Touch,

    /// Eye/gaze tracking device
    EyeTracker,

    /// Binary switch device (sip-and-puff, head switch, BCI binary)
    Switch,

    /// Agentic AI / machine interactor (Squirrel, API client)
    Agent,

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

    /// Display perceivability changed (app focused/unfocused)
    DisplayVisible {
        /// Whether the display is perceivable to the user
        visible: bool,
        /// When the event occurred
        timestamp: Instant,
    },

    /// Voice/speech command from microphone or speech recognizer.
    ///
    /// Toadstool (or the OS accessibility layer) provides the transcript;
    /// petalTongue maps it to an `InteractionIntent`.
    VoiceCommand {
        /// Recognized speech transcript.
        transcript: String,
        /// Recognition confidence (0.0–1.0).
        confidence: f64,
        /// When the event occurred.
        timestamp: Instant,
    },

    /// Gesture event (hand, body, or device motion).
    Gesture {
        /// What type of gesture was detected.
        gesture_type: GestureType,
        /// Magnitude or intensity (0.0–1.0 normalized).
        magnitude: f64,
        /// When the event occurred.
        timestamp: Instant,
    },

    /// Touch event from a touchscreen or pressure surface.
    Touch {
        /// X coordinate (display-space pixels).
        x: f32,
        /// Y coordinate (display-space pixels).
        y: f32,
        /// Pressure level (0.0–1.0, 0.0 = hover if supported).
        pressure: f32,
        /// When the event occurred.
        timestamp: Instant,
    },

    /// Eye/gaze tracking position.
    GazePosition {
        /// X coordinate on display (pixels).
        x: f32,
        /// Y coordinate on display (pixels).
        y: f32,
        /// How long the gaze has been fixated at this position (ms).
        fixation_ms: u64,
        /// When the event occurred.
        timestamp: Instant,
    },

    /// Single-switch or binary input activation (sip-and-puff, head switch, blink).
    SwitchActivation {
        /// Which switch was activated (0-indexed).
        switch_id: u8,
        /// When the event occurred.
        timestamp: Instant,
    },

    /// Command from an agentic AI (Squirrel or other machine interactor).
    AgentCommand {
        /// Semantic intent expressed as a verb (e.g. "select", "navigate").
        intent: String,
        /// Structured parameters for the command.
        parameters: serde_json::Value,
        /// When the event occurred.
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

/// Classification of gesture events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureType {
    /// Swipe in a direction.
    Swipe(GestureDirection),
    /// Pinch (zoom in).
    PinchIn,
    /// Spread (zoom out).
    PinchOut,
    /// Rotation gesture.
    Rotate,
    /// Wave / attention-getting gesture.
    Wave,
    /// Point at target.
    Point,
    /// Grab / grip.
    Grab,
    /// Release / open hand.
    Release,
    /// Custom gesture with a name.
    Custom(String),
}

/// Direction for swipe/directional gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureDirection {
    /// Swipe up.
    Up,
    /// Swipe down.
    Down,
    /// Swipe left.
    Left,
    /// Swipe right.
    Right,
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
            | Self::VoiceCommand { timestamp, .. }
            | Self::Gesture { timestamp, .. }
            | Self::Touch { timestamp, .. }
            | Self::GazePosition { timestamp, .. }
            | Self::SwitchActivation { timestamp, .. }
            | Self::AgentCommand { timestamp, .. }
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
                | Self::VoiceCommand { .. }
                | Self::Gesture { .. }
                | Self::Touch { .. }
                | Self::GazePosition { .. }
                | Self::SwitchActivation { .. }
                | Self::AgentCommand { .. }
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
    /// Named key (e.g. "Space", "`CapsLock`") for non-printable keys
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
