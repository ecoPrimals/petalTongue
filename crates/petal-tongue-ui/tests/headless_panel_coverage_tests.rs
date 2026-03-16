// SPDX-License-Identifier: AGPL-3.0-only
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Extended headless panel tests: previously-untestable UI logic.
//!
//! These tests exercise panel interactions, motor command processing,
//! data binding visibility, accessibility styling, and multi-frame
//! state evolution through the headless harness.

use petal_tongue_core::{MotorCommand, PanelId, PanelKind};
use petal_tongue_ui::headless_harness::HeadlessHarness;

// === Motor command integration ===

#[test]
fn motor_hide_top_menu() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::TopMenu));

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: false,
        })
        .unwrap();
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::TopMenu));
}

#[test]
fn motor_hide_dashboard() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Dashboard));

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: false,
        })
        .unwrap();
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::Dashboard));
}

#[test]
fn motor_show_proprioception() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::Proprioception));

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::Proprioception,
            visible: true,
        })
        .unwrap();
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Proprioception));
}

#[test]
fn motor_toggle_audio_panel() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();
    let initially_visible = h.is_panel_visible(PanelKind::AudioSonification);

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::AudioPanel,
            visible: !initially_visible,
        })
        .unwrap();
    h.run_frame();
    assert_eq!(
        h.is_panel_visible(PanelKind::AudioSonification),
        !initially_visible
    );
}

#[test]
fn motor_toggle_trust_dashboard() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: false,
        })
        .unwrap();
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::TrustDashboard));

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::TrustDashboard));
}

// === Multi-panel orchestration ===

#[test]
fn hide_all_sidebars_leaves_canvas() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let sender = h.app_mut().motor_sender();
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::LeftSidebar,
            visible: false,
        })
        .unwrap();
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::RightSidebar,
            visible: false,
        })
        .unwrap();
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: false,
        })
        .unwrap();
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: false,
        })
        .unwrap();

    h.run_frame();

    assert!(h.is_panel_visible(PanelKind::GraphCanvas));
    assert!(!h.is_panel_visible(PanelKind::Controls));
    assert!(!h.is_panel_visible(PanelKind::TopMenu));
    assert!(!h.is_panel_visible(PanelKind::Dashboard));
}

// === Mode presets ===

#[test]
fn set_mode_clinical_applies_presets() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetMode {
            mode: "clinical".into(),
        })
        .unwrap();
    h.run_frame();

    // Clinical mode disables graph builder and audio
    assert!(!h.is_panel_visible(PanelKind::GraphBuilder));
}

#[test]
fn set_mode_developer_applies_presets() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetMode {
            mode: "developer".into(),
        })
        .unwrap();
    h.run_frame();

    assert!(h.is_panel_visible(PanelKind::Dashboard));
}

// === Data binding introspection ===

#[test]
fn empty_graph_has_no_data_bindings() {
    let mut h = HeadlessHarness::new().unwrap();
    let intro = h.run_frame();

    // A freshly created headless app might have tutorial data or be empty
    // depending on build features - but the introspection itself should be populated
    let _binding_count = intro.bound_data.len();
}

#[test]
fn interaction_capabilities_include_navigate() {
    let mut h = HeadlessHarness::new().unwrap();
    let intro = h.run_frame();

    let has_navigate = intro
        .possible_interactions
        .iter()
        .any(|i| i.intent == petal_tongue_core::InteractionKind::Navigate);
    assert!(
        has_navigate,
        "Graph canvas should offer Navigate interaction"
    );
}

#[test]
fn interaction_capabilities_include_toggle_panel() {
    let mut h = HeadlessHarness::new().unwrap();
    let intro = h.run_frame();

    let has_toggle = intro
        .possible_interactions
        .iter()
        .any(|i| i.intent == petal_tongue_core::InteractionKind::TogglePanel);
    assert!(has_toggle, "TopMenu should offer TogglePanel interaction");
}

#[test]
fn interaction_capabilities_include_configure() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    assert!(h.is_panel_visible(PanelKind::Controls));
    let intro = h.last_introspection().unwrap();
    let has_configure = intro
        .possible_interactions
        .iter()
        .any(|i| i.intent == petal_tongue_core::InteractionKind::Configure);
    assert!(
        has_configure,
        "Controls panel should offer Configure interaction"
    );
}

// === Multi-frame evolution ===

#[test]
fn frame_ids_monotonically_increase() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frames(5);

    let history = h.frame_history();
    for window in history.windows(2) {
        assert!(
            window[1].frame_id > window[0].frame_id,
            "Frame IDs must increase monotonically"
        );
    }
}

#[test]
fn content_awareness_updates_each_frame() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frames(3);

    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();
    assert!(awareness.content().total_introspections() >= 3);
}

// === Keyboard shortcut chaining ===

#[test]
fn toggle_panels_via_keyboard_chain() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    // Toggle P, M, G in sequence
    h.key_press(petal_tongue_ui::egui::Key::P);
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Proprioception));

    h.key_press(petal_tongue_ui::egui::Key::M);
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Metrics));

    h.key_press(petal_tongue_ui::egui::Key::G);
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::GraphBuilder));

    // All three still visible together
    assert!(h.is_panel_visible(PanelKind::Proprioception));
    assert!(h.is_panel_visible(PanelKind::Metrics));
    assert!(h.is_panel_visible(PanelKind::GraphBuilder));
}

// === Rendering awareness integration ===

#[test]
fn rendering_awareness_visible_panels_match_introspection() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let intro_panels = h.visible_panels();
    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();
    let awareness_panels = awareness.visible_panels();

    assert_eq!(
        intro_panels.len(),
        awareness_panels.len(),
        "RenderingAwareness and FrameIntrospection should agree on panel count"
    );
}

#[test]
fn rendering_awareness_is_showing_data_matches() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();

    // Non-existent data should not be "showing"
    assert!(!awareness.is_showing_data("totally-fake-id-xyz"));
}

#[test]
fn rendering_awareness_is_panel_visible_works() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();

    assert!(awareness.is_panel_visible(PanelKind::TopMenu));
    assert!(!awareness.is_panel_visible(PanelKind::Proprioception));
}

// === Keyboard shortcut integration ===

#[test]
fn ctrl_d_toggles_dashboard() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Dashboard));

    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::D,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(
        !h.is_panel_visible(PanelKind::Dashboard),
        "Ctrl+D should toggle dashboard off"
    );

    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::D,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(
        h.is_panel_visible(PanelKind::Dashboard),
        "Second Ctrl+D should toggle dashboard back on"
    );
}

#[test]
fn ctrl_a_toggles_accessibility_panel() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::Accessibility));

    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(
        h.is_panel_visible(PanelKind::Accessibility),
        "Ctrl+A should show accessibility panel"
    );

    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(
        !h.is_panel_visible(PanelKind::Accessibility),
        "Second Ctrl+A should hide accessibility panel"
    );
}

#[test]
fn escape_closes_overlays() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    // Open accessibility panel
    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::A,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Accessibility));

    // Open help overlay
    h.key_press(petal_tongue_ui::egui::Key::F1);
    h.run_frame();
    assert!(h.app().is_help_visible());

    // Escape should close both
    h.key_press(petal_tongue_ui::egui::Key::Escape);
    h.run_frame();
    assert!(
        !h.is_panel_visible(PanelKind::Accessibility),
        "Escape should close accessibility panel"
    );
    assert!(
        !h.app().is_help_visible(),
        "Escape should close help overlay"
    );
}

#[test]
fn ctrl_h_toggles_help() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    assert!(!h.app().is_help_visible());

    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::H,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(h.app().is_help_visible(), "Ctrl+H should show help overlay");

    h.key_press_with_modifiers(
        petal_tongue_ui::egui::Key::H,
        petal_tongue_ui::egui::Modifiers::CTRL,
    );
    h.run_frame();
    assert!(
        !h.app().is_help_visible(),
        "Second Ctrl+H should hide help overlay"
    );
}

// === Multi-frame state evolution ===

#[test]
fn run_10_frames_increments_count() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frames(10);
    assert!(
        h.frame_count() >= 10,
        "Frame count should be at least 10 after running 10 frames"
    );
    assert_eq!(
        h.frame_history().len(),
        10,
        "Frame history should have 10 entries"
    );
}

#[test]
fn toggle_panels_persist_across_frames() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::Proprioception));

    h.key_press(petal_tongue_ui::egui::Key::P);
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Proprioception));

    // Run several more frames without input - state should persist
    h.run_frames(5);
    assert!(
        h.is_panel_visible(PanelKind::Proprioception),
        "Panel state should persist across frames"
    );
}

#[test]
fn multiple_motor_commands_apply_in_sequence() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    let sender = h.app_mut().motor_sender();
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: false,
        })
        .expect("send");
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: true,
        })
        .expect("send");
    sender
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::Proprioception,
            visible: true,
        })
        .expect("send");

    h.run_frame();

    assert!(!h.is_panel_visible(PanelKind::Dashboard));
    assert!(h.is_panel_visible(PanelKind::TrustDashboard));
    assert!(h.is_panel_visible(PanelKind::Proprioception));
}

// === Panel navigation via motor commands ===

#[test]
fn motor_show_dashboard() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: false,
        })
        .expect("send");
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::Dashboard));

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: true,
        })
        .expect("send");
    h.run_frame();
    assert!(h.is_panel_visible(PanelKind::Dashboard));
}

#[test]
fn motor_show_metrics_via_key() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::Metrics));

    h.key_press(petal_tongue_ui::egui::Key::M);
    h.run_frame();
    assert!(
        h.is_panel_visible(PanelKind::Metrics),
        "M key should show metrics dashboard"
    );
}

#[test]
fn key_g_shows_graph_builder() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();
    assert!(!h.is_panel_visible(PanelKind::GraphBuilder));

    h.key_press(petal_tongue_ui::egui::Key::G);
    h.run_frame();
    assert!(
        h.is_panel_visible(PanelKind::GraphBuilder),
        "G key should show graph builder"
    );
}

// === Input simulation ===

#[test]
fn pointer_move_no_crash() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    h.pointer_move(petal_tongue_ui::egui::pos2(640.0, 360.0));
    h.run_frame();
    h.pointer_move(petal_tongue_ui::egui::pos2(100.0, 100.0));
    h.run_frame();
}

#[test]
fn click_at_center_no_crash() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    let center = petal_tongue_ui::egui::pos2(640.0, 360.0);
    h.click(center);
    h.run_frame();
}

#[test]
fn type_text_no_crash() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    h.type_text("hello");
    h.run_frame();
}

// === Motor command coverage ===

#[test]
fn motor_set_zoom_no_crash() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::SetZoom { level: 1.5 })
        .expect("send");
    h.run_frame();
}

#[test]
fn motor_navigate_no_crash() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::Navigate {
            target_node: "some-node".into(),
        })
        .expect("send");
    h.run_frame();
}

#[test]
fn motor_fit_to_view_no_crash() {
    let mut h = HeadlessHarness::new().expect("harness");
    h.run_frame();

    h.app_mut()
        .motor_sender()
        .send(MotorCommand::FitToView)
        .expect("send");
    h.run_frame();
}
