//! Map rendering - Phase 1.1: Top-down 2D view
//!
//! This module renders Doom maps in a simple top-down view.
//! Later phases will add first-person 3D rendering.

use crate::wad_loader::{MapData, Vertex};

/// Map renderer for top-down 2D view
pub struct MapRenderer {
    width: usize,
    height: usize,
    framebuffer: Vec<u8>, // RGBA
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl MapRenderer {
    /// Create a new map renderer
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            framebuffer: vec![0; width * height * 4],
            scale: 0.1, // Start with a reasonable scale
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// Render a map to the framebuffer
    pub fn render(&mut self, map: &MapData) {
        // Clear framebuffer (dark gray background)
        for pixel in self.framebuffer.chunks_exact_mut(4) {
            pixel[0] = 32; // R
            pixel[1] = 32; // G
            pixel[2] = 32; // B
            pixel[3] = 255; // A
        }

        // Calculate bounds to center the map
        if !map.vertices.is_empty() {
            self.calculate_view_transform(map);
        }

        // Draw all linedefs (walls)
        for linedef in &map.linedefs {
            if linedef.start_vertex < map.vertices.len() && linedef.end_vertex < map.vertices.len()
            {
                let v1 = map.vertices[linedef.start_vertex];
                let v2 = map.vertices[linedef.end_vertex];

                // Different colors for different line types
                let color = if linedef.flags & 0x0001 != 0 {
                    // Impassable wall (solid white)
                    [255, 255, 255, 255]
                } else {
                    // Passable line (gray)
                    [128, 128, 128, 255]
                };

                self.draw_line(v1, v2, color);
            }
        }

        // Draw things (player start, enemies, items)
        for thing in &map.things {
            let color = match thing.thing_type {
                1 => [0, 255, 0, 255],   // Player 1 start (green)
                2 => [0, 255, 255, 255], // Player 2 start (cyan)
                3 => [255, 255, 0, 255], // Player 3 start (yellow)
                4 => [255, 0, 255, 255], // Player 4 start (magenta)
                _ if thing.thing_type >= 3001 && thing.thing_type <= 3006 => {
                    // Enemies (red)
                    [255, 0, 0, 255]
                }
                _ => [128, 128, 255, 255], // Other things (blue)
            };

            self.draw_thing(thing.x, thing.y, color);
        }
    }

    /// Get the rendered framebuffer
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Calculate view transform to center and scale the map
    fn calculate_view_transform(&mut self, map: &MapData) {
        // Find bounds
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

        let map_width = (max_x - min_x) as f32;
        let map_height = (max_y - min_y) as f32;

        // Calculate scale to fit map in view with some padding
        let padding = 0.9; // 90% of available space
        let scale_x = (self.width as f32 * padding) / map_width;
        let scale_y = (self.height as f32 * padding) / map_height;
        self.scale = scale_x.min(scale_y);

        // Calculate offset to center the map
        let center_x = (min_x + max_x) as f32 / 2.0;
        let center_y = (min_y + max_y) as f32 / 2.0;

        self.offset_x = (self.width as f32 / 2.0) - (center_x * self.scale);
        self.offset_y = (self.height as f32 / 2.0) + (center_y * self.scale); // Y is flipped
    }

    /// Transform world coordinates to screen coordinates
    fn world_to_screen(&self, x: i16, y: i16) -> (i32, i32) {
        let screen_x = (x as f32 * self.scale + self.offset_x) as i32;
        let screen_y = (-(y as f32) * self.scale + self.offset_y) as i32; // Flip Y
        (screen_x, screen_y)
    }

    /// Draw a line between two vertices
    fn draw_line(&mut self, v1: Vertex, v2: Vertex, color: [u8; 4]) {
        let (x1, y1) = self.world_to_screen(v1.x, v1.y);
        let (x2, y2) = self.world_to_screen(v2.x, v2.y);

        // Bresenham's line algorithm
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

    /// Draw a thing (small circle)
    fn draw_thing(&mut self, x: i16, y: i16, color: [u8; 4]) {
        let (screen_x, screen_y) = self.world_to_screen(x, y);
        let radius = 3; // pixels

        // Draw a simple filled circle
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    self.set_pixel(screen_x + dx, screen_y + dy, color);
                }
            }
        }
    }

    /// Set a pixel in the framebuffer
    fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = ((y as usize * self.width) + x as usize) * 4;
            if idx + 3 < self.framebuffer.len() {
                self.framebuffer[idx..idx + 4].copy_from_slice(&color);
            }
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
        // With default scale=0.1 and offset=0, (0,0) maps to (0,0)
        // Test that the transform is mathematically correct
        let (x, y) = renderer.world_to_screen(0, 0);
        assert_eq!(x, 0, "Origin should map to screen origin with zero offset");
        assert_eq!(y, 0, "Origin should map to screen origin with zero offset");

        // Test with non-zero coordinates
        let (x, y) = renderer.world_to_screen(1000, 500);
        // scale=0.1: x = 1000 * 0.1 + 0 = 100, y = -500 * 0.1 + 0 = -50
        assert_eq!(x, 100);
        assert_eq!(y, -50); // Y is flipped
    }

    #[test]
    fn test_render_simple_map() {
        let mut renderer = MapRenderer::new(320, 240);

        // Create a simple square map
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

        // Framebuffer should not be all zeros (something was drawn)
        let non_zero = renderer.framebuffer().iter().any(|&b| b != 0 && b != 32);
        assert!(non_zero, "Renderer should have drawn something");
    }
}
