// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensor types: trait, capabilities, events, input types.

use std::time::Instant;

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
#[expect(
    clippy::struct_excessive_bools,
    reason = "capability flags are naturally boolean"
)]
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
    /// A compute provider (or the OS accessibility layer) provides the transcript;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn ts() -> Instant {
        Instant::now()
    }

    #[test]
    fn sensor_error_display_strings() {
        assert_eq!(
            SensorError::Unavailable("camera".into()).to_string(),
            "sensor unavailable: camera"
        );
        assert_eq!(
            SensorError::Io(io::Error::new(io::ErrorKind::NotFound, "device")).to_string(),
            "sensor I/O error: device"
        );
        assert_eq!(
            SensorError::DataFormat("bad frame".into()).to_string(),
            "sensor data error: bad frame"
        );
        assert_eq!(
            SensorError::Timeout("poll".into()).to_string(),
            "sensor timeout: poll"
        );
    }

    #[test]
    fn has_capability_returns_correct_booleans() {
        let caps = SensorCapabilities {
            sensor_type: SensorType::Mouse,
            input: true,
            output: false,
            spatial: true,
            temporal: false,
            continuous: true,
            discrete: true,
            bidirectional: false,
        };
        assert!(caps.has_capability(SensorCapability::Input));
        assert!(!caps.has_capability(SensorCapability::Output));
        assert!(caps.has_capability(SensorCapability::Spatial));
        assert!(!caps.has_capability(SensorCapability::Temporal));
        assert!(caps.has_capability(SensorCapability::Continuous));
        assert!(caps.has_capability(SensorCapability::Discrete));
        assert!(!caps.has_capability(SensorCapability::Bidirectional));
    }

    #[test]
    fn sensor_event_timestamp_extracts_correctly() {
        let t_click = ts();
        let click = SensorEvent::Click {
            x: 1.0,
            y: 2.0,
            button: MouseButton::Left,
            timestamp: t_click,
        };
        assert_eq!(click.timestamp(), t_click);

        let t_heartbeat = ts();
        let heartbeat = SensorEvent::Heartbeat {
            latency: std::time::Duration::from_millis(5),
            timestamp: t_heartbeat,
        };
        assert_eq!(heartbeat.timestamp(), t_heartbeat);

        let t_generic = ts();
        let generic = SensorEvent::Generic {
            data: "payload".into(),
            timestamp: t_generic,
        };
        assert_eq!(generic.timestamp(), t_generic);
    }

    #[test]
    fn is_user_interaction_true_for_interactive_events() {
        let t = ts();
        let interactive = [
            SensorEvent::Click {
                x: 0.0,
                y: 0.0,
                button: MouseButton::Left,
                timestamp: t,
            },
            SensorEvent::KeyPress {
                key: Key::Enter,
                modifiers: Modifiers::none(),
                timestamp: t,
            },
            SensorEvent::ButtonPress {
                button: 1,
                timestamp: t,
            },
            SensorEvent::Scroll {
                delta_x: 0.0,
                delta_y: 1.0,
                timestamp: t,
            },
            SensorEvent::VoiceCommand {
                transcript: "hello".into(),
                confidence: 0.9,
                timestamp: t,
            },
            SensorEvent::Gesture {
                gesture_type: GestureType::Wave,
                magnitude: 0.5,
                timestamp: t,
            },
            SensorEvent::Touch {
                x: 10.0,
                y: 20.0,
                pressure: 1.0,
                timestamp: t,
            },
            SensorEvent::GazePosition {
                x: 100.0,
                y: 200.0,
                fixation_ms: 50,
                timestamp: t,
            },
            SensorEvent::SwitchActivation {
                switch_id: 0,
                timestamp: t,
            },
            SensorEvent::AgentCommand {
                intent: "select".into(),
                parameters: serde_json::json!({}),
                timestamp: t,
            },
        ];
        for event in interactive {
            assert!(event.is_user_interaction());
        }
    }

    #[test]
    fn is_user_interaction_false_for_passive_events() {
        let t = ts();
        let passive = [
            SensorEvent::Position {
                x: 0.0,
                y: 0.0,
                timestamp: t,
            },
            SensorEvent::Heartbeat {
                latency: std::time::Duration::ZERO,
                timestamp: t,
            },
            SensorEvent::FrameAcknowledged {
                frame_id: 42,
                timestamp: t,
            },
            SensorEvent::DisplayVisible {
                visible: true,
                timestamp: t,
            },
            SensorEvent::AudioLevel {
                amplitude: 0.5,
                frequency: None,
                timestamp: t,
            },
            SensorEvent::Temperature {
                celsius: 22.0,
                timestamp: t,
            },
            SensorEvent::Generic {
                data: "raw".into(),
                timestamp: t,
            },
        ];
        for event in passive {
            assert!(!event.is_user_interaction());
        }
    }

    #[test]
    fn is_confirmation_true_for_confirmation_events() {
        let t = ts();
        let confirmations = [
            SensorEvent::Heartbeat {
                latency: std::time::Duration::from_millis(1),
                timestamp: t,
            },
            SensorEvent::FrameAcknowledged {
                frame_id: 1,
                timestamp: t,
            },
            SensorEvent::DisplayVisible {
                visible: false,
                timestamp: t,
            },
        ];
        for event in confirmations {
            assert!(event.is_confirmation());
        }
    }

    #[test]
    fn is_confirmation_false_for_non_confirmation_events() {
        let t = ts();
        let non_confirmations = [
            SensorEvent::Click {
                x: 0.0,
                y: 0.0,
                button: MouseButton::Left,
                timestamp: t,
            },
            SensorEvent::KeyPress {
                key: Key::Tab,
                modifiers: Modifiers::none(),
                timestamp: t,
            },
            SensorEvent::Position {
                x: 1.0,
                y: 1.0,
                timestamp: t,
            },
            SensorEvent::Generic {
                data: "x".into(),
                timestamp: t,
            },
        ];
        for event in non_confirmations {
            assert!(!event.is_confirmation());
        }
    }

    #[test]
    fn modifiers_none_all_false() {
        let m = Modifiers::none();
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.shift);
        assert!(!m.meta);
    }

    #[test]
    fn modifiers_ctrl_only_ctrl_true() {
        let m = Modifiers::ctrl();
        assert!(m.ctrl);
        assert!(!m.alt);
        assert!(!m.shift);
        assert!(!m.meta);
    }

    #[test]
    fn key_debug_format() {
        assert_eq!(format!("{:?}", Key::Escape), "Escape");
        assert_eq!(format!("{:?}", Key::Char('a')), "Char('a')");
        assert_eq!(format!("{:?}", Key::F(5)), "F(5)");
    }

    #[test]
    fn mouse_button_debug_format() {
        assert_eq!(format!("{:?}", MouseButton::Left), "Left");
        assert_eq!(format!("{:?}", MouseButton::Other(4)), "Other(4)");
    }

    #[test]
    fn gesture_type_equality() {
        assert_eq!(GestureType::Wave, GestureType::Wave);
        assert_eq!(
            GestureType::Swipe(GestureDirection::Up),
            GestureType::Swipe(GestureDirection::Up)
        );
        assert_ne!(
            GestureType::Swipe(GestureDirection::Up),
            GestureType::Swipe(GestureDirection::Down)
        );
        assert_ne!(GestureType::PinchIn, GestureType::PinchOut);
    }

    #[test]
    fn gesture_direction_equality() {
        assert_eq!(GestureDirection::Left, GestureDirection::Left);
        assert_ne!(GestureDirection::Left, GestureDirection::Right);
    }
}
