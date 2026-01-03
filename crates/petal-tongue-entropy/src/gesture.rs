//! Gesture entropy capture (motion, touch)
//!
//! Captures sensor data (accelerometer, gyroscope) and touch patterns.

use crate::types::*;
use crate::quality::{variance, weighted_quality};

/// Gesture entropy capturer (stub for Phase 5)
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
    pub fn add_sensor_reading(
        &mut self,
        accel: Vec3,
        gyro: Vec3,
        timestamp: std::time::Duration,
    ) {
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
        // Stub: Analyze gesture patterns
        0.7 // Placeholder
    }

    fn calculate_timing_entropy(&self) -> f64 {
        // Stub: Analyze timing between readings
        0.7 // Placeholder
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
            Vec3 { x: 0.1, y: 0.2, z: 0.3 },
            Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            std::time::Duration::from_secs(0),
        );

        let quality = capture.assess_quality();
        assert!(quality.sensor_diversity > 0.0);
    }
}

