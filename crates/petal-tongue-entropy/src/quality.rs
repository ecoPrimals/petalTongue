//! Quality assessment algorithms

use std::collections::HashMap;
use std::time::Duration;

/// Calculate Shannon entropy of a discrete distribution
///
/// Shannon entropy: H(X) = -Σ p(x) * log₂(p(x))
///
/// # Arguments
///
/// * `values` - Slice of values to analyze
///
/// # Returns
///
/// Shannon entropy [0.0-1.0], normalized by log₂(n) where n is the number of unique values
pub fn shannon_entropy<T: std::hash::Hash + Eq + Clone>(values: &[T]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    // Count occurrences
    let mut counts: HashMap<T, usize> = HashMap::new();
    for value in values {
        *counts.entry(value.clone()).or_insert(0) += 1;
    }

    let total = values.len() as f64;
    let num_unique = counts.len() as f64;

    // Calculate entropy
    let entropy: f64 = counts
        .values()
        .map(|&count| {
            let p = count as f64 / total;
            -p * p.log2()
        })
        .sum();

    // Normalize by log₂(num_unique) to get [0.0-1.0] range
    if num_unique <= 1.0 {
        0.0
    } else {
        entropy / num_unique.log2()
    }
}

/// Calculate variance of a sequence of floating-point values
///
/// Variance: σ² = E[(X - μ)²]
///
/// # Arguments
///
/// * `values` - Slice of f64 values
///
/// # Returns
///
/// Variance as f64, normalized to [0.0-1.0] using sigmoid(σ/4)
pub fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    // Calculate mean
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

    // Calculate variance
    let var: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

    // Normalize using sigmoid(σ/4) to get [0.0-1.0]
    // Standard deviation / 4 gives good sensitivity
    let std_dev = var.sqrt();
    sigmoid(std_dev / 4.0)
}

/// Sigmoid function for normalization
///
/// sigmoid(x) = 1 / (1 + e^(-x))
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Create histogram buckets from floating-point values
///
/// # Arguments
///
/// * `values` - Slice of values
/// * `num_buckets` - Number of histogram buckets
///
/// # Returns
///
/// Vector of bucket indices
pub fn create_histogram_buckets(values: &[f64], num_buckets: usize) -> Vec<usize> {
    if values.is_empty() || num_buckets == 0 {
        return vec![];
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max - min).abs() < f64::EPSILON {
        // All values are the same
        return vec![0; values.len()];
    }

    values
        .iter()
        .map(|&v| {
            let normalized = (v - min) / (max - min);
            let bucket = (normalized * num_buckets as f64).floor() as usize;
            bucket.min(num_buckets - 1)
        })
        .collect()
}

/// Calculate timing entropy from durations
///
/// Analyzes inter-event intervals for natural human rhythm.
///
/// # Arguments
///
/// * `durations` - Slice of inter-event durations
///
/// # Returns
///
/// Timing entropy [0.0-1.0]
pub fn timing_entropy(durations: &[Duration]) -> f64 {
    if durations.len() < 2 {
        return 0.0;
    }

    // Convert to milliseconds
    let ms_values: Vec<u64> = durations.iter().map(|d| d.as_millis() as u64).collect();

    // Create histogram buckets (10 buckets for timing)
    let ms_f64: Vec<f64> = ms_values.iter().map(|&x| x as f64).collect();
    let buckets = create_histogram_buckets(&ms_f64, 10);

    // Calculate Shannon entropy of buckets
    shannon_entropy(&buckets)
}

/// Calculate weighted quality score
///
/// # Arguments
///
/// * `components` - Slice of (score, weight) tuples
///
/// # Returns
///
/// Weighted average [0.0-1.0]
pub fn weighted_quality(components: &[(f64, f64)]) -> f64 {
    if components.is_empty() {
        return 0.0;
    }

    let total_weight: f64 = components.iter().map(|(_, w)| w).sum();
    if total_weight == 0.0 {
        return 0.0;
    }

    components
        .iter()
        .map(|(score, weight)| score * weight)
        .sum::<f64>()
        / total_weight
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_shannon_entropy_uniform() {
        // Uniform distribution should have high entropy
        let values = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let entropy = shannon_entropy(&values);
        assert!(entropy > 0.9); // Close to 1.0
    }

    #[test]
    fn test_shannon_entropy_single_value() {
        // Single value should have zero entropy
        let values = vec![1, 1, 1, 1, 1];
        let entropy = shannon_entropy(&values);
        assert_relative_eq!(entropy, 0.0, epsilon = 0.01);
    }

    #[test]
    fn test_variance_zero() {
        let values = vec![1.0, 1.0, 1.0];
        let var = variance(&values);
        assert_relative_eq!(var, 0.5, epsilon = 0.1); // sigmoid(0) ≈ 0.5
    }

    #[test]
    fn test_variance_nonzero() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let var = variance(&values);
        assert!(var > 0.5); // Should be > 0.5 for spread data
    }

    #[test]
    fn test_histogram_buckets() {
        let values = vec![0.0, 0.5, 1.0, 1.5, 2.0];
        let buckets = create_histogram_buckets(&values, 2);
        assert_eq!(buckets.len(), 5);
        assert_eq!(buckets[0], 0); // 0.0 → bucket 0
        assert_eq!(buckets[4], 1); // 2.0 → bucket 1
    }

    #[test]
    fn test_timing_entropy() {
        let durations = vec![
            Duration::from_millis(100),
            Duration::from_millis(150),
            Duration::from_millis(200),
            Duration::from_millis(250),
        ];
        let entropy = timing_entropy(&durations);
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_weighted_quality() {
        let components = vec![
            (0.8, 1.0), // 80% with weight 1.0
            (0.6, 0.5), // 60% with weight 0.5
        ];
        let quality = weighted_quality(&components);
        // (0.8*1.0 + 0.6*0.5) / (1.0 + 0.5) = 1.1 / 1.5 ≈ 0.733
        assert_relative_eq!(quality, 0.733, epsilon = 0.01);
    }
}
