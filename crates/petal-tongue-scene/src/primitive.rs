// SPDX-License-Identifier: AGPL-3.0-only
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
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Convert from 8-bit RGBA.
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
    pub fn carries_data(&self) -> bool {
        self.data_id().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-10;

    #[test]
    fn color_rgb() {
        let c = Color::rgb(0.5, 0.25, 1.0);
        assert!((c.r - 0.5).abs() < EPS);
        assert!((c.g - 0.25).abs() < EPS);
        assert!((c.b - 1.0).abs() < EPS);
        assert!((c.a - 1.0).abs() < EPS);
    }

    #[test]
    fn color_rgba() {
        let c = Color::rgba(1.0, 0.0, 0.5, 0.5);
        assert!((c.r - 1.0).abs() < EPS);
        assert!((c.g - 0.0).abs() < EPS);
        assert!((c.b - 0.5).abs() < EPS);
        assert!((c.a - 0.5).abs() < EPS);
    }

    #[test]
    fn color_from_rgba8() {
        let c = Color::from_rgba8(255, 128, 0, 64);
        assert!((c.r - 1.0).abs() < EPS);
        assert!((c.g - 128.0 / 255.0).abs() < EPS);
        assert!((c.b - 0.0).abs() < EPS);
        assert!((c.a - 64.0 / 255.0).abs() < EPS);
    }

    #[test]
    fn color_default() {
        let c = Color::default();
        assert_eq!(c, Color::BLACK);
    }

    #[test]
    fn stroke_style_default() {
        let s = StrokeStyle::default();
        assert_eq!(s.color, Color::BLACK);
        assert!((s.width - 1.0).abs() < EPS);
        assert_eq!(s.cap, LineCap::Butt);
        assert_eq!(s.join, LineJoin::Miter);
    }

    #[test]
    fn primitive_data_id_extraction() {
        let with_id = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            fill: Some(Color::rgb(1.0, 0.0, 0.0)),
            stroke: None,
            data_id: Some("foo".to_string()),
        };
        assert_eq!(with_id.data_id(), Some("foo"));

        let without_id = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        assert_eq!(without_id.data_id(), None);
    }

    #[test]
    fn primitive_carries_data() {
        let with_id = Primitive::Line {
            points: vec![[0.0, 0.0], [1.0, 1.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: Some("line-1".to_string()),
        };
        assert!(with_id.carries_data());

        let without_id = Primitive::Line {
            points: vec![[0.0, 0.0], [1.0, 1.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: None,
        };
        assert!(!without_id.carries_data());
    }

    #[test]
    fn serialization_roundtrip_point() {
        let point = Primitive::Point {
            x: 10.0,
            y: 20.0,
            radius: 3.0,
            fill: Some(Color::WHITE),
            stroke: Some(StrokeStyle::default()),
            data_id: Some("pt-1".to_string()),
        };
        let json = serde_json::to_string(&point).unwrap();
        let decoded: Primitive = serde_json::from_str(&json).unwrap();
        assert_eq!(point, decoded);
    }

    #[test]
    fn serialization_roundtrip_line() {
        let line = Primitive::Line {
            points: vec![[0.0, 0.0], [100.0, 50.0], [200.0, 0.0]],
            stroke: StrokeStyle {
                color: Color::from_rgba8(255, 0, 0, 255),
                width: 2.0,
                cap: LineCap::Round,
                join: LineJoin::Bevel,
            },
            closed: true,
            data_id: Some("line-2".to_string()),
        };
        let json = serde_json::to_string(&line).unwrap();
        let decoded: Primitive = serde_json::from_str(&json).unwrap();
        assert_eq!(line, decoded);
    }
}
