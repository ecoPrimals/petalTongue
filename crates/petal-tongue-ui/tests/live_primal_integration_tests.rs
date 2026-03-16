// SPDX-License-Identifier: AGPL-3.0-only
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests exercising the full sensor → subscriber → visualization pipeline.
//!
//! These tests verify the wiring between petalTongue components without requiring
//! live primals. For testing with real biomeOS/ludoSpring, see `docs/LIVE_TESTING.md`.

use petal_tongue_core::{DataBinding, KeyModifiersIpc, SensorEventBatch, SensorEventIpc};
use petal_tongue_ipc::{
    InteractionEventNotification, InteractionSubscriberRegistry, SensorStreamRegistry,
    VisualizationRenderRequest, VisualizationState,
};
use serde_json::json;
use std::sync::{Arc, RwLock};

// ─────────────────────────────────────────────────────────────────────
// Sensor stream pipeline
// ─────────────────────────────────────────────────────────────────────

#[test]
fn sensor_stream_subscribe_broadcast_poll() {
    let reg = Arc::new(RwLock::new(SensorStreamRegistry::new()));

    // A mock ludoSpring subscriber registers
    let sub_id = reg.write().unwrap().subscribe();
    assert!(sub_id.starts_with("sensor-sub-"));

    // UI broadcasts pointer + key events (simulating sensor_feed output)
    let events = vec![
        SensorEventIpc::PointerMove {
            x: 100.0,
            y: 200.0,
            timestamp_ms: 1000,
        },
        SensorEventIpc::Click {
            x: 100.0,
            y: 200.0,
            button: "left".to_string(),
            timestamp_ms: 1001,
        },
        SensorEventIpc::KeyPress {
            key: "Space".to_string(),
            modifiers: KeyModifiersIpc::default(),
            timestamp_ms: 1002,
        },
    ];

    reg.write().unwrap().broadcast(&events);

    // Subscriber polls and receives all events
    let batch: SensorEventBatch = reg.write().unwrap().poll(&sub_id);
    assert_eq!(batch.events.len(), 3);
    assert_eq!(batch.subscription_id, sub_id);

    // Second poll should be empty (events were drained)
    let batch2 = reg.write().unwrap().poll(&sub_id);
    assert!(batch2.is_empty());
}

#[test]
fn sensor_stream_multiple_subscribers() {
    let reg = Arc::new(RwLock::new(SensorStreamRegistry::new()));

    let sub1 = reg.write().unwrap().subscribe();
    let sub2 = reg.write().unwrap().subscribe();

    let events = vec![SensorEventIpc::Scroll {
        delta_x: 0.0,
        delta_y: -5.0,
        timestamp_ms: 2000,
    }];

    reg.write().unwrap().broadcast(&events);

    assert_eq!(reg.write().unwrap().poll(&sub1).events.len(), 1);
    assert_eq!(reg.write().unwrap().poll(&sub2).events.len(), 1);
}

#[test]
fn sensor_stream_unsubscribe_stops_delivery() {
    let reg = Arc::new(RwLock::new(SensorStreamRegistry::new()));

    let sub = reg.write().unwrap().subscribe();
    reg.write().unwrap().unsubscribe(&sub);

    let events = vec![SensorEventIpc::PointerMove {
        x: 50.0,
        y: 50.0,
        timestamp_ms: 3000,
    }];
    reg.write().unwrap().broadcast(&events);

    let batch = reg.write().unwrap().poll(&sub);
    assert!(batch.events.is_empty());
}

// ─────────────────────────────────────────────────────────────────────
// Interaction event pipeline
// ─────────────────────────────────────────────────────────────────────

#[test]
fn interaction_subscribe_broadcast_poll() {
    let reg = Arc::new(RwLock::new(InteractionSubscriberRegistry::new()));

    let subscribed = reg.write().unwrap().subscribe("mock-ludospring");
    assert!(subscribed);

    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec!["node-42".to_string()],
        timestamp: "2026-03-10T00:00:00Z".to_string(),
        perspective_id: None,
    };

    reg.write().unwrap().broadcast(&event);

    let polled = reg.write().unwrap().poll("mock-ludospring");
    assert_eq!(polled.len(), 1);
    assert_eq!(polled[0].event_type, "select");
    assert_eq!(polled[0].targets, vec!["node-42"]);
}

#[test]
fn interaction_deselect_event() {
    let reg = Arc::new(RwLock::new(InteractionSubscriberRegistry::new()));
    reg.write().unwrap().subscribe("observer");

    let event = InteractionEventNotification {
        event_type: "deselect".to_string(),
        targets: vec![],
        timestamp: "2026-03-10T00:00:01Z".to_string(),
        perspective_id: None,
    };

    reg.write().unwrap().broadcast(&event);

    let polled = reg.write().unwrap().poll("observer");
    assert_eq!(polled.len(), 1);
    assert_eq!(polled[0].event_type, "deselect");
    assert!(polled[0].targets.is_empty());
}

// ─────────────────────────────────────────────────────────────────────
// Game data channel → DataBinding mapping
// ─────────────────────────────────────────────────────────────────────

#[test]
fn game_channel_engagement_roundtrip() {
    let payload = json!({
        "channel": "EngagementCurve",
        "timestamps": [0.0, 1.0, 2.0, 3.0, 4.0],
        "engagement": [0.1, 0.4, 0.8, 0.6, 0.3]
    });

    let binding =
        petal_tongue_ui::game_data_channel::map_game_channel(&payload).expect("should map");

    match &binding {
        DataBinding::TimeSeries {
            x_values, y_values, ..
        } => {
            assert_eq!(x_values.len(), 5);
            assert_eq!(y_values.len(), 5);
        }
        _ => panic!("expected TimeSeries"),
    }

    // Verify the binding serializes correctly for IPC
    let json_str = serde_json::to_string(&binding).expect("serialize");
    assert!(json_str.contains("timeseries"));
}

#[test]
fn game_channel_flow_timeline_roundtrip() {
    let payload = json!({
        "channel": "FlowTimeline",
        "flow_states": ["anxiety", "flow", "boredom", "flow"],
        "durations": [5.0, 30.0, 3.0, 22.0]
    });

    let binding =
        petal_tongue_ui::game_data_channel::map_game_channel(&payload).expect("should map");

    match &binding {
        DataBinding::Bar {
            categories, values, ..
        } => {
            assert_eq!(categories.len(), 4);
            assert_eq!(values.len(), 4);
        }
        _ => panic!("expected Bar"),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Full pipeline: sensor → broadcast → game channel → visualization state
// ─────────────────────────────────────────────────────────────────────

#[test]
fn full_pipeline_sensor_to_game_visualization() {
    // 1. Create shared registries (simulating IPC server)
    let sensor_reg = Arc::new(RwLock::new(SensorStreamRegistry::new()));
    let viz_state = Arc::new(RwLock::new(VisualizationState::new()));

    // 2. "ludoSpring" subscribes to sensor stream
    let sub_id = sensor_reg.write().unwrap().subscribe();

    // 3. UI broadcasts sensor events
    let events = vec![
        SensorEventIpc::PointerMove {
            x: 400.0,
            y: 300.0,
            timestamp_ms: 5000,
        },
        SensorEventIpc::Click {
            x: 400.0,
            y: 300.0,
            button: "left".to_string(),
            timestamp_ms: 5001,
        },
    ];
    sensor_reg.write().unwrap().broadcast(&events);

    // 4. ludoSpring polls sensor events
    let batch = sensor_reg.write().unwrap().poll(&sub_id);
    assert_eq!(batch.events.len(), 2);

    // 5. ludoSpring computes engagement from the events and sends game data
    let game_payload = json!({
        "channel": "EngagementCurve",
        "timestamps": [5.0, 5.001],
        "engagement": [0.7, 0.9]
    });
    let binding = petal_tongue_ui::game_data_channel::map_game_channel(&game_payload).expect("map");

    // 6. ludoSpring calls visualization.render (simulated by writing to viz state)
    let render_req = VisualizationRenderRequest {
        session_id: "ludospring-flow-analysis".to_string(),
        title: "Flow Analysis".to_string(),
        bindings: vec![binding],
        thresholds: vec![],
        domain: Some("game".to_string()),
        ui_config: None,
    };
    viz_state.write().unwrap().handle_render(render_req);

    // 7. UI polls visualization state and finds the session
    let sessions = viz_state.read().unwrap();
    let session = sessions
        .sessions()
        .get("ludospring-flow-analysis")
        .expect("should find ludoSpring session in viz state");
    assert_eq!(session.title, "Flow Analysis");
    assert_eq!(session.domain.as_deref(), Some("game"));
    assert_eq!(session.bindings.len(), 1);
}

// ─────────────────────────────────────────────────────────────────────
// Neural registration capabilities
// ─────────────────────────────────────────────────────────────────────

#[test]
fn neural_registration_capabilities_complete() {
    let caps = petal_tongue_ui::neural_registration::petaltongue_capabilities();
    assert!(caps.contains(&"ui.render"));
    assert!(caps.contains(&"visualization.render"));
    assert!(caps.contains(&"ipc.json-rpc"));
    assert!(caps.contains(&"interaction.sensor_stream"));
}
