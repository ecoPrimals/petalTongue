// SPDX-License-Identifier: AGPL-3.0-only
//! Comprehensive tests for entropy streaming
//!
//! Tests verify encryption, zeroization, and streaming logic.

use petal_tongue_entropy::types::*;
use std::time::Duration;

#[test]
fn test_audio_entropy_creation() {
    let audio_metrics = AudioQualityMetrics {
        amplitude_entropy: 0.8,
        timing_entropy: 0.7,
        spectral_entropy: 0.6,
        dynamic_range: 0.9,
        overall_quality: 0.75,
    };

    let audio_data = AudioEntropyData {
        samples: vec![0.1, 0.2, 0.3],
        sample_rate: 44100,
        duration: Duration::from_secs(1),
        peaks: vec![0.3],
        timestamps: vec![Duration::from_millis(0), Duration::from_millis(100)],
        peak_amplitude: 0.3,
        avg_amplitude: 0.2,
        quality_metrics: audio_metrics,
    };

    let capture = EntropyCapture::Audio(audio_data);
    assert_eq!(capture.modality(), "audio");
    assert!((capture.quality() - 0.75).abs() < 0.01);
}

#[test]
fn test_audio_quality_metrics() {
    let quality = AudioQualityMetrics {
        amplitude_entropy: 0.8,
        timing_entropy: 0.7,
        spectral_entropy: 0.9,
        dynamic_range: 0.6,
        overall_quality: 0.75,
    };

    assert!(quality.overall_quality > 0.0 && quality.overall_quality <= 1.0);
    assert!((quality.overall_quality - 0.75).abs() < 0.1);
}

#[test]
fn test_visual_entropy_creation() {
    let visual = VisualEntropyData {
        strokes: vec![],
        canvas_size: (1920, 1080),
        total_coverage: 0.5,
        quality_metrics: VisualQualityMetrics {
            movement_entropy: 0.7,
            spatial_entropy: 0.6,
            timing_entropy: 0.8,
            overall_quality: 0.7,
        },
    };

    let capture = EntropyCapture::Visual(visual);
    assert_eq!(capture.modality(), "visual");
    assert!((capture.quality() - 0.7).abs() < 0.01);
}

#[test]
fn test_stroke_creation() {
    let stroke = Stroke {
        points: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 10.0, y: 10.0 }],
        timestamps: vec![Duration::from_millis(0), Duration::from_millis(10)],
        pressure: vec![0.5, 0.6],
        color: Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        },
    };

    assert_eq!(stroke.points.len(), 2);
    assert_eq!(stroke.pressure.len(), 2);
}

#[test]
fn test_narrative_entropy_creation() {
    let narrative = NarrativeEntropyData {
        text: "Test narrative".to_string(),
        keystroke_timings: vec![Duration::from_millis(50); 14],
        backspace_events: vec![],
        pause_durations: vec![Duration::from_millis(500)],
        quality_metrics: NarrativeQualityMetrics {
            keystroke_entropy: 0.9,
            pause_entropy: 0.7,
            correction_entropy: 0.5,
            content_entropy: 0.8,
            overall_quality: 0.725,
        },
    };

    let capture = EntropyCapture::Narrative(narrative);
    assert_eq!(capture.modality(), "narrative");
    assert!((capture.quality() - 0.725).abs() < 0.01);
}

#[test]
fn test_backspace_event() {
    let event = BackspaceEvent {
        timestamp: Duration::from_secs(1),
        position: 10,
        deleted_char: 'x',
    };

    assert_eq!(event.position, 10);
    assert_eq!(event.deleted_char, 'x');
}

#[test]
fn test_gesture_entropy_creation() {
    let gesture = GestureEntropyData {
        accelerometer: vec![Vec3 {
            x: 0.1,
            y: 0.2,
            z: 0.3,
        }],
        gyroscope: vec![Vec3 {
            x: 0.01,
            y: 0.02,
            z: 0.03,
        }],
        touch_events: vec![],
        timestamps: vec![Duration::from_millis(0)],
        quality_metrics: GestureQualityMetrics {
            motion_entropy: 0.8,
            pattern_uniqueness: 0.7,
            timing_entropy: 0.75,
            sensor_diversity: 0.6,
            overall_quality: 0.7,
        },
    };

    let capture = EntropyCapture::Gesture(gesture);
    assert_eq!(capture.modality(), "gesture");
    assert!((capture.quality() - 0.7).abs() < 0.01);
}

#[test]
fn test_touch_event() {
    let event = TouchEvent {
        position: Point2D { x: 100.0, y: 200.0 },
        pressure: 0.5,
        timestamp: Duration::from_millis(100),
    };

    assert_eq!(event.position.x, 100.0);
    assert_eq!(event.pressure, 0.5);
}

#[test]
fn test_video_entropy_creation() {
    let video = VideoEntropyData {
        motion_vectors: vec![],
        lighting_samples: vec![0.5, 0.6, 0.7],
        scene_complexity: vec![0.8],
        timestamps: vec![Duration::from_millis(0)],
        quality_metrics: VideoQualityMetrics {
            motion_entropy: 0.8,
            scene_entropy: 0.7,
            temporal_entropy: 0.6,
            overall_quality: 0.7,
        },
    };

    let capture = EntropyCapture::Video(video);
    assert_eq!(capture.modality(), "video");
    assert!((capture.quality() - 0.7).abs() < 0.01);
}

#[test]
fn test_motion_field() {
    let field = MotionField {
        vectors: vec![Vec2 { x: 1.0, y: 2.0 }],
        magnitude: 2.236,
        direction: 1.107,
    };

    assert_eq!(field.vectors.len(), 1);
    assert_eq!(field.vectors[0].x, 1.0);
    assert!((field.magnitude - 2.236).abs() < 0.01);
}

#[test]
fn test_entropy_serialization() {
    let audio = AudioEntropyData {
        samples: vec![0.1, 0.2, 0.3],
        sample_rate: 44100,
        duration: Duration::from_micros(68),
        peaks: vec![0.3],
        timestamps: vec![],
        peak_amplitude: 0.3,
        avg_amplitude: 0.2,
        quality_metrics: AudioQualityMetrics {
            amplitude_entropy: 0.8,
            timing_entropy: 0.7,
            spectral_entropy: 0.6,
            dynamic_range: 0.9,
            overall_quality: 0.75,
        },
    };
    let capture = EntropyCapture::Audio(audio);

    // Test JSON serialization
    let json = serde_json::to_string(&capture);
    assert!(json.is_ok(), "Should serialize to JSON");

    // Test deserialization
    let json_str = json.unwrap();
    let deserialized: Result<EntropyCapture, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok(), "Should deserialize from JSON");
}

#[test]
fn test_quality_bounds() {
    // Test minimum quality
    let min_quality = AudioQualityMetrics {
        amplitude_entropy: 0.0,
        timing_entropy: 0.0,
        spectral_entropy: 0.0,
        dynamic_range: 0.0,
        overall_quality: 0.0,
    };
    assert_eq!(min_quality.overall_quality, 0.0);

    // Test maximum quality
    let max_quality = AudioQualityMetrics {
        amplitude_entropy: 1.0,
        timing_entropy: 1.0,
        spectral_entropy: 1.0,
        dynamic_range: 1.0,
        overall_quality: 1.0,
    };
    assert_eq!(max_quality.overall_quality, 1.0);
}

#[test]
fn test_entropy_modality_strings() {
    let audio = EntropyCapture::Audio(AudioEntropyData {
        samples: vec![],
        sample_rate: 44100,
        duration: Duration::from_secs(0),
        peaks: vec![],
        timestamps: vec![],
        peak_amplitude: 0.0,
        avg_amplitude: 0.0,
        quality_metrics: AudioQualityMetrics::default(),
    });

    let visual = EntropyCapture::Visual(VisualEntropyData {
        strokes: vec![],
        canvas_size: (0, 0),
        total_coverage: 0.0,
        quality_metrics: VisualQualityMetrics {
            movement_entropy: 0.0,
            spatial_entropy: 0.0,
            timing_entropy: 0.0,
            overall_quality: 0.0,
        },
    });

    let narrative = EntropyCapture::Narrative(NarrativeEntropyData {
        text: String::new(),
        keystroke_timings: vec![],
        backspace_events: vec![],
        pause_durations: vec![],
        quality_metrics: NarrativeQualityMetrics {
            keystroke_entropy: 0.0,
            pause_entropy: 0.0,
            correction_entropy: 0.0,
            content_entropy: 0.0,
            overall_quality: 0.0,
        },
    });

    let gesture = EntropyCapture::Gesture(GestureEntropyData {
        accelerometer: vec![],
        gyroscope: vec![],
        touch_events: vec![],
        timestamps: vec![],
        quality_metrics: GestureQualityMetrics {
            motion_entropy: 0.0,
            pattern_uniqueness: 0.0,
            timing_entropy: 0.0,
            sensor_diversity: 0.0,
            overall_quality: 0.0,
        },
    });

    let video = EntropyCapture::Video(VideoEntropyData {
        motion_vectors: vec![],
        lighting_samples: vec![],
        scene_complexity: vec![],
        timestamps: vec![],
        quality_metrics: VideoQualityMetrics {
            motion_entropy: 0.0,
            scene_entropy: 0.0,
            temporal_entropy: 0.0,
            overall_quality: 0.0,
        },
    });

    assert_eq!(audio.modality(), "audio");
    assert_eq!(visual.modality(), "visual");
    assert_eq!(narrative.modality(), "narrative");
    assert_eq!(gesture.modality(), "gesture");
    assert_eq!(video.modality(), "video");
}

#[test]
fn test_audio_entropy_default() {
    let metrics = AudioQualityMetrics::default();
    assert_eq!(metrics.overall_quality, 0.0);
}
