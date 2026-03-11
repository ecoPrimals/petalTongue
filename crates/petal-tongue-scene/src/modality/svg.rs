// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Write;

use bytes::Bytes;

use crate::primitive::{AnchorPoint, Color, FillRule, Primitive};
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

use super::{ModalityCompiler, ModalityOutput};

/// Compiles scene graph to SVG.
#[derive(Debug, Clone, Default)]
pub struct SvgCompiler;

impl SvgCompiler {
    /// Create a new SVG compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ModalityCompiler for SvgCompiler {
    fn name(&self) -> &'static str {
        "SvgCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let mut buf = String::new();
        buf.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600">"#);

        for (transform, prim) in scene.flatten() {
            Self::emit_primitive(&mut buf, prim, &transform);
        }

        buf.push_str("</svg>");
        ModalityOutput::Svg(Bytes::from(buf.into_bytes()))
    }
}

impl SvgCompiler {
    #[expect(
        clippy::too_many_lines,
        reason = "emit_primitive is a single match over primitive variants"
    )]
    fn emit_primitive(buf: &mut String, prim: &Primitive, transform: &Transform2D) {
        match prim {
            Primitive::Point {
                x,
                y,
                radius,
                fill,
                stroke,
                ..
            } => {
                let (x, y) = transform.apply(*x, *y);
                let fill_attr = fill
                    .map(Self::color_attr)
                    .unwrap_or_else(|| "none".to_string());
                let stroke_attr = stroke.as_ref().map_or_else(
                    || "stroke=\"none\"".to_string(),
                    |s| {
                        format!(
                            r#"stroke="{}" stroke-width="{}""#,
                            Self::color_attr(s.color),
                            s.width
                        )
                    },
                );
                let _ = write!(
                    buf,
                    r#"<circle cx="{x}" cy="{y}" r="{radius}" fill="{fill_attr}" {stroke_attr} />"#
                );
            }
            Primitive::Line { points, stroke, .. } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|&[px, py]| {
                        let (sx, sy) = transform.apply(px, py);
                        format!("{sx},{sy}")
                    })
                    .collect();
                let _ = write!(
                    buf,
                    r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="{}" />"#,
                    pts.join(" "),
                    Self::color_attr(stroke.color),
                    stroke.width
                );
            }
            Primitive::Rect {
                x,
                y,
                width,
                height,
                fill,
                stroke,
                corner_radius,
                ..
            } => {
                let (x, y) = transform.apply(*x, *y);
                let fill_attr = fill
                    .map(Self::color_attr)
                    .unwrap_or_else(|| "none".to_string());
                let stroke_attr = stroke.as_ref().map_or_else(
                    || "stroke=\"none\"".to_string(),
                    |s| {
                        format!(
                            r#"stroke="{}" stroke-width="{}""#,
                            Self::color_attr(s.color),
                            s.width
                        )
                    },
                );
                let _ = write!(
                    buf,
                    r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" rx="{corner_radius}" fill="{fill_attr}" {stroke_attr} />"#
                );
            }
            Primitive::Text {
                x,
                y,
                content,
                font_size,
                color,
                anchor,
                ..
            } => {
                let (sx, sy) = transform.apply(*x, *y);
                let anchor_str = match anchor {
                    AnchorPoint::TopLeft | AnchorPoint::CenterLeft | AnchorPoint::BottomLeft => {
                        "start"
                    }
                    AnchorPoint::TopCenter | AnchorPoint::Center | AnchorPoint::BottomCenter => {
                        "middle"
                    }
                    AnchorPoint::TopRight | AnchorPoint::CenterRight | AnchorPoint::BottomRight => {
                        "end"
                    }
                };
                let escaped = content
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                let _ = write!(
                    buf,
                    r#"<text x="{sx}" y="{sy}" font-size="{font_size}" fill="{}" text-anchor="{anchor_str}">{escaped}</text>"#,
                    Self::color_attr(*color)
                );
            }
            Primitive::Polygon {
                points,
                fill,
                stroke,
                fill_rule,
                ..
            } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|&[px, py]| {
                        let (sx, sy) = transform.apply(px, py);
                        format!("{sx},{sy}")
                    })
                    .collect();
                let fill_rule_str = match fill_rule {
                    FillRule::EvenOdd => "evenodd",
                    FillRule::NonZero => "nonzero",
                };
                let stroke_attr = stroke.as_ref().map_or_else(
                    || "stroke=\"none\"".to_string(),
                    |s| {
                        format!(
                            r#"stroke="{}" stroke-width="{}""#,
                            Self::color_attr(s.color),
                            s.width
                        )
                    },
                );
                let _ = write!(
                    buf,
                    r#"<polygon points="{}" fill="{}" fill-rule="{}" {stroke_attr} />"#,
                    pts.join(" "),
                    Self::color_attr(*fill),
                    fill_rule_str
                );
            }
            Primitive::Arc {
                cx,
                cy,
                radius,
                start_angle,
                end_angle,
                fill,
                stroke,
                ..
            } => {
                let (cx, cy) = transform.apply(*cx, *cy);
                let x1 = cx + radius * start_angle.cos();
                let y1 = cy + radius * start_angle.sin();
                let x2 = cx + radius * end_angle.cos();
                let y2 = cy + radius * end_angle.sin();
                let large = (end_angle - start_angle).abs() > std::f64::consts::PI;
                let d = format!(
                    "M {x1} {y1} A {radius} {radius} 0 {} 1 {x2} {y2}",
                    i32::from(large)
                );
                let fill_attr = fill
                    .map(Self::color_attr)
                    .unwrap_or_else(|| "none".to_string());
                let stroke_attr = stroke.as_ref().map_or_else(
                    || "stroke=\"none\"".to_string(),
                    |s| {
                        format!(
                            r#"stroke="{}" stroke-width="{}""#,
                            Self::color_attr(s.color),
                            s.width
                        )
                    },
                );
                let _ = write!(buf, r#"<path d="{d}" fill="{fill_attr}" {stroke_attr} />"#);
            }
            Primitive::BezierPath {
                start,
                segments,
                stroke,
                fill,
                fill_rule,
                ..
            } => {
                let (sx, sy) = transform.apply(start[0], start[1]);
                let mut d = format!("M {sx} {sy}");
                for seg in segments {
                    let (cp1x, cp1y) = transform.apply(seg.cp1[0], seg.cp1[1]);
                    let (cp2x, cp2y) = transform.apply(seg.cp2[0], seg.cp2[1]);
                    let (ex, ey) = transform.apply(seg.end[0], seg.end[1]);
                    let _ = write!(d, " C {cp1x} {cp1y}, {cp2x} {cp2y}, {ex} {ey}");
                }
                let fill_attr = fill
                    .map(Self::color_attr)
                    .unwrap_or_else(|| "none".to_string());
                let fill_rule_str = match fill_rule {
                    FillRule::EvenOdd => "evenodd",
                    FillRule::NonZero => "nonzero",
                };
                let _ = write!(
                    buf,
                    r#"<path d="{d}" fill="{fill_attr}" fill-rule="{fill_rule_str}" stroke="{}" stroke-width="{}" />"#,
                    Self::color_attr(stroke.color),
                    stroke.width
                );
            }
            Primitive::Mesh { .. } => {
                // 3D only; skip for 2D SVG
            }
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "color components clamped to [0,255] before cast"
    )]
    fn color_attr(c: Color) -> String {
        let r = (c.r * 255.0).clamp(0.0, 255.0);
        let g = (c.g * 255.0).clamp(0.0, 255.0);
        let b = (c.b * 255.0).clamp(0.0, 255.0);
        format!("rgb({},{},{})", r as u8, g as u8, b as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn svg_compiler_produces_valid_svg() {
        let compiler = SvgCompiler::new();
        let graph = SceneGraph::new();
        let out = compiler.compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<svg"));
        assert!(s.contains("</svg>"));
    }

    #[test]
    fn svg_compiler_handles_point_as_circle() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 10.0,
            y: 20.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<circle"));
    }

    #[test]
    fn svg_compiler_handles_line_as_polyline() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Line {
            points: vec![[10.0, 20.0], [50.0, 60.0], [90.0, 30.0]],
            stroke: crate::primitive::StrokeStyle::default(),
            closed: false,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("line").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<polyline"));
        assert!(s.contains("10,20"));
        assert!(s.contains("90,30"));
    }

    #[test]
    fn svg_compiler_handles_rect() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 5.0,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("rect").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<rect"));
        assert!(s.contains("width=\"100\""));
        assert!(s.contains("height=\"50\""));
        assert!(s.contains("rx=\"5\""));
    }

    #[test]
    fn svg_compiler_handles_polygon() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Polygon {
            points: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: crate::primitive::FillRule::NonZero,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("poly").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<polygon"));
        assert!(s.contains("fill-rule=\"nonzero\""));
    }

    #[test]
    fn svg_compiler_handles_arc() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Arc {
            cx: 100.0,
            cy: 100.0,
            radius: 50.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::PI,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("arc").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<path"));
        assert!(s.contains("d="));
    }

    #[test]
    fn svg_compiler_handles_bezier_path() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![crate::primitive::BezierSegment {
                cp1: [0.0, 50.0],
                cp2: [100.0, 50.0],
                end: [100.0, 0.0],
            }],
            stroke: crate::primitive::StrokeStyle::default(),
            fill: None,
            fill_rule: crate::primitive::FillRule::NonZero,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("bezier").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("<path"));
        assert!(s.contains("d="));
    }

    #[test]
    fn svg_compiler_escapes_text_content() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Text {
            x: 0.0,
            y: 0.0,
            content: "A & B < C > D".to_string(),
            font_size: 12.0,
            color: Color::BLACK,
            anchor: crate::primitive::AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("t").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("&amp;"));
        assert!(s.contains("&lt;"));
        assert!(s.contains("&gt;"));
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use crate::primitive::{Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};
    use proptest::prelude::*;

    proptest! {
        /// SVG compiler always produces valid XML wrapping: starts with <svg, ends with </svg>.
        #[test]
        fn prop_svg_valid_xml_wrapping(x in 0.0f64..800.0, y in 0.0f64..600.0, radius in 0.1f64..50.0) {
            let compiler = SvgCompiler::new();
            let mut graph = SceneGraph::new();
            let prim = Primitive::Point {
                x,
                y,
                radius,
                fill: Some(Color::BLACK),
                stroke: None,
                data_id: None,
            };
            graph.add_to_root(SceneNode::new("p").with_primitive(prim));
            let out = compiler.compile(&graph);
            let ModalityOutput::Svg(b) = &out else {
                panic!("expected Svg output");
            };
            let s = std::str::from_utf8(b.as_ref()).unwrap();
            prop_assert!(s.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
            prop_assert!(s.ends_with("</svg>"));
        }
    }
}
