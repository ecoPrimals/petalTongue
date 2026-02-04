//! First-person raycasting renderer - Phase 1.2
//!
//! Renders Doom maps from a first-person perspective using raycasting.
//! This is simpler than full BSP rendering but gets us visible results quickly!

use crate::wad_loader::{MapData, Vertex};
use std::f32::consts::PI;

/// First-person raycasting renderer
pub struct RaycastRenderer {
    width: usize,
    height: usize,
    framebuffer: Vec<u8>, // RGBA

    // Player state
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,  // Radians
    pub player_height: f32, // Z coordinate

    // Camera settings
    fov: f32,             // Field of view in radians
    render_distance: f32, // Maximum distance to render
}

impl RaycastRenderer {
    /// Create a new raycasting renderer
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            framebuffer: vec![0; width * height * 4],
            player_x: 0.0,
            player_y: 0.0,
            player_angle: 0.0,
            player_height: 41.0, // Standard Doom player height
            fov: PI / 2.0,       // 90 degrees
            render_distance: 4096.0,
        }
    }

    /// Set player position from map things
    pub fn set_player_start(&mut self, map: &MapData) {
        // Find player 1 start (thing type 1)
        for thing in &map.things {
            if thing.thing_type == 1 {
                self.player_x = thing.x as f32;
                self.player_y = thing.y as f32;
                // Doom angles: 0=east, 90=north, 180=west, 270=south
                // Convert to radians: 0=east, PI/2=north, PI=west, 3PI/2=south
                self.player_angle = (thing.angle as f32).to_radians();
                break;
            }
        }
    }

    /// Render the map from first-person perspective
    pub fn render(&mut self, map: &MapData) {
        // Clear framebuffer (sky color - light blue)
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) * 4;

                // Sky gradient (top) to ground color (bottom)
                if y < self.height / 2 {
                    // Sky (light blue)
                    self.framebuffer[idx] = 100;
                    self.framebuffer[idx + 1] = 150;
                    self.framebuffer[idx + 2] = 200;
                } else {
                    // Floor (dark gray)
                    self.framebuffer[idx] = 64;
                    self.framebuffer[idx + 1] = 64;
                    self.framebuffer[idx + 2] = 64;
                }
                self.framebuffer[idx + 3] = 255;
            }
        }

        // Cast a ray for each screen column
        for x in 0..self.width {
            self.cast_and_draw_ray(map, x);
        }
    }

    /// Cast a ray for one screen column and draw the result
    fn cast_and_draw_ray(&mut self, map: &MapData, screen_x: usize) {
        // Calculate ray angle
        let ray_angle = self.calculate_ray_angle(screen_x);

        // Cast ray to find nearest wall
        if let Some((distance, wall_color)) = self.cast_ray(map, ray_angle) {
            // Calculate wall height on screen
            let wall_height = self.calculate_wall_height(distance);

            // Apply distance-based shading (fog effect)
            let shading = (1.0 - (distance / self.render_distance)).max(0.0);
            let shaded_color = [
                (wall_color[0] as f32 * shading) as u8,
                (wall_color[1] as f32 * shading) as u8,
                (wall_color[2] as f32 * shading) as u8,
                255,
            ];

            // Draw vertical wall slice
            self.draw_wall_column(screen_x, wall_height, shaded_color);
        }
    }

    /// Calculate ray angle for a given screen column
    fn calculate_ray_angle(&self, screen_x: usize) -> f32 {
        // Map screen X to angle relative to player
        let screen_center = self.width as f32 / 2.0;
        let x_offset = screen_x as f32 - screen_center;
        let angle_offset = (x_offset / screen_center) * (self.fov / 2.0);

        self.player_angle + angle_offset
    }

    /// Cast a ray and return distance to nearest wall and its color
    fn cast_ray(&self, map: &MapData, angle: f32) -> Option<(f32, [u8; 3])> {
        let ray_dx = angle.cos();
        let ray_dy = angle.sin();

        let mut nearest_distance = self.render_distance;
        let mut nearest_color = [255, 255, 255]; // Default white

        // Check intersection with each linedef
        for linedef in &map.linedefs {
            if linedef.start_vertex >= map.vertices.len()
                || linedef.end_vertex >= map.vertices.len()
            {
                continue;
            }

            let v1 = &map.vertices[linedef.start_vertex];
            let v2 = &map.vertices[linedef.end_vertex];

            // Check if ray intersects this line segment
            if let Some(distance) =
                self.ray_line_intersection(self.player_x, self.player_y, ray_dx, ray_dy, v1, v2)
            {
                if distance < nearest_distance && distance > 0.1 {
                    nearest_distance = distance;

                    // Color based on wall type
                    if linedef.flags & 0x0001 != 0 {
                        // Impassable wall (solid)
                        nearest_color = [180, 180, 180]; // Light gray
                    } else {
                        // Passable line
                        nearest_color = [100, 100, 100]; // Dark gray
                    }
                }
            }
        }

        if nearest_distance < self.render_distance {
            Some((nearest_distance, nearest_color))
        } else {
            None
        }
    }

    /// Check if ray intersects line segment, return distance
    fn ray_line_intersection(
        &self,
        ray_x: f32,
        ray_y: f32,
        ray_dx: f32,
        ray_dy: f32,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Option<f32> {
        let x1 = v1.x as f32;
        let y1 = v1.y as f32;
        let x2 = v2.x as f32;
        let y2 = v2.y as f32;

        let line_dx = x2 - x1;
        let line_dy = y2 - y1;

        // Solve parametric equations:
        // ray_x + t * ray_dx = x1 + u * line_dx
        // ray_y + t * ray_dy = y1 + u * line_dy

        let denominator = ray_dx * line_dy - ray_dy * line_dx;

        if denominator.abs() < 0.0001 {
            // Lines are parallel
            return None;
        }

        let u = ((ray_x - x1) * ray_dy - (ray_y - y1) * ray_dx) / denominator;
        let t = ((ray_x - x1) * line_dy - (ray_y - y1) * line_dx) / denominator;

        // Check if intersection is on both line segment and ray (idiomatic range check)
        if (0.0..=1.0).contains(&u) && t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }

    /// Calculate wall height on screen based on distance
    fn calculate_wall_height(&self, distance: f32) -> usize {
        // Perspective projection
        // wall_height = (actual_height / distance) * projection_plane_distance

        let wall_actual_height = 128.0; // Standard Doom wall height
        let projection_distance = (self.width as f32 / 2.0) / (self.fov / 2.0).tan();

        let projected_height = (wall_actual_height / distance) * projection_distance;

        (projected_height as usize).min(self.height * 2) // Cap at 2x screen height
    }

    /// Draw a vertical wall column
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

    /// Get the rendered framebuffer
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Move player forward/backward
    pub fn move_forward(&mut self, amount: f32) {
        self.player_x += self.player_angle.cos() * amount;
        self.player_y += self.player_angle.sin() * amount;
    }

    /// Move player left/right (strafe)
    pub fn move_strafe(&mut self, amount: f32) {
        let strafe_angle = self.player_angle + PI / 2.0;
        self.player_x += strafe_angle.cos() * amount;
        self.player_y += strafe_angle.sin() * amount;
    }

    /// Rotate player (turn)
    pub fn rotate(&mut self, amount: f32) {
        self.player_angle += amount;

        // Keep angle in 0..2PI range
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

        // Center of screen should give player angle
        let center_angle = renderer.calculate_ray_angle(160);
        assert!((center_angle - renderer.player_angle).abs() < 0.01);

        // Left edge should give negative offset
        let left_angle = renderer.calculate_ray_angle(0);
        assert!(left_angle < renderer.player_angle);

        // Right edge should give positive offset
        let right_angle = renderer.calculate_ray_angle(319);
        assert!(right_angle > renderer.player_angle);
    }

    #[test]
    fn test_player_movement() {
        let mut renderer = RaycastRenderer::new(320, 240);
        renderer.player_x = 100.0;
        renderer.player_y = 100.0;
        renderer.player_angle = 0.0; // Facing east

        // Move forward (east)
        renderer.move_forward(10.0);
        assert!((renderer.player_x - 110.0).abs() < 0.1);
        assert!((renderer.player_y - 100.0).abs() < 0.1);

        // Rotate and move
        renderer.rotate(PI / 2.0); // Face north
        renderer.move_forward(10.0);
        assert!((renderer.player_y - 110.0).abs() < 0.1);
    }
}
