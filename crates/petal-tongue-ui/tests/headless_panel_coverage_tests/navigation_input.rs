// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{MotorCommand, PanelId, PanelKind};
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
