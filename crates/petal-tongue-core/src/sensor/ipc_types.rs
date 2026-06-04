// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC-serializable sensor event types.
//!
//! These mirror the internal `SensorEvent` enum but use millisecond timestamps
//! and string keys so they can be serialized over JSON-RPC. Batched per tick
//! (16.67ms at 60 Hz) to avoid flooding the IPC channel.

use serde::{Deserialize, Serialize};

/// A batch of sensor events for IPC delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEventBatch {
    /// Subscription identifier (returned by `interaction.sensor_stream.subscribe`).
    pub subscription_id: String,
    /// Events collected since the last poll or tick.
    pub events: Vec<SensorEventIpc>,
}

/// Keyboard modifier state for IPC.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyModifiersIpc {
    /// Control key held.
    pub ctrl: bool,
    /// Alt / Option key held.
    pub alt: bool,
    /// Shift key held.
    pub shift: bool,
    /// Meta / Super / Command key held.
    pub meta: bool,
}

/// A single sensor event in IPC-serializable form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SensorEventIpc {
    /// Pointer/mouse movement.
    #[serde(rename = "pointer_move")]
    PointerMove {
        /// X coordinate in window-relative pixels.
        x: f32,
        /// Y coordinate in window-relative pixels.
        y: f32,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Mouse/pointer click.
    #[serde(rename = "click")]
    Click {
        /// X coordinate.
        x: f32,
        /// Y coordinate.
        y: f32,
        /// Button name (`"left"`, `"right"`, `"middle"`).
        button: String,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Keyboard key press.
    #[serde(rename = "key_press")]
    KeyPress {
        /// Key name (e.g. `"A"`, `"Enter"`, `"Escape"`).
        key: String,
        /// Active modifier keys.
        modifiers: KeyModifiersIpc,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Keyboard key release.
    #[serde(rename = "key_release")]
    KeyRelease {
        /// Key name.
        key: String,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Scroll wheel event.
    #[serde(rename = "scroll")]
    Scroll {
        /// Horizontal scroll delta.
        delta_x: f32,
        /// Vertical scroll delta.
        delta_y: f32,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Window/viewport gained focus.
    #[serde(rename = "focus_gained")]
    FocusGained {
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Window/viewport lost focus.
    #[serde(rename = "focus_lost")]
    FocusLost {
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Viewport resized.
    #[serde(rename = "window_resize")]
    WindowResize {
        /// New width in logical pixels.
        width: f32,
        /// New height in logical pixels.
        height: f32,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
    /// Text input (character composition).
    #[serde(rename = "text_input")]
    TextInput {
        /// The composed text string.
        text: String,
        /// Unix epoch milliseconds.
        timestamp_ms: u64,
    },
}

impl SensorEventBatch {
    /// Create an empty batch for a given subscription.
    #[must_use]
    pub const fn new(subscription_id: String) -> Self {
        Self {
            subscription_id,
            events: Vec::new(),
        }
    }

    /// Whether this batch has any events.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Number of events in this batch.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_new_is_empty() {
        let batch = SensorEventBatch::new("sub-1".to_owned());
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.subscription_id, "sub-1");
    }

    #[test]
    fn batch_with_events() {
        let mut batch = SensorEventBatch::new("sub-2".to_owned());
        batch.events.push(SensorEventIpc::PointerMove {
            x: 100.0,
            y: 200.0,
            timestamp_ms: 1000,
        });
        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn serialize_pointer_move() {
        let event = SensorEventIpc::PointerMove {
            x: 10.5,
            y: 20.3,
            timestamp_ms: 42,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"pointer_move\""));
        assert!(json.contains("10.5"));
    }

    #[test]
    fn serialize_click() {
        let event = SensorEventIpc::Click {
            x: 5.0,
            y: 6.0,
            button: "left".to_owned(),
            timestamp_ms: 100,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"click\""));
        assert!(json.contains("\"button\":\"left\""));
    }

    #[test]
    fn serialize_key_press() {
        let event = SensorEventIpc::KeyPress {
            key: "A".to_owned(),
            modifiers: KeyModifiersIpc {
                ctrl: true,
                ..Default::default()
            },
            timestamp_ms: 200,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"key_press\""));
        assert!(json.contains("\"ctrl\":true"));
    }

    #[test]
    fn serialize_key_release() {
        let event = SensorEventIpc::KeyRelease {
            key: "B".to_owned(),
            timestamp_ms: 300,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"key_release\""));
    }

    #[test]
    fn serialize_scroll() {
        let event = SensorEventIpc::Scroll {
            delta_x: 0.0,
            delta_y: -3.5,
            timestamp_ms: 400,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"scroll\""));
    }

    #[test]
    fn roundtrip_batch() {
        let mut batch = SensorEventBatch::new("sub-rt".to_owned());
        batch.events.push(SensorEventIpc::PointerMove {
            x: 1.0,
            y: 2.0,
            timestamp_ms: 10,
        });
        batch.events.push(SensorEventIpc::Click {
            x: 1.0,
            y: 2.0,
            button: "left".to_owned(),
            timestamp_ms: 20,
        });
        let json = serde_json::to_string(&batch).expect("serialize");
        let decoded: SensorEventBatch = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.subscription_id, "sub-rt");
        assert_eq!(decoded.events.len(), 2);
    }

    #[test]
    fn key_modifiers_default() {
        let mods = KeyModifiersIpc::default();
        assert!(!mods.ctrl && !mods.alt && !mods.shift && !mods.meta);
    }

    #[test]
    fn serialize_focus_gained() {
        let event = SensorEventIpc::FocusGained { timestamp_ms: 500 };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"focus_gained\""));
    }

    #[test]
    fn serialize_focus_lost() {
        let event = SensorEventIpc::FocusLost { timestamp_ms: 600 };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"focus_lost\""));
    }

    #[test]
    fn serialize_window_resize() {
        let event = SensorEventIpc::WindowResize {
            width: 1920.0,
            height: 1080.0,
            timestamp_ms: 700,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"window_resize\""));
        assert!(json.contains("1920"));
    }

    #[test]
    fn serialize_text_input() {
        let event = SensorEventIpc::TextInput {
            text: "hello".to_owned(),
            timestamp_ms: 800,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"text_input\""));
        assert!(json.contains("\"text\":\"hello\""));
    }

    #[test]
    fn roundtrip_new_event_types() {
        let events = vec![
            SensorEventIpc::FocusGained { timestamp_ms: 1 },
            SensorEventIpc::FocusLost { timestamp_ms: 2 },
            SensorEventIpc::WindowResize {
                width: 800.0,
                height: 600.0,
                timestamp_ms: 3,
            },
            SensorEventIpc::TextInput {
                text: "a".to_owned(),
                timestamp_ms: 4,
            },
        ];
        for event in &events {
            let json = serde_json::to_string(event).expect("serialize");
            let decoded: SensorEventIpc = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&decoded, event);
        }
    }
}
