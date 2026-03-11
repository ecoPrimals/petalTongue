// SPDX-License-Identifier: AGPL-3.0-only
//! Headless harness integration tests.
//!
//! These tests exercise the UI through the `HeadlessHarness`, proving that
//! petalTongue can introspect its own visual state without a display.

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn harness_creates_successfully() {
    let harness = HeadlessHarness::new();
    assert!(harness.is_ok(), "HeadlessHarness::new() should succeed");
}

#[test]
fn first_frame_runs_and_produces_introspection() {
    let mut harness = HeadlessHarness::new().unwrap();
    let intro = harness.run_frame();
    assert!(
        intro.frame_id > 0,
        "Frame ID should be positive after running a frame"
    );
    assert!(
        !intro.visible_panels.is_empty(),
        "At least one panel should exist in the introspection"
    );
}

#[test]
fn default_panels_visible() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    assert!(
        harness.is_panel_visible(PanelKind::TopMenu),
        "TopMenu should be visible by default"
    );
    assert!(
        harness.is_panel_visible(PanelKind::Controls),
        "Controls panel should be visible by default"
    );
    assert!(
        harness.is_panel_visible(PanelKind::GraphCanvas),
        "Graph canvas should always be visible"
    );
}

#[test]
fn dashboard_visible_by_default() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    assert!(
        harness.is_panel_visible(PanelKind::Dashboard),
        "Dashboard should be visible by default"
    );
}

#[test]
fn proprioception_hidden_by_default() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    assert!(
        !harness.is_panel_visible(PanelKind::Proprioception),
        "Proprioception panel should be hidden by default"
    );
}

#[test]
fn graph_builder_hidden_by_default() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    assert!(
        !harness.is_panel_visible(PanelKind::GraphBuilder),
        "Graph builder should be hidden by default"
    );
}

#[test]
fn metrics_hidden_by_default() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    assert!(
        !harness.is_panel_visible(PanelKind::Metrics),
        "Metrics panel should be hidden by default"
    );
}

#[test]
fn keyboard_p_toggles_proprioception() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::Proprioception));

    harness.key_press(petal_tongue_ui::egui::Key::P);
    harness.run_frame();
    assert!(
        harness.is_panel_visible(PanelKind::Proprioception),
        "P key should toggle proprioception panel on"
    );

    harness.key_press(petal_tongue_ui::egui::Key::P);
    harness.run_frame();
    assert!(
        !harness.is_panel_visible(PanelKind::Proprioception),
        "Second P key press should toggle proprioception panel off"
    );
}

#[test]
fn keyboard_m_toggles_metrics() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::Metrics));

    harness.key_press(petal_tongue_ui::egui::Key::M);
    harness.run_frame();
    assert!(
        harness.is_panel_visible(PanelKind::Metrics),
        "M key should toggle metrics dashboard on"
    );
}

#[test]
fn keyboard_g_toggles_graph_builder() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::GraphBuilder));

    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frame();
    assert!(
        harness.is_panel_visible(PanelKind::GraphBuilder),
        "G key should toggle graph builder on"
    );
}

#[test]
fn multiple_frames_increment_count() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frames(5);
    assert!(
        harness.frame_count() >= 5,
        "Frame count should be at least 5 after running 5 frames"
    );
    assert_eq!(
        harness.frame_history().len(),
        5,
        "Frame history should have 5 entries"
    );
}

#[test]
fn frame_history_captures_changes() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    let panels_before = harness.visible_panels();

    harness.key_press(petal_tongue_ui::egui::Key::P);
    harness.run_frame();
    let panels_after = harness.visible_panels();

    assert_ne!(
        panels_before, panels_after,
        "Panel state should change after toggling with key press"
    );
}

#[test]
fn introspection_has_possible_interactions() {
    let mut harness = HeadlessHarness::new().unwrap();
    let intro = harness.run_frame();
    assert!(
        !intro.possible_interactions.is_empty(),
        "There should be possible interactions (navigate, toggle, etc)"
    );
}

#[test]
fn introspection_reports_active_modality() {
    let mut harness = HeadlessHarness::new().unwrap();
    let intro = harness.run_frame();
    assert!(
        !intro.active_modalities.is_empty(),
        "At least GUI modality should be active"
    );
}

#[test]
fn rendering_awareness_receives_content() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    let awareness_arc = harness.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();
    assert!(
        awareness.content().has_content(),
        "RenderingAwareness should have content after a frame"
    );
    assert!(
        awareness.current_content().is_some(),
        "current_content() should return the last introspection"
    );
}

#[test]
fn self_assessment_reports_content_awareness() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frames(3);

    let awareness_arc = harness.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();
    let assessment = awareness.assess_self();

    assert!(
        assessment.can_render,
        "Motor system should be functional after frames"
    );
}

#[test]
fn custom_screen_size_works() {
    let harness = HeadlessHarness::with_screen_size(800.0, 600.0);
    assert!(harness.is_ok());
    let mut harness = harness.unwrap();
    harness.run_frame();
    assert!(!harness.visible_panels().is_empty());
}

#[test]
fn motor_command_changes_panel_visibility() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    assert!(harness.is_panel_visible(PanelKind::Controls));

    let sender = harness.app_mut().motor_sender();
    sender
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::LeftSidebar,
            visible: false,
        })
        .unwrap();

    harness.run_frame();
    assert!(
        !harness.is_panel_visible(PanelKind::Controls),
        "Controls panel should be hidden after motor command"
    );
}

#[test]
fn visible_panel_count_matches() {
    let mut harness = HeadlessHarness::new().unwrap();
    let intro = harness.run_frame();

    let visible_count = intro.visible_panel_count();
    let visible_kinds = intro.visible_panel_kinds();

    assert_eq!(
        visible_count,
        visible_kinds.len(),
        "visible_panel_count() and visible_panel_kinds().len() should agree"
    );
}

#[test]
fn tessellate_does_not_panic() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    let primitives = harness.tessellate();
    // Tessellation should produce some output (at least background)
    let _ = primitives;
}

#[test]
fn tutorial_data_reflects_current_architecture() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();

    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);

    harness.run_frame();
    let intro = harness.run_frame();

    let has_bound_data = !intro.bound_data.is_empty();
    assert!(
        has_bound_data,
        "After loading tutorial data, introspection should report bound data objects"
    );

    let has_petaltongue = intro
        .bound_data
        .iter()
        .any(|d| d.data_object_id.contains("petaltongue"));
    assert!(
        has_petaltongue,
        "Tutorial data should include a petalTongue node in bound data"
    );
}

#[test]
fn tutorial_primals_use_capability_taxonomy() {
    use petal_tongue_core::{GraphEngine, LayoutAlgorithm};
    use petal_tongue_ui::tutorial_mode::TutorialMode;
    use std::sync::{Arc, RwLock};

    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();
    let petal = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "petaltongue-tutorial")
        .expect("petalTongue tutorial node should exist");

    assert!(
        petal.info.capabilities.iter().any(|c| c.starts_with("ui.")),
        "petalTongue should have ui.* capabilities"
    );
    assert!(
        petal
            .info
            .capabilities
            .iter()
            .any(|c| c.starts_with("ipc.")),
        "petalTongue should have ipc.* capabilities"
    );
    assert!(
        petal.info.endpoint.starts_with("unix://"),
        "Endpoint should be a Unix socket URI"
    );

    let security = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "security-example")
        .expect("Security example node should exist");

    assert!(
        security
            .info
            .capabilities
            .iter()
            .any(|c| c.starts_with("security.")),
        "Security example should have security.* capabilities"
    );

    let discovery = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "discovery-example")
        .expect("Discovery example node should exist");

    assert!(
        discovery
            .info
            .capabilities
            .iter()
            .any(|c| c.starts_with("discovery.")),
        "Discovery example should have discovery.* capabilities"
    );

    let edges = graph.edges();
    assert!(
        edges.iter().any(|e| e.edge_type == "ipc.discovery"),
        "Should have semantic ipc.discovery edge"
    );
    assert!(
        edges.iter().any(|e| e.edge_type == "ipc.trust"),
        "Should have semantic ipc.trust edge"
    );
}
