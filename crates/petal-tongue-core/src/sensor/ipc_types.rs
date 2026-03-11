// SPDX-License-Identifier: AGPL-3.0-only
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
        let batch = SensorEventBatch::new("sub-1".to_string());
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.subscription_id, "sub-1");
    }

    #[test]
    fn batch_with_events() {
        let mut batch = SensorEventBatch::new("sub-2".to_string());
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
            button: "left".to_string(),
            timestamp_ms: 100,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("\"type\":\"click\""));
        assert!(json.contains("\"button\":\"left\""));
    }

    #[test]
    fn serialize_key_press() {
        let event = SensorEventIpc::KeyPress {
            key: "A".to_string(),
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
            key: "B".to_string(),
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
        let mut batch = SensorEventBatch::new("sub-rt".to_string());
        batch.events.push(SensorEventIpc::PointerMove {
            x: 1.0,
            y: 2.0,
            timestamp_ms: 10,
        });
        batch.events.push(SensorEventIpc::Click {
            x: 1.0,
            y: 2.0,
            button: "left".to_string(),
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
}
