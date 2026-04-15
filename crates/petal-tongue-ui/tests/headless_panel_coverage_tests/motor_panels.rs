// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{MotorCommand, PanelId, PanelKind};
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
