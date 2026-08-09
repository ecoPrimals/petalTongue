// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene-agnostic frame output for the doom renderer.
//!
//! `DoomFrame` is a vector of colored rectangles produced by `render_to_frame()`.
//! Consumers (like `petal-tongue-ui`) can convert this to their own scene format
//! without coupling `doom-core` to any specific scene graph implementation.

/// A single colored rectangle in the rendered frame.
#[derive(Debug, Clone)]
pub struct FrameRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    /// Semantic identifier for traceability (e.g. "sky", "floor", "wall:3:120").
    pub data_id: String,
}

/// A rendered doom frame as a collection of rectangles.
///
/// This is the scene-agnostic output of `DoomInstance::render_frame()` and
/// `RaycastRenderer::render_to_frame()`. Each rectangle represents a visible
/// region with a solid color, suitable for conversion to any scene format.
#[derive(Debug, Clone, Default)]
pub struct DoomFrame {
    pub rects: Vec<FrameRect>,
}

impl DoomFrame {
    #[must_use]
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    pub fn push(&mut self, rect: FrameRect) {
        self.rects.push(rect);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
}
