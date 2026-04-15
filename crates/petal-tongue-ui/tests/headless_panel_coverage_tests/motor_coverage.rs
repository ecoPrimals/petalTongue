// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::MotorCommand;
use petal_tongue_ui::headless_harness::HeadlessHarness;

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
