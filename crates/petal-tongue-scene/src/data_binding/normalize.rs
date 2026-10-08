// SPDX-License-Identifier: AGPL-3.0-or-later
//! Value normalization strategies for data bindings.
//!
//! Applied **before** the grammar compiler so that multi-metric panels
//! (e.g., IPs=199, UA Pool=12, Blame%=45) render with comparable scales.

use petal_tongue_types::Normalization;

/// Apply a normalization strategy to a slice of values **in-place**.
///
/// For column-wise normalization (faceted bars, heatmaps), call this once
/// per column with the values from that column across all rows/groups.
pub fn apply_normalization(values: &mut [f64], norm: &Normalization) {
    if values.is_empty() {
        return;
    }
    match norm {
        Normalization::None => {}
        Normalization::MinMax => normalize_min_max(values),
        Normalization::Symmetric => normalize_symmetric(values),
        Normalization::ZScore => normalize_z_score(values),
        Normalization::Log1p => normalize_log1p(values),
        Normalization::Rank => normalize_rank(values),
        Normalization::LeakyRelu { alpha } => normalize_leaky_relu(values, *alpha),
    }
}

/// `[0, 1]` — `(x - min) / (max - min)`
fn normalize_min_max(values: &mut [f64]) {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    if range.abs() < f64::EPSILON {
        // All values equal → map to 0.5 (midpoint)
        for v in values.iter_mut() {
            *v = 0.5;
        }
    } else {
        for v in values.iter_mut() {
            *v = (*v - min) / range;
        }
    }
}

/// `[-1, 1]` — `2 * (x - min) / (max - min) - 1`
fn normalize_symmetric(values: &mut [f64]) {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    if range.abs() < f64::EPSILON {
        for v in values.iter_mut() {
            *v = 0.0;
        }
    } else {
        for v in values.iter_mut() {
            *v = 2.0 * (*v - min) / range - 1.0;
        }
    }
}

/// Z-score: `(x - mean) / std`
fn normalize_z_score(values: &mut [f64]) {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    if std < f64::EPSILON {
        for v in values.iter_mut() {
            *v = 0.0;
        }
    } else {
        for v in values.iter_mut() {
            *v = (*v - mean) / std;
        }
    }
}

/// Log1p: `ln(1 + |x|) * sign(x)`
fn normalize_log1p(values: &mut [f64]) {
    for v in values.iter_mut() {
        *v = (1.0 + v.abs()).ln() * v.signum();
    }
}

/// Ordinal rank normalized to `[0, 1]`.
fn normalize_rank(values: &mut [f64]) {
    let n = values.len();
    if n <= 1 {
        for v in values.iter_mut() {
            *v = 0.5;
        }
        return;
    }
    // Build (index, value) sorted by value
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    // Assign ranks (average tied ranks)
    let mut ranks = vec![0.0_f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < f64::EPSILON {
            j += 1;
        }
        // Average rank for tied group
        #[expect(clippy::cast_precision_loss, reason = "rank indices")]
        let avg_rank = (i + j - 1) as f64 / 2.0;
        for item in &indexed[i..j] {
            ranks[item.0] = avg_rank;
        }
        i = j;
    }
    // Normalize to [0, 1]
    #[expect(clippy::cast_precision_loss, reason = "rank normalization")]
    let denom = (n - 1) as f64;
    for (idx, v) in values.iter_mut().enumerate() {
        *v = if denom > 0.0 { ranks[idx] / denom } else { 0.5 };
    }
}

/// Leaky ReLU: `max(α·x, x)`
fn normalize_leaky_relu(values: &mut [f64], alpha: f64) {
    for v in values.iter_mut() {
        if *v < 0.0 {
            *v *= alpha;
        }
    }
}

/// Normalize faceted bar data per-category across groups.
///
/// Given groups where each group has the same categories, this normalizes
/// each category column independently. For example, with categories
/// `["IPs", "UA Pool", "Blame%"]`, all IP values across groups are
/// normalized together, all UA Pool values together, etc.
///
/// Returns the normalized values as a `Vec<Vec<f64>>` (one inner vec per group).
pub fn normalize_faceted_columns(
    groups: &[petal_tongue_types::FacetGroup],
    norm: &Normalization,
) -> Vec<Vec<f64>> {
    if groups.is_empty() || *norm == Normalization::None {
        return groups.iter().map(|g| g.values.clone()).collect();
    }
    let n_cats = groups.first().map_or(0, |g| g.categories.len());
    let n_groups = groups.len();

    // Start with a copy of all values
    let mut result: Vec<Vec<f64>> = groups.iter().map(|g| g.values.clone()).collect();

    // Normalize each category column independently
    for col in 0..n_cats {
        let mut column: Vec<f64> = result
            .iter()
            .map(|row| row.get(col).copied().unwrap_or(0.0))
            .collect();
        apply_normalization(&mut column, norm);
        for (row_idx, val) in column.into_iter().enumerate().take(n_groups) {
            if col < result[row_idx].len() {
                result[row_idx][col] = val;
            }
        }
    }
    result
}

/// Normalize heatmap values per-column across rows.
///
/// `values` is a flat row-major array with `n_cols` columns.
/// Each column is normalized independently.
pub fn normalize_heatmap_columns(values: &mut [f64], n_cols: usize, norm: &Normalization) {
    if n_cols == 0 || *norm == Normalization::None {
        return;
    }
    let n_rows = values.len() / n_cols;
    for col in 0..n_cols {
        let mut column: Vec<f64> = (0..n_rows)
            .map(|row| values.get(row * n_cols + col).copied().unwrap_or(0.0))
            .collect();
        apply_normalization(&mut column, norm);
        for (row, val) in column.into_iter().enumerate() {
            if let Some(cell) = values.get_mut(row * n_cols + col) {
                *cell = val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_max_basic() {
        let mut vals = vec![10.0, 20.0, 30.0];
        apply_normalization(&mut vals, &Normalization::MinMax);
        assert!((vals[0] - 0.0).abs() < 1e-9);
        assert!((vals[1] - 0.5).abs() < 1e-9);
        assert!((vals[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn min_max_all_equal() {
        let mut vals = vec![5.0, 5.0, 5.0];
        apply_normalization(&mut vals, &Normalization::MinMax);
        assert!(vals.iter().all(|v| (*v - 0.5).abs() < 1e-9));
    }

    #[test]
    fn symmetric_basic() {
        let mut vals = vec![0.0, 50.0, 100.0];
        apply_normalization(&mut vals, &Normalization::Symmetric);
        assert!((vals[0] - (-1.0)).abs() < 1e-9);
        assert!((vals[1] - 0.0).abs() < 1e-9);
        assert!((vals[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn z_score_basic() {
        let mut vals = vec![2.0, 4.0, 6.0];
        apply_normalization(&mut vals, &Normalization::ZScore);
        let sum: f64 = vals.iter().sum();
        assert!(sum.abs() < 1e-9, "z-score mean should be ~0, got {sum}");
    }

    #[test]
    fn log1p_positive() {
        let mut vals = vec![0.0, 99.0, 9999.0];
        apply_normalization(&mut vals, &Normalization::Log1p);
        assert!((vals[0] - 0.0).abs() < 1e-9);
        assert!(vals[1] > 0.0 && vals[1] < 99.0);
        assert!(vals[2] > vals[1]);
    }

    #[test]
    fn log1p_negative() {
        let mut vals = vec![-10.0, 0.0, 10.0];
        apply_normalization(&mut vals, &Normalization::Log1p);
        assert!(vals[0] < 0.0);
        assert!((vals[1] - 0.0).abs() < 1e-9);
        assert!(vals[2] > 0.0);
        assert!((vals[0].abs() - vals[2]).abs() < 1e-9, "symmetric around zero");
    }

    #[test]
    fn rank_basic() {
        let mut vals = vec![100.0, 1.0, 50.0];
        apply_normalization(&mut vals, &Normalization::Rank);
        assert!((vals[0] - 1.0).abs() < 1e-9);
        assert!((vals[1] - 0.0).abs() < 1e-9);
        assert!((vals[2] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn rank_tied() {
        let mut vals = vec![10.0, 10.0, 20.0];
        apply_normalization(&mut vals, &Normalization::Rank);
        assert!((vals[0] - vals[1]).abs() < 1e-9, "tied values get same rank");
        assert!(vals[2] > vals[0]);
    }

    #[test]
    fn leaky_relu_basic() {
        let mut vals = vec![-10.0, 0.0, 10.0];
        apply_normalization(&mut vals, &Normalization::LeakyRelu { alpha: 0.1 });
        assert!((vals[0] - (-1.0)).abs() < 1e-9);
        assert!((vals[1] - 0.0).abs() < 1e-9);
        assert!((vals[2] - 10.0).abs() < 1e-9);
    }

    #[test]
    fn faceted_columns_min_max() {
        let groups = vec![
            petal_tongue_types::FacetGroup {
                key: "Meta".into(),
                categories: vec!["IPs".into(), "UA".into()],
                values: vec![199.0, 12.0],
            },
            petal_tongue_types::FacetGroup {
                key: "Self".into(),
                categories: vec!["IPs".into(), "UA".into()],
                values: vec![1.0, 1.0],
            },
        ];
        let result = normalize_faceted_columns(&groups, &Normalization::MinMax);
        // IPs: 199→1.0, 1→0.0; UA: 12→1.0, 1→0.0
        assert!((result[0][0] - 1.0).abs() < 1e-9);
        assert!((result[1][0] - 0.0).abs() < 1e-9);
        assert!((result[0][1] - 1.0).abs() < 1e-9);
        assert!((result[1][1] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn heatmap_columns_z_score() {
        // 2 rows × 3 cols
        let mut vals = vec![10.0, 100.0, 1.0, 30.0, 200.0, 3.0];
        normalize_heatmap_columns(&mut vals, 3, &Normalization::ZScore);
        // Column 0: [10, 30], Column 1: [100, 200], Column 2: [1, 3]
        // Each column should sum to ~0 (z-score property)
        assert!((vals[0] + vals[3]).abs() < 1e-9);
        assert!((vals[1] + vals[4]).abs() < 1e-9);
        assert!((vals[2] + vals[5]).abs() < 1e-9);
    }

    #[test]
    fn none_is_identity() {
        let mut vals = vec![1.0, 2.0, 3.0];
        let orig = vals.clone();
        apply_normalization(&mut vals, &Normalization::None);
        assert_eq!(vals, orig);
    }
}
