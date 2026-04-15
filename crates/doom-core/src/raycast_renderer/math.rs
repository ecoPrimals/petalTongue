// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ray / projection helpers for the raycasting renderer.

use crate::wad_loader::Vertex;
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

/// Ray vs segment intersection: returns ray parameter `t` along `(ray_x, ray_y) + t * (ray_dx, ray_dy)`.
#[expect(
    clippy::similar_names,
    reason = "ray_dx/ray_dy/line_dx/line_dy are standard geometry names"
)]
pub fn ray_line_intersection(
    ray_x: f32,
    ray_y: f32,
    ray_dx: f32,
    ray_dy: f32,
    v1: Vertex,
    v2: Vertex,
) -> Option<f32> {
    let x1 = f32::from(v1.x);
    let y1 = f32::from(v1.y);
    let x2 = f32::from(v2.x);
    let y2 = f32::from(v2.y);

    let line_dx = x2 - x1;
    let line_dy = y2 - y1;

    let denominator = ray_dx.mul_add(line_dy, -(ray_dy * line_dx));

    if denominator.abs() < 0.0001 {
        return None;
    }

    let u = (ray_x - x1).mul_add(ray_dy, -((ray_y - y1) * ray_dx)) / denominator;
    let t = (ray_x - x1).mul_add(line_dy, -((ray_y - y1) * line_dx)) / denominator;

    if (0.0..=1.0).contains(&u) && t >= 0.0 {
        Some(t)
    } else {
        None
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "screen pixel indices fit losslessly in f32"
)]
pub fn screen_ray_angle(screen_x: usize, width: usize, player_angle: f32, fov: f32) -> f32 {
    let screen_center = width as f32 / 2.0;
    let x_offset = screen_x as f32 - screen_center;
    let angle_offset = (x_offset / screen_center) * (fov / 2.0);
    player_angle + angle_offset
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "projected height always positive and capped"
)]
pub fn projected_wall_height(distance: f32, width: usize, height: usize, fov: f32) -> usize {
    let wall_actual_height: f32 = 128.0;
    let projection_distance = (width as f32 / 2.0) / (fov / 2.0).tan();
    let projected_height = (wall_actual_height / distance) * projection_distance;
    (projected_height as usize).min(height * 2)
}

pub fn distance_shading(distance: f32, render_distance: f32) -> f32 {
    (1.0 - (distance / render_distance)).max(0.0)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "shading clamps to 0..1 so result fits in u8"
)]
pub fn shade_wall_rgba(wall_color: [u8; 3], shading: f32) -> [u8; 4] {
    [
        (f32::from(wall_color[0]) * shading) as u8,
        (f32::from(wall_color[1]) * shading) as u8,
        (f32::from(wall_color[2]) * shading) as u8,
        255,
    ]
}

/// Normalizes an angle to `[0, 2π)`.
pub fn normalize_angle_0_two_pi(angle: f32) -> f32 {
    angle.rem_euclid(TWO_PI)
}
