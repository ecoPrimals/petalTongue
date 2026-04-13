// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_cpu_stream() {
    let mut stream = CpuStream::new();
    stream.push_value(0.45);

    assert_eq!(stream.value(), 0.45);
    assert_eq!(stream.range(), (0.0, 1.0));
    assert_eq!(stream.label(), "CPU Usage");
}

#[test]
fn test_cpu_stream_empty_value() {
    let stream = CpuStream::new();
    assert_eq!(stream.value(), 0.0);
}

#[test]
fn test_cpu_stream_history_capped() {
    let mut stream = CpuStream::new();
    for i in 0..150 {
        stream.push_value(f64::from(i) / 150.0);
    }
    let hist = stream.history().expect("has history");
    assert!(hist.len() <= 120);
}

#[test]
fn test_audio_representation() {
    let mut stream = CpuStream::new();
    stream.push_value(0.5);
    let renderer = SystemMetricRenderer;

    let audio = renderer.render_audio(&stream);
    assert!(audio.is_some());

    let audio = audio.expect("CPU stream should have audio");
    assert!(audio.frequency >= 200.0 && audio.frequency <= 2000.0);
    assert!(audio.volume > 0.0);
}

#[test]
fn test_text_representation() {
    let mut stream = CpuStream::new();
    stream.push_value(0.65); // 65% -> active range (0.5-0.8)

    let renderer = SystemMetricRenderer;
    let text = renderer.render_text(&stream);

    assert!(text.contains("CPU Usage"));
    assert!(text.contains("65"));
    assert!(text.contains("active"));
}

#[test]
fn test_text_representation_idle() {
    let mut stream = CpuStream::new();
    stream.push_value(0.3);
    let renderer = SystemMetricRenderer;
    let text = renderer.render_text(&stream);
    assert!(text.contains("idle"));
}

#[test]
fn test_text_representation_busy() {
    let mut stream = CpuStream::new();
    stream.push_value(0.9);
    let renderer = SystemMetricRenderer;
    let text = renderer.render_text(&stream);
    assert!(text.contains("busy"));
}

#[test]
fn test_memory_stream() {
    let mut stream = MemoryStream::new(8 * 1024 * 1024 * 1024); // 8GB
    stream.push_value(4 * 1024 * 1024 * 1024); // 4GB used
    assert_eq!(stream.value(), 0.5);
    assert_eq!(stream.range(), (0.0, 1.0));
    assert_eq!(stream.label(), "Memory Usage");
}

#[test]
fn test_memory_stream_empty() {
    let stream = MemoryStream::new(1024);
    assert_eq!(stream.value(), 0.0);
}

#[test]
fn test_memory_stream_small_total() {
    let mut stream = MemoryStream::new(1024);
    stream.push_value(256);
    assert!((stream.value() - 0.25).abs() < 0.01);
}

#[test]
fn test_network_stream() {
    let mut stream = NetworkStream::new();
    stream.push_value(500_000.0); // 500 Kbps
    assert!(stream.value() > 0.0);
    assert!(stream.value() <= 1.0);
}

#[test]
fn test_network_stream_auto_normalize() {
    let mut stream = NetworkStream::new();
    stream.push_value(2_000_000.0);
    stream.push_value(1_000_000.0);
    assert!(stream.value() <= 1.0);
}

#[test]
fn test_modality_preferences_default() {
    let prefs = ModalityPreferences::default();
    assert!(prefs.visual_enabled);
    assert_eq!(prefs.visual_opacity, 1.0);
    assert!(!prefs.audio_enabled);
    assert_eq!(prefs.audio_volume, 0.5);
    assert!(prefs.text_enabled);
    assert!(!prefs.haptic_enabled);
}

#[test]
fn test_render_audio_memory() {
    let mut stream = MemoryStream::new(1024);
    stream.push_value(512);
    let renderer = SystemMetricRenderer;
    let audio = renderer.render_audio(&stream);
    assert!(audio.is_some());
    let audio = audio.expect("Memory stream should have audio");
    assert_eq!(audio.frequency, 400.0);
    assert!((audio.volume - 0.25).abs() < 0.01);
}

#[test]
fn test_render_audio_network() {
    let mut stream = NetworkStream::new();
    stream.push_value(500_000.0);
    let renderer = SystemMetricRenderer;
    let audio = renderer.render_audio(&stream);
    assert!(audio.is_some());
    let audio = audio.expect("Network stream should have audio");
    assert_eq!(audio.frequency, 800.0);
}

#[test]
fn test_render_audio_unknown_label() {
    struct UnknownStream;
    impl DataStream for UnknownStream {
        fn value(&self) -> f64 {
            0.5
        }
        fn range(&self) -> (f64, f64) {
            (0.0, 1.0)
        }
        fn label(&self) -> &'static str {
            "Unknown"
        }
    }
    let stream = UnknownStream;
    let renderer = SystemMetricRenderer;
    let audio = renderer.render_audio(&stream);
    assert!(audio.is_none());
}

#[test]
fn test_render_haptic() {
    let mut stream = CpuStream::new();
    stream.push_value(0.9);
    let renderer = SystemMetricRenderer;
    let haptic = renderer.render_haptic(&stream);
    assert!(haptic.is_some());
    let haptic = haptic.expect("should have haptic");
    assert!(haptic.intensity > 0.8);
    assert_eq!(haptic.pattern, HapticPatternType::Pulse);
}

#[test]
fn test_render_haptic_continuous() {
    let mut stream = CpuStream::new();
    stream.push_value(0.3);
    let renderer = SystemMetricRenderer;
    let haptic = renderer.render_haptic(&stream);
    assert!(haptic.is_some());
    let haptic = haptic.expect("should have haptic");
    assert_eq!(haptic.pattern, HapticPatternType::Continuous);
}

#[test]
fn test_haptic_pattern_types() {
    assert_eq!(HapticPatternType::Continuous, HapticPatternType::Continuous);
    assert_eq!(HapticPatternType::Pulse, HapticPatternType::Pulse);
    assert_eq!(HapticPatternType::Rhythm, HapticPatternType::Rhythm);
}

#[test]
fn test_cpu_stream_default() {
    let stream = CpuStream::default();
    assert_eq!(stream.label(), "CPU Usage");
}

#[test]
fn test_network_stream_default() {
    let stream = NetworkStream::default();
    assert_eq!(stream.label(), "Network Traffic");
}
