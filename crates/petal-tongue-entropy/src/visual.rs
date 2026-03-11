// SPDX-License-Identifier: AGPL-3.0-only
//! Visual entropy capture (drawing, painting)
//!
//! Captures stroke patterns, spatial coverage, and timing for visual creativity.

use crate::quality::{create_histogram_buckets, shannon_entropy, variance, weighted_quality};
use crate::types::{Color, Point2D, Stroke, VisualEntropyData, VisualQualityMetrics};
use std::time::Duration;

/// Compute Shannon entropy from color channel histograms (R, G, B buckets).
///
/// Histograms each channel into 8 buckets and computes entropy per channel,
/// then returns the average. Empty input returns 0.0.
///
/// # Arguments
/// * `colors` - Slice of RGBA colors
///
/// # Returns
/// Average Shannon entropy [0.0-1.0] across R, G, B channels, or 0.0 for empty input
#[must_use]
pub fn compute_color_entropy(colors: &[Color]) -> f64 {
    const NUM_BUCKETS: usize = 8;

    if colors.is_empty() {
        return 0.0;
    }

    let r_vals: Vec<f64> = colors.iter().map(|c| f64::from(c.r)).collect();
    let g_vals: Vec<f64> = colors.iter().map(|c| f64::from(c.g)).collect();
    let b_vals: Vec<f64> = colors.iter().map(|c| f64::from(c.b)).collect();
    let r_buckets = create_histogram_buckets(&r_vals, NUM_BUCKETS);
    let g_buckets = create_histogram_buckets(&g_vals, NUM_BUCKETS);
    let b_buckets = create_histogram_buckets(&b_vals, NUM_BUCKETS);

    let r_entropy = shannon_entropy(&r_buckets);
    let g_entropy = shannon_entropy(&g_buckets);
    let b_entropy = shannon_entropy(&b_buckets);

    (r_entropy + g_entropy + b_entropy) / 3.0
}

/// Compute luminance (perceived brightness) for a color.
fn luminance(c: Color) -> f64 {
    0.299 * f64::from(c.r) + 0.587 * f64::from(c.g) + 0.114 * f64::from(c.b)
}

/// Quantify visual complexity from color diversity, brightness variance,
/// and spatial frequency estimate.
///
/// # Arguments
/// * `strokes` - Drawing strokes (points, colors, timestamps)
/// * `canvas_size` - (width, height) of the canvas
///
/// # Returns
/// Complexity score [0.0-1.0], or 0.0 for empty strokes
pub fn visual_complexity(strokes: &[Stroke], canvas_size: (u32, u32)) -> f64 {
    if strokes.is_empty() {
        return 0.0;
    }

    let (cw, ch) = (f64::from(canvas_size.0), f64::from(canvas_size.1));
    let canvas_area = (cw * ch).max(1.0);

    // 1. Color diversity: Shannon entropy of color distribution
    let colors: Vec<Color> = strokes.iter().map(|s| s.color).collect();
    let color_component = compute_color_entropy(&colors);

    // 2. Brightness variance: variance of luminance across strokes
    let luminances: Vec<f64> = colors.iter().copied().map(luminance).collect();
    let brightness_component = if luminances.len() >= 2 {
        variance(&luminances)
    } else {
        0.5
    };

    // 3. Spatial frequency estimate: points per unit area (density)
    let total_points: usize = strokes.iter().map(|s| s.points.len()).sum();
    let point_density = total_points as f64 / canvas_area;
    let spatial_component = (point_density / 0.01).min(1.0);

    (color_component * 0.4 + brightness_component * 0.3 + spatial_component * 0.3).clamp(0.0, 1.0)
}

/// Visual entropy capturer
pub struct VisualEntropyCapture {
    strokes: Vec<Stroke>,
    canvas_size: (u32, u32),
}

impl VisualEntropyCapture {
    /// Create a new visual entropy capturer
    #[must_use]
    pub const fn new(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            strokes: Vec::new(),
            canvas_size: (canvas_width, canvas_height),
        }
    }

    /// Add a stroke
    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.push(stroke);
    }

    /// Assess current quality
    #[must_use]
    pub fn assess_quality(&self) -> VisualQualityMetrics {
        if self.strokes.is_empty() {
            return VisualQualityMetrics {
                movement_entropy: 0.0,
                spatial_entropy: 0.0,
                timing_entropy: 0.0,
                overall_quality: 0.0,
            };
        }

        // Calculate movement entropy (direction changes)
        let movement_entropy = self.calculate_movement_entropy();

        // Calculate spatial entropy (coverage variance)
        let spatial_entropy = self.calculate_spatial_entropy();

        // Calculate timing entropy (inter-stroke intervals)
        let timing_entropy = self.calculate_timing_entropy();

        // Weighted average
        let overall_quality = weighted_quality(&[
            (movement_entropy, 0.3),
            (spatial_entropy, 0.4),
            (timing_entropy, 0.3),
        ]);

        VisualQualityMetrics {
            movement_entropy,
            spatial_entropy,
            timing_entropy,
            overall_quality,
        }
    }

    fn calculate_movement_entropy(&self) -> f64 {
        let all_points: Vec<Point2D> = self
            .strokes
            .iter()
            .flat_map(|s| s.points.iter().copied())
            .collect();
        if all_points.len() < 2 {
            return 0.0;
        }
        crate::gesture::analyze_gesture_entropy(&all_points)
    }

    fn calculate_spatial_entropy(&self) -> f64 {
        let (cw, ch) = (f64::from(self.canvas_size.0), f64::from(self.canvas_size.1));
        if cw < 1.0 || ch < 1.0 {
            return 0.0;
        }
        visual_complexity(&self.strokes, self.canvas_size)
    }

    fn calculate_timing_entropy(&self) -> f64 {
        let mut intervals: Vec<Duration> = Vec::new();
        for stroke in &self.strokes {
            for w in stroke.timestamps.windows(2) {
                let dt = w[1].saturating_sub(w[0]);
                intervals.push(dt);
            }
        }
        if intervals.len() < 2 {
            return 0.0;
        }
        crate::quality::timing_entropy(&intervals)
    }

    fn calculate_total_coverage(&self) -> f64 {
        let (cw, ch) = (self.canvas_size.0 as f32, self.canvas_size.1 as f32);
        if cw < 1.0 || ch < 1.0 {
            return 0.0;
        }
        let all_points: Vec<Point2D> = self
            .strokes
            .iter()
            .flat_map(|s| s.points.iter().copied())
            .collect();
        if all_points.is_empty() {
            return 0.0;
        }
        let (min_x, max_x) = all_points
            .iter()
            .map(|p| p.x)
            .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let (min_y, max_y) = all_points
            .iter()
            .map(|p| p.y)
            .fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
        let bbox_area = (max_x - min_x).max(0.0) * (max_y - min_y).max(0.0);
        let canvas_area = cw * ch;
        f64::from((bbox_area / canvas_area).min(1.0))
    }

    /// Finalize and create entropy data
    pub fn finalize(self) -> anyhow::Result<VisualEntropyData> {
        let quality_metrics = self.assess_quality();
        let total_coverage = self.calculate_total_coverage();

        Ok(VisualEntropyData {
            strokes: self.strokes,
            canvas_size: self.canvas_size,
            total_coverage,
            quality_metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn test_visual_capture_creation() {
        let capture = VisualEntropyCapture::new(800, 600);
        assert_eq!(capture.canvas_size, (800, 600));
    }

    #[test]
    fn test_visual_quality_empty() {
        let capture = VisualEntropyCapture::new(800, 600);
        let quality = capture.assess_quality();
        assert_eq!(quality.overall_quality, 0.0);
    }

    #[test]
    fn test_compute_color_entropy_empty() {
        let colors: Vec<Color> = vec![];
        assert_eq!(compute_color_entropy(&colors), 0.0);
    }

    #[test]
    fn test_compute_color_entropy_uniform() {
        let colors = vec![
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        ];
        let entropy = compute_color_entropy(&colors);
        assert!(entropy > 0.0 && entropy <= 1.0);
    }

    #[test]
    fn test_compute_color_entropy_single() {
        let colors = vec![Color {
            r: 100,
            g: 100,
            b: 100,
            a: 255,
        }];
        assert_eq!(compute_color_entropy(&colors), 0.0);
    }

    #[test]
    fn test_visual_complexity_empty() {
        let strokes: Vec<Stroke> = vec![];
        assert_eq!(visual_complexity(&strokes, (800, 600)), 0.0);
    }

    #[test]
    fn test_visual_complexity_single_stroke() {
        let stroke = Stroke {
            points: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 100.0, y: 100.0 }],
            timestamps: vec![Duration::from_millis(0), Duration::from_millis(100)],
            pressure: vec![0.5, 0.6],
            color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        };
        let complexity = visual_complexity(&[stroke], (800, 600));
        assert!(complexity > 0.0 && complexity <= 1.0);
    }

    #[test]
    fn test_visual_add_stroke_and_finalize() {
        let mut capture = VisualEntropyCapture::new(800, 600);
        let stroke = Stroke {
            points: vec![Point2D { x: 10.0, y: 20.0 }, Point2D { x: 50.0, y: 60.0 }],
            timestamps: vec![Duration::from_millis(0), Duration::from_millis(50)],
            pressure: vec![0.5, 0.6],
            color: Color {
                r: 255,
                g: 128,
                b: 0,
                a: 255,
            },
        };
        capture.add_stroke(stroke);
        let data = capture.finalize().expect("finalize");
        assert_eq!(data.strokes.len(), 1);
        assert_eq!(data.canvas_size, (800, 600));
        assert!(data.total_coverage >= 0.0 && data.total_coverage <= 1.0);
    }

    #[test]
    fn test_visual_quality_with_strokes() {
        let mut capture = VisualEntropyCapture::new(800, 600);
        let stroke = Stroke {
            points: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 100.0, y: 0.0 },
                Point2D { x: 100.0, y: 100.0 },
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(50),
                Duration::from_millis(100),
            ],
            pressure: vec![0.5, 0.5, 0.5],
            color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        };
        capture.add_stroke(stroke);
        let quality = capture.assess_quality();
        assert!(quality.overall_quality >= 0.0);
    }
}
