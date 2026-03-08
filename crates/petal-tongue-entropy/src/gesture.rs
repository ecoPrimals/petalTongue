// SPDX-License-Identifier: AGPL-3.0-only
//! Gesture entropy capture (motion, touch)
//!
//! Captures sensor data (accelerometer, gyroscope) and touch patterns.

use crate::quality::{create_histogram_buckets, shannon_entropy, variance, weighted_quality};
use crate::types::*;
use std::f64::consts::PI;
use std::time::Duration;

/// Compute Shannon entropy from gesture point deltas.
///
/// Histograms movement angles (atan2(dy, dx)) into buckets and computes
/// information entropy. Higher entropy indicates more diverse movement directions.
///
/// # Arguments
/// * `points` - Sequence of (x, y) gesture points
///
/// # Returns
/// Shannon entropy [0.0-1.0], or 0.0 for empty or single-point input
pub fn analyze_gesture_entropy(points: &[Point2D]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }

    // Compute movement angles from deltas
    let mut angles: Vec<f64> = Vec::with_capacity(points.len() - 1);
    for i in 0..points.len() - 1 {
        let dx = (points[i + 1].x - points[i].x) as f64;
        let dy = (points[i + 1].y - points[i].y) as f64;

        // Skip zero-length segments (no movement)
        if dx.abs() < 1e-10 && dy.abs() < 1e-10 {
            continue;
        }

        let angle = dy.atan2(dx);
        // Normalize to [0, 2π) for consistent bucketing
        let normalized = if angle >= 0.0 {
            angle
        } else {
            angle + 2.0 * PI
        };
        angles.push(normalized);
    }

    if angles.is_empty() {
        return 0.0;
    }

    // Histogram into 16 direction buckets (22.5° each)
    const NUM_BUCKETS: usize = 16;
    let buckets = create_histogram_buckets(&angles, NUM_BUCKETS);

    if buckets.is_empty() {
        return 0.0;
    }

    // Shannon entropy of bucket distribution, normalized to [0, 1]
    shannon_entropy(&buckets)
}

/// Quantify gesture complexity from path length, direction changes, and speed variance.
///
/// Combines: path length (normalized), direction change count, and speed variance
/// into a [0.0-1.0] complexity score.
///
/// # Arguments
/// * `points` - Sequence of (x, y) gesture points
/// * `timestamps` - Optional timestamps for each point (for speed variance).
///   If empty or length mismatch, speed variance contributes 0.
///
/// # Returns
/// Complexity score [0.0-1.0], or 0.0 for empty/single-point input
pub fn gesture_complexity(points: &[Point2D], timestamps: &[Duration]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }

    // 1. Path length (sum of segment lengths)
    let mut path_length = 0.0_f64;
    let mut direction_changes = 0_usize;
    let mut prev_angle: Option<f64> = None;
    const ANGLE_THRESHOLD: f64 = 0.5; // ~29° change to count as direction change

    for i in 0..points.len() - 1 {
        let dx = (points[i + 1].x - points[i].x) as f64;
        let dy = (points[i + 1].y - points[i].y) as f64;
        let seg_len = (dx * dx + dy * dy).sqrt();
        path_length += seg_len;

        if seg_len > 1e-10 {
            let angle = dy.atan2(dx);
            if let Some(prev) = prev_angle {
                let mut delta = (angle - prev).abs();
                if delta > PI {
                    delta = 2.0 * PI - delta;
                }
                if delta > ANGLE_THRESHOLD {
                    direction_changes += 1;
                }
            }
            prev_angle = Some(angle);
        }
    }

    if path_length < 1e-10 {
        return 0.0;
    }

    // 2. Path length component: normalize by typical gesture scale (e.g. 500px)
    let path_component = (path_length / 500.0).min(1.0);

    // 3. Direction change component: more changes = more complex
    let max_reasonable_changes = (points.len() - 1).max(1);
    let direction_component = (direction_changes as f64 / max_reasonable_changes as f64).min(1.0);

    // 4. Speed variance (if timestamps available)
    let speed_component = if timestamps.len() >= 2 && timestamps.len() == points.len() {
        let mut speeds: Vec<f64> = Vec::with_capacity(points.len() - 1);
        for i in 0..points.len() - 1 {
            let dt_ms = timestamps[i + 1]
                .saturating_sub(timestamps[i])
                .as_secs_f64()
                * 1000.0;
            if dt_ms > 1e-6 {
                let dx = (points[i + 1].x - points[i].x) as f64;
                let dy = (points[i + 1].y - points[i].y) as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                speeds.push(dist / dt_ms);
            }
        }
        if speeds.len() >= 2 {
            variance(&speeds)
        } else {
            0.5
        }
    } else {
        0.5
    };

    // Weighted combination
    let complexity = (path_component * 0.3 + direction_component * 0.4 + speed_component * 0.3)
        .min(1.0)
        .max(0.0);

    complexity
}

/// Gesture entropy capturer
pub struct GestureEntropyCapture {
    accelerometer: Vec<Vec3>,
    gyroscope: Vec<Vec3>,
    touch_events: Vec<TouchEvent>,
    timestamps: Vec<std::time::Duration>,
}

impl GestureEntropyCapture {
    /// Create a new gesture entropy capturer
    pub fn new() -> Self {
        Self {
            accelerometer: Vec::new(),
            gyroscope: Vec::new(),
            touch_events: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    /// Add sensor reading
    pub fn add_sensor_reading(&mut self, accel: Vec3, gyro: Vec3, timestamp: std::time::Duration) {
        self.accelerometer.push(accel);
        self.gyroscope.push(gyro);
        self.timestamps.push(timestamp);
    }

    /// Add touch event
    pub fn add_touch(&mut self, event: TouchEvent) {
        self.touch_events.push(event);
    }

    /// Assess current quality
    pub fn assess_quality(&self) -> GestureQualityMetrics {
        if self.accelerometer.is_empty() && self.touch_events.is_empty() {
            return GestureQualityMetrics {
                motion_entropy: 0.0,
                pattern_uniqueness: 0.0,
                timing_entropy: 0.0,
                sensor_diversity: 0.0,
                overall_quality: 0.0,
            };
        }

        let motion_entropy = self.calculate_motion_entropy();
        let pattern_uniqueness = self.calculate_pattern_uniqueness();
        let timing_entropy = self.calculate_timing_entropy();
        let sensor_diversity = self.calculate_sensor_diversity();

        let overall_quality = weighted_quality(&[
            (motion_entropy, 0.3),
            (pattern_uniqueness, 0.3),
            (timing_entropy, 0.2),
            (sensor_diversity, 0.2),
        ]);

        GestureQualityMetrics {
            motion_entropy,
            pattern_uniqueness,
            timing_entropy,
            sensor_diversity,
            overall_quality,
        }
    }

    fn calculate_motion_entropy(&self) -> f64 {
        // Stub: Calculate variance of motion magnitudes
        if self.accelerometer.is_empty() {
            return 0.0;
        }

        let magnitudes: Vec<f64> = self
            .accelerometer
            .iter()
            .map(|v| ((v.x * v.x + v.y * v.y + v.z * v.z) as f64).sqrt())
            .collect();

        variance(&magnitudes)
    }

    fn calculate_pattern_uniqueness(&self) -> f64 {
        let points: Vec<Point2D> = self.touch_events.iter().map(|e| e.position).collect();
        if points.len() < 2 {
            return 0.0;
        }
        analyze_gesture_entropy(&points)
    }

    fn calculate_timing_entropy(&self) -> f64 {
        if self.timestamps.len() < 2 {
            return 0.0;
        }
        let durations: Vec<Duration> = self
            .timestamps
            .windows(2)
            .map(|w| w[1].saturating_sub(w[0]))
            .collect();
        crate::quality::timing_entropy(&durations)
    }

    fn calculate_sensor_diversity(&self) -> f64 {
        // Count active sensor types
        let mut count = 0;
        if !self.accelerometer.is_empty() {
            count += 1;
        }
        if !self.gyroscope.is_empty() {
            count += 1;
        }
        if !self.touch_events.is_empty() {
            count += 1;
        }

        count as f64 / 3.0 // Normalize to [0.0-1.0]
    }

    /// Finalize and create entropy data
    pub fn finalize(self) -> anyhow::Result<GestureEntropyData> {
        let quality_metrics = self.assess_quality();

        Ok(GestureEntropyData {
            accelerometer: self.accelerometer,
            gyroscope: self.gyroscope,
            touch_events: self.touch_events,
            timestamps: self.timestamps,
            quality_metrics,
        })
    }
}

impl Default for GestureEntropyCapture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_capture_creation() {
        let capture = GestureEntropyCapture::new();
        assert!(capture.accelerometer.is_empty());
    }

    #[test]
    fn test_gesture_quality_empty() {
        let capture = GestureEntropyCapture::new();
        let quality = capture.assess_quality();
        assert_eq!(quality.overall_quality, 0.0);
    }

    #[test]
    fn test_gesture_sensor_diversity() {
        let mut capture = GestureEntropyCapture::new();

        // Add accelerometer reading
        capture.add_sensor_reading(
            Vec3 {
                x: 0.1,
                y: 0.2,
                z: 0.3,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            std::time::Duration::from_secs(0),
        );

        let quality = capture.assess_quality();
        assert!(quality.sensor_diversity > 0.0);
    }

    #[test]
    fn test_analyze_gesture_entropy_empty() {
        let points: Vec<Point2D> = vec![];
        assert_eq!(analyze_gesture_entropy(&points), 0.0);
    }

    #[test]
    fn test_analyze_gesture_entropy_single_point() {
        let points = vec![Point2D { x: 0.0, y: 0.0 }];
        assert_eq!(analyze_gesture_entropy(&points), 0.0);
    }

    #[test]
    fn test_analyze_gesture_entropy_straight_line() {
        let points = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 10.0, y: 0.0 },
            Point2D { x: 20.0, y: 0.0 },
        ];
        assert_eq!(analyze_gesture_entropy(&points), 0.0);
    }

    #[test]
    fn test_analyze_gesture_entropy_diverse_directions() {
        let points = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 10.0, y: 0.0 },
            Point2D { x: 10.0, y: 10.0 },
            Point2D { x: 0.0, y: 10.0 },
            Point2D { x: 0.0, y: 0.0 },
        ];
        let entropy = analyze_gesture_entropy(&points);
        assert!(entropy > 0.5);
    }

    #[test]
    fn test_gesture_complexity_empty() {
        let points: Vec<Point2D> = vec![];
        assert_eq!(gesture_complexity(&points, &[]), 0.0);
    }

    #[test]
    fn test_gesture_complexity_simple_path() {
        let points = vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 100.0, y: 0.0 }];
        let timestamps = vec![Duration::from_millis(0), Duration::from_millis(100)];
        let complexity = gesture_complexity(&points, &timestamps);
        assert!(complexity > 0.0 && complexity <= 1.0);
    }

    #[test]
    fn test_gesture_complexity_zigzag() {
        let points = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 50.0, y: 50.0 },
            Point2D { x: 100.0, y: 0.0 },
            Point2D { x: 150.0, y: 50.0 },
        ];
        let timestamps = vec![
            Duration::from_millis(0),
            Duration::from_millis(50),
            Duration::from_millis(100),
            Duration::from_millis(150),
        ];
        let complexity = gesture_complexity(&points, &timestamps);
        assert!(complexity > 0.0 && complexity <= 1.0);
    }
}
