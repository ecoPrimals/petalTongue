// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{MotorCommand, PanelId, PanelKind};
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
