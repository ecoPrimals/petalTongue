// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless integration tests: multi-frame sequences, mode switching, and continuous rendering.

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn multi_frame_sequence_20_frames() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.run_frames(19);
    assert!(harness.frame_count() >= 20);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.visible_panels.is_empty());
}

#[test]
fn multi_frame_sequence_mode_switch_mid_sequence() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frames(3);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "presentation".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "developer".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    assert!(harness.is_panel_visible(PanelKind::Proprioception));
}

#[test]
fn continuous_mode_enables_game_loop() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetContinuousMode { enabled: true })
        .unwrap();
    harness.run_frames(10);
    let _ = harness.tessellate();
}

#[test]
fn continuous_mode_with_physics_and_animation() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetContinuousMode { enabled: true })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPhysics { enabled: true })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetSceneAnimation { enabled: true })
        .unwrap();
    harness.run_frames(15);
    let _ = harness.tessellate();
}
