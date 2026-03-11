// SPDX-License-Identifier: AGPL-3.0-only
//! Data binding compiler utilities: histogram binning, threshold resolution.

use petal_tongue_core::ThresholdRange;

/// Resolve the threshold status for a value against a set of threshold ranges.
///
/// Returns "critical" > "warning" > "normal" > "unknown" (highest severity wins).
pub fn resolve_threshold_status(value: f64, thresholds: &[ThresholdRange]) -> &'static str {
    let mut best: Option<&'static str> = None;
    for t in thresholds {
        if value >= t.min && value <= t.max {
            let status = match t.status.as_str() {
                "critical" => Some("critical"),
                "warning" => Some("warning"),
                "normal" => Some("normal"),
                _ => continue,
            };
            best = match (best, status) {
                (Some("critical"), _) | (_, Some("critical")) => Some("critical"),
                (Some("warning"), _) | (_, Some("warning")) => Some("warning"),
                (_, s) => s.or(best),
            };
        }
    }
    best.unwrap_or("unknown")
}

/// Bin continuous values into a histogram.
pub fn histogram_bins(values: &[f64], n_bins: usize) -> (Vec<f64>, Vec<f64>) {
    if values.is_empty() || n_bins == 0 {
        return (vec![], vec![]);
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        return (vec![min], vec![values.len() as f64]);
    }
    let bin_width = (max - min) / n_bins as f64;
    let mut counts = vec![0.0_f64; n_bins];
    let mut centers = Vec::with_capacity(n_bins);
    for i in 0..n_bins {
        centers.push((i as f64 + 0.5).mul_add(bin_width, min));
    }
    for v in values {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "histogram bin index bounded by n_bins"
        )]
        let bin = ((*v - min) / bin_width).floor() as usize;
        let bin = bin.min(n_bins - 1);
        counts[bin] += 1.0;
    }
    (centers, counts)
}
