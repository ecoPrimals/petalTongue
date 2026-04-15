// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::{TrafficFlow, TrafficMetrics, TrafficView, primal_lane_layout};

#[test]
fn primal_lane_layout_empty() {
    let layout = primal_lane_layout(0, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert!(layout.is_empty());
}

#[test]
fn primal_lane_layout_single() {
    let layout = primal_lane_layout(1, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 1);
    let (y, left_x, right_x) = layout[0];
    assert!((y - 300.0).abs() < f32::EPSILON);
    assert!((left_x - 80.0).abs() < f32::EPSILON);
    assert!((right_x - 720.0).abs() < f32::EPSILON);
}

#[test]
fn primal_lane_layout_three() {
    let layout = primal_lane_layout(3, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 3);
    let node_height = (600.0 - 40.0) / 3.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 20.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 80.0).abs() < f32::EPSILON);
        assert!((right_x - 720.0).abs() < f32::EPSILON);
    }
}

#[test]
fn max_volume_no_flows() {
    let view = TrafficView::new();
    assert_eq!(view.max_volume(), 1);
}

#[test]
fn max_volume_all_zero() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    view.add_flow(TrafficFlow {
        from: "b".to_string(),
        to: "c".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    assert_eq!(view.max_volume(), 0);
}

#[test]
fn max_volume_very_large() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 1,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    view.add_flow(TrafficFlow {
        from: "b".to_string(),
        to: "c".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: u64::MAX,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    assert_eq!(view.max_volume(), u64::MAX);
}

#[test]
fn primal_lane_layout_one_primal() {
    let layout = primal_lane_layout(1, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 1);
    let (y, left_x, right_x) = layout[0];
    assert!((y - 300.0).abs() < f32::EPSILON);
    assert!((left_x - 80.0).abs() < f32::EPSILON);
    assert!((right_x - 720.0).abs() < f32::EPSILON);
}

#[test]
fn primal_lane_layout_twenty_primals() {
    let layout = primal_lane_layout(20, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 20);
    let node_height = (600.0 - 40.0) / 20.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 20.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 80.0).abs() < f32::EPSILON);
        assert!((right_x - 720.0).abs() < f32::EPSILON);
    }
}

#[test]
fn primal_lane_layout_very_narrow_rect() {
    let layout = primal_lane_layout(3, 0.0, 0.0, 100.0, 50.0, 5.0, 30.0);
    assert_eq!(layout.len(), 3);
    let node_height = (50.0 - 10.0) / 3.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 5.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 20.0).abs() < f32::EPSILON);
        assert!((right_x - 80.0).abs() < f32::EPSILON);
    }
}
