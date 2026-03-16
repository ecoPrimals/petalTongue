// SPDX-License-Identifier: AGPL-3.0-or-later
//! Golden pixel test infrastructure for deterministic scene graph rendering.

use crate::provenance::{ProvenanceBuffer, ProvenanceRenderer};
use crate::scene_graph::SceneGraph;

/// A golden test case: scene + viewport dimensions.
pub struct GoldenTest {
    pub name: String,
    pub scene: SceneGraph,
    pub width: u32,
    pub height: u32,
}

/// Result of verifying a region against expected `data_id`.
pub struct GoldenTestResult {
    pub passed: bool,
    pub mismatched_pixels: usize,
    pub total_pixels: usize,
}

/// Tufte-aware verification result including data-ink and coverage metrics.
pub struct TufteVerificationResult {
    /// Whether the verification passed within tolerances.
    pub passed: bool,
    /// Fraction of pixels that carry data (have provenance). Range: 0.0..=1.0.
    pub data_ink_ratio: f64,
    /// Fraction of viewport covered by data primitives. Range: 0.0..=1.0.
    pub coverage: f64,
    /// The data-ink tolerance used for this check.
    pub data_ink_tolerance: f64,
    /// The coverage tolerance used for this check.
    pub coverage_tolerance: f64,
}

impl GoldenTest {
    #[must_use]
    pub fn new(name: impl Into<String>, scene: SceneGraph, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            scene,
            width,
            height,
        }
    }

    /// Renders the scene to a provenance buffer.
    #[must_use]
    pub fn render(&self) -> ProvenanceBuffer {
        ProvenanceRenderer.render(&self.scene, self.width, self.height)
    }

    /// Verifies a single pixel: if `expected_data_id` is None, pixel must have no provenance;
    /// otherwise pixel must have that `data_id`.
    #[must_use]
    pub fn verify_pixel(
        &self,
        buffer: &ProvenanceBuffer,
        x: u32,
        y: u32,
        expected_data_id: Option<&str>,
    ) -> bool {
        let actual = buffer.get(x, y).and_then(|p| p.data_id.as_deref());
        expected_data_id.map_or_else(|| actual.is_none(), |expected| actual == Some(expected))
    }

    /// Verifies all pixels in rect (x, y, width, height) have the expected `data_id`.
    #[must_use]
    pub fn verify_region(
        &self,
        buffer: &ProvenanceBuffer,
        rect: (u32, u32, u32, u32),
        expected_data_id: Option<&str>,
    ) -> GoldenTestResult {
        let (x0, y0, w, h) = rect;
        let mut mismatched = 0usize;
        let mut total = 0usize;
        for y in y0..y0.saturating_add(h).min(buffer.height()) {
            for x in x0..x0.saturating_add(w).min(buffer.width()) {
                total += 1;
                if !self.verify_pixel(buffer, x, y, expected_data_id) {
                    mismatched += 1;
                }
            }
        }
        GoldenTestResult {
            passed: mismatched == 0,
            mismatched_pixels: mismatched,
            total_pixels: total,
        }
    }

    /// Verify that a rendered buffer meets Tufte data-ink and coverage thresholds.
    ///
    /// Uses the ludoSpring V14 tolerance constants by default:
    /// - `data_ink_tolerance`: max fraction of non-data pixels (default `UI_DATA_INK_TOL = 0.01`)
    /// - `coverage_tolerance`: max fraction of uncovered viewport (default `UI_COVERAGE_TOL = 0.05`)
    #[must_use]
    pub fn verify_tufte(
        &self,
        buffer: &ProvenanceBuffer,
        data_ink_tolerance: f64,
        coverage_tolerance: f64,
    ) -> TufteVerificationResult {
        let total = u64::from(buffer.width()) * u64::from(buffer.height());
        if total == 0 {
            return TufteVerificationResult {
                passed: true,
                data_ink_ratio: 1.0,
                coverage: 1.0,
                data_ink_tolerance,
                coverage_tolerance,
            };
        }
        let mut data_pixels: u64 = 0;
        for y in 0..buffer.height() {
            for x in 0..buffer.width() {
                if buffer.get(x, y).is_some() {
                    data_pixels += 1;
                }
            }
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "pixel count fits within f64 mantissa for any practical viewport"
        )]
        let data_ink_ratio = data_pixels as f64 / total as f64;
        let coverage = data_ink_ratio;
        let chartjunk = 1.0 - data_ink_ratio;
        let uncovered = 1.0 - coverage;
        let passed = chartjunk <= data_ink_tolerance && uncovered <= coverage_tolerance;
        TufteVerificationResult {
            passed,
            data_ink_ratio,
            coverage,
            data_ink_tolerance,
            coverage_tolerance,
        }
    }

    /// Returns coordinates where two buffers differ (comparing `data_id` of each pixel).
    #[must_use]
    pub fn pixel_diff(a: &ProvenanceBuffer, b: &ProvenanceBuffer) -> Vec<(u32, u32)> {
        let w = a.width().min(b.width());
        let h = a.height().min(b.height());
        let mut diff = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let da = a.get(x, y).and_then(|p| p.data_id.as_deref());
                let db = b.get(x, y).and_then(|p| p.data_id.as_deref());
                if da != db {
                    diff.push((x, y));
                }
            }
        }
        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Color, Primitive};
    use crate::scene_graph::SceneNode;

    fn rect(x: f64, y: f64, w: f64, h: f64, data_id: Option<&str>) -> Primitive {
        Primitive::Rect {
            x,
            y,
            width: w,
            height: h,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 0.0,
            data_id: data_id.map(String::from),
        }
    }

    #[test]
    fn single_rect_golden_test() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("r1").with_primitive(rect(
            1.0,
            2.0,
            3.0,
            2.0,
            Some("data-a"),
        )));
        let gt = GoldenTest::new("single_rect", scene, 10, 10);
        let buf = gt.render();
        assert!(gt.verify_pixel(&buf, 1, 2, Some("data-a")));
        assert!(gt.verify_pixel(&buf, 3, 3, Some("data-a")));
        assert!(gt.verify_pixel(&buf, 0, 0, None));
        assert!(gt.verify_pixel(&buf, 5, 5, None));
        let inside = gt.verify_region(&buf, (1, 2, 3, 2), Some("data-a"));
        assert!(inside.passed, "inside rect should match");
        let outside = gt.verify_region(&buf, (0, 0, 1, 1), None);
        assert!(outside.passed, "outside should be None");
    }

    #[test]
    fn overlapping_shapes_topmost_wins() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("bottom").with_primitive(rect(
            0.0,
            0.0,
            5.0,
            5.0,
            Some("bottom-id"),
        )));
        scene.add_to_root(SceneNode::new("top").with_primitive(rect(
            2.0,
            2.0,
            3.0,
            3.0,
            Some("top-id"),
        )));
        let gt = GoldenTest::new("overlap", scene, 10, 10);
        let buf = gt.render();
        assert!(gt.verify_pixel(&buf, 1, 1, Some("bottom-id")));
        assert!(gt.verify_pixel(&buf, 3, 3, Some("top-id")));
    }

    #[test]
    fn pixel_diff_identical_returns_empty() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n").with_primitive(rect(0.0, 0.0, 2.0, 2.0, Some("x"))));
        let buf = ProvenanceRenderer.render(&scene, 5, 5);
        let diff = GoldenTest::pixel_diff(&buf, &buf);
        assert!(diff.is_empty());
    }

    #[test]
    fn tufte_full_coverage_passes() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("bg").with_primitive(rect(
            0.0,
            0.0,
            10.0,
            10.0,
            Some("bg"),
        )));
        let gt = GoldenTest::new("tufte_full", scene, 10, 10);
        let buf = gt.render();
        let result = gt.verify_tufte(&buf, 0.01, 0.05);
        assert!(result.passed);
        assert!((result.data_ink_ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tufte_low_coverage_fails() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("tiny").with_primitive(rect(
            0.0,
            0.0,
            1.0,
            1.0,
            Some("dot"),
        )));
        let gt = GoldenTest::new("tufte_sparse", scene, 100, 100);
        let buf = gt.render();
        let result = gt.verify_tufte(&buf, 0.01, 0.05);
        assert!(!result.passed, "sparse scene should fail Tufte coverage");
        assert!(result.coverage < 0.05);
    }

    #[test]
    fn pixel_diff_different_returns_positions() {
        let mut s1 = SceneGraph::new();
        s1.add_to_root(SceneNode::new("a").with_primitive(rect(0.0, 0.0, 2.0, 2.0, Some("a"))));
        let mut s2 = SceneGraph::new();
        s2.add_to_root(SceneNode::new("b").with_primitive(rect(0.0, 0.0, 2.0, 2.0, Some("b"))));
        let buf1 = ProvenanceRenderer.render(&s1, 5, 5);
        let buf2 = ProvenanceRenderer.render(&s2, 5, 5);
        let diff = GoldenTest::pixel_diff(&buf1, &buf2);
        assert!(!diff.is_empty());
        assert!(diff.contains(&(1, 1)));
    }
}
