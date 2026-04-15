// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
