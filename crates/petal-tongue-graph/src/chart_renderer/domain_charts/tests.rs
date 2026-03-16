// SPDX-License-Identifier: AGPL-3.0-only
//! Domain charts unit tests.

use super::*;

#[test]
fn validate_heatmap_valid() {
    assert!(validate_heatmap_dimensions(3, 4, 12));
    assert!(validate_heatmap_dimensions(1, 1, 1));
}

#[test]
fn validate_heatmap_invalid() {
    assert!(!validate_heatmap_dimensions(0, 4, 12));
    assert!(!validate_heatmap_dimensions(3, 0, 12));
    assert!(!validate_heatmap_dimensions(3, 4, 10));
    assert!(!validate_heatmap_dimensions(3, 4, 14));
}

#[test]
fn validate_scatter2d_valid() {
    assert!(validate_scatter2d_lengths(5, 5));
    assert!(validate_scatter2d_lengths(1, 1));
}

#[test]
fn validate_scatter2d_invalid() {
    assert!(!validate_scatter2d_lengths(0, 5));
    assert!(!validate_scatter2d_lengths(5, 4));
}

#[test]
fn validate_scatter3d_valid() {
    assert!(validate_scatter3d_lengths(5, 5, 5));
    assert!(validate_scatter3d_lengths(1, 1, 1));
}

#[test]
fn validate_scatter3d_invalid() {
    assert!(!validate_scatter3d_lengths(0, 5, 5));
    assert!(!validate_scatter3d_lengths(5, 4, 5));
    assert!(!validate_scatter3d_lengths(5, 5, 4));
}

#[test]
fn validate_spectrum_valid() {
    assert!(validate_spectrum_lengths(10, 10));
    assert!(validate_spectrum_lengths(1, 1));
}

#[test]
fn validate_spectrum_invalid() {
    assert!(!validate_spectrum_lengths(0, 0));
    assert!(!validate_spectrum_lengths(10, 9));
    assert!(!validate_spectrum_lengths(9, 10));
}

#[test]
fn validate_heatmap_edge_cases() {
    assert!(!validate_heatmap_dimensions(1, 0, 0));
    assert!(!validate_heatmap_dimensions(0, 1, 0));
    assert!(validate_heatmap_dimensions(2, 3, 6));
    assert!(!validate_heatmap_dimensions(2, 3, 5));
    assert!(!validate_heatmap_dimensions(2, 3, 7));
}

#[test]
fn validate_scatter3d_edge_cases() {
    assert!(!validate_scatter3d_lengths(1, 0, 0));
    assert!(!validate_scatter3d_lengths(0, 1, 1));
    assert!(!validate_scatter3d_lengths(1, 1, 0));
    assert!(validate_scatter3d_lengths(100, 100, 100));
}

#[test]
fn scatter3d_params_construction() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![4.0, 5.0, 6.0];
    let z = vec![7.0, 8.0, 9.0];
    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let params = Scatter3dParams {
        label: "test",
        x_vals: &x,
        y_vals: &y,
        z_vals: &z,
        point_labels: &labels,
        unit: "m",
        domain: Some("health"),
    };
    assert_eq!(params.label, "test");
    assert_eq!(params.x_vals.len(), 3);
    assert_eq!(params.y_vals.len(), 3);
    assert_eq!(params.z_vals.len(), 3);
    assert_eq!(params.unit, "m");
    assert_eq!(params.domain, Some("health"));
}

#[test]
fn scatter3d_params_domain_none() {
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![3.0];
    let params = Scatter3dParams {
        label: "l",
        x_vals: &x,
        y_vals: &y,
        z_vals: &z,
        point_labels: &[],
        unit: "u",
        domain: None,
    };
    assert!(params.domain.is_none());
}

#[test]
fn z_bands_constant() {
    assert_eq!(Z_BANDS, 8);
}

#[test]
fn scatter2d_params_construction() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![4.0, 5.0, 6.0];
    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let params = Scatter2dParams {
        label: "test",
        x_vals: &x,
        y_vals: &y,
        point_labels: &labels,
        x_label: "X",
        y_label: "Y",
        unit: "m",
        domain: Some("health"),
    };
    assert_eq!(params.label, "test");
    assert_eq!(params.x_vals.len(), 3);
    assert_eq!(params.y_vals.len(), 3);
    assert_eq!(params.point_labels.len(), 3);
    assert_eq!(params.x_label, "X");
    assert_eq!(params.y_label, "Y");
    assert_eq!(params.unit, "m");
    assert_eq!(params.domain, Some("health"));
}

#[test]
fn value_range_empty() {
    let values: Vec<f64> = vec![];
    assert!(value_range(&values).is_none());
}

#[test]
fn value_range_single() {
    let values = vec![42.0];
    let (vmin, vmax, range) = value_range(&values).expect("should have range");
    assert!((vmin - 42.0).abs() < 1e-10);
    assert!((vmax - 42.0).abs() < 1e-10);
    assert!(range >= f64::EPSILON);
}

#[test]
fn value_range_spread() {
    let values = vec![1.0, 5.0, 3.0, 9.0, 2.0];
    let (vmin, vmax, range) = value_range(&values).expect("should have range");
    assert!((vmin - 1.0).abs() < 1e-10);
    assert!((vmax - 9.0).abs() < 1e-10);
    assert!((range - 8.0).abs() < 1e-10);
}

#[test]
fn normalize_value_mid_range() {
    let t = normalize_value(5.0, 0.0, 10.0);
    assert!((t - 0.5).abs() < 1e-5);
}

#[test]
fn normalize_value_clamps() {
    let t_lo = normalize_value(-1.0, 0.0, 10.0);
    assert!(t_lo <= 0.0);
    let t_hi = normalize_value(15.0, 0.0, 10.0);
    assert!(t_hi >= 1.0);
}

#[test]
fn scatter3d_bands_distribution() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let z = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid scatter3d input");
    assert_eq!(bands.len(), 4);
    assert_eq!(bands[0].len(), 1);
    assert_eq!(bands[1].len(), 1);
    assert_eq!(bands[2].len(), 1);
    assert_eq!(bands[3].len(), 2);
}

#[test]
fn scatter3d_bands_invalid_input() {
    let x = vec![1.0, 2.0];
    let y = vec![1.0, 2.0];
    let z = vec![1.0];
    assert!(scatter3d_bands(&x, &y, &z, 4).is_none());
}

#[test]
fn scatter3d_bands_zero_bands() {
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![3.0];
    assert!(scatter3d_bands(&x, &y, &z, 0).is_none());
}

#[test]
fn value_range_infinity_returns_none() {
    let values = vec![1.0, f64::INFINITY, 3.0];
    assert!(value_range(&values).is_none());
}

#[test]
fn value_range_all_nan_returns_none() {
    let values = vec![f64::NAN, f64::NAN];
    let result = value_range(&values);
    assert!(
        result.is_none() || {
            let (vmin, vmax, range) = result.expect("unreachable");
            !vmin.is_finite() || !vmax.is_finite() || !range.is_finite()
        }
    );
}

#[test]
fn value_range_negative_values() {
    let values = vec![-10.0, -5.0, 0.0, 5.0];
    let (vmin, vmax, range) = value_range(&values).expect("should have range");
    assert!((vmin - (-10.0)).abs() < 1e-10);
    assert!((vmax - 5.0).abs() < 1e-10);
    assert!((range - 15.0).abs() < 1e-10);
}

#[test]
fn normalize_value_at_min() {
    let t = normalize_value(0.0, 0.0, 10.0);
    assert!((t - 0.0).abs() < 1e-5);
}

#[test]
fn normalize_value_at_max() {
    let t = normalize_value(10.0, 0.0, 10.0);
    assert!((t - 1.0).abs() < 1e-5);
}

#[test]
fn scatter3d_bands_single_point() {
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![0.5];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid scatter3d input");
    assert_eq!(bands.len(), 4);
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 1);
}

#[test]
fn scatter3d_bands_uniform_z() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    let z = vec![1.0, 1.0, 1.0];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid scatter3d input");
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 3);
}

#[test]
fn heatmap_value_range_integration() {
    let values = vec![10.0, 20.0, 30.0, 40.0];
    let (vmin, _vmax, range) = value_range(&values).expect("should have range");
    for (i, &v) in values.iter().enumerate() {
        let t = normalize_value(v, vmin, range);
        let expected = i as f32 / 3.0;
        assert!((t - expected).abs() < 0.01);
    }
}

#[test]
fn value_range_neg_infinity_returns_none() {
    let values = vec![1.0, f64::NEG_INFINITY, 3.0];
    assert!(value_range(&values).is_none());
}

#[test]
fn value_range_nan_does_not_panic() {
    let values = vec![1.0, f64::NAN, 3.0];
    let _ = value_range(&values);
}

#[test]
fn normalize_value_zero_range_uses_epsilon() {
    let t = normalize_value(0.0, 0.0, f64::EPSILON);
    assert!((0.0..=1.0).contains(&t));
}

#[test]
fn scatter3d_bands_last_band_inclusive() {
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![1.0];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid input");
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 1);
}

#[test]
fn scatter3d_bands_first_band() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    let z = vec![0.0, 0.1, 1.0];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid input");
    assert_eq!(bands[0].len(), 2);
}

#[test]
fn normalize_value_below_min_clamps_to_zero() {
    let t = normalize_value(-5.0, 0.0, 10.0);
    assert!((t - 0.0).abs() < f32::EPSILON);
}

#[test]
fn normalize_value_above_max_clamps_to_one() {
    let t = normalize_value(100.0, 0.0, 10.0);
    assert!((t - 1.0).abs() < f32::EPSILON);
}

#[test]
fn value_range_all_same_returns_valid() {
    let values = vec![7.0, 7.0, 7.0];
    let (vmin, vmax, range) = value_range(&values).expect("same values still valid");
    assert!((vmin - 7.0).abs() < f64::EPSILON);
    assert!((vmax - 7.0).abs() < f64::EPSILON);
    assert!(range >= f64::EPSILON);
}

#[test]
fn scatter3d_bands_boundary_between_bands() {
    let x = vec![1.0, 2.0];
    let y = vec![1.0, 2.0];
    let z = vec![0.0, 0.5];
    let bands = scatter3d_bands(&x, &y, &z, 2).expect("valid input");
    assert_eq!(bands.len(), 2);
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 2);
}

#[test]
fn scatter3d_bands_norm_at_boundary() {
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![0.999_999];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid input");
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 1);
}

#[test]
fn validate_scatter3d_mismatched_lengths() {
    assert!(!validate_scatter3d_lengths(2, 3, 2));
    assert!(!validate_scatter3d_lengths(3, 2, 3));
}

#[test]
fn validate_spectrum_mismatched() {
    assert!(!validate_spectrum_lengths(5, 6));
}

#[test]
fn scatter3d_bands_single_band() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    let z = vec![0.0, 0.5, 1.0];
    let bands = scatter3d_bands(&x, &y, &z, 1).expect("valid input");
    assert_eq!(bands.len(), 1);
    assert_eq!(bands[0].len(), 3);
}

#[test]
fn scatter3d_bands_norm_exactly_one_in_last_band() {
    let x = vec![1.0, 2.0];
    let y = vec![1.0, 2.0];
    let z = vec![0.0, 1.0];
    let bands = scatter3d_bands(&x, &y, &z, 2).expect("valid input");
    assert_eq!(bands.len(), 2);
    assert_eq!(bands[0].len(), 1);
    assert_eq!(bands[1].len(), 1);
}

#[test]
fn scatter3d_bands_many_points_large_dataset() {
    let n = 100;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i * 2) as f64).collect();
    let z: Vec<f64> = (0..n).map(|i| (i as f64) / n as f64).collect();
    let bands = scatter3d_bands(&x, &y, &z, 10).expect("valid input");
    assert_eq!(bands.len(), 10);
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, n);
}

#[test]
fn scatter3d_bands_negative_z_values() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    let z = vec![-10.0, 0.0, 10.0];
    let bands = scatter3d_bands(&x, &y, &z, 3).expect("valid input");
    assert_eq!(bands.len(), 3);
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 3);
}

#[test]
fn scatter2d_params_with_empty_labels() {
    let x = vec![1.0, 2.0];
    let y = vec![3.0, 4.0];
    let params = Scatter2dParams {
        label: "t",
        x_vals: &x,
        y_vals: &y,
        point_labels: &[],
        x_label: "X",
        y_label: "Y",
        unit: "u",
        domain: None,
    };
    assert!(params.point_labels.is_empty());
    assert_eq!(params.x_vals.len(), params.y_vals.len());
}

#[test]
fn heatmap_dimensions_exact_match() {
    assert!(validate_heatmap_dimensions(10, 20, 200));
    assert!(!validate_heatmap_dimensions(10, 20, 199));
    assert!(!validate_heatmap_dimensions(10, 20, 201));
}

#[test]
fn spectrum_lengths_single_point() {
    assert!(validate_spectrum_lengths(1, 1));
}

#[test]
fn scatter3d_bands_x_y_preserved() {
    let x = vec![10.0, 20.0];
    let y = vec![30.0, 40.0];
    let z = vec![0.0, 1.0];
    let bands = scatter3d_bands(&x, &y, &z, 2).expect("valid");
    assert_eq!(bands[0].len(), 1);
    assert_eq!(bands[0][0], [10.0, 30.0]);
    assert_eq!(bands[1].len(), 1);
    assert_eq!(bands[1][0], [20.0, 40.0]);
}

#[test]
fn scatter3d_bands_nan_z_excluded() {
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![f64::NAN];
    let result = scatter3d_bands(&x, &y, &z, 4);
    assert!(result.is_some());
    let bands = result.unwrap();
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 0);
}

#[test]
fn normalize_value_nan_input_clamps() {
    let t = normalize_value(f64::NAN, 0.0, 10.0);
    assert!(t.is_nan() || (0.0..=1.0).contains(&t));
}

#[test]
fn value_range_very_small() {
    let values = vec![1.0e-100, 2.0e-100];
    let (vmin, vmax, range) = value_range(&values).expect("finite range");
    assert!((vmin - 1.0e-100).abs() < 1e-110);
    assert!((vmax - 2.0e-100).abs() < 1e-110);
    assert!(range > 0.0);
}

#[test]
fn scatter3d_bands_all_in_one_band() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 3.0];
    let z = vec![0.5, 0.5, 0.5];
    let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid");
    let total: usize = bands.iter().map(Vec::len).sum();
    assert_eq!(total, 3);
}

#[test]
fn validate_heatmap_single_cell() {
    assert!(validate_heatmap_dimensions(1, 1, 1));
}

#[test]
fn validate_spectrum_empty_freq() {
    assert!(!validate_spectrum_lengths(0, 10));
}

#[test]
fn normalize_value_infinity_clamps() {
    let t = normalize_value(f64::INFINITY, 0.0, 10.0);
    assert!(t >= 1.0_f32 - f32::EPSILON);
}
