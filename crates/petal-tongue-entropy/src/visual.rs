//! Visual entropy capture (drawing, painting)
//!
//! Captures stroke patterns, spatial coverage, and timing for visual creativity.

use crate::quality::weighted_quality;
use crate::types::*;

/// Visual entropy capturer (stub for Phase 3)
pub struct VisualEntropyCapture {
    strokes: Vec<Stroke>,
    canvas_size: (u32, u32),
}

impl VisualEntropyCapture {
    /// Create a new visual entropy capturer
    pub fn new(canvas_width: u32, canvas_height: u32) -> Self {
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
        // Stub: Calculate direction changes across strokes
        0.7 // Placeholder
    }

    fn calculate_spatial_entropy(&self) -> f64 {
        // Stub: Calculate coverage variance
        0.7 // Placeholder
    }

    fn calculate_timing_entropy(&self) -> f64 {
        // Stub: Calculate inter-stroke timing
        0.7 // Placeholder
    }

    /// Finalize and create entropy data
    pub fn finalize(self) -> anyhow::Result<VisualEntropyData> {
        let quality_metrics = self.assess_quality();

        // Calculate total coverage (stub)
        let total_coverage = 0.5; // Placeholder

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
}
