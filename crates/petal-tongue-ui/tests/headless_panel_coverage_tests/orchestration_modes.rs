// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{MotorCommand, PanelId, PanelKind};
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
