// SPDX-License-Identifier: AGPL-3.0-only
//! Axis types: NumberLine (1D) and Axes (2D Cartesian).

use serde::{Deserialize, Serialize};

use crate::math::MathObject;
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};

/// A number line: axis with tick marks and optional labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberLine {
    /// Start value (data space).
    pub start: f64,
    /// End value (data space).
    pub end: f64,
    /// Step between ticks.
    pub step: f64,
    /// Origin x in screen coordinates.
    pub origin_x: f64,
    /// Origin y in screen coordinates.
    pub origin_y: f64,
    /// Pixel length of the axis.
    pub length: f64,
    /// Color for axis and ticks.
    pub color: Color,
    /// Whether to show numeric labels.
    pub show_labels: bool,
    /// Font size for labels.
    pub label_font_size: f64,
}

impl Default for NumberLine {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 10.0,
            step: 1.0,
            origin_x: 0.0,
            origin_y: 0.0,
            length: 400.0,
            color: Color::BLACK,
            show_labels: true,
            label_font_size: 12.0,
        }
    }
}

impl NumberLine {
    /// Map a data value to screen x (horizontal number line).
    fn data_to_screen_x(&self, value: f64) -> f64 {
        let t = (value - self.start) / (self.end - self.start);
        self.origin_x + t * self.length
    }
}

impl MathObject for NumberLine {
    fn to_primitives(&self) -> Vec<Primitive> {
        let mut prims = Vec::new();
        let stroke = StrokeStyle {
            color: self.color,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        // Axis line (horizontal)
        let x0 = self.origin_x;
        let x1 = self.origin_x + self.length;
        prims.push(Primitive::Line {
            points: vec![[x0, self.origin_y], [x1, self.origin_y]],
            stroke,
            closed: false,
            data_id: None,
        });

        // Tick marks
        let mut v = self.start;
        while v <= self.end {
            let sx = self.data_to_screen_x(v);
            let tick_len = 5.0;
            prims.push(Primitive::Line {
                points: vec![[sx, self.origin_y], [sx, self.origin_y - tick_len]],
                stroke,
                closed: false,
                data_id: None,
            });
            v += self.step;
        }

        // Labels
        if self.show_labels {
            let mut v = self.start;
            while v <= self.end {
                let sx = self.data_to_screen_x(v);
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "axis labels are integer tick values"
                )]
                let label = if (v - v.round()).abs() < 1e-10 {
                    format!("{}", v.round() as i64)
                } else {
                    format!("{v:.2}")
                };
                prims.push(Primitive::Text {
                    x: sx,
                    y: self.origin_y - 8.0,
                    content: label,
                    font_size: self.label_font_size,
                    color: self.color,
                    anchor: AnchorPoint::TopCenter,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
                v += self.step;
            }
        }

        prims
    }
}

/// 2D axes with x and y number lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axes {
    /// X range: (min, max, step).
    pub x_range: (f64, f64, f64),
    /// Y range: (min, max, step).
    pub y_range: (f64, f64, f64),
    /// Origin in screen coordinates.
    pub origin: (f64, f64),
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
    /// Color for axes.
    pub color: Color,
    /// Whether to show labels.
    pub show_labels: bool,
    /// Font size for labels.
    pub label_font_size: f64,
}

impl Default for Axes {
    fn default() -> Self {
        Self {
            x_range: (-10.0, 10.0, 2.0),
            y_range: (-10.0, 10.0, 2.0),
            origin: (200.0, 200.0),
            width: 400.0,
            height: 400.0,
            color: Color::BLACK,
            show_labels: true,
            label_font_size: 12.0,
        }
    }
}

impl Axes {
    /// Map data coordinates (x, y) to screen coordinates.
    pub fn data_to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let (x_min, x_max, _) = self.x_range;
        let (y_min, y_max, _) = self.y_range;
        let tx = (x - x_min) / (x_max - x_min);
        let ty = (y - y_min) / (y_max - y_min);
        let sx = self.origin.0 + tx * self.width;
        let sy = self.origin.1 - ty * self.height; // y flipped (screen y down)
        (sx, sy)
    }

    /// Map screen coordinates to data coordinates.
    pub fn screen_to_data(&self, sx: f64, sy: f64) -> (f64, f64) {
        let (x_min, x_max, _) = self.x_range;
        let (y_min, y_max, _) = self.y_range;
        let tx = (sx - self.origin.0) / self.width;
        let ty = (sy - self.origin.1) / -self.height;
        let x = x_min + tx * (x_max - x_min);
        let y = y_min + ty * (y_max - y_min);
        (x, y)
    }
}

impl MathObject for Axes {
    #[expect(
        clippy::too_many_lines,
        reason = "axis rendering is a cohesive sequence: x-axis, y-axis, ticks, labels, gridlines"
    )]
    fn to_primitives(&self) -> Vec<Primitive> {
        let mut prims = Vec::new();
        let stroke = StrokeStyle {
            color: self.color,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let (x_min, x_max, x_step) = self.x_range;
        let (y_min, y_max, y_step) = self.y_range;
        let (ox, oy) = self.origin;

        // X axis (horizontal)
        prims.push(Primitive::Line {
            points: vec![[ox, oy], [ox + self.width, oy]],
            stroke,
            closed: false,
            data_id: None,
        });
        // X ticks
        let mut v = x_min;
        while v <= x_max {
            let (sx, _) = self.data_to_screen(v, 0.0);
            prims.push(Primitive::Line {
                points: vec![[sx, oy], [sx, oy + 5.0]],
                stroke,
                closed: false,
                data_id: None,
            });
            if self.show_labels {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "axis labels are integer tick values"
                )]
                prims.push(Primitive::Text {
                    x: sx,
                    y: oy + 8.0,
                    content: format!("{}", v.round() as i64),
                    font_size: self.label_font_size,
                    color: self.color,
                    anchor: AnchorPoint::TopCenter,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
            }
            v += x_step;
        }
        // X arrow head
        let arrow_size = 8.0;
        let (sx_end, _) = self.data_to_screen(x_max, 0.0);
        prims.push(Primitive::Line {
            points: vec![[sx_end, oy], [sx_end - arrow_size, oy - arrow_size * 0.5]],
            stroke,
            closed: false,
            data_id: None,
        });
        prims.push(Primitive::Line {
            points: vec![[sx_end, oy], [sx_end - arrow_size, oy + arrow_size * 0.5]],
            stroke,
            closed: false,
            data_id: None,
        });

        // Y axis (vertical)
        prims.push(Primitive::Line {
            points: vec![[ox, oy], [ox, oy - self.height]],
            stroke,
            closed: false,
            data_id: None,
        });
        // Y ticks
        let mut v = y_min;
        while v <= y_max {
            let (_, sy) = self.data_to_screen(0.0, v);
            prims.push(Primitive::Line {
                points: vec![[ox, sy], [ox - 5.0, sy]],
                stroke,
                closed: false,
                data_id: None,
            });
            if self.show_labels {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "axis labels are integer tick values"
                )]
                prims.push(Primitive::Text {
                    x: ox - 8.0,
                    y: sy,
                    content: format!("{}", v.round() as i64),
                    font_size: self.label_font_size,
                    color: self.color,
                    anchor: AnchorPoint::CenterRight,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
            }
            v += y_step;
        }
        // Y arrow head
        let (_, sy_end) = self.data_to_screen(0.0, y_max);
        prims.push(Primitive::Line {
            points: vec![[ox, sy_end], [ox - arrow_size * 0.5, sy_end + arrow_size]],
            stroke,
            closed: false,
            data_id: None,
        });
        prims.push(Primitive::Line {
            points: vec![[ox, sy_end], [ox + arrow_size * 0.5, sy_end + arrow_size]],
            stroke,
            closed: false,
            data_id: None,
        });

        prims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    #[test]
    fn numberline_default_values() {
        let nl = NumberLine::default();
        assert!((nl.start - 0.0).abs() < EPS);
        assert!((nl.end - 10.0).abs() < EPS);
        assert!((nl.step - 1.0).abs() < EPS);
        assert!((nl.length - 400.0).abs() < EPS);
        assert!(nl.show_labels);
    }

    #[test]
    fn numberline_to_primitives_produces_axis_line() {
        let nl = NumberLine::default();
        let prims = nl.to_primitives();
        assert!(
            !prims.is_empty(),
            "NumberLine should produce at least axis line"
        );
        let line_count = prims
            .iter()
            .filter(|p| matches!(p, Primitive::Line { .. }))
            .count();
        assert!(line_count >= 11, "0..10 with step 1 gives 11 ticks + axis");
    }

    #[test]
    fn numberline_to_primitives_with_labels() {
        let nl = NumberLine {
            start: 0.0,
            end: 5.0,
            step: 1.0,
            show_labels: true,
            ..NumberLine::default()
        };
        let prims = nl.to_primitives();
        let text_count = prims
            .iter()
            .filter(|p| matches!(p, Primitive::Text { .. }))
            .count();
        assert!(text_count >= 5, "should have labels at 0,1,2,3,4,5");
    }

    #[test]
    fn axes_default_values() {
        let axes = Axes::default();
        assert!((axes.x_range.0 - (-10.0)).abs() < EPS);
        assert!((axes.x_range.1 - 10.0).abs() < EPS);
        assert!((axes.y_range.0 - (-10.0)).abs() < EPS);
        assert!((axes.origin.0 - 200.0).abs() < EPS);
        assert!(axes.show_labels);
    }

    #[test]
    fn axes_data_to_screen_origin() {
        let axes = Axes {
            x_range: (0.0, 10.0, 1.0),
            y_range: (0.0, 10.0, 1.0),
            origin: (100.0, 200.0),
            width: 100.0,
            height: 100.0,
            ..Axes::default()
        };
        let (sx, sy) = axes.data_to_screen(0.0, 0.0);
        assert!((sx - 100.0).abs() < EPS);
        assert!((sy - 200.0).abs() < EPS);
    }

    #[test]
    fn axes_data_to_screen_max() {
        let axes = Axes {
            x_range: (0.0, 10.0, 1.0),
            y_range: (0.0, 10.0, 1.0),
            origin: (0.0, 100.0),
            width: 100.0,
            height: 100.0,
            ..Axes::default()
        };
        let (sx, sy) = axes.data_to_screen(10.0, 10.0);
        assert!((sx - 100.0).abs() < EPS);
        assert!((sy - 0.0).abs() < EPS, "y flipped: max y at top");
    }

    #[test]
    fn axes_screen_to_data_roundtrip() {
        let axes = Axes::default();
        let (x, y) = (-3.5, 7.2);
        let (sx, sy) = axes.data_to_screen(x, y);
        let (x2, y2) = axes.screen_to_data(sx, sy);
        assert!((x - x2).abs() < EPS);
        assert!((y - y2).abs() < EPS);
    }

    #[test]
    fn axes_to_primitives_has_x_and_y_axes() {
        let axes = Axes::default();
        let prims = axes.to_primitives();
        assert!(!prims.is_empty());
        let has_x = prims.iter().any(|p| {
            if let Primitive::Line { points, .. } = p {
                points.len() >= 2 && (points[0][1] - points[1][1]).abs() < EPS
            } else {
                false
            }
        });
        assert!(has_x, "should have horizontal x-axis");
    }

    #[test]
    fn axes_show_labels_false_no_text() {
        let axes = Axes {
            show_labels: false,
            ..Axes::default()
        };
        let prims = axes.to_primitives();
        let text_count = prims
            .iter()
            .filter(|p| matches!(p, Primitive::Text { .. }))
            .count();
        assert_eq!(text_count, 0);
    }

    #[test]
    fn axes_custom_label_font_size() {
        let axes = Axes {
            label_font_size: 16.0,
            ..Axes::default()
        };
        let prims = axes.to_primitives();
        let text_prims: Vec<_> = prims
            .iter()
            .filter_map(|p| {
                if let Primitive::Text { font_size, .. } = p {
                    Some(*font_size)
                } else {
                    None
                }
            })
            .collect();
        assert!(!text_prims.is_empty());
        assert!(text_prims.iter().all(|&s| (s - 16.0).abs() < EPS));
    }

    #[test]
    fn numberline_decimal_labels() {
        let nl = NumberLine {
            start: 0.0,
            end: 1.0,
            step: 0.25,
            show_labels: true,
            ..NumberLine::default()
        };
        let prims = nl.to_primitives();
        let texts: Vec<_> = prims
            .iter()
            .filter_map(|p| {
                if let Primitive::Text { content, .. } = p {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(!texts.is_empty(), "should have decimal labels like 0.25");
    }
}
