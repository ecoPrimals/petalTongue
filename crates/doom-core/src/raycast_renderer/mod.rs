// SPDX-License-Identifier: AGPL-3.0-or-later
//! First-person raycasting renderer - Phase 1.2
//!
//! Renders Doom maps from a first-person perspective using raycasting.

mod math;
#[cfg(test)]
mod tests;

use crate::wad_loader::MapData;
use math::{
    distance_shading, normalize_angle_0_two_pi, projected_wall_height, ray_line_intersection,
    screen_ray_angle, shade_wall_rgba,
};
use petal_tongue_scene::primitive::{Color as SceneColor, Primitive};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
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
        fill_sky_and_floor(&mut self.framebuffer, self.width, self.height);

        for x in 0..self.width {
            self.cast_and_draw_ray(map, x);
        }
    }

    fn cast_and_draw_ray(&mut self, map: &MapData, screen_x: usize) {
        let ray_angle = self.calculate_ray_angle(screen_x);

        if let Some((distance, wall_color)) = self.cast_ray(map, ray_angle) {
            let wall_height = self.calculate_wall_height(distance);
            let shading = distance_shading(distance, self.render_distance);
            let shaded_color = shade_wall_rgba(wall_color, shading);

            draw_wall_column(
                &mut self.framebuffer,
                self.width,
                self.height,
                screen_x,
                wall_height,
                shaded_color,
            );
        }
    }

    pub(super) fn calculate_ray_angle(&self, screen_x: usize) -> f32 {
        screen_ray_angle(screen_x, self.width, self.player_angle, self.fov)
    }

    fn cast_ray(&self, map: &MapData, angle: f32) -> Option<(f32, [u8; 3])> {
        self.cast_ray_to_linedef(map, angle)
            .map(|(distance, color, _)| (distance, color))
    }

    /// Like `cast_ray` but also returns the linedef index that was hit.
    fn cast_ray_with_linedef(&self, map: &MapData, angle: f32) -> Option<(f32, [u8; 3], usize)> {
        self.cast_ray_to_linedef(map, angle)
    }

    #[expect(clippy::similar_names, reason = "standard raycasting names")]
    fn cast_ray_to_linedef(&self, map: &MapData, angle: f32) -> Option<(f32, [u8; 3], usize)> {
        let ray_dx = angle.cos();
        let ray_dy = angle.sin();

        let mut nearest_distance = self.render_distance;
        let mut nearest_color = [255u8, 255, 255];
        let mut nearest_linedef = 0usize;

        for (idx, linedef) in map.linedefs.iter().enumerate() {
            if linedef.start_vertex >= map.vertices.len()
                || linedef.end_vertex >= map.vertices.len()
            {
                continue;
            }
            let v1 = map.vertices[linedef.start_vertex];
            let v2 = map.vertices[linedef.end_vertex];

            if let Some(distance) =
                ray_line_intersection(self.player_x, self.player_y, ray_dx, ray_dy, v1, v2)
                && distance < nearest_distance
                && distance > 0.1
            {
                nearest_distance = distance;
                nearest_linedef = idx;
                nearest_color = if linedef.flags & 0x0001 != 0 {
                    [180, 180, 180]
                } else {
                    [100, 100, 100]
                };
            }
        }

        if nearest_distance < self.render_distance {
            Some((nearest_distance, nearest_color, nearest_linedef))
        } else {
            None
        }
    }

    pub(super) fn calculate_wall_height(&self, distance: f32) -> usize {
        projected_wall_height(distance, self.width, self.height, self.fov)
    }

    /// Render the map to a `SceneGraph` instead of a raw framebuffer.
    ///
    /// Every pixel region is represented by a `Primitive::Rect` with a `data_id`
    /// encoding its origin: `"sky"`, `"floor"`, or `"wall:<linedef_index>:<column>"`.
    #[expect(
        clippy::cast_precision_loss,
        reason = "screen dimensions and indices fit losslessly in f64/f32"
    )]
    #[must_use]
    pub fn render_to_scene(&self, map: &MapData) -> SceneGraph {
        let mut scene = SceneGraph::new();
        let scene_width = self.width as f64;
        let scene_height = self.height as f64;
        add_sky_and_floor_planes(&mut scene, scene_width, scene_height);

        for x in 0..self.width {
            let ray_angle = self.calculate_ray_angle(x);
            if let Some((distance, wall_color, linedef_idx)) =
                self.cast_ray_with_linedef(map, ray_angle)
            {
                let wall_height = self.calculate_wall_height(distance);
                let shading = distance_shading(distance, self.render_distance);
                let wall_rgb = [
                    (f32::from(wall_color[0]) * shading) / 255.0,
                    (f32::from(wall_color[1]) * shading) / 255.0,
                    (f32::from(wall_color[2]) * shading) / 255.0,
                ];
                append_scene_wall_column(
                    &mut scene,
                    x,
                    self.height,
                    wall_height,
                    wall_rgb,
                    linedef_idx,
                );
            }
        }

        scene
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

    /// Rotate player (turn). Angle is normalized to [0, 2π).
    pub fn rotate(&mut self, amount: f32) {
        self.player_angle = normalize_angle_0_two_pi(self.player_angle + amount);
    }
}

fn add_sky_and_floor_planes(scene: &mut SceneGraph, scene_width: f64, scene_height: f64) {
    let half_height = scene_height / 2.0;

    scene.add_to_root(SceneNode::new("sky").with_primitive(Primitive::Rect {
        x: 0.0,
        y: 0.0,
        width: scene_width,
        height: half_height,
        fill: Some(SceneColor::from_rgba8(100, 150, 200, 255)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("sky".to_string()),
    }));
    scene.add_to_root(SceneNode::new("floor").with_primitive(Primitive::Rect {
        x: 0.0,
        y: half_height,
        width: scene_width,
        height: half_height,
        fill: Some(SceneColor::from_rgba8(64, 64, 64, 255)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("floor".to_string()),
    }));
}

#[expect(
    clippy::cast_precision_loss,
    reason = "scene column geometry uses f64 screen space"
)]
fn append_scene_wall_column(
    scene: &mut SceneGraph,
    column_x: usize,
    screen_height: usize,
    wall_height: usize,
    wall_rgb: [f32; 3],
    linedef_idx: usize,
) {
    let [r, g, b] = wall_rgb;
    let screen_center = screen_height / 2;
    let half_wall = wall_height / 2;
    let y_start = screen_center.saturating_sub(half_wall);
    let y_end = (screen_center + half_wall).min(screen_height);
    let col_height = (y_end - y_start) as f64;

    if col_height <= 0.0 {
        return;
    }

    let node_id = format!("wall_{column_x}");
    let data_id = format!("wall:{linedef_idx}:{column_x}");
    scene.add_to_root(SceneNode::new(node_id).with_primitive(Primitive::Rect {
        x: column_x as f64,
        y: y_start as f64,
        width: 1.0,
        height: col_height,
        fill: Some(SceneColor::rgba(r, g, b, 1.0)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some(data_id),
    }));
}

fn fill_sky_and_floor(framebuffer: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;

            if y < height / 2 {
                framebuffer[idx] = 100;
                framebuffer[idx + 1] = 150;
                framebuffer[idx + 2] = 200;
            } else {
                framebuffer[idx] = 64;
                framebuffer[idx + 1] = 64;
                framebuffer[idx + 2] = 64;
            }
            framebuffer[idx + 3] = 255;
        }
    }
}

fn draw_wall_column(
    framebuffer: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    wall_height: usize,
    color: [u8; 4],
) {
    let screen_center = height / 2;
    let half_height = wall_height / 2;

    let y_start = screen_center.saturating_sub(half_height);
    let y_end = (screen_center + half_height).min(height);

    for y in y_start..y_end {
        let idx = (y * width + x) * 4;
        if idx + 3 < framebuffer.len() {
            framebuffer[idx..idx + 4].copy_from_slice(&color);
        }
    }
}
