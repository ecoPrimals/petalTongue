// SPDX-License-Identifier: AGPL-3.0-only
//! Sensor abstraction layer - Universal input system
//!
//! petalTongue discovers sensors at runtime and understands their capabilities.
//! No hardcoded knowledge of specific devices - only capability-based discovery.

mod ipc_types;
mod registry;
mod types;

pub use ipc_types::{KeyModifiersIpc, SensorEventBatch, SensorEventIpc};
pub use registry::{SensorRegistry, SensorStats};
pub use types::{
    Key, Modifiers, MouseButton, Sensor, SensorCapabilities, SensorCapability, SensorEvent,
    SensorType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

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

    #[test]
    fn test_sensor_event_timestamp() {
        let ts = Instant::now();
        let pos = SensorEvent::Position {
            x: 1.0,
            y: 2.0,
            timestamp: ts,
        };
        assert_eq!(pos.timestamp(), ts);

        let click = SensorEvent::Click {
            x: 0.0,
            y: 0.0,
            button: MouseButton::Left,
            timestamp: ts,
        };
        assert_eq!(click.timestamp(), ts);

        let generic = SensorEvent::Generic {
            data: "test".to_string(),
            timestamp: ts,
        };
        assert_eq!(generic.timestamp(), ts);
    }

    #[test]
    fn test_sensor_event_user_interaction_all_variants() {
        let ts = Instant::now();
        assert!(
            SensorEvent::Click {
                x: 0.0,
                y: 0.0,
                button: MouseButton::Left,
                timestamp: ts,
            }
            .is_user_interaction()
        );
        assert!(
            SensorEvent::KeyPress {
                key: Key::Char('a'),
                modifiers: Modifiers::none(),
                timestamp: ts,
            }
            .is_user_interaction()
        );
        assert!(
            SensorEvent::ButtonPress {
                button: 1,
                timestamp: ts,
            }
            .is_user_interaction()
        );
        assert!(
            SensorEvent::Scroll {
                delta_x: 0.0,
                delta_y: 1.0,
                timestamp: ts,
            }
            .is_user_interaction()
        );

        assert!(
            !SensorEvent::Position {
                x: 0.0,
                y: 0.0,
                timestamp: ts
            }
            .is_user_interaction()
        );
        assert!(
            !SensorEvent::KeyRelease {
                key: Key::Escape,
                modifiers: Modifiers::none(),
                timestamp: ts,
            }
            .is_user_interaction()
        );
    }

    #[test]
    fn test_sensor_event_confirmation_all_variants() {
        let ts = Instant::now();
        assert!(
            SensorEvent::Heartbeat {
                latency: std::time::Duration::ZERO,
                timestamp: ts,
            }
            .is_confirmation()
        );
        assert!(
            SensorEvent::FrameAcknowledged {
                frame_id: 1,
                timestamp: ts,
            }
            .is_confirmation()
        );
        assert!(
            SensorEvent::DisplayVisible {
                visible: true,
                timestamp: ts,
            }
            .is_confirmation()
        );

        assert!(
            !SensorEvent::Position {
                x: 0.0,
                y: 0.0,
                timestamp: ts
            }
            .is_confirmation()
        );
    }

    #[test]
    fn test_sensor_capabilities_all() {
        let caps = SensorCapabilities {
            sensor_type: SensorType::Mouse,
            input: true,
            output: false,
            spatial: true,
            temporal: true,
            continuous: true,
            discrete: true,
            bidirectional: false,
        };
        assert!(caps.has_capability(SensorCapability::Input));
        assert!(caps.has_capability(SensorCapability::Spatial));
        assert!(caps.has_capability(SensorCapability::Temporal));
        assert!(caps.has_capability(SensorCapability::Continuous));
        assert!(caps.has_capability(SensorCapability::Discrete));
        assert!(!caps.has_capability(SensorCapability::Output));
        assert!(!caps.has_capability(SensorCapability::Bidirectional));
    }

    #[test]
    fn test_sensor_registry_empty() {
        let registry = SensorRegistry::new();
        assert_eq!(registry.sensors().len(), 0);
        assert_eq!(registry.active_count(), 0);
        assert!(!registry.has_capability(SensorCapability::Input));
        let stats = registry.stats();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.active, 0);
    }

    #[test]
    fn test_sensor_registry_default() {
        let registry = SensorRegistry::default();
        assert_eq!(registry.sensors().len(), 0);
    }

    #[test]
    fn test_mouse_button_variants() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_eq!(MouseButton::Right, MouseButton::Right);
        assert_eq!(MouseButton::Middle, MouseButton::Middle);
        assert_eq!(MouseButton::Other(4), MouseButton::Other(4));
    }

    #[test]
    fn test_key_variants() {
        assert_eq!(Key::Char('x'), Key::Char('x'));
        assert_eq!(
            Key::Named("Enter".to_string()),
            Key::Named("Enter".to_string())
        );
        assert_eq!(Key::Escape, Key::Escape);
        assert_eq!(Key::F(1), Key::F(1));
    }

    #[test]
    fn test_sensor_type_variants() {
        assert_eq!(SensorType::Screen, SensorType::Screen);
        assert_eq!(SensorType::Keyboard, SensorType::Keyboard);
        assert_eq!(SensorType::Mouse, SensorType::Mouse);
    }

    #[test]
    fn test_sensor_capability_all_variants() {
        let caps = SensorCapabilities {
            sensor_type: SensorType::Mouse,
            input: true,
            output: true,
            spatial: true,
            temporal: true,
            continuous: true,
            discrete: true,
            bidirectional: true,
        };
        assert!(caps.has_capability(SensorCapability::Input));
        assert!(caps.has_capability(SensorCapability::Output));
        assert!(caps.has_capability(SensorCapability::Spatial));
        assert!(caps.has_capability(SensorCapability::Temporal));
        assert!(caps.has_capability(SensorCapability::Continuous));
        assert!(caps.has_capability(SensorCapability::Discrete));
        assert!(caps.has_capability(SensorCapability::Bidirectional));
    }

    #[test]
    fn test_sensor_event_timestamp_all_variants() {
        let ts = Instant::now();
        assert_eq!(
            SensorEvent::AudioLevel {
                amplitude: 0.5,
                frequency: Some(440.0),
                timestamp: ts,
            }
            .timestamp(),
            ts
        );
        assert_eq!(
            SensorEvent::Temperature {
                celsius: 25.0,
                timestamp: ts,
            }
            .timestamp(),
            ts
        );
    }

    #[test]
    fn test_modifiers_alt_shift_meta() {
        let m = Modifiers {
            ctrl: false,
            alt: true,
            shift: true,
            meta: true,
        };
        assert!(m.alt && m.shift && m.meta);
    }

    #[test]
    fn test_sensor_registry_register_and_sensors_by_type() {
        use crate::sensor::registry::mock_sensor::MockSensor;
        let mut registry = SensorRegistry::new();
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
        registry.register(Box::new(MockSensor::new("kb", caps)));
        assert_eq!(registry.sensors().len(), 1);
        let by_type = registry.sensors_by_type(SensorType::Keyboard);
        assert_eq!(by_type.len(), 1);
        assert!(registry.has_capability(SensorCapability::Input));
    }

    #[tokio::test]
    async fn test_sensor_registry_poll_all() {
        use crate::sensor::registry::mock_sensor::MockSensor;
        let mut registry = SensorRegistry::new();
        let caps = SensorCapabilities {
            sensor_type: SensorType::Mouse,
            input: true,
            output: false,
            spatial: true,
            temporal: true,
            continuous: true,
            discrete: true,
            bidirectional: false,
        };
        registry.register(Box::new(MockSensor::new("mouse", caps)));
        let events = registry.poll_all().await.expect("poll");
        assert!(events.is_empty());
        assert!(registry.stats().last_poll.is_some());
    }

    #[test]
    fn test_sensor_event_batch_construction() {
        let batch = SensorEventBatch::new("sub-123".to_string());
        assert_eq!(batch.subscription_id, "sub-123");
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_sensor_event_batch_with_events() {
        let mut batch = SensorEventBatch::new("sub".to_string());
        batch.events.push(SensorEventIpc::PointerMove {
            x: 1.0,
            y: 2.0,
            timestamp_ms: 1000,
        });
        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_key_modifiers_ipc_construction() {
        let mods = KeyModifiersIpc {
            ctrl: true,
            alt: false,
            shift: true,
            meta: false,
        };
        assert!(mods.ctrl && mods.shift);
    }

    #[test]
    fn test_key_modifiers_ipc_default() {
        let mods = KeyModifiersIpc::default();
        assert!(!mods.ctrl && !mods.alt && !mods.shift && !mods.meta);
    }

    #[test]
    fn test_sensor_event_ipc_serde_roundtrip() {
        let event = SensorEventIpc::Click {
            x: 10.0,
            y: 20.0,
            button: "right".to_string(),
            timestamp_ms: 5000,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: SensorEventIpc = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, event);
    }
}
