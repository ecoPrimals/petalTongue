// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rendering primitives -- the atomic visual elements of a scene.
//!
//! Every visualization compiles to a collection of `Primitive` values.
//! Modality compilers translate these to backend-specific output
//! (egui paint commands, SVG elements, audio parameters, terminal cells).

use serde::{Deserialize, Serialize};

/// RGBA color with floating-point channels (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Convert from 8-bit RGBA.
    #[must_use]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Stroke style for lines and outlines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f32,
    pub cap: LineCap,
    pub join: LineJoin,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        }
    }
}

/// Line cap style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Line join style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Fill rule for closed paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillRule {
    EvenOdd,
    NonZero,
}

/// Text anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorPoint {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// A cubic Bezier segment (two control points + endpoint).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BezierSegment {
    pub cp1: [f64; 2],
    pub cp2: [f64; 2],
    pub end: [f64; 2],
}

/// A vertex in a 3D mesh.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: [f64; 3],
    pub normal: [f64; 3],
    pub color: Color,
}

/// The atomic rendering primitives.
///
/// These map directly to output in any modality:
/// - egui: paint commands
/// - SVG: XML elements
/// - ratatui: character cells
/// - Audio: spatial sound parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Primitive {
    /// A filled or stroked circle.
    Point {
        x: f64,
        y: f64,
        radius: f64,
        fill: Option<Color>,
        stroke: Option<StrokeStyle>,
        /// Optional data object ID for hit-testing.
        data_id: Option<String>,
    },

    /// A polyline or polygon outline.
    Line {
        points: Vec<[f64; 2]>,
        stroke: StrokeStyle,
        closed: bool,
        data_id: Option<String>,
    },

    /// A filled rectangle.
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: Option<Color>,
        stroke: Option<StrokeStyle>,
        corner_radius: f64,
        data_id: Option<String>,
    },

    /// Rendered text.
    Text {
        x: f64,
        y: f64,
        content: String,
        font_size: f64,
        color: Color,
        anchor: AnchorPoint,
        bold: bool,
        italic: bool,
        data_id: Option<String>,
    },

    /// A filled polygon.
    Polygon {
        points: Vec<[f64; 2]>,
        fill: Color,
        stroke: Option<StrokeStyle>,
        fill_rule: FillRule,
        data_id: Option<String>,
    },

    /// A circular arc (pie slice or ring segment).
    Arc {
        cx: f64,
        cy: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        fill: Option<Color>,
        stroke: Option<StrokeStyle>,
        data_id: Option<String>,
    },

    /// A cubic Bezier path.
    BezierPath {
        start: [f64; 2],
        segments: Vec<BezierSegment>,
        stroke: StrokeStyle,
        fill: Option<Color>,
        fill_rule: FillRule,
        data_id: Option<String>,
    },

    /// A 3D triangle mesh (for Toadstool GPU rendering).
    Mesh {
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        data_id: Option<String>,
    },
}

impl Primitive {
    /// Get the data object ID for hit-testing, if any.
    #[must_use]
    pub fn data_id(&self) -> Option<&str> {
        match self {
            Self::Point { data_id, .. }
            | Self::Line { data_id, .. }
            | Self::Rect { data_id, .. }
            | Self::Text { data_id, .. }
            | Self::Polygon { data_id, .. }
            | Self::Arc { data_id, .. }
            | Self::BezierPath { data_id, .. }
            | Self::Mesh { data_id, .. } => data_id.as_deref(),
        }
    }

    /// Returns true if this primitive carries a data reference.
    #[must_use]
    pub fn carries_data(&self) -> bool {
        self.data_id().is_some()
    }
}

#[cfg(test)]
mod tests;
