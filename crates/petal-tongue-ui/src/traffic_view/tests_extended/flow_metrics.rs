// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::{
    ColorScheme, TrafficFlow, TrafficMetrics, calculate_flow_color, calculate_flow_width,
    prepare_flow_detail,
};

#[test]
fn calculate_flow_color_volume_zero() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Volume);
    assert_eq!(c[0], 0);
    assert_eq!(c[1], 255);
}

#[test]
fn calculate_flow_color_volume_max() {
    let m = TrafficMetrics {
        bytes_per_second: 200_000,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Volume);
    assert_eq!(c[0], 255);
    assert_eq!(c[1], 0);
}

#[test]
fn calculate_flow_color_latency_fast() {
    let m = TrafficMetrics {
        avg_latency_ms: 0.0,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Latency);
    assert_eq!(c[0], 0);
}

#[test]
fn calculate_flow_color_error_high() {
    let m = TrafficMetrics {
        error_rate: 0.5,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::ErrorRate);
    assert!(c[0] > 200);
}

#[test]
fn calculate_flow_width_min() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 10000, 2.0, 40.0);
    assert!((w - 2.0).abs() < f32::EPSILON);
}

#[test]
fn calculate_flow_width_max() {
    let m = TrafficMetrics {
        bytes_per_second: 10000,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 10000, 2.0, 40.0);
    assert!((w - 40.0).abs() < f32::EPSILON);
}

#[test]
fn prepare_flow_detail_formats() {
    let flow = TrafficFlow {
        from: "alpha".to_string(),
        to: "beta".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 5000,
            requests_per_second: 42.5,
            avg_latency_ms: 12.34,
            error_rate: 0.05,
        },
        color: [255, 0, 0, 255],
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.from, "alpha");
    assert_eq!(d.to, "beta");
    assert_eq!(d.volume_label, "5000 B/s");
    assert_eq!(d.requests_label, "42.5 req/s");
    assert_eq!(d.latency_label, "12.34 ms");
    assert_eq!(d.error_rate_label, "5.00%");
}

#[test]
fn calculate_flow_width_max_volume_zero_no_panic() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 0, 2.0, 40.0);
    assert!(
        (2.0..=40.0).contains(&w),
        "result should be in valid range: {w}"
    );
    assert!(
        (w - 2.0).abs() < 0.01,
        "zero volume with max_vol guard should give min_width: {w}"
    );
}

#[test]
fn calculate_flow_color_volume_variants() {
    let zero = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert_eq!(zero[0], 0);
    assert_eq!(zero[1], 255);
    let mid = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 50_000,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert!(mid[0] > 0);
    assert!(mid[1] < 255);
    let high = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 150_000,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert_eq!(high[0], 255);
    assert_eq!(high[1], 0);
}

#[test]
fn calculate_flow_color_latency_variants() {
    let zero = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 0.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert_eq!(zero[0], 0);
    let mid = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 50.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert!(mid[0] > 0);
    assert!(mid[1] < 255);
    let high = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 150.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert_eq!(high[0], 255);
}

#[test]
fn calculate_flow_color_error_rate_variants() {
    let zero = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.0,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert_eq!(zero[0], 0);
    let mid = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.1,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert!(mid[0] > 0);
    let high = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 1.0,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert_eq!(high[0], 255);
}

#[test]
fn prepare_flow_detail_zero_metrics() {
    let flow = TrafficFlow {
        from: "src".to_string(),
        to: "dst".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 0,
            requests_per_second: 0.0,
            avg_latency_ms: 0.0,
            error_rate: 0.0,
        },
        color: [0, 255, 0, 255],
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.volume_label, "0 B/s");
    assert_eq!(d.requests_label, "0.0 req/s");
    assert_eq!(d.latency_label, "0.00 ms");
    assert_eq!(d.error_rate_label, "0.00%");
}

#[test]
fn prepare_flow_detail_high_metrics() {
    let flow = TrafficFlow {
        from: "src".to_string(),
        to: "dst".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 999_999_999,
            requests_per_second: 12345.67,
            avg_latency_ms: 999.99,
            error_rate: 0.999,
        },
        color: [0, 255, 0, 255],
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.volume_label, "999999999 B/s");
    assert_eq!(d.requests_label, "12345.7 req/s");
    assert_eq!(d.latency_label, "999.99 ms");
    assert_eq!(d.error_rate_label, "99.90%");
}
