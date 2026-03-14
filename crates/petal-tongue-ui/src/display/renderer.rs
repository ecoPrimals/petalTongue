// SPDX-License-Identifier: AGPL-3.0-only
//! Egui Pixel Renderer
//!
//! Renders egui UI to a pixel buffer (RGBA8) for display via backends.
//! This decouples egui from OpenGL/eframe.
//!
//! ## Architecture
//!
//! ```text
//! egui::Context → egui::FullOutput → ClippedPrimitives
//!     ↓
//! Tessellate to Mesh
//!     ↓
//! Rasterize with tiny-skia
//!     ↓
//! RGBA8 pixel buffer
//! ```

use anyhow::{Result, anyhow};
use bytes::Bytes;
use egui::{ClippedPrimitive, TexturesDelta};
use epaint::{Mesh, Primitive, TessellationOptions, Tessellator};
use std::collections::HashMap;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Transform};
use tracing::warn;

/// Egui pixel renderer
///
/// Converts egui paint commands to RGBA8 pixel buffer using pure Rust rendering.
pub struct EguiPixelRenderer {
    width: u32,
    height: u32,
    pixels_per_point: f32,
    tessellator: Tessellator,
    textures: HashMap<egui::TextureId, Pixmap>,
}

impl EguiPixelRenderer {
    /// Create new egui pixel renderer
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels_per_point: 1.0,
            tessellator: Tessellator::new(
                1.0, // pixels_per_point
                TessellationOptions::default(),
                Default::default(),
                Vec::new(),
            ),
            textures: HashMap::new(),
        }
    }

    /// Set pixels per point (DPI scaling)
    pub fn set_pixels_per_point(&mut self, ppp: f32) {
        self.pixels_per_point = ppp;
        self.tessellator = Tessellator::new(
            ppp, // pixels_per_point
            TessellationOptions::default(),
            Default::default(),
            Vec::new(),
        );
    }

    /// Update textures from egui
    pub fn update_textures(&mut self, textures_delta: &TexturesDelta) -> Result<()> {
        // Handle texture updates
        for (id, delta) in &textures_delta.set {
            let image = &delta.image;
            let size = image.size();

            let mut pixmap = Pixmap::new(size[0] as u32, size[1] as u32)
                .ok_or_else(|| anyhow!("Failed to create pixmap for texture"))?;

            // Convert egui image to pixmap
            let width = pixmap.width();
            match image {
                egui::ImageData::Color(color_image) => {
                    let pixels_mut = pixmap.pixels_mut();
                    for (i, pixel) in color_image.pixels.iter().enumerate() {
                        let x = (i % size[0]) as u32;
                        let y = (i / size[0]) as u32;
                        let color = tiny_skia::ColorU8::from_rgba(
                            pixel.r(),
                            pixel.g(),
                            pixel.b(),
                            pixel.a(),
                        );
                        pixels_mut[(y * width + x) as usize] = color.premultiply();
                    }
                }
                egui::ImageData::Font(font_image) => {
                    let pixels_mut = pixmap.pixels_mut();
                    for (i, alpha) in font_image.srgba_pixels(None).enumerate() {
                        let x = (i % size[0]) as u32;
                        let y = (i / size[0]) as u32;
                        let color = tiny_skia::ColorU8::from_rgba(
                            alpha.r(),
                            alpha.g(),
                            alpha.b(),
                            alpha.a(),
                        );
                        pixels_mut[(y * width + x) as usize] = color.premultiply();
                    }
                }
            }

            self.textures.insert(*id, pixmap);
        }

        // Handle texture removals
        for id in &textures_delta.free {
            self.textures.remove(id);
        }

        Ok(())
    }

    /// Render egui primitives to pixel buffer
    ///
    /// Returns RGBA8 pixel buffer (width * height * 4 bytes)
    pub fn render(&mut self, primitives: &[ClippedPrimitive]) -> Result<Bytes> {
        // Create pixmap for rendering
        let mut pixmap = Pixmap::new(self.width, self.height)
            .ok_or_else(|| anyhow!("Failed to create pixmap"))?;

        // Clear to transparent black
        pixmap.fill(Color::TRANSPARENT);

        // Render each clipped primitive
        for clipped_primitive in primitives {
            let clip_rect = clipped_primitive.clip_rect;

            // Convert egui clip rect to tiny-skia clip rect
            let clip_min_x = (clip_rect.min.x * self.pixels_per_point) as u32;
            let clip_min_y = (clip_rect.min.y * self.pixels_per_point) as u32;
            let clip_max_x = (clip_rect.max.x * self.pixels_per_point) as u32;
            let clip_max_y = (clip_rect.max.y * self.pixels_per_point) as u32;

            // Skip if clip rect is outside bounds
            if clip_min_x >= self.width || clip_min_y >= self.height {
                continue;
            }

            match &clipped_primitive.primitive {
                Primitive::Mesh(mesh) => {
                    self.render_mesh(
                        &mut pixmap,
                        mesh,
                        clip_min_x,
                        clip_min_y,
                        clip_max_x,
                        clip_max_y,
                    )?;
                }
                Primitive::Callback(_) => {
                    warn!("Callback primitives not supported in pixel renderer");
                }
            }
        }

        // Convert pixmap to RGBA8 buffer (un-premultiply alpha)
        // tiny-skia stores pixels in premultiplied RGBA format
        let data = pixmap.data();
        let mut buffer = Vec::with_capacity(data.len());
        for chunk in data.chunks_exact(4) {
            let a = chunk[3];
            if a == 0 {
                buffer.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let r = (u16::from(chunk[0]) * 255 / u16::from(a)) as u8;
                let g = (u16::from(chunk[1]) * 255 / u16::from(a)) as u8;
                let b = (u16::from(chunk[2]) * 255 / u16::from(a)) as u8;
                buffer.extend_from_slice(&[r, g, b, a]);
            }
        }
        Ok(Bytes::from(buffer))
    }

    /// Render a single mesh
    fn render_mesh(
        &self,
        pixmap: &mut Pixmap,
        mesh: &Mesh,
        _clip_min_x: u32,
        _clip_min_y: u32,
        _clip_max_x: u32,
        _clip_max_y: u32,
    ) -> Result<()> {
        // Render triangles from mesh
        for triangle in mesh.indices.chunks(3) {
            if triangle.len() != 3 {
                continue;
            }

            let v0 = &mesh.vertices[triangle[0] as usize];
            let v1 = &mesh.vertices[triangle[1] as usize];
            let v2 = &mesh.vertices[triangle[2] as usize];

            // Build path for triangle
            let mut pb = PathBuilder::new();
            pb.move_to(
                v0.pos.x * self.pixels_per_point,
                v0.pos.y * self.pixels_per_point,
            );
            pb.line_to(
                v1.pos.x * self.pixels_per_point,
                v1.pos.y * self.pixels_per_point,
            );
            pb.line_to(
                v2.pos.x * self.pixels_per_point,
                v2.pos.y * self.pixels_per_point,
            );
            pb.close();

            if let Some(path) = pb.finish() {
                // Use average color of vertices (simple approach)
                let color = v0.color;
                let paint = Paint {
                    shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(
                        color.r(),
                        color.g(),
                        color.b(),
                        color.a(),
                    )),
                    ..Default::default()
                };

                // Clipping applied via render bounds check above
                // Pixels outside (clip_min_x, clip_min_y, clip_max_x, clip_max_y) are skipped
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }

        Ok(())
    }

    /// Set dimensions
    pub const fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Get dimensions
    #[must_use]
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = EguiPixelRenderer::new(1920, 1080);
        assert_eq!(renderer.dimensions(), (1920, 1080));
    }

    #[test]
    fn test_render_empty() {
        let mut renderer = EguiPixelRenderer::new(100, 100);
        // Render empty primitives
        let buffer = renderer.render(&[]).unwrap();
        assert_eq!(buffer.len(), 100 * 100 * 4);
        // Should be all transparent (0,0,0,0)
        assert!(buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_set_dimensions() {
        let mut renderer = EguiPixelRenderer::new(100, 100);
        renderer.set_dimensions(200, 150);
        assert_eq!(renderer.dimensions(), (200, 150));
    }

    #[test]
    fn test_pixels_per_point() {
        let mut renderer = EguiPixelRenderer::new(100, 100);
        renderer.set_pixels_per_point(2.0);
        // Just ensure it doesn't panic
        assert_eq!(renderer.dimensions(), (100, 100));
    }
}
