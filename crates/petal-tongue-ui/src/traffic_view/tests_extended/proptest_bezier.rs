// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::bezier_control_points;

mod proptest_impl {
    use super::super::super::bezier_control_points;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn bezier_control_points_symmetry(
            from_x in -500.0f32..500.0f32,
            from_y in -500.0f32..500.0f32,
            to_x in -500.0f32..500.0f32,
            to_y in -500.0f32..500.0f32,
        ) {
            let (ctrl1, ctrl2) = bezier_control_points(from_x, from_y, to_x, to_y);
            let dx = to_x - from_x;
            let offset = dx.abs() * 0.3;
            let expect_ctrl1_x = from_x + offset * dx.signum();
            let expect_ctrl2_x = to_x - offset * dx.signum();
            assert!((ctrl1[0] - expect_ctrl1_x).abs() < 0.001, "ctrl1.x");
            assert!((ctrl1[1] - from_y).abs() < 0.001, "ctrl1.y");
            assert!((ctrl2[0] - expect_ctrl2_x).abs() < 0.001, "ctrl2.x");
            assert!((ctrl2[1] - to_y).abs() < 0.001, "ctrl2.y");
        }
    }
}

#[test]
fn bezier_control_points_left_to_right() {
    let (ctrl1, ctrl2) = bezier_control_points(100.0, 50.0, 400.0, 100.0);
    let dx = 400.0_f32 - 100.0;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - (100.0 + offset)).abs() < f32::EPSILON);
    assert!((ctrl1[1] - 50.0).abs() < f32::EPSILON);
    assert!((ctrl2[0] - (400.0 - offset)).abs() < f32::EPSILON);
    assert!((ctrl2[1] - 100.0).abs() < f32::EPSILON);
}

#[test]
fn bezier_control_points_right_to_left() {
    let (ctrl1, ctrl2) = bezier_control_points(400.0, 50.0, 100.0, 100.0);
    let dx: f32 = 100.0 - 400.0;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - dx.signum().mul_add(offset, 400.0)).abs() < f32::EPSILON);
    assert!((ctrl2[0] - dx.signum().mul_add(-offset, 100.0)).abs() < f32::EPSILON);
}

#[test]
fn bezier_control_points_diagonal() {
    let (ctrl1, ctrl2) = bezier_control_points(0.0, 0.0, 100.0, 100.0);
    let dx = 100.0_f32;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - offset).abs() < f32::EPSILON);
    assert!((ctrl1[1] - 0.0).abs() < f32::EPSILON);
    assert!((ctrl2[0] - (100.0 - offset)).abs() < f32::EPSILON);
    assert!((ctrl2[1] - 100.0).abs() < f32::EPSILON);
}

#[test]
fn bezier_control_points_reversed() {
    let (ctrl1, ctrl2) = bezier_control_points(200.0, 100.0, 50.0, 50.0);
    let dx = 50.0_f32 - 200.0;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - (200.0 + offset * dx.signum())).abs() < f32::EPSILON);
    assert!((ctrl2[0] - (50.0 - offset * dx.signum())).abs() < f32::EPSILON);
}
