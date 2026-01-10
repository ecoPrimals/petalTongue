//! Comprehensive tests for quality assessment
//!
//! Tests verify Shannon entropy, variance, and histogram algorithms.

use petal_tongue_entropy::quality::{create_histogram_buckets, shannon_entropy, variance};

#[test]
fn test_shannon_entropy_uniform() {
    // Perfectly uniform distribution should have maximum entropy
    let values = vec![1, 2, 3, 4, 5];
    let entropy = shannon_entropy(&values);

    // Should be close to 1.0 (maximum for uniform distribution)
    assert!(
        entropy > 0.9,
        "Uniform distribution should have high entropy: {}",
        entropy
    );
}

#[test]
fn test_shannon_entropy_single_value() {
    // Single repeated value should have zero entropy
    let values = vec![1, 1, 1, 1, 1];
    let entropy = shannon_entropy(&values);

    assert_eq!(entropy, 0.0, "Single value should have zero entropy");
}

#[test]
fn test_shannon_entropy_empty() {
    let values: Vec<i32> = vec![];
    let entropy = shannon_entropy(&values);

    assert_eq!(entropy, 0.0, "Empty sequence should have zero entropy");
}

#[test]
fn test_shannon_entropy_binary() {
    // Binary distribution [0,1,0,1,0,1,0,1]
    let values = vec![0, 1, 0, 1, 0, 1, 0, 1];
    let entropy = shannon_entropy(&values);

    // Should be 1.0 for perfectly balanced binary
    assert!(
        (entropy - 1.0).abs() < 0.01,
        "Binary distribution entropy: {}",
        entropy
    );
}

#[test]
fn test_shannon_entropy_skewed() {
    // Skewed distribution should have lower entropy than uniform
    let values = vec![1, 1, 1, 1, 1, 2];
    let entropy = shannon_entropy(&values);

    assert!(
        entropy < 0.7,
        "Skewed distribution should have low entropy: {}",
        entropy
    );
}

#[test]
fn test_variance_zero() {
    // No variance (all same values)
    let values = vec![5.0, 5.0, 5.0, 5.0];
    let var = variance(&values);

    // Should be very close to 0.5 (sigmoid of 0)
    assert!(
        (var - 0.5).abs() < 0.1,
        "No variance should give ~0.5: {}",
        var
    );
}

#[test]
fn test_variance_high() {
    // High variance
    let values = vec![0.0, 100.0, 0.0, 100.0];
    let var = variance(&values);

    // Should be close to 1.0 (high variance after sigmoid)
    assert!(var > 0.9, "High variance should give high score: {}", var);
}

#[test]
fn test_variance_empty() {
    let values: Vec<f64> = vec![];
    let var = variance(&values);

    assert_eq!(var, 0.0, "Empty sequence should have zero variance");
}

#[test]
fn test_variance_single_value() {
    let values = vec![42.0];
    let var = variance(&values);

    assert_eq!(var, 0.0, "Single value should have zero variance");
}

#[test]
fn test_variance_two_values() {
    let values = vec![10.0, 20.0];
    let var = variance(&values);

    // Should have some variance
    assert!(var > 0.0, "Two different values should have variance");
}

#[test]
fn test_variance_normalization() {
    // Test that variance is normalized to [0,1] range
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let var = variance(&values);

    assert!(
        var >= 0.0 && var <= 1.0,
        "Variance should be normalized: {}",
        var
    );
}

#[test]
fn test_create_histogram_buckets_empty() {
    let values: Vec<f64> = vec![];
    let buckets = create_histogram_buckets(&values, 10);

    assert_eq!(buckets.len(), 0, "Empty values should give empty buckets");
}

#[test]
fn test_create_histogram_buckets_uniform() {
    let values = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    let buckets = create_histogram_buckets(&values, 4);

    assert_eq!(buckets.len(), 5, "Should create buckets for all values");

    // Check that buckets are in valid range [0, 3]
    for &bucket in &buckets {
        assert!(bucket < 4, "Bucket index should be < num_buckets");
    }
}

#[test]
fn test_create_histogram_buckets_single_value() {
    let values = vec![5.0, 5.0, 5.0];
    let buckets = create_histogram_buckets(&values, 10);

    // All should map to same bucket
    assert_eq!(buckets.len(), 3);
    assert_eq!(buckets[0], buckets[1]);
    assert_eq!(buckets[1], buckets[2]);
}

#[test]
fn test_create_histogram_buckets_zero_buckets() {
    let values = vec![1.0, 2.0, 3.0];
    let buckets = create_histogram_buckets(&values, 0);

    assert_eq!(buckets.len(), 0, "Zero buckets should return empty");
}

#[test]
fn test_create_histogram_buckets_distribution() {
    let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let buckets = create_histogram_buckets(&values, 10);

    assert_eq!(buckets.len(), 100);

    // Check that buckets are well-distributed
    let max_bucket = buckets.iter().max().unwrap();
    assert_eq!(*max_bucket, 9, "Max bucket should be num_buckets - 1");
}

#[test]
fn test_shannon_entropy_with_strings() {
    let values = vec!["a", "b", "c", "d", "e"];
    let entropy = shannon_entropy(&values);

    assert!(
        entropy > 0.9,
        "String entropy should be high for unique values"
    );
}

#[test]
fn test_variance_negative_values() {
    let values = vec![-10.0, -5.0, 0.0, 5.0, 10.0];
    let var = variance(&values);

    assert!(var > 0.0, "Negative values should still have variance");
}

#[test]
fn test_variance_large_values() {
    let values = vec![1000.0, 2000.0, 3000.0, 4000.0, 5000.0];
    let var = variance(&values);

    assert!(var > 0.9, "Large spread should have high variance");
}

#[test]
fn test_histogram_edge_cases() {
    // Test with min == max (all same values)
    let values = vec![42.0, 42.0, 42.0];
    let buckets = create_histogram_buckets(&values, 10);

    // Should handle gracefully
    assert_eq!(buckets.len(), 3);
}

#[test]
fn test_shannon_entropy_chars() {
    let text = "aaabbc".chars().collect::<Vec<_>>();
    let entropy = shannon_entropy(&text);

    // Should have moderate entropy (not uniform, not single)
    assert!(entropy > 0.0 && entropy < 1.0);
}

#[test]
fn test_variance_consistency() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    // Calculate variance multiple times
    let var1 = variance(&values);
    let var2 = variance(&values);

    assert_eq!(var1, var2, "Variance should be consistent");
}

#[test]
fn test_histogram_buckets_boundary_values() {
    // Test that boundary values are handled correctly
    let values = vec![0.0, 0.0, 1.0, 1.0];
    let buckets = create_histogram_buckets(&values, 2);

    assert_eq!(buckets.len(), 4);
    // First two should be in bucket 0, last two in bucket 1 (or all in same if range is 0)
}

#[test]
fn test_shannon_entropy_large_dataset() {
    // Test with larger dataset
    let values: Vec<i32> = (0..1000).map(|i| i % 10).collect();
    let entropy = shannon_entropy(&values);

    // Should have high entropy (10 unique values repeated)
    assert!(entropy > 0.8, "Large dataset entropy: {}", entropy);
}

#[test]
fn test_variance_floating_point_precision() {
    let values = vec![1.0, 1.0000001, 1.0000002];
    let var = variance(&values);

    // Very small differences should give low variance
    assert!(
        var < 0.6,
        "Tiny differences should have low variance: {}",
        var
    );
}

#[test]
fn test_histogram_many_buckets() {
    let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let buckets = create_histogram_buckets(&values, 100);

    assert_eq!(buckets.len(), 100);

    // With 100 buckets for 100 values, we should have good distribution
    let unique_buckets: std::collections::HashSet<_> = buckets.iter().collect();
    assert!(unique_buckets.len() > 50, "Should use many buckets");
}
