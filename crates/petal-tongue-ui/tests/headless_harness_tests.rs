// SPDX-License-Identifier: AGPL-3.0-only
//! Headless harness integration tests.
//!
//! Core tests: mode switching, panel toggles, basic rendering.
//! See `headless_motor_command_tests.rs` and `headless_integration_tests.rs` for more.

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
fn ctrl_a_toggles_accessibility_panel() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::Accessibility));

    harness.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    harness.run_frame();
    assert!(
        harness.is_panel_visible(PanelKind::Accessibility),
        "Ctrl+A should show accessibility panel"
    );

    harness.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    harness.run_frame();
    assert!(
        !harness.is_panel_visible(PanelKind::Accessibility),
        "Second Ctrl+A should hide accessibility panel"
    );
}

#[test]
fn keyboard_escape_closes_overlays() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    harness.run_frame();
    assert!(harness.is_panel_visible(PanelKind::Accessibility));

    harness.key_press(petal_tongue_ui::egui::Key::Escape);
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::Accessibility));
}

#[test]
fn keyboard_tab_cycles_focus() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::Tab);
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::Tab);
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn keyboard_enter_submits() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::Enter);
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn keyboard_arrows_navigate() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::ArrowUp);
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::ArrowDown);
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::ArrowLeft);
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::ArrowRight);
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn ctrl_d_toggles_dashboard() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    let initial = harness.is_panel_visible(PanelKind::Dashboard);

    harness.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::D,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    harness.run_frame();
    assert_eq!(harness.is_panel_visible(PanelKind::Dashboard), !initial);
}

#[test]
fn multiple_frames_stability() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    for _ in 0..5 {
        harness.run_frame();
    }

    assert!(harness.frame_count() >= 6);
    let intro = harness.last_introspection().expect("frame ran");
    assert!(!intro.visible_panels.is_empty());
}

#[test]
fn panel_count_consistency() {
    let mut harness = HeadlessHarness::new().unwrap();
    let intro = harness.run_frame();
    let initial_count = intro.visible_panel_count();
    assert!(
        initial_count > 0,
        "After first frame, panel_count should be > 0"
    );

    harness.run_frame();
    harness.run_frame();
    let later = harness.last_introspection().expect("frames ran");
    assert_eq!(
        later.visible_panel_count(),
        initial_count,
        "Panel count should stay stable across frames"
    );
}

#[test]
fn accessibility_panel_then_escape() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    harness.run_frame();
    assert!(harness.is_panel_visible(PanelKind::Accessibility));
    harness.key_press(petal_tongue_ui::egui::Key::Escape);
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::Accessibility));
}
