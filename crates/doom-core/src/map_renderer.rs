// SPDX-License-Identifier: AGPL-3.0-only
//! Map rendering - Phase 1.1: Top-down 2D view
//!
//! This module renders Doom maps in a simple top-down view.

use crate::wad_loader::{MapData, Vertex};

/// Map renderer for top-down 2D view.
pub struct MapRenderer {
    width: usize,
    height: usize,
    framebuffer: Vec<u8>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl MapRenderer {
    /// Create a new map renderer with the given framebuffer dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            framebuffer: vec![0; width * height * 4],
            scale: 0.1,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// Render a map to the framebuffer.
    pub fn render(&mut self, map: &MapData) {
        for pixel in self.framebuffer.chunks_exact_mut(4) {
            pixel[0] = 32;
            pixel[1] = 32;
            pixel[2] = 32;
            pixel[3] = 255;
        }

        if !map.vertices.is_empty() {
            self.calculate_view_transform(map);
        }

        for linedef in &map.linedefs {
            if linedef.start_vertex < map.vertices.len() && linedef.end_vertex < map.vertices.len()
            {
                let v1 = map.vertices[linedef.start_vertex];
                let v2 = map.vertices[linedef.end_vertex];

                let color = if linedef.flags & 0x0001 != 0 {
                    [255, 255, 255, 255]
                } else {
                    [128, 128, 128, 255]
                };

                self.draw_line(v1, v2, color);
            }
        }

        for thing in &map.things {
            let color = match thing.thing_type {
                1 => [0, 255, 0, 255],
                2 => [0, 255, 255, 255],
                3 => [255, 255, 0, 255],
                4 => [255, 0, 255, 255],
                3001..=3006 => [255, 0, 0, 255],
                _ => [128, 128, 255, 255],
            };

            self.draw_thing(thing.x, thing.y, color);
        }
    }

    /// Get the rendered framebuffer.
    #[must_use]
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "framebuffer dimensions are small enough for f32"
    )]
    fn calculate_view_transform(&mut self, map: &MapData) {
        let mut min_x = i16::MAX;
        let mut max_x = i16::MIN;
        let mut min_y = i16::MAX;
        let mut max_y = i16::MIN;

        for vertex in &map.vertices {
            min_x = min_x.min(vertex.x);
            max_x = max_x.max(vertex.x);
            min_y = min_y.min(vertex.y);
            max_y = max_y.max(vertex.y);
        }

        let map_width = f32::from(max_x - min_x);
        let map_height = f32::from(max_y - min_y);

        let padding = 0.9;
        let scale_x = (self.width as f32 * padding) / map_width;
        let scale_y = (self.height as f32 * padding) / map_height;
        self.scale = scale_x.min(scale_y);

        let center_x = f32::from(min_x + max_x) / 2.0;
        let center_y = f32::from(min_y + max_y) / 2.0;

        self.offset_x = (self.width as f32 / 2.0) - (center_x * self.scale);
        self.offset_y = (self.height as f32 / 2.0) + (center_y * self.scale);
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "screen coordinates fit in i32"
    )]
    fn world_to_screen(&self, x: i16, y: i16) -> (i32, i32) {
        let screen_x = (f32::from(x) * self.scale + self.offset_x) as i32;
        let screen_y = (-f32::from(y) * self.scale + self.offset_y) as i32;
        (screen_x, screen_y)
    }

    fn draw_line(&mut self, v1: Vertex, v2: Vertex, color: [u8; 4]) {
        let (x1, y1) = self.world_to_screen(v1.x, v1.y);
        let (x2, y2) = self.world_to_screen(v2.x, v2.y);

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            self.set_pixel(x, y, color);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_thing(&mut self, x: i16, y: i16, color: [u8; 4]) {
        let (screen_x, screen_y) = self.world_to_screen(x, y);
        let radius = 3;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    self.set_pixel(screen_x + dx, screen_y + dy, color);
                }
            }
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        let Ok(ux) = usize::try_from(x) else { return };
        let Ok(uy) = usize::try_from(y) else { return };
        if ux < self.width && uy < self.height {
            let idx = (uy * self.width + ux) * 4;
            self.framebuffer[idx..idx + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad_loader::LineDef;

    #[test]
    fn test_renderer_creation() {
        let renderer = MapRenderer::new(640, 480);
        assert_eq!(renderer.framebuffer().len(), 640 * 480 * 4);
    }

    #[test]
    fn test_world_to_screen() {
        let renderer = MapRenderer::new(640, 480);
        let (x, y) = renderer.world_to_screen(0, 0);
        assert_eq!(x, 0, "Origin should map to screen origin with zero offset");
        assert_eq!(y, 0, "Origin should map to screen origin with zero offset");

        let (x, y) = renderer.world_to_screen(1000, 500);
        assert_eq!(x, 100);
        assert_eq!(y, -50);
    }

    #[test]
    fn test_render_simple_map() {
        let mut renderer = MapRenderer::new(320, 240);

        let map = MapData {
            name: "TEST".to_string(),
            vertices: vec![
                Vertex { x: 0, y: 0 },
                Vertex { x: 100, y: 0 },
                Vertex { x: 100, y: 100 },
                Vertex { x: 0, y: 100 },
            ],
            linedefs: vec![
                LineDef {
                    start_vertex: 0,
                    end_vertex: 1,
                    flags: 1,
                    line_type: 0,
                    sector_tag: 0,
                },
                LineDef {
                    start_vertex: 1,
                    end_vertex: 2,
                    flags: 1,
                    line_type: 0,
                    sector_tag: 0,
                },
                LineDef {
                    start_vertex: 2,
                    end_vertex: 3,
                    flags: 1,
                    line_type: 0,
                    sector_tag: 0,
                },
                LineDef {
                    start_vertex: 3,
                    end_vertex: 0,
                    flags: 1,
                    line_type: 0,
                    sector_tag: 0,
                },
            ],
            sectors: vec![],
            things: vec![],
        };

        renderer.render(&map);

        let non_zero = renderer.framebuffer().iter().any(|&b| b != 0 && b != 32);
        assert!(non_zero, "Renderer should have drawn something");
    }
}
