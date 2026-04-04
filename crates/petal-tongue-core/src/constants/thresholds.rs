// SPDX-License-Identifier: AGPL-3.0-or-later
//! Health check thresholds.

/// CPU usage percentage above which to emit a warning
pub const CPU_WARNING: f64 = 80.0;
/// Memory usage percentage above which to emit a warning
pub const MEMORY_WARNING: f64 = 50.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_constants() {
        assert!((CPU_WARNING - 80.0).abs() < f64::EPSILON);
        assert!((MEMORY_WARNING - 50.0).abs() < f64::EPSILON);
    }
}
