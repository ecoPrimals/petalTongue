// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Instant;

use super::gesture::GestureType;
use super::input::{Key, Modifiers, MouseButton};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensor::types::{GestureType, Key, Modifiers, MouseButton};
    use std::time::Instant;

    fn ts() -> Instant {
        Instant::now()
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
}
