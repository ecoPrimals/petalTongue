// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn gauge_status_normal() {
    assert_eq!(
        gauge_status_for_value(50.0, &[0.0, 100.0], &[0.0, 100.0]),
        GaugeStatus::Normal
    );
    assert_eq!(
        gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
    assert_eq!(
        gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn gauge_status_warning() {
    assert_eq!(
        gauge_status_for_value(105.0, &[0.0, 100.0], &[100.0, 120.0]),
        GaugeStatus::Warning
    );
    assert_eq!(
        gauge_status_for_value(-5.0, &[0.0, 100.0], &[-10.0, 0.0]),
        GaugeStatus::Warning
    );
}

#[test]
fn gauge_status_critical() {
    assert_eq!(
        gauge_status_for_value(150.0, &[0.0, 100.0], &[100.0, 120.0]),
        GaugeStatus::Critical
    );
    assert_eq!(
        gauge_status_for_value(-20.0, &[0.0, 100.0], &[-10.0, 0.0]),
        GaugeStatus::Critical
    );
}

#[test]
fn distribution_bins_spread() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 2.5, 3.5];
    let result = distribution_bins(&values, 5);
    let (lo, hi, counts) = result.unwrap();
    assert!((lo - 1.0).abs() < 1e-10);
    assert!((hi - 5.0).abs() < 1e-10);
    assert_eq!(counts.len(), 5);
    assert_eq!(counts.iter().sum::<u32>(), 7);
}

#[test]
fn distribution_bins_no_spread() {
    let values = vec![42.0, 42.0, 42.0];
    let result = distribution_bins(&values, 10);
    assert!(result.is_none());
}

#[test]
fn distribution_bins_empty() {
    let values: Vec<f64> = vec![];
    let result = distribution_bins(&values, 10);
    assert!(result.is_none());
}

#[test]
fn distribution_bins_zero_bins() {
    let values = vec![1.0, 2.0, 3.0];
    let result = distribution_bins(&values, 0);
    assert!(result.is_none());
}

#[test]
fn distribution_bins_single_bin() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = distribution_bins(&values, 1);
    let (lo, hi, counts) = result.unwrap();
    assert!((lo - 1.0).abs() < 1e-10);
    assert!((hi - 5.0).abs() < 1e-10);
    assert_eq!(counts.len(), 1);
    assert_eq!(counts[0], 5);
}

#[test]
fn distribution_bins_boundary_values() {
    let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]; // 5.0 at boundary
    let result = distribution_bins(&values, 5);
    let (_lo, _hi, counts) = result.unwrap();
    assert_eq!(counts.iter().sum::<u32>(), 7);
    assert!(counts[counts.len() - 1] >= 1); // last bin gets 5.0 values
}

#[test]
fn gauge_status_boundary_normal() {
    assert_eq!(
        gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
    assert_eq!(
        gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn gauge_status_boundary_warning() {
    assert_eq!(
        gauge_status_for_value(100.0, &[0.0, 99.0], &[99.0, 120.0]),
        GaugeStatus::Warning
    );
    assert_eq!(
        gauge_status_for_value(0.0, &[1.0, 100.0], &[-10.0, 1.0]),
        GaugeStatus::Warning
    );
}

#[test]
fn gauge_status_enum_variants() {
    assert_eq!(GaugeStatus::Normal, GaugeStatus::Normal);
    assert_ne!(GaugeStatus::Normal, GaugeStatus::Warning);
    assert_ne!(GaugeStatus::Warning, GaugeStatus::Critical);
    assert_ne!(GaugeStatus::Normal, GaugeStatus::Critical);
}

#[test]
fn node_detail_fields() {
    let node = NodeDetail {
        name: "Test".to_string(),
        health: 80,
        status: "Active".to_string(),
        capabilities: vec!["a.b.c".to_string(), "x".to_string()],
        data_bindings: vec![],
    };
    assert_eq!(node.name, "Test");
    assert_eq!(node.health, 80);
    assert_eq!(node.status, "Active");
    assert_eq!(node.capabilities.len(), 2);
    assert_eq!(node.capabilities[0], "a.b.c");
    assert_eq!(node.capabilities[1], "x");
}

#[test]
fn node_detail_capability_short_name_logic() {
    // Test the rsplit('.').next().unwrap_or(cap) logic used in draw_node_detail
    fn short(cap: &str) -> String {
        cap.rsplit('.').next().unwrap_or(cap).to_string()
    }
    assert_eq!(short("ui.render"), "render");
    assert_eq!(short("ui.graph"), "graph");
    assert_eq!(short("a.b.c"), "c");
    assert_eq!(short("no-dots"), "no-dots");
    assert_eq!(short(""), "");
}

#[test]
fn bar_chart_category_fallback() {
    // Test categories.get(i).map_or("?", String::as_str) logic
    let categories = ["A".to_string(), "B".to_string()];
    let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
    assert_eq!(name_for(0), "A");
    assert_eq!(name_for(1), "B");
    assert_eq!(name_for(2), "?");
}

#[test]
fn bar_chart_empty_categories() {
    let categories: Vec<String> = vec![];
    let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
    assert_eq!(name_for(0), "?");
}

#[test]
fn distribution_bins_infinity_returns_none() {
    let values = vec![1.0, f64::INFINITY, 3.0];
    let result = distribution_bins(&values, 5);
    assert!(result.is_none());
}

#[test]
fn distribution_bins_value_exceeds_bin_index() {
    // Values that could produce idx >= n_bins before min()
    let values = vec![0.0, 0.1, 99.0, 100.0, 1000.0];
    let result = distribution_bins(&values, 5);
    let (lo, hi, counts) = result.expect("should have bins");
    assert!(lo.is_finite());
    assert!(hi.is_finite());
    assert_eq!(counts.len(), 5);
    assert_eq!(counts.iter().sum::<u32>(), 5);
}

#[test]
fn gauge_status_value_at_normal_boundary_low() {
    assert_eq!(
        gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn gauge_status_value_at_normal_boundary_high() {
    assert_eq!(
        gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn node_detail_default() {
    let node = NodeDetail::default();
    assert!(node.name.is_empty());
    assert_eq!(node.health, 0);
    assert!(node.status.is_empty());
    assert!(node.capabilities.is_empty());
    assert!(node.data_bindings.is_empty());
}

#[test]
fn node_detail_capability_rsplit_dot() {
    fn short(cap: &str) -> &str {
        cap.rsplit('.').next().unwrap_or(cap)
    }
    assert_eq!(short("a.b.c.d"), "d");
    assert_eq!(short("single"), "single");
}

#[test]
fn distribution_bins_negative_values() {
    let values = vec![-10.0, -5.0, 0.0, 5.0, 10.0];
    let result = distribution_bins(&values, 5);
    let (lo, hi, counts) = result.expect("negative range valid");
    assert!((lo - (-10.0)).abs() < f64::EPSILON);
    assert!((hi - 10.0).abs() < f64::EPSILON);
    assert_eq!(counts.iter().sum::<u32>(), 5);
}

#[test]
fn gauge_status_value_at_warning_boundary() {
    assert_eq!(
        gauge_status_for_value(120.0, &[0.0, 100.0], &[100.0, 120.0]),
        GaugeStatus::Warning
    );
    assert_eq!(
        gauge_status_for_value(-10.0, &[0.0, 100.0], &[-10.0, 0.0]),
        GaugeStatus::Warning
    );
}

#[test]
fn gauge_status_value_at_critical_boundary() {
    assert_eq!(
        gauge_status_for_value(121.0, &[0.0, 100.0], &[100.0, 120.0]),
        GaugeStatus::Critical
    );
}

#[test]
fn distribution_bins_single_data_point_returns_none() {
    let values = vec![42.0];
    let result = distribution_bins(&values, 5);
    assert!(
        result.is_none(),
        "single point has zero range, bin_width <= 0"
    );
}

#[test]
fn distribution_bins_large_dataset() {
    let values: Vec<f64> = (0..1000).map(|i| f64::from(i) / 10.0).collect();
    let result = distribution_bins(&values, 20);
    let (lo, hi, counts) = result.expect("large dataset valid");
    assert!((lo - 0.0).abs() < f64::EPSILON);
    assert!((hi - 99.9).abs() < 0.01);
    assert_eq!(counts.len(), 20);
    assert_eq!(counts.iter().sum::<u32>(), 1000);
}

#[test]
fn distribution_bins_with_nan_uses_finite_range() {
    let values = vec![1.0, f64::NAN, 3.0];
    let result = distribution_bins(&values, 5);
    assert!(
        result.is_some(),
        "min/max ignores NaN, range 1..3 is finite"
    );
    let (lo, hi, counts) = result.unwrap();
    assert!((lo - 1.0).abs() < f64::EPSILON);
    assert!((hi - 3.0).abs() < f64::EPSILON);
    assert_eq!(counts.iter().sum::<u32>(), 3);
}

#[test]
fn distribution_bins_neg_infinity_returns_none() {
    let values = vec![1.0, f64::NEG_INFINITY, 3.0];
    let result = distribution_bins(&values, 5);
    assert!(result.is_none());
}

#[test]
fn gauge_status_value_at_min_boundary() {
    assert_eq!(
        gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn gauge_status_value_at_max_boundary() {
    assert_eq!(
        gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn bar_chart_values_more_than_categories() {
    let categories: Vec<String> = ["A".to_string(), "B".to_string()].into();
    let values: Vec<f64> = [1.0, 2.0, 3.0].into();
    assert!(values.len() > categories.len());
    let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
    assert_eq!(name_for(0), "A");
    assert_eq!(name_for(1), "B");
    assert_eq!(name_for(2), "?");
}

#[test]
fn bar_chart_empty_values() {
    let categories: Vec<String> = vec![];
    let _values: Vec<f64> = vec![];
    let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
    assert_eq!(name_for(0), "?");
}

#[test]
fn bar_chart_single_bar() {
    let categories: Vec<String> = ["Only".to_string()].into();
    let values: Vec<f64> = [42.0].into();
    let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
    assert_eq!(name_for(0), "Only");
    assert!((values[0] - 42.0).abs() < f64::EPSILON);
}

#[test]
fn bar_chart_negative_values() {
    let values = vec![-5.0, 0.0, 5.0];
    let result = distribution_bins(&values, 3);
    let (lo, hi, counts) = result.expect("negative range valid");
    assert!((lo - (-5.0)).abs() < f64::EPSILON);
    assert!((hi - 5.0).abs() < f64::EPSILON);
    assert_eq!(counts.iter().sum::<u32>(), 3);
}

#[test]
fn node_detail_clone() {
    let node = NodeDetail {
        name: "x".to_string(),
        health: 50,
        status: "ok".to_string(),
        capabilities: vec!["a".to_string()],
        data_bindings: vec![],
    };
    let cloned = node.clone();
    assert_eq!(cloned.name, node.name);
    assert_eq!(cloned.health, node.health);
}

#[test]
fn gauge_status_overlapping_ranges_normal_takes_precedence() {
    assert_eq!(
        gauge_status_for_value(50.0, &[0.0, 100.0], &[0.0, 100.0]),
        GaugeStatus::Normal
    );
}

#[test]
fn distribution_bins_bar_center_formula() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let (lo, hi, counts) = distribution_bins(&values, 5).expect("valid");
    let bin_width = (hi - lo) / 5.0;
    for (i, &c) in counts.iter().enumerate() {
        if c > 0 {
            let center = (i as f64 + 0.5).mul_add(bin_width, lo);
            assert!(center >= lo && center <= hi);
        }
    }
}

#[test]
fn gauge_frac_computation() {
    let frac = |value: f64, min: f64, max: f64| ((value - min) / (max - min)).clamp(0.0, 1.0);
    assert!((frac(50.0, 0.0, 100.0) - 0.5).abs() < f64::EPSILON);
    assert!((frac(0.0, 0.0, 100.0) - 0.0).abs() < f64::EPSILON);
    assert!((frac(100.0, 0.0, 100.0) - 1.0).abs() < f64::EPSILON);
    assert!(frac(-10.0, 0.0, 100.0) <= 0.0);
    assert!(frac(150.0, 0.0, 100.0) >= 1.0);
}

#[test]
fn gauge_normal_range_fraction() {
    let normal_left =
        |n0: f64, _n1: f64, min: f64, max: f64| ((n0 - min) / (max - min)).clamp(0.0, 1.0);
    let normal_right =
        |_n0: f64, n1: f64, min: f64, max: f64| ((n1 - min) / (max - min)).clamp(0.0, 1.0);
    let min = 0.0;
    let max = 100.0;
    let nl = normal_left(20.0, 80.0, min, max);
    let nr = normal_right(20.0, 80.0, min, max);
    assert!((nl - 0.2).abs() < f64::EPSILON);
    assert!((nr - 0.8).abs() < f64::EPSILON);
}

#[test]
fn distribution_bins_idx_floor() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let (lo, hi, counts) = distribution_bins(&values, 5).expect("valid");
    let bin_width = (hi - lo) / 5.0;
    let idx_for = |v: f64| {
        let idx = ((v - lo) / bin_width).floor() as usize;
        idx.min(4)
    };
    assert_eq!(idx_for(1.0), 0);
    assert_eq!(idx_for(2.0), 1);
    assert_eq!(idx_for(3.0), 2);
    assert_eq!(idx_for(4.0), 3);
    assert_eq!(idx_for(5.0), 4);
    assert_eq!(counts.iter().sum::<u32>(), 5, "all values must be binned");
}

#[test]
fn gauge_status_warning_at_upper_bound() {
    assert_eq!(
        gauge_status_for_value(120.0, &[0.0, 100.0], &[100.0, 120.0]),
        GaugeStatus::Warning
    );
}

#[test]
fn gauge_status_critical_above_warning() {
    assert_eq!(
        gauge_status_for_value(200.0, &[0.0, 100.0], &[100.0, 150.0]),
        GaugeStatus::Critical
    );
}

#[test]
fn distribution_bins_nan_in_values() {
    let values = vec![1.0, f64::NAN, 5.0];
    let result = distribution_bins(&values, 3);
    assert!(result.is_some());
    let (lo, hi, counts) = result.unwrap();
    assert!((lo - 1.0).abs() < f64::EPSILON);
    assert!((hi - 5.0).abs() < f64::EPSILON);
    assert_eq!(counts.iter().sum::<u32>(), 3);
}

#[test]
fn gauge_status_normal_inner() {
    assert_eq!(
        gauge_status_for_value(50.0, &[20.0, 80.0], &[10.0, 90.0]),
        GaugeStatus::Normal
    );
}
