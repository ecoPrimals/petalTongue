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
        let json = serde_json::to_string(&point).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
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
        let json = serde_json::to_string(&line).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(line, decoded);
    }

    #[test]
    fn color_constants() {
        assert!((Color::BLACK.r - 0.0).abs() < f32::EPSILON);
        assert!((Color::BLACK.a - 1.0).abs() < f32::EPSILON);
        assert!((Color::WHITE.r - 1.0).abs() < f32::EPSILON);
        assert!((Color::WHITE.g - 1.0).abs() < f32::EPSILON);
        assert!((Color::TRANSPARENT.a - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn line_cap_variants() {
        assert_eq!(LineCap::Butt, LineCap::Butt);
        assert_ne!(LineCap::Butt, LineCap::Round);
        assert_ne!(LineCap::Round, LineCap::Square);
    }

    #[test]
    fn line_join_variants() {
        assert_eq!(LineJoin::Miter, LineJoin::Miter);
        assert_ne!(LineJoin::Miter, LineJoin::Round);
        assert_ne!(LineJoin::Round, LineJoin::Bevel);
    }

    #[test]
    fn fill_rule_variants() {
        assert_eq!(FillRule::EvenOdd, FillRule::EvenOdd);
        assert_ne!(FillRule::EvenOdd, FillRule::NonZero);
    }

    #[test]
    fn anchor_point_variants() {
        assert_eq!(AnchorPoint::TopLeft, AnchorPoint::TopLeft);
        assert_eq!(AnchorPoint::Center, AnchorPoint::Center);
        assert_ne!(AnchorPoint::TopLeft, AnchorPoint::BottomRight);
    }

    #[test]
    fn primitive_rect_data_id() {
        let rect = Primitive::Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 0.0,
            data_id: Some("rect-1".to_string()),
        };
        assert_eq!(rect.data_id(), Some("rect-1"));
        assert!(rect.carries_data());
    }

    #[test]
    fn primitive_text_data_id() {
        let text = Primitive::Text {
            x: 0.0,
            y: 0.0,
            content: "Hello".to_string(),
            font_size: 12.0,
            color: Color::BLACK,
            anchor: AnchorPoint::Center,
            bold: false,
            italic: false,
            data_id: Some("text-1".to_string()),
        };
        assert_eq!(text.data_id(), Some("text-1"));
        assert!(text.carries_data());
    }

    #[test]
    fn primitive_polygon_data_id() {
        let poly = Primitive::Polygon {
            points: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("poly-1".to_string()),
        };
        assert_eq!(poly.data_id(), Some("poly-1"));
        assert!(poly.carries_data());
    }

    #[test]
    fn primitive_arc_data_id() {
        let arc = Primitive::Arc {
            cx: 0.0,
            cy: 0.0,
            radius: 10.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::PI,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("arc-1".to_string()),
        };
        assert_eq!(arc.data_id(), Some("arc-1"));
        assert!(arc.carries_data());
    }

    #[test]
    fn primitive_bezier_path_data_id() {
        let path = Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![],
            stroke: StrokeStyle::default(),
            fill: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("path-1".to_string()),
        };
        assert_eq!(path.data_id(), Some("path-1"));
        assert!(path.carries_data());
    }

    #[test]
    fn primitive_mesh_data_id() {
        let mesh = Primitive::Mesh {
            vertices: vec![],
            indices: vec![],
            data_id: Some("mesh-1".to_string()),
        };
        assert_eq!(mesh.data_id(), Some("mesh-1"));
        assert!(mesh.carries_data());
    }

    #[test]
    fn primitive_rect_no_data_id() {
        let rect = Primitive::Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            fill: None,
            stroke: None,
            corner_radius: 4.0,
            data_id: None,
        };
        assert_eq!(rect.data_id(), None);
        assert!(!rect.carries_data());
    }

    #[test]
    fn bezier_segment_construction() {
        let seg = BezierSegment {
            cp1: [1.0, 2.0],
            cp2: [3.0, 4.0],
            end: [5.0, 6.0],
        };
        assert!((seg.cp1[0] - 1.0).abs() < 1e-10);
        assert!((seg.end[1] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn mesh_vertex_construction() {
        let v = MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            color: Color::WHITE,
        };
        assert!((v.position[2] - 3.0).abs() < 1e-10);
        assert!((v.normal[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn serialization_roundtrip_rect() {
        let rect = Primitive::Rect {
            x: 5.0,
            y: 10.0,
            width: 50.0,
            height: 25.0,
            fill: Some(Color::rgba(0.5, 0.5, 0.5, 0.5)),
            stroke: Some(StrokeStyle::default()),
            corner_radius: 8.0,
            data_id: None,
        };
        let json = serde_json::to_string(&rect).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(rect, decoded);
    }

    #[test]
    fn serialization_roundtrip_arc() {
        let arc = Primitive::Arc {
            cx: 100.0,
            cy: 100.0,
            radius: 50.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::FRAC_PI_2,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        let json = serde_json::to_string(&arc).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(arc, decoded);
    }

    #[test]
    fn serialization_roundtrip_text() {
        let text = Primitive::Text {
            x: 10.0,
            y: 20.0,
            content: "Hello World".to_string(),
            font_size: 14.0,
            color: Color::BLACK,
            anchor: AnchorPoint::Center,
            bold: true,
            italic: false,
            data_id: Some("text-id".to_string()),
        };
        let json = serde_json::to_string(&text).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(text, decoded);
    }

    #[test]
    fn serialization_roundtrip_polygon() {
        let poly = Primitive::Polygon {
            points: vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]],
            fill: Color::rgba(0.5, 0.5, 0.5, 0.8),
            stroke: Some(StrokeStyle::default()),
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        let json = serde_json::to_string(&poly).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(poly, decoded);
    }

    #[test]
    fn serialization_roundtrip_bezier_path() {
        let path = Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![BezierSegment {
                cp1: [1.0, 2.0],
                cp2: [3.0, 4.0],
                end: [5.0, 6.0],
            }],
            stroke: StrokeStyle::default(),
            fill: Some(Color::WHITE),
            fill_rule: FillRule::EvenOdd,
            data_id: Some("path-id".to_string()),
        };
        let json = serde_json::to_string(&path).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(path, decoded);
    }

    #[test]
    fn serialization_roundtrip_mesh() {
        let mesh = Primitive::Mesh {
            vertices: vec![MeshVertex {
                position: [1.0, 2.0, 3.0],
                normal: [0.0, 1.0, 0.0],
                color: Color::WHITE,
            }],
            indices: vec![0, 1, 2],
            data_id: Some("mesh-id".to_string()),
        };
        let json = serde_json::to_string(&mesh).expect("serialization should succeed");
        let decoded: Primitive =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(mesh, decoded);
    }

    #[test]
    fn color_serialization_roundtrip() {
        let c = Color::rgba(0.25, 0.5, 0.75, 0.9);
        let json = serde_json::to_string(&c).expect("serialization should succeed");
        let decoded: Color = serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((c.r - decoded.r).abs() < EPS);
        assert!((c.g - decoded.g).abs() < EPS);
        assert!((c.b - decoded.b).abs() < EPS);
        assert!((c.a - decoded.a).abs() < EPS);
    }

    #[test]
    fn stroke_style_serialization_roundtrip() {
        let s = StrokeStyle {
            color: Color::from_rgba8(255, 0, 0, 255),
            width: 2.5,
            cap: LineCap::Round,
            join: LineJoin::Bevel,
        };
        let json = serde_json::to_string(&s).expect("serialization should succeed");
        let decoded: StrokeStyle =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(s.color.r, decoded.color.r);
        assert!((s.width - decoded.width).abs() < EPS);
        assert_eq!(s.cap, decoded.cap);
        assert_eq!(s.join, decoded.join);
    }

    #[test]
    fn line_cap_serialization_roundtrip() {
        for cap in [LineCap::Butt, LineCap::Round, LineCap::Square] {
            let json = serde_json::to_string(&cap).expect("serialization should succeed");
            let decoded: LineCap =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(cap, decoded);
        }
    }

    #[test]
    fn line_join_serialization_roundtrip() {
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            let json = serde_json::to_string(&join).expect("serialization should succeed");
            let decoded: LineJoin =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(join, decoded);
        }
    }

    #[test]
    fn fill_rule_serialization_roundtrip() {
        for rule in [FillRule::EvenOdd, FillRule::NonZero] {
            let json = serde_json::to_string(&rule).expect("serialization should succeed");
            let decoded: FillRule =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(rule, decoded);
        }
    }

    #[test]
    fn anchor_point_serialization_roundtrip() {
        let anchors = [
            AnchorPoint::TopLeft,
            AnchorPoint::TopCenter,
            AnchorPoint::TopRight,
            AnchorPoint::CenterLeft,
            AnchorPoint::Center,
            AnchorPoint::CenterRight,
            AnchorPoint::BottomLeft,
            AnchorPoint::BottomCenter,
            AnchorPoint::BottomRight,
        ];
        for anchor in anchors {
            let json = serde_json::to_string(&anchor).expect("serialization should succeed");
            let decoded: AnchorPoint =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(anchor, decoded);
        }
    }

    #[test]
    fn bezier_segment_serialization_roundtrip() {
        let seg = BezierSegment {
            cp1: [1.0, 2.0],
            cp2: [3.0, 4.0],
            end: [5.0, 6.0],
        };
        let json = serde_json::to_string(&seg).expect("serialization should succeed");
        let decoded: BezierSegment =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((seg.cp1[0] - decoded.cp1[0]).abs() < 1e-10);
        assert!((seg.end[1] - decoded.end[1]).abs() < 1e-10);
    }

    #[test]
    fn mesh_vertex_serialization_roundtrip() {
        let v = MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            color: Color::WHITE,
        };
        let json = serde_json::to_string(&v).expect("serialization should succeed");
        let decoded: MeshVertex =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((v.position[2] - decoded.position[2]).abs() < 1e-10);
    }

    #[test]
    fn color_debug_formatting() {
        let c = Color::rgb(1.0, 0.0, 0.0);
        let s = format!("{c:?}");
        assert!(!s.is_empty());
    }

    #[test]
    fn primitive_debug_formatting() {
        let p = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        let s = format!("{p:?}");
        assert!(s.contains("Point"));
    }

    #[test]
    fn primitive_line_closed() {
        let line = Primitive::Line {
            points: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            stroke: StrokeStyle::default(),
            closed: true,
            data_id: None,
        };
        assert!(!line.carries_data());
        assert_eq!(line.data_id(), None);
    }

    #[test]
    fn primitive_bezier_path_empty_segments() {
        let path = Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![],
            stroke: StrokeStyle::default(),
            fill: None,
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        assert_eq!(path.data_id(), None);
        assert!(!path.carries_data());
    }

    #[test]
    fn primitive_mesh_empty_vertices() {
        let mesh = Primitive::Mesh {
            vertices: vec![],
            indices: vec![],
            data_id: None,
        };
        assert_eq!(mesh.data_id(), None);
    }

    #[test]
    fn stroke_style_clone() {
        let s = StrokeStyle::default();
        let cloned = s;
        assert_eq!(s.color.r, cloned.color.r);
    }

    #[test]
    fn color_clone() {
        let c = Color::rgb(0.5, 0.5, 0.5);
        let cloned = c;
        assert_eq!(c, cloned);
    }

    #[test]
    fn color_partial_eq() {
        let a = Color::rgb(1.0, 0.0, 0.0);
        let b = Color::rgb(1.0, 0.0, 0.0);
        let c = Color::rgb(0.0, 1.0, 0.0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
