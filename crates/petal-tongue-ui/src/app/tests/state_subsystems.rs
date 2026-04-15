// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::*;
use std::sync::{Arc, RwLock};

#[test]
fn set_visualization_state_and_active_session_count() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    assert_eq!(app.active_session_count(), 0);
    let state = Arc::new(RwLock::new(petal_tongue_ipc::VisualizationState::new()));
    app.set_visualization_state(state);
    assert_eq!(app.active_session_count(), 0);
}

#[test]
fn set_sensor_stream_and_interaction_subscribers() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let sensor_reg = Arc::new(RwLock::new(petal_tongue_ipc::SensorStreamRegistry::new()));
    let interaction_reg = Arc::new(RwLock::new(
        petal_tongue_ipc::InteractionSubscriberRegistry::new(),
    ));
    app.set_sensor_stream(sensor_reg);
    app.set_interaction_subscribers(interaction_reg);
}

#[test]
fn update_headless_single_frame() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_headless(ctx);
    });
}

#[test]
fn tick_clock_access() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let clock = app.tick_clock();
    let _ = clock;
}

#[test]
fn headless_app_refresh_graph_data() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.refresh_graph_data();
    let _introspection = app.introspect();
}
