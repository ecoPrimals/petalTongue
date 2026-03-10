// SPDX-License-Identifier: AGPL-3.0-only
//! Modality compilers: scene graph to output formats.
//!
//! Each compiler produces a different output modality: SVG, terminal cells,
//! audio parameters, GPU commands, or text descriptions for accessibility.

use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::primitive::{AnchorPoint, Color, FillRule, Primitive};
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

/// Output of a modality compiler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModalityOutput {
    /// SVG document string.
    Svg(String),
    /// Terminal character grid.
    TerminalCells(Vec<Vec<char>>),
    /// Audio synthesis parameters.
    AudioParams(Vec<AudioParam>),
    /// Raw GPU command bytes.
    GpuCommands(Vec<u8>),
    /// Text description for accessibility.
    Description(String),
}

/// Audio parameter for a single datum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParam {
    /// Frequency in Hz (e.g. 200–2000).
    pub frequency: f64,
    /// Amplitude 0.0 to 1.0.
    pub amplitude: f64,
    /// Pan -1.0 (left) to 1.0 (right).
    pub pan: f64,
    /// Duration in seconds.
    pub duration_secs: f64,
}

/// Trait for compiling a scene graph to a specific output modality.
pub trait ModalityCompiler: Send + Sync {
    /// Compile the scene graph to output.
    fn compile(&self, scene: &SceneGraph) -> ModalityOutput;

    /// Human-readable compiler name.
    fn name(&self) -> &'static str;
}

// -----------------------------------------------------------------------------
// SVG Compiler
// -----------------------------------------------------------------------------

/// Compiles scene graph to SVG.
#[derive(Debug, Clone, Default)]
pub struct SvgCompiler;

impl SvgCompiler {
    /// Create a new SVG compiler.
    #[must_use]
    pub fn new() -> Self {
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
        ModalityOutput::Svg(buf)
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
                    large as i32
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

// -----------------------------------------------------------------------------
// Audio Compiler
// -----------------------------------------------------------------------------

/// Compiles scene graph to audio parameters.
/// Maps data-carrying primitives: x→pan, y→frequency, size→amplitude.
#[derive(Debug, Clone, Default)]
pub struct AudioCompiler;

impl AudioCompiler {
    /// Create a new audio compiler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ModalityCompiler for AudioCompiler {
    fn name(&self) -> &'static str {
        "AudioCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let mut params = Vec::new();
        for (transform, prim) in scene.flatten() {
            if !prim.carries_data() {
                continue;
            }
            let (x, y, size) = match prim {
                Primitive::Point { x, y, radius, .. } => {
                    let (sx, sy) = transform.apply(*x, *y);
                    (sx, sy, *radius)
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let (sx, sy) = transform.apply(*x, *y);
                    let s = (width * height).sqrt() / 100.0;
                    (sx, sy, s)
                }
                Primitive::Line { points, .. } => {
                    if points.is_empty() {
                        continue;
                    }
                    let (sx, sy) = transform.apply(points[0][0], points[0][1]);
                    (sx, sy, 1.0)
                }
                _ => continue,
            };
            // Normalize to typical ranges: x→pan [-1,1], y→freq [200,2000], size→amp [0,1]
            let pan = (x / 400.0 - 0.5) * 2.0;
            let pan = pan.clamp(-1.0, 1.0);
            let freq = 200.0 + (y / 600.0) * 1800.0;
            let freq = freq.clamp(200.0, 2000.0);
            let amplitude = (size / 10.0).clamp(0.0, 1.0);
            params.push(AudioParam {
                frequency: freq,
                amplitude,
                pan,
                duration_secs: 0.1,
            });
        }
        ModalityOutput::AudioParams(params)
    }
}

// -----------------------------------------------------------------------------
// Description Compiler
// -----------------------------------------------------------------------------

/// Compiles scene graph to text description for accessibility.
#[derive(Debug, Clone, Default)]
pub struct DescriptionCompiler;

impl DescriptionCompiler {
    /// Create a new description compiler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ModalityCompiler for DescriptionCompiler {
    fn name(&self) -> &'static str {
        "DescriptionCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let node_count = scene.node_count();
        let prim_count = scene.total_primitives();
        let flat = scene.flatten();
        let mut type_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for (_, prim) in &flat {
            let name = match prim {
                Primitive::Point { .. } => "Point",
                Primitive::Line { .. } => "Line",
                Primitive::Rect { .. } => "Rect",
                Primitive::Text { .. } => "Text",
                Primitive::Polygon { .. } => "Polygon",
                Primitive::Arc { .. } => "Arc",
                Primitive::BezierPath { .. } => "BezierPath",
                Primitive::Mesh { .. } => "Mesh",
            };
            *type_counts.entry(name).or_insert(0) += 1;
        }
        let type_desc: Vec<String> = type_counts
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect();
        let labels: Vec<&str> = flat
            .iter()
            .filter_map(|(_, p)| {
                if let Primitive::Text { content, .. } = p {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();
        let mut desc = format!(
            "Scene with {} nodes and {} primitives. Primitive types: {}.",
            node_count,
            prim_count,
            type_desc.join(", ")
        );
        if !labels.is_empty() {
            let _ = write!(desc, " Labels: {}.", labels.join(", "));
        }
        ModalityOutput::Description(desc)
    }
}

// -----------------------------------------------------------------------------
// Terminal Compiler (ratatui character grid)
// -----------------------------------------------------------------------------

/// Compiles scene graph to a terminal character grid (for ratatui rendering).
///
/// Maps primitives to a 2D character array: points become markers, lines become
/// line-drawing characters, rectangles become boxes, and text is placed directly.
#[derive(Debug, Clone, Default)]
pub struct TerminalCompiler {
    width: usize,
    height: usize,
}

impl TerminalCompiler {
    /// Create a terminal compiler with the given dimensions (columns x rows).
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    fn plot_point(grid: &mut [Vec<char>], col: usize, row: usize, ch: char) {
        if row < grid.len() && col < grid[0].len() {
            grid[row][col] = ch;
        }
    }

    fn plot_line(grid: &mut [Vec<char>], points: &[[f64; 2]], w: usize, h: usize) {
        for pair in points.windows(2) {
            let (x0, y0) = Self::to_cell(pair[0][0], pair[0][1], w, h);
            let (x1, y1) = Self::to_cell(pair[1][0], pair[1][1], w, h);
            Self::bresenham(grid, x0, y0, x1, y1);
        }
    }

    fn to_cell(x: f64, y: f64, w: usize, h: usize) -> (usize, usize) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to [0, w-1] and [0, h-1] before cast"
        )]
        let col = (x / 800.0 * w as f64).clamp(0.0, (w - 1) as f64) as usize;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to [0, w-1] and [0, h-1] before cast"
        )]
        let row = (y / 600.0 * h as f64).clamp(0.0, (h - 1) as f64) as usize;
        (col, row)
    }

    #[allow(clippy::cast_possible_wrap)]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "y,x are bounds-checked in loop condition before indexing"
    )]
    fn bresenham(grid: &mut [Vec<char>], x0: usize, y0: usize, x1: usize, y1: usize) {
        let (mut x, mut y) = (x0 as i64, y0 as i64);
        let x1i = x1 as i64;
        let y1i = y1 as i64;
        let (dx, dy) = ((x1i - x).abs(), (y1i - y).abs());
        let sx: i64 = if x0 < x1 { 1 } else { -1 };
        let sy: i64 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let rows = grid.len() as i64;
        let cols = if grid.is_empty() {
            0
        } else {
            grid[0].len() as i64
        };

        loop {
            if y >= 0 && y < rows && x >= 0 && x < cols {
                let ch = if dx > dy * 2 {
                    '─'
                } else if dy > dx * 2 {
                    '│'
                } else {
                    '·'
                };
                grid[y as usize][x as usize] = ch;
            }
            if x == x1i && y == y1i {
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
}

impl ModalityCompiler for TerminalCompiler {
    fn name(&self) -> &'static str {
        "TerminalCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let w = if self.width > 0 { self.width } else { 80 };
        let h = if self.height > 0 { self.height } else { 24 };
        let mut grid = vec![vec![' '; w]; h];

        for (_transform, prim) in scene.flatten() {
            match prim {
                Primitive::Point { x, y, .. } => {
                    let (col, row) = Self::to_cell(*x, *y, w, h);
                    Self::plot_point(&mut grid, col, row, '●');
                }
                Primitive::Line { points, .. } => {
                    Self::plot_line(&mut grid, points, w, h);
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let (c0, r0) = Self::to_cell(*x, *y, w, h);
                    let (c1, r1) = Self::to_cell(x + width, y + height, w, h);
                    for col in c0..=c1.min(w - 1) {
                        Self::plot_point(&mut grid, col, r0, '─');
                        Self::plot_point(&mut grid, col, r1.min(h - 1), '─');
                    }
                    for row in r0..=r1.min(h - 1) {
                        Self::plot_point(&mut grid, c0, row, '│');
                        Self::plot_point(&mut grid, c1.min(w - 1), row, '│');
                    }
                }
                Primitive::Text { x, y, content, .. } => {
                    let (col, row) = Self::to_cell(*x, *y, w, h);
                    for (i, ch) in content.chars().enumerate() {
                        if col + i < w && row < h {
                            grid[row][col + i] = ch;
                        }
                    }
                }
                _ => {}
            }
        }

        ModalityOutput::TerminalCells(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::Primitive;
    use crate::scene_graph::SceneGraph;

    #[test]
    fn svg_compiler_produces_valid_svg() {
        let compiler = SvgCompiler::new();
        let graph = SceneGraph::new();
        let out = compiler.compile(&graph);
        let ModalityOutput::Svg(s) = &out else {
            panic!("expected Svg");
        };
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
        graph.add_to_root(crate::scene_graph::SceneNode::new("p").with_primitive(prim));
        let out = SvgCompiler::new().compile(&graph);
        let ModalityOutput::Svg(s) = &out else {
            panic!("expected Svg");
        };
        assert!(s.contains("<circle"));
    }

    #[test]
    fn audio_compiler_produces_params_from_points() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 100.0,
            y: 300.0,
            radius: 5.0,
            fill: None,
            stroke: None,
            data_id: Some("d1".to_string()),
        };
        graph.add_to_root(crate::scene_graph::SceneNode::new("p").with_primitive(prim));
        let out = AudioCompiler::new().compile(&graph);
        let ModalityOutput::AudioParams(params) = &out else {
            panic!("expected AudioParams");
        };
        assert_eq!(params.len(), 1);
        assert!(params[0].frequency >= 200.0 && params[0].frequency <= 2000.0);
        assert!(params[0].amplitude >= 0.0 && params[0].amplitude <= 1.0);
    }

    #[test]
    fn terminal_compiler_plots_points() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 400.0,
            y: 300.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        graph.add_to_root(crate::scene_graph::SceneNode::new("p").with_primitive(prim));
        let out = TerminalCompiler::new(80, 24).compile(&graph);
        let ModalityOutput::TerminalCells(grid) = &out else {
            panic!("expected TerminalCells");
        };
        assert_eq!(grid.len(), 24);
        assert_eq!(grid[0].len(), 80);
        let has_marker = grid.iter().any(|row| row.contains(&'●'));
        assert!(has_marker, "Grid should contain a point marker");
    }

    #[test]
    fn terminal_compiler_renders_text() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Text {
            x: 0.0,
            y: 0.0,
            content: "Hi".to_string(),
            font_size: 12.0,
            color: Color::WHITE,
            anchor: crate::primitive::AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        };
        graph.add_to_root(crate::scene_graph::SceneNode::new("t").with_primitive(prim));
        let out = TerminalCompiler::new(40, 10).compile(&graph);
        let ModalityOutput::TerminalCells(grid) = &out else {
            panic!("expected TerminalCells");
        };
        assert_eq!(grid[0][0], 'H');
        assert_eq!(grid[0][1], 'i');
    }

    #[test]
    fn description_compiler_describes_node_count() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(crate::scene_graph::SceneNode::new("a"));
        graph.add_to_root(crate::scene_graph::SceneNode::new("b"));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(s) = &out else {
            panic!("expected Description");
        };
        assert!(s.contains("3 nodes")); // root + a + b
    }
}
