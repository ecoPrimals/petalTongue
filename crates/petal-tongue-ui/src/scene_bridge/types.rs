// SPDX-License-Identifier: AGPL-3.0-only
//! Provenance and hit-map types for scene bridge rendering.

use egui::{Pos2, Rect};

/// Provenance of a single rendered region — the answer to "what produced this pixel?"
#[derive(Debug, Clone)]
pub struct PixelProvenance {
    /// The scene-graph node that owns the primitive.
    pub node_id: String,
    /// Index of the primitive within the flattened scene output.
    pub primitive_index: usize,
    /// The `data_id` from the primitive, if any.
    pub data_id: Option<String>,
    /// World-space coordinates (pre-offset) of the primitive origin.
    pub world_x: f64,
    pub world_y: f64,
}

/// Per-frame spatial index mapping screen regions to their scene-graph source.
///
/// Built alongside `paint_scene_tracked` so every egui shape can be traced
/// back through the scene graph to the originating data.
#[derive(Debug, Clone, Default)]
pub struct FrameHitMap {
    entries: Vec<(Rect, PixelProvenance)>,
}

impl FrameHitMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a rendered region with its provenance.
    pub fn register(&mut self, screen_rect: Rect, provenance: PixelProvenance) {
        self.entries.push((screen_rect, provenance));
    }

    /// Query the hit map for the topmost entry at the given screen position.
    ///
    /// Returns the last (topmost) entry whose bounding rect contains the point,
    /// matching the painter's back-to-front draw order.
    #[must_use]
    pub fn query(&self, screen_x: f32, screen_y: f32) -> Option<&PixelProvenance> {
        let pos = Pos2::new(screen_x, screen_y);
        self.entries
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(pos))
            .map(|(_, prov)| prov)
    }

    /// Number of registered entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the hit map is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate all entries.
    pub fn iter(&self) -> impl Iterator<Item = &(Rect, PixelProvenance)> {
        self.entries.iter()
    }
}
