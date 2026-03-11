// SPDX-License-Identifier: AGPL-3.0-only
//! Core types for entropy capture

use serde::{Deserialize, Serialize};
use std::time::Duration;
use zeroize::Zeroize;

/// Unified entropy capture type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropyCapture {
    /// Audio entropy (singing, speaking)
    Audio(AudioEntropyData),
    /// Visual entropy (drawing, painting)
    Visual(VisualEntropyData),
    /// Narrative entropy (storytelling, typing)
    Narrative(NarrativeEntropyData),
    /// Gesture entropy (motion, touch)
    Gesture(GestureEntropyData),
    /// Video entropy (camera motion)
    Video(VideoEntropyData),
}

impl EntropyCapture {
    /// Get the overall quality score for this entropy capture
    #[must_use]
    pub const fn quality(&self) -> f64 {
        match self {
            Self::Audio(data) => data.quality_metrics.overall_quality,
            Self::Visual(data) => data.quality_metrics.overall_quality,
            Self::Narrative(data) => data.quality_metrics.overall_quality,
            Self::Gesture(data) => data.quality_metrics.overall_quality,
            Self::Video(data) => data.quality_metrics.overall_quality,
        }
    }

    /// Get the modality name
    #[must_use]
    pub const fn modality(&self) -> &'static str {
        match self {
            Self::Audio(_) => "audio",
            Self::Visual(_) => "visual",
            Self::Narrative(_) => "narrative",
            Self::Gesture(_) => "gesture",
            Self::Video(_) => "video",
        }
    }
}

// Audio types

/// Audio entropy capture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEntropyData {
    /// Raw audio samples (manually zeroized on drop)
    pub samples: Vec<f32>,
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Capture duration
    #[serde(with = "serde_duration")]
    pub duration: Duration,
    /// Peak amplitudes per chunk
    pub peaks: Vec<f32>,
    /// Sample timestamps (for timing entropy)
    #[serde(with = "serde_duration_vec")]
    pub timestamps: Vec<Duration>,
    /// Peak amplitude (maximum)
    pub peak_amplitude: f32,
    /// Average amplitude
    pub avg_amplitude: f32,
    /// Quality metrics
    pub quality_metrics: AudioQualityMetrics,
}

impl Drop for AudioEntropyData {
    fn drop(&mut self) {
        // Manually zeroize sensitive audio data
        use zeroize::Zeroize;
        self.samples.zeroize();
        self.peaks.zeroize();
    }
}

/// Audio quality metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct AudioQualityMetrics {
    /// Amplitude entropy (volume variation) [0.0-1.0]
    pub amplitude_entropy: f64,
    /// Timing entropy (natural rhythm) [0.0-1.0]
    pub timing_entropy: f64,
    /// Spectral entropy (frequency distribution) [0.0-1.0]
    pub spectral_entropy: f64,
    /// Dynamic range (amplitude variance) [0.0-1.0]
    pub dynamic_range: f64,
    /// Overall quality (weighted average) [0.0-1.0]
    pub overall_quality: f64,
}

impl Default for AudioQualityMetrics {
    fn default() -> Self {
        Self {
            amplitude_entropy: 0.0,
            timing_entropy: 0.0,
            spectral_entropy: 0.0,
            dynamic_range: 0.0,
            overall_quality: 0.0,
        }
    }
}

// Serde helpers for Duration
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "duration in ms fits u64 for serialization"
        )]
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

mod serde_duration_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(durations: &[Duration], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "duration in ms fits u64 for serialization"
        )]
        let millis: Vec<u64> = durations.iter().map(|d| d.as_millis() as u64).collect();
        millis.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = Vec::<u64>::deserialize(deserializer)?;
        Ok(millis.into_iter().map(Duration::from_millis).collect())
    }
}

// Visual types

/// Visual entropy capture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEntropyData {
    /// Strokes (paths, timing, pressure)
    pub strokes: Vec<Stroke>,
    /// Canvas size (width, height)
    pub canvas_size: (u32, u32),
    /// Total coverage (percentage [0.0-1.0])
    pub total_coverage: f64,
    /// Quality metrics
    pub quality_metrics: VisualQualityMetrics,
}

impl Drop for VisualEntropyData {
    fn drop(&mut self) {
        // Manually zeroize sensitive stroke data
        use zeroize::Zeroize;
        for stroke in &mut self.strokes {
            stroke.points.zeroize();
            stroke.pressure.zeroize();
        }
    }
}

/// A single drawing stroke
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    /// Points along the stroke
    pub points: Vec<Point2D>,
    /// Timestamp for each point (relative to start)
    pub timestamps: Vec<Duration>,
    /// Pressure at each point [0.0-1.0]
    pub pressure: Vec<f32>,
    /// Color (RGBA)
    pub color: Color,
}

/// 2D point
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct Point2D {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

/// Color (RGBA)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct Color {
    /// Red [0-255]
    pub r: u8,
    /// Green [0-255]
    pub g: u8,
    /// Blue [0-255]
    pub b: u8,
    /// Alpha [0-255]
    pub a: u8,
}

/// Visual quality metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct VisualQualityMetrics {
    /// Movement entropy (natural motion) [0.0-1.0]
    pub movement_entropy: f64,
    /// Spatial entropy (coverage & variance) [0.0-1.0]
    pub spatial_entropy: f64,
    /// Timing entropy (natural rhythm) [0.0-1.0]
    pub timing_entropy: f64,
    /// Overall quality (weighted average) [0.0-1.0]
    pub overall_quality: f64,
}

// Narrative types

/// Narrative entropy capture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEntropyData {
    /// The story text (manually zeroized on drop)
    pub text: String,
    /// Keystroke timings (inter-key intervals)
    pub keystroke_timings: Vec<Duration>,
    /// Backspace events (corrections)
    pub backspace_events: Vec<BackspaceEvent>,
    /// Pause durations (thinking time)
    pub pause_durations: Vec<Duration>,
    /// Quality metrics
    pub quality_metrics: NarrativeQualityMetrics,
}

impl Drop for NarrativeEntropyData {
    fn drop(&mut self) {
        // Manually zeroize sensitive text data
        use zeroize::Zeroize;
        self.text.zeroize();
    }
}

/// A backspace event (correction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackspaceEvent {
    /// Timestamp of the backspace
    pub timestamp: Duration,
    /// Position in the text
    pub position: usize,
    /// Character that was deleted
    pub deleted_char: char,
}

/// Narrative quality metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct NarrativeQualityMetrics {
    /// Keystroke entropy (typing rhythm) [0.0-1.0]
    pub keystroke_entropy: f64,
    /// Pause entropy (thinking patterns) [0.0-1.0]
    pub pause_entropy: f64,
    /// Correction entropy (human mistakes) [0.0-1.0]
    pub correction_entropy: f64,
    /// Content entropy (story uniqueness) [0.0-1.0]
    pub content_entropy: f64,
    /// Overall quality (weighted average) [0.0-1.0]
    pub overall_quality: f64,
}

// Gesture types

/// Gesture entropy capture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEntropyData {
    /// Accelerometer readings
    pub accelerometer: Vec<Vec3>,
    /// Gyroscope readings
    pub gyroscope: Vec<Vec3>,
    /// Touch events
    pub touch_events: Vec<TouchEvent>,
    /// Timestamps for each reading
    pub timestamps: Vec<Duration>,
    /// Quality metrics
    pub quality_metrics: GestureQualityMetrics,
}

impl Drop for GestureEntropyData {
    fn drop(&mut self) {
        // Manually zeroize sensitive sensor data
        use zeroize::Zeroize;
        self.accelerometer.zeroize();
        self.gyroscope.zeroize();
        for event in &mut self.touch_events {
            event.position.zeroize();
        }
    }
}

/// 3D vector (for sensors)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct Vec3 {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
    /// Z component
    pub z: f32,
}

/// Touch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchEvent {
    /// Position on screen
    pub position: Point2D,
    /// Pressure [0.0-1.0]
    pub pressure: f32,
    /// Timestamp
    pub timestamp: Duration,
}

/// Gesture quality metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct GestureQualityMetrics {
    /// Motion entropy (movement variance) [0.0-1.0]
    pub motion_entropy: f64,
    /// Pattern uniqueness (your signature) [0.0-1.0]
    pub pattern_uniqueness: f64,
    /// Timing entropy (natural rhythm) [0.0-1.0]
    pub timing_entropy: f64,
    /// Sensor diversity (multi-source bonus) [0.0-1.0]
    pub sensor_diversity: f64,
    /// Overall quality (weighted average) [0.0-1.0]
    pub overall_quality: f64,
}

// Video types

/// Video entropy capture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEntropyData {
    /// Motion vectors (frame-to-frame)
    pub motion_vectors: Vec<MotionField>,
    /// Lighting samples
    pub lighting_samples: Vec<f32>,
    /// Scene complexity scores
    pub scene_complexity: Vec<f32>,
    /// Timestamps for each frame
    pub timestamps: Vec<Duration>,
    /// Quality metrics
    pub quality_metrics: VideoQualityMetrics,
}

impl Drop for VideoEntropyData {
    fn drop(&mut self) {
        // Manually zeroize sensitive motion data
        use zeroize::Zeroize;
        for field in &mut self.motion_vectors {
            field.vectors.zeroize();
        }
        self.lighting_samples.zeroize();
        self.scene_complexity.zeroize();
    }
}

/// Motion field (per frame)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionField {
    /// Motion vectors (2D)
    pub vectors: Vec<Vec2>,
    /// Average magnitude
    pub magnitude: f32,
    /// Average direction (radians)
    pub direction: f32,
}

/// 2D vector (for motion)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct Vec2 {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
}

/// Video quality metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Zeroize)]
pub struct VideoQualityMetrics {
    /// Motion entropy (movement variance) [0.0-1.0]
    pub motion_entropy: f64,
    /// Scene entropy (visual complexity) [0.0-1.0]
    pub scene_entropy: f64,
    /// Temporal entropy (time-based patterns) [0.0-1.0]
    pub temporal_entropy: f64,
    /// Overall quality (weighted average) [0.0-1.0]
    pub overall_quality: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroize;

    #[test]
    fn test_entropy_capture_quality() {
        let audio_metrics = AudioQualityMetrics {
            amplitude_entropy: 0.8,
            timing_entropy: 0.8,
            spectral_entropy: 0.7,
            dynamic_range: 0.9,
            overall_quality: 0.8,
        };

        let audio_data = AudioEntropyData {
            samples: vec![0.1, 0.2, 0.3],
            sample_rate: 44100,
            duration: Duration::from_secs(30),
            peaks: vec![0.3, 0.5, 0.4],
            timestamps: vec![Duration::from_millis(0), Duration::from_millis(100)],
            peak_amplitude: 0.5,
            avg_amplitude: 0.4,
            quality_metrics: audio_metrics,
        };

        let capture = EntropyCapture::Audio(audio_data);
        assert!((capture.quality() - 0.8).abs() < 0.01);
        assert_eq!(capture.modality(), "audio");
    }

    #[test]
    fn test_zeroization() {
        let mut audio_data = AudioEntropyData {
            samples: vec![1.0, 2.0, 3.0],
            sample_rate: 44100,
            duration: Duration::from_secs(1),
            peaks: vec![3.0],
            timestamps: vec![Duration::from_millis(0)],
            peak_amplitude: 3.0,
            avg_amplitude: 2.0,
            quality_metrics: AudioQualityMetrics {
                amplitude_entropy: 0.5,
                timing_entropy: 0.5,
                spectral_entropy: 0.5,
                dynamic_range: 0.5,
                overall_quality: 0.5,
            },
        };

        audio_data.samples.zeroize();
        assert!(audio_data.samples.iter().all(|&x| x == 0.0));
    }
}
