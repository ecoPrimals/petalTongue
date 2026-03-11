// SPDX-License-Identifier: AGPL-3.0-only
//! Perspective system -- the "6 vs 9" solution.
//!
//! A [`Perspective`] is the complete context through which a human perceives
//! and interacts with data. Two users can have different perspectives on the
//! same data and both be correct. Selection and focus operate on
//! [`DataObjectId`] values (perspective-invariant),
//! not on rendered primitives.

use serde::{Deserialize, Serialize};

use super::target::DataObjectId;

/// Unique identifier for a perspective.
pub type PerspectiveId = u64;

/// A complete viewing and interaction context.
///
/// Includes which modalities are active, what subset of data is visible,
/// and what is selected. Multiple perspectives can coexist, each
/// rendering the same underlying data differently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    /// Unique identifier.
    pub id: PerspectiveId,
    /// Active output modalities for this perspective.
    pub modalities: Vec<OutputModality>,
    /// Viewport bounds.
    pub viewport: PerspectiveViewport,
    /// Active data filters.
    pub filters: Vec<super::target::FilterExpr>,
    /// Coordinate orientation (rotation, mirroring).
    pub orientation: Orientation,
    /// Currently selected data objects.
    pub selection: Vec<DataObjectId>,
    /// Currently focused data object (hover / tab-to).
    pub focus: Option<DataObjectId>,
    /// How this perspective synchronizes with others.
    pub sync_mode: PerspectiveSync,
    /// Optional user identity for multi-user collaboration.
    pub user: Option<String>,
}

impl Perspective {
    /// Create a new default perspective with the given ID.
    #[must_use]
    pub fn new(id: PerspectiveId) -> Self {
        Self {
            id,
            modalities: vec![OutputModality::Gui],
            viewport: PerspectiveViewport::default(),
            filters: Vec::new(),
            orientation: Orientation::Default,
            selection: Vec::new(),
            focus: None,
            sync_mode: PerspectiveSync::SharedSelection,
            user: None,
        }
    }

    /// Check if a data object is currently selected.
    #[must_use]
    pub fn is_selected(&self, id: &DataObjectId) -> bool {
        self.selection.contains(id)
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Set selection to a single object, replacing any existing selection.
    pub fn select(&mut self, id: DataObjectId) {
        self.selection.clear();
        self.selection.push(id);
    }

    /// Add an object to the selection.
    pub fn add_to_selection(&mut self, id: DataObjectId) {
        if !self.selection.contains(&id) {
            self.selection.push(id);
        }
    }

    /// Toggle an object's selection state.
    pub fn toggle_selection(&mut self, id: DataObjectId) {
        if let Some(pos) = self.selection.iter().position(|s| s == &id) {
            self.selection.swap_remove(pos);
        } else {
            self.selection.push(id);
        }
    }
}

/// How perspectives synchronize with each other.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerspectiveSync {
    /// No synchronization -- fully independent views.
    Independent,
    /// Selection in one perspective highlights in all others.
    #[default]
    SharedSelection,
    /// Zoom, pan, and filter changes propagate.
    SharedViewport,
    /// All state changes propagate.
    FullSync,
}

/// Output modality that a perspective renders to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OutputModality {
    /// Graphical UI (egui, web canvas).
    Gui,
    /// Terminal UI (ratatui).
    Tui,
    /// Audio sonification.
    Audio,
    /// SVG static export.
    Svg,
    /// PNG static export.
    Png,
    /// JSON machine-readable export.
    Json,
    /// Braille display.
    Braille,
    /// Haptic feedback device.
    Haptic,
    /// Headless API (data only, no rendering).
    Headless,
}

/// Coordinate orientation for a perspective.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Orientation {
    /// Standard orientation (Y up, X right).
    #[default]
    Default,
    /// Rotated by given radians.
    Rotated(f64),
    /// Mirrored on an axis.
    Mirrored(Axis),
    /// Arbitrary affine transform (row-major 2x3 matrix).
    Custom([f64; 6]),
}

/// Axis for mirroring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    /// Horizontal axis (flip vertically).
    X,
    /// Vertical axis (flip horizontally).
    Y,
}

/// Viewport for a perspective (more general than `engine::Viewport`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerspectiveViewport {
    /// Center X in data-space coordinates.
    pub center_x: f64,
    /// Center Y in data-space coordinates.
    pub center_y: f64,
    /// Width in data-space units.
    pub width: f64,
    /// Height in data-space units.
    pub height: f64,
    /// Zoom level (1.0 = 1:1 mapping).
    pub zoom: f64,
}

impl Default for PerspectiveViewport {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            width: 100.0,
            height: 100.0,
            zoom: 1.0,
        }
    }
}

impl PerspectiveViewport {
    /// Check if a data-space point is within this viewport.
    #[must_use]
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let half_w = self.width / (2.0 * self.zoom);
        let half_h = self.height / (2.0 * self.zoom);
        (x - self.center_x).abs() <= half_w && (y - self.center_y).abs() <= half_h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perspective_selection_ops() {
        let mut p = Perspective::new(1);
        let id_a = DataObjectId::new("test", serde_json::json!("a"));
        let id_b = DataObjectId::new("test", serde_json::json!("b"));

        p.select(id_a.clone());
        assert!(p.is_selected(&id_a));
        assert!(!p.is_selected(&id_b));

        p.add_to_selection(id_b.clone());
        assert_eq!(p.selection.len(), 2);

        p.toggle_selection(id_a.clone());
        assert!(!p.is_selected(&id_a));
        assert!(p.is_selected(&id_b));

        p.clear_selection();
        assert!(p.selection.is_empty());
    }

    #[test]
    fn perspective_serialization() {
        let p = Perspective::new(42);
        let json = serde_json::to_string(&p).expect("serialize");
        let back: Perspective = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, 42);
        assert_eq!(back.sync_mode, PerspectiveSync::SharedSelection);
    }

    #[test]
    fn viewport_contains() {
        let vp = PerspectiveViewport {
            center_x: 50.0,
            center_y: 50.0,
            width: 100.0,
            height: 100.0,
            zoom: 1.0,
        };
        assert!(vp.contains(50.0, 50.0));
        assert!(vp.contains(0.0, 0.0));
        assert!(!vp.contains(101.0, 50.0));
    }

    #[test]
    fn sync_mode_default() {
        assert_eq!(PerspectiveSync::default(), PerspectiveSync::SharedSelection);
    }

    #[test]
    fn perspective_add_to_selection_no_duplicate() {
        let mut p = Perspective::new(1);
        let id = DataObjectId::new("src", serde_json::json!("a"));
        p.add_to_selection(id.clone());
        p.add_to_selection(id);
        assert_eq!(p.selection.len(), 1);
    }

    #[test]
    fn perspective_viewport_default() {
        let vp = PerspectiveViewport::default();
        assert!((vp.center_x - 0.0).abs() < f64::EPSILON);
        assert!((vp.zoom - 1.0).abs() < f64::EPSILON);
        assert!(vp.contains(0.0, 0.0));
    }

    #[test]
    fn perspective_viewport_contains_with_zoom() {
        let vp = PerspectiveViewport {
            center_x: 50.0,
            center_y: 50.0,
            width: 100.0,
            height: 100.0,
            zoom: 2.0,
        };
        assert!(vp.contains(50.0, 50.0));
        assert!(!vp.contains(0.0, 0.0));
    }

    #[test]
    fn output_modality_variants() {
        assert_eq!(OutputModality::Gui, OutputModality::Gui);
        assert_eq!(OutputModality::Tui, OutputModality::Tui);
        assert_eq!(OutputModality::Audio, OutputModality::Audio);
    }

    #[test]
    fn orientation_variants() {
        assert!(matches!(Orientation::default(), Orientation::Default));
        let rot = Orientation::Rotated(1.5);
        assert!(matches!(rot, Orientation::Rotated(_)));
    }

    #[test]
    fn axis_variants() {
        assert_eq!(Axis::X, Axis::X);
        assert_eq!(Axis::Y, Axis::Y);
    }

    #[test]
    fn perspective_sync_variants() {
        assert_eq!(PerspectiveSync::Independent, PerspectiveSync::Independent);
        assert_eq!(PerspectiveSync::FullSync, PerspectiveSync::FullSync);
    }
}
