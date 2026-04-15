// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::*;

#[test]
fn headless_app_introspect() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    let _ = introspection.frame_id;
    assert!(!introspection.visible_panels.is_empty());
    assert!(
        introspection
            .active_modalities
            .contains(&petal_tongue_core::interaction::perspective::OutputModality::Gui)
    );
}

#[test]
fn headless_app_motor_sender() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let tx = app.motor_sender();
    let _ = tx;
}

#[test]
fn headless_app_graph_handle() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let handle = app.graph_handle();
    let _guard = handle.read().unwrap();
}

#[test]
fn headless_app_getters() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    assert_eq!(app.frame_count(), 0);
    assert!(!app.is_help_visible());
    assert!(!app.is_continuous_mode());
    assert_eq!(app.active_session_count(), 0);
}

#[test]
fn introspect_panel_snapshots() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    let panel_ids: Vec<_> = introspection
        .visible_panels
        .iter()
        .map(|p| format!("{:?}", p.id))
        .collect();
    assert!(
        panel_ids.iter().any(|s| s.contains("TopMenu")),
        "should have TopMenu in panels"
    );
    assert!(
        panel_ids.iter().any(|s| s.contains("graph_canvas")),
        "should have graph_canvas"
    );
}

#[test]
fn introspect_bound_data_from_default_graph() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    for bd in &introspection.bound_data {
        assert!(
            !bd.data_object_id.is_empty(),
            "bound data objects must have an id"
        );
    }
}

#[test]
fn introspect_possible_interactions() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    assert!(
        !introspection.possible_interactions.is_empty(),
        "should have at least Navigate"
    );
    assert!(
        introspection
            .possible_interactions
            .iter()
            .any(|i| matches!(i.intent, petal_tongue_core::InteractionKind::Navigate)),
        "should have Navigate"
    );
}

#[test]
fn headless_app_introspect_with_visibility() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let i1 = app.introspect();
    let visible_count = i1.visible_panels.iter().filter(|p| p.visible).count();
    assert!(visible_count >= 1);
}
