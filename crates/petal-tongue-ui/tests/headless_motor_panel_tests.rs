// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Motor command integration tests — panel visibility and mode switching.
//!
//! Split from `headless_motor_command_tests.rs` for maintainability.

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
fn motor_command_set_mode_clinical() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "clinical".to_string(),
        })
        .unwrap();
    harness.run_frame();

    assert!(
        harness.is_panel_visible(PanelKind::Dashboard),
        "Clinical mode: dashboard visible"
    );
    assert!(
        !harness.is_panel_visible(PanelKind::Proprioception),
        "Clinical mode: proprioception hidden"
    );
    assert!(
        !harness.is_panel_visible(PanelKind::AudioSonification),
        "Clinical mode: audio panel hidden"
    );
}

#[test]
fn motor_command_set_mode_developer() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "developer".to_string(),
        })
        .unwrap();
    harness.run_frame();

    assert!(
        harness.is_panel_visible(PanelKind::Proprioception),
        "Developer mode: proprioception visible"
    );
    assert!(
        harness.is_panel_visible(PanelKind::AudioSonification),
        "Developer mode: audio panel visible"
    );
}

#[test]
fn motor_command_set_mode_presentation() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "presentation".to_string(),
        })
        .unwrap();
    harness.run_frame();

    assert!(
        !harness.is_panel_visible(PanelKind::TopMenu),
        "Presentation mode: top menu hidden"
    );
    assert!(
        !harness.is_panel_visible(PanelKind::Controls),
        "Presentation mode: controls hidden"
    );
}

#[test]
fn motor_command_set_mode_research() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "research".to_string(),
        })
        .unwrap();
    harness.run_frame();

    assert!(harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    assert!(harness.is_panel_visible(PanelKind::Dashboard));
    assert!(!harness.is_panel_visible(PanelKind::AudioSonification));
}

#[test]
fn motor_command_set_mode_full() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "full".to_string(),
        })
        .unwrap();
    harness.run_frame();

    assert!(harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::AudioSonification));
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
}

#[test]
fn motor_command_set_mode_patient_facing() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "patient-facing".to_string(),
        })
        .unwrap();
    harness.run_frame();

    assert!(harness.is_panel_visible(PanelKind::TopMenu));
    assert!(!harness.is_panel_visible(PanelKind::Controls));
    assert!(!harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::GraphCanvas));
}

#[test]
fn motor_command_panel_combinations_all_visible() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    let sender = harness.app_mut().motor_sender();
    sender
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::Proprioception,
            visible: true,
        })
        .unwrap();
    sender
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    harness.key_press(petal_tongue_ui::egui::Key::M);
    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frame();

    assert!(harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    assert!(harness.is_panel_visible(PanelKind::Metrics));
    assert!(harness.is_panel_visible(PanelKind::GraphBuilder));
}

#[test]
fn motor_command_panel_combinations_minimal() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    let sender = harness.app_mut().motor_sender();
    sender
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::LeftSidebar,
            visible: false,
        })
        .unwrap();
    sender
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::RightSidebar,
            visible: false,
        })
        .unwrap();
    sender
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TopMenu,
            visible: false,
        })
        .unwrap();
    harness.run_frame();

    assert!(!harness.is_panel_visible(PanelKind::Controls));
    assert!(!harness.is_panel_visible(PanelKind::TopMenu));
    assert!(harness.is_panel_visible(PanelKind::GraphCanvas));
}

#[test]
fn motor_command_graph_stats_toggle() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::GraphStats,
            visible: true,
        })
        .unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::GraphStats,
            visible: false,
        })
        .unwrap();
    harness.run_frame();
}

#[test]
fn motor_command_toggles_audio_panel() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    let initially_visible = harness.is_panel_visible(PanelKind::AudioSonification);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::AudioPanel,
            visible: !initially_visible,
        })
        .unwrap();
    harness.run_frame();
    assert_eq!(
        harness.is_panel_visible(PanelKind::AudioSonification),
        !initially_visible
    );
}

#[test]
fn panel_toggle_system_dashboard() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    let initial = harness.is_panel_visible(PanelKind::Dashboard);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::SystemDashboard,
            visible: !initial,
        })
        .unwrap();
    harness.run_frame();
    assert_eq!(harness.is_panel_visible(PanelKind::Dashboard), !initial);
}

#[test]
fn panel_toggle_trust_dashboard() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    let initial = harness.is_panel_visible(PanelKind::TrustDashboard);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: !initial,
        })
        .unwrap();
    harness.run_frame();
    assert_eq!(
        harness.is_panel_visible(PanelKind::TrustDashboard),
        !initial
    );
}

#[test]
fn panel_toggle_graph_stats() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::GraphStats,
            visible: true,
        })
        .unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::GraphStats,
            visible: false,
        })
        .unwrap();
    harness.run_frame();
}

#[test]
fn panel_toggle_right_sidebar() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::RightSidebar,
            visible: false,
        })
        .unwrap();
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::Dashboard));
    assert!(!harness.is_panel_visible(PanelKind::TrustDashboard));
    assert!(!harness.is_panel_visible(PanelKind::AudioSonification));

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::RightSidebar,
            visible: true,
        })
        .unwrap();
    harness.run_frame();
}

#[test]
fn audio_panel_solo_visible() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::AudioPanel,
            visible: true,
        })
        .unwrap();
    harness.run_frames(3);
    assert!(harness.is_panel_visible(PanelKind::AudioSonification));
    let _ = harness.tessellate();
}

#[test]
fn system_dashboard_with_all_panels() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::SystemDashboard,
            visible: true,
        })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::Dashboard));
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    let _ = harness.tessellate();
}

#[test]
fn multi_modal_all_panels_visible() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::Proprioception,
            visible: true,
        })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::AudioPanel,
            visible: true,
        })
        .unwrap();
    harness.key_press(petal_tongue_ui::egui::Key::M);
    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    assert!(harness.is_panel_visible(PanelKind::AudioSonification));
    assert!(harness.is_panel_visible(PanelKind::Metrics));
    assert!(harness.is_panel_visible(PanelKind::GraphBuilder));
    let _ = harness.tessellate();
}

#[test]
fn rapid_mode_switching() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    for mode in ["clinical", "developer", "presentation", "research", "full"] {
        harness
            .app_mut()
            .motor_sender()
            .send(petal_tongue_core::MotorCommand::SetMode {
                mode: mode.to_string(),
            })
            .unwrap();
        harness.run_frame();
    }
    harness.run_frames(3);
    let _ = harness.tessellate();
}
