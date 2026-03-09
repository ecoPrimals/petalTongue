// SPDX-License-Identifier: AGPL-3.0-only
//! First-person raycasting renderer - Phase 1.2
//!
//! Renders Doom maps from a first-person perspective using raycasting.

use crate::wad_loader::{MapData, Vertex};
use std::f32::consts::PI;

/// First-person raycasting renderer.
pub struct RaycastRenderer {
    width: usize,
    height: usize,
    framebuffer: Vec<u8>,
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,
    pub player_height: f32,
    fov: f32,
    render_distance: f32,
}

impl RaycastRenderer {
    /// Create a new raycasting renderer with the given framebuffer dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            framebuffer: vec![0; width * height * 4],
            player_x: 0.0,
            player_y: 0.0,
            player_angle: 0.0,
            player_height: 41.0,
            fov: PI / 2.0,
            render_distance: 4096.0,
        }
    }

    /// Set player position from map things (finds player 1 start).
    pub fn set_player_start(&mut self, map: &MapData) {
        for thing in &map.things {
            if thing.thing_type == 1 {
                self.player_x = f32::from(thing.x);
                self.player_y = f32::from(thing.y);
                self.player_angle = f32::from(thing.angle).to_radians();
                break;
            }
        }
    }

    /// Render the map from first-person perspective.
    pub fn render(&mut self, map: &MapData) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) * 4;

                if y < self.height / 2 {
                    self.framebuffer[idx] = 100;
                    self.framebuffer[idx + 1] = 150;
                    self.framebuffer[idx + 2] = 200;
                } else {
                    self.framebuffer[idx] = 64;
                    self.framebuffer[idx + 1] = 64;
                    self.framebuffer[idx + 2] = 64;
                }
                self.framebuffer[idx + 3] = 255;
            }
        }

        for x in 0..self.width {
            self.cast_and_draw_ray(map, x);
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "shading clamps to 0..1 so result fits in u8"
    )]
    fn cast_and_draw_ray(&mut self, map: &MapData, screen_x: usize) {
        let ray_angle = self.calculate_ray_angle(screen_x);

        if let Some((distance, wall_color)) = self.cast_ray(map, ray_angle) {
            let wall_height = self.calculate_wall_height(distance);

            let shading = (1.0 - (distance / self.render_distance)).max(0.0);
            let shaded_color = [
                (f32::from(wall_color[0]) * shading) as u8,
                (f32::from(wall_color[1]) * shading) as u8,
                (f32::from(wall_color[2]) * shading) as u8,
                255,
            ];

            self.draw_wall_column(screen_x, wall_height, shaded_color);
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "screen pixel indices fit losslessly in f32"
    )]
    fn calculate_ray_angle(&self, screen_x: usize) -> f32 {
        let screen_center = self.width as f32 / 2.0;
        let x_offset = screen_x as f32 - screen_center;
        let angle_offset = (x_offset / screen_center) * (self.fov / 2.0);
        self.player_angle + angle_offset
    }

    #[expect(
        clippy::similar_names,
        reason = "ray_dx/ray_dy are standard raycasting nomenclature"
    )]
    fn cast_ray(&self, map: &MapData, angle: f32) -> Option<(f32, [u8; 3])> {
        let ray_dx = angle.cos();
        let ray_dy = angle.sin();

        let mut nearest_distance = self.render_distance;
        let mut nearest_color = [255u8, 255, 255];

        for linedef in &map.linedefs {
            if linedef.start_vertex >= map.vertices.len()
                || linedef.end_vertex >= map.vertices.len()
            {
                continue;
            }

            let v1 = map.vertices[linedef.start_vertex];
            let v2 = map.vertices[linedef.end_vertex];

            if let Some(distance) =
                Self::ray_line_intersection(self.player_x, self.player_y, ray_dx, ray_dy, v1, v2)
                && distance < nearest_distance
                && distance > 0.1
            {
                nearest_distance = distance;
                nearest_color = if linedef.flags & 0x0001 != 0 {
                    [180, 180, 180]
                } else {
                    [100, 100, 100]
                };
            }
        }

        if nearest_distance < self.render_distance {
            Some((nearest_distance, nearest_color))
        } else {
            None
        }
    }

    #[expect(
        clippy::similar_names,
        reason = "ray_dx/ray_dy/line_dx/line_dy are standard geometry names"
    )]
    fn ray_line_intersection(
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

        let denominator = ray_dx * line_dy - ray_dy * line_dx;

        if denominator.abs() < 0.0001 {
            return None;
        }

        let u = ((ray_x - x1) * ray_dy - (ray_y - y1) * ray_dx) / denominator;
        let t = ((ray_x - x1) * line_dy - (ray_y - y1) * line_dx) / denominator;

        if (0.0..=1.0).contains(&u) && t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "projected height always positive and capped"
    )]
    fn calculate_wall_height(&self, distance: f32) -> usize {
        let wall_actual_height: f32 = 128.0;
        let projection_distance = (self.width as f32 / 2.0) / (self.fov / 2.0).tan();
        let projected_height = (wall_actual_height / distance) * projection_distance;
        (projected_height as usize).min(self.height * 2)
    }

    fn draw_wall_column(&mut self, x: usize, wall_height: usize, color: [u8; 4]) {
        let screen_center = self.height / 2;
        let half_height = wall_height / 2;

        let y_start = screen_center.saturating_sub(half_height);
        let y_end = (screen_center + half_height).min(self.height);

        for y in y_start..y_end {
            let idx = (y * self.width + x) * 4;
            if idx + 3 < self.framebuffer.len() {
                self.framebuffer[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    /// Get the rendered framebuffer.
    #[must_use]
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Move player forward/backward.
    pub fn move_forward(&mut self, amount: f32) {
        self.player_x += self.player_angle.cos() * amount;
        self.player_y += self.player_angle.sin() * amount;
    }

    /// Move player left/right (strafe).
    pub fn move_strafe(&mut self, amount: f32) {
        let strafe_angle = self.player_angle + PI / 2.0;
        self.player_x += strafe_angle.cos() * amount;
        self.player_y += strafe_angle.sin() * amount;
    }

    /// Rotate player (turn).
    pub fn rotate(&mut self, amount: f32) {
        self.player_angle += amount;

        while self.player_angle < 0.0 {
            self.player_angle += 2.0 * PI;
        }
        while self.player_angle >= 2.0 * PI {
            self.player_angle -= 2.0 * PI;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = RaycastRenderer::new(320, 240);
        assert_eq!(renderer.framebuffer().len(), 320 * 240 * 4);
    }

    #[test]
    fn test_ray_angle_calculation() {
        let renderer = RaycastRenderer::new(320, 240);

        let center_angle = renderer.calculate_ray_angle(160);
        assert!((center_angle - renderer.player_angle).abs() < 0.01);

        let left_angle = renderer.calculate_ray_angle(0);
        assert!(left_angle < renderer.player_angle);

        let right_angle = renderer.calculate_ray_angle(319);
        assert!(right_angle > renderer.player_angle);
    }

    #[test]
    fn test_player_movement() {
        let mut renderer = RaycastRenderer::new(320, 240);
        renderer.player_x = 100.0;
        renderer.player_y = 100.0;
        renderer.player_angle = 0.0;

        renderer.move_forward(10.0);
        assert!((renderer.player_x - 110.0).abs() < 0.1);
        assert!((renderer.player_y - 100.0).abs() < 0.1);

        renderer.rotate(PI / 2.0);
        renderer.move_forward(10.0);
        assert!((renderer.player_y - 110.0).abs() < 0.1);
    }

    #[test]
    fn test_render_empty_map() {
        use crate::wad_loader::MapData;

        let mut renderer = RaycastRenderer::new(64, 64);
        let map = MapData {
            name: "TEST".to_string(),
            vertices: vec![],
            linedefs: vec![],
            sectors: vec![],
            things: vec![],
        };

        renderer.render(&map);
        assert_eq!(renderer.framebuffer().len(), 64 * 64 * 4);
        assert_eq!(renderer.framebuffer()[0], 100);
        assert_eq!(renderer.framebuffer()[3], 255);
    }

    #[test]
    fn test_set_player_start() {
        use crate::wad_loader::{MapData, Thing};

        let mut renderer = RaycastRenderer::new(64, 64);
        let map = MapData {
            name: "TEST".to_string(),
            vertices: vec![],
            linedefs: vec![],
            sectors: vec![],
            things: vec![Thing {
                x: 64,
                y: 64,
                angle: 90,
                thing_type: 1,
                flags: 0,
            }],
        };

        renderer.set_player_start(&map);
        assert!((renderer.player_x - 64.0).abs() < f32::EPSILON);
        assert!((renderer.player_y - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rotate_wrapping() {
        let mut renderer = RaycastRenderer::new(64, 64);
        renderer.player_angle = 0.0;

        renderer.rotate(2.0 * PI);
        assert!((renderer.player_angle - 0.0).abs() < 0.01);

        renderer.rotate(-2.0 * PI);
        assert!((renderer.player_angle - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_move_strafe() {
        let mut renderer = RaycastRenderer::new(64, 64);
        renderer.player_x = 0.0;
        renderer.player_y = 0.0;
        renderer.player_angle = 0.0;

        renderer.move_strafe(10.0);
        assert!((renderer.player_y - 10.0).abs() < 0.1);
    }
}
