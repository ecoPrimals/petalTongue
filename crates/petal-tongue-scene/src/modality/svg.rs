// SPDX-License-Identifier: AGPL-3.0-or-later

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
                    let (c1x, c1y) = transform.apply(seg.cp1[0], seg.cp1[1]);
                    let (c2x, c2y) = transform.apply(seg.cp2[0], seg.cp2[1]);
                    let (ex, ey) = transform.apply(seg.end[0], seg.end[1]);
                    let _ = write!(d, " C {c1x} {c1y}, {c2x} {c2y}, {ex} {ey}");
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
#[path = "svg_tests.rs"]
mod tests;
